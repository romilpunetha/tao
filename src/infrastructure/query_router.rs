use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use serde::Serialize;

use crate::error::{AppResult, AppError};
use crate::infrastructure::shard_topology::{ShardTopology, ShardId, ShardInfo, ShardHealth};

/// Information about a specific shard (no operations, just metadata)
#[derive(Debug, Clone)]
pub struct TaoShardInfo {
    pub shard_id: ShardId,
    pub connection_string: String,
    pub health: ShardHealth,
    pub region: String,
}

impl TaoShardInfo {
    pub fn from_shard_info(shard_info: &ShardInfo) -> Self {
        Self {
            shard_id: shard_info.shard_id,
            connection_string: shard_info.connection_string.clone(),
            health: shard_info.health,
            region: shard_info.region.clone(),
        }
    }
}

/// The main TAO Query Router - Provides database instances for operations
/// This determines which shard to route requests to and provides the database connection
pub struct TaoQueryRouter {
    /// Shard topology manager
    topology: Arc<RwLock<ShardTopology>>,
    /// Database instances for each shard (initialized at startup)
    shard_databases: Arc<RwLock<HashMap<ShardId, Arc<dyn crate::infrastructure::DatabaseInterface>>>>,
    /// Router configuration
    config: QueryRouterConfig,
    /// Write-Ahead Log for cross-shard transactions
    wal: Option<Arc<crate::infrastructure::write_ahead_log::TaoWriteAheadLog>>,
    /// Eventual Consistency Manager for complex cross-shard operations
    consistency_manager: Option<Arc<crate::infrastructure::eventual_consistency::EventualConsistencyManager>>,
}

#[derive(Debug, Clone)]
pub struct QueryRouterConfig {
    pub replication_factor: usize,
    pub health_check_interval_ms: u64,
    pub max_retry_attempts: u32,
    pub enable_read_from_replicas: bool,
}

impl Default for QueryRouterConfig {
    fn default() -> Self {
        Self {
            replication_factor: 2,
            health_check_interval_ms: 30_000, // 30 seconds
            max_retry_attempts: 3,
            enable_read_from_replicas: true,
        }
    }
}

impl TaoQueryRouter {
    pub async fn new(config: QueryRouterConfig) -> Self {
        let topology = Arc::new(RwLock::new(ShardTopology::new(config.replication_factor)));
        let shard_databases = Arc::new(RwLock::new(HashMap::new()));

        Self {
            topology,
            shard_databases,
            config,
            wal: None,
            consistency_manager: None,
        }
    }

    /// Create a new query router with full distributed system integration
    pub async fn new_distributed(
        config: QueryRouterConfig, 
        wal: Arc<crate::infrastructure::write_ahead_log::TaoWriteAheadLog>,
        consistency_manager: Arc<crate::infrastructure::eventual_consistency::EventualConsistencyManager>
    ) -> Self {
        let topology = Arc::new(RwLock::new(ShardTopology::new(config.replication_factor)));
        let shard_databases = Arc::new(RwLock::new(HashMap::new()));

        Self {
            topology,
            shard_databases,
            config,
            wal: Some(wal),
            consistency_manager: Some(consistency_manager),
        }
    }

    /// Add a new shard with its database connection
    pub async fn add_shard(&self, shard_info: ShardInfo, database: Arc<dyn crate::infrastructure::DatabaseInterface>) -> AppResult<()> {
        let shard_id = shard_info.shard_id;
        
        // Add to topology
        {
            let mut topology = self.topology.write().await;
            topology.add_shard(shard_info);
        }
        
        // Store database connection
        {
            let mut databases = self.shard_databases.write().await;
            databases.insert(shard_id, database);
        }

        info!("Successfully added shard {} with database connection to query router", shard_id);
        Ok(())
    }

    /// =========================================================================
    /// ROUTING METHODS - Pure routing logic, provides database instances
    /// =========================================================================

    /// Determine which shard should handle an object based on owner ID
    pub async fn get_shard_for_owner(&self, owner_id: i64) -> AppResult<ShardId> {
        let mut topology = self.topology.write().await;
        topology.get_shard_for_owner(owner_id)
            .ok_or_else(|| AppError::Validation("No healthy shards available".to_string()))
    }

    /// Determine which shard contains an object based on object ID
    pub async fn get_shard_for_object(&self, object_id: i64) -> ShardId {
        let topology = self.topology.read().await;
        topology.get_shard_for_object(object_id)
    }

    /// Get database instance for a shard - This is the key method TAO uses
    pub async fn get_database_for_shard(&self, shard_id: ShardId) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let databases = self.shard_databases.read().await;
        databases.get(&shard_id)
            .cloned()
            .ok_or_else(|| AppError::ShardError(format!("Database for shard {} not available", shard_id)))
    }

    /// Get database instance for an object (convenience method)
    pub async fn get_database_for_object(&self, object_id: i64) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let shard_id = self.get_shard_for_object(object_id).await;
        self.get_database_for_shard(shard_id).await
    }

    /// Get database instance for an owner (convenience method)
    pub async fn get_database_for_owner(&self, owner_id: i64) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let shard_id = self.get_shard_for_owner(owner_id).await?;
        self.get_database_for_shard(shard_id).await
    }

    /// Get all available shards
    pub async fn get_all_shards(&self) -> Vec<ShardId> {
        let databases = self.shard_databases.read().await;
        databases.keys().copied().collect()
    }

    /// =========================================================================
    /// CROSS-SHARD OPERATIONS (using WAL)
    /// =========================================================================

    /// Execute a cross-shard operation using WAL for atomicity
    pub async fn execute_cross_shard_operation(&self, operations: Vec<crate::infrastructure::write_ahead_log::TaoOperation>) -> AppResult<()> {
        if let Some(ref wal) = self.wal {
            let txn_id = wal.execute_cross_shard_transaction(operations).await?;
            info!("Cross-shard operation queued for WAL processing: {}", txn_id);
            Ok(())
        } else {
            Err(AppError::Internal("WAL not available for cross-shard operations".to_string()))
        }
    }

    /// =========================================================================
    /// HIGH-LEVEL SOCIAL OPERATIONS (using Consistency Manager)
    /// =========================================================================

    /// Create a follow relationship between two users (high-level operation)
    pub async fn create_follow_relationship(&self, follower_id: i64, followee_id: i64) -> AppResult<()> {
        if let Some(ref consistency_manager) = self.consistency_manager {
            let txn_id = consistency_manager.handle_follow_relationship(follower_id, followee_id).await?;
            info!("Follow relationship created with transaction ID: {}", txn_id);
            Ok(())
        } else {
            Err(AppError::Internal("Consistency manager not available".to_string()))
        }
    }

    /// Handle a like operation (user likes a post)
    pub async fn create_like_operation(&self, user_id: i64, post_id: i64) -> AppResult<()> {
        if let Some(ref consistency_manager) = self.consistency_manager {
            let txn_id = consistency_manager.handle_like_operation(user_id, post_id).await?;
            info!("Like operation created with transaction ID: {}", txn_id);
            Ok(())
        } else {
            Err(AppError::Internal("Consistency manager not available".to_string()))
        }
    }

    /// Check if the system has full distributed capabilities
    pub fn has_full_distributed_capabilities(&self) -> bool {
        self.wal.is_some() && self.consistency_manager.is_some()
    }

    /// Get router statistics
    pub async fn get_stats(&self) -> QueryRouterStats {
        let topology_stats = {
            let topology = self.topology.read().await;
            topology.get_stats()
        };

        let shard_count = {
            let databases = self.shard_databases.read().await;
            databases.len()
        };

        QueryRouterStats {
            topology_stats,
            active_connections: shard_count,
            replication_factor: self.config.replication_factor,
        }
    }
}

impl std::fmt::Debug for TaoQueryRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaoQueryRouter")
            .field("config", &self.config)
            .field("wal", &self.wal.is_some())
            .field("consistency_manager", &self.consistency_manager.is_some())
            .finish()
    }
}

#[derive(Debug, Serialize)]
pub struct QueryRouterStats {
    pub topology_stats: crate::infrastructure::shard_topology::TopologyStats,
    pub active_connections: usize,
    pub replication_factor: usize,
}