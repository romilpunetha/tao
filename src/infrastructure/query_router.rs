use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::{AppError, AppResult};
use crate::infrastructure::shard_topology::{
    ConsistentHashingShardManager, ShardHealth, ShardId, ShardInfo, ShardManager, ShardTopology,
};
use crate::infrastructure::write_ahead_log::TaoOperation;

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
    pub shard_manager: Arc<dyn ShardManager + Send + Sync>,
    /// Database instances for each shard (initialized at startup)
    shard_databases:
        Arc<RwLock<HashMap<ShardId, Arc<dyn crate::infrastructure::DatabaseInterface>>>>,
    /// Router configuration
    config: QueryRouterConfig,
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
        let shard_manager = Arc::new(ConsistentHashingShardManager::new(topology));
        let shard_databases = Arc::new(RwLock::new(HashMap::new()));

        Self {
            shard_manager,
            shard_databases,
            config,
        }
    }

    /// Add a new shard with its database connection
    pub async fn add_shard(
        &self,
        shard_info: ShardInfo,
        database: Arc<dyn crate::infrastructure::DatabaseInterface>,
    ) -> AppResult<()> {
        let shard_id = shard_info.shard_id;

        // Add to topology
        {
            self.shard_manager.add_shard(shard_info).await;
        }

        // Store database connection
        {
            let mut databases = self.shard_databases.write().await;
            databases.insert(shard_id, database);
        }

        info!(
            "Successfully added shard {} with database connection to query router",
            shard_id
        );
        Ok(())
    }

    /// =========================================================================
    /// ROUTING METHODS - Pure routing logic, provides database instances
    /// =========================================================================

    /// Determine which shard should handle an object based on owner ID
    pub async fn get_shard_for_owner(&self, owner_id: i64) -> AppResult<ShardId> {
        self.shard_manager.get_shard_for_owner(owner_id).await
    }

    /// Determine which shard contains an object based on object ID
    pub async fn get_shard_for_object(&self, object_id: i64) -> ShardId {
        self.shard_manager.get_shard_for_object(object_id).await
    }

    /// Get database instance for a shard - This is the key method TAO uses
    pub async fn get_database_for_shard(
        &self,
        shard_id: ShardId,
    ) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let databases = self.shard_databases.read().await;
        databases.get(&shard_id).cloned().ok_or_else(|| {
            AppError::ShardError(format!("Database for shard {} not available", shard_id))
        })
    }

    /// Get database instance for an object (convenience method)
    pub async fn get_database_for_object(
        &self,
        object_id: i64,
    ) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let shard_id = self.get_shard_for_object(object_id).await;
        self.get_database_for_shard(shard_id).await
    }

    /// Get database instance for an owner (convenience method)
    pub async fn get_database_for_owner(
        &self,
        owner_id: i64,
    ) -> AppResult<Arc<dyn crate::infrastructure::DatabaseInterface>> {
        let shard_id = self.get_shard_for_owner(owner_id).await?;
        self.get_database_for_shard(shard_id).await
    }

    /// Get all available shards
    pub async fn get_all_shards(&self) -> Vec<ShardId> {
        let databases = self.shard_databases.read().await;
        databases.keys().copied().collect()
    }

    /// =========================================================================
    /// EXECUTION METHODS - Executes operations on their respective shards
    /// =========================================================================

    /// Execute operations for a specific shard only
    /// This is used by the Tao orchestrator to execute operations on the correct shard
    pub async fn execute_operations_for_shard(
        &self,
        shard_id: ShardId,
        operations: &[&TaoOperation],
    ) -> AppResult<()> {
        // Get the database for this shard
        let db = match self.get_database_for_shard(shard_id).await {
            Ok(db) => db,
            Err(e) => {
                error!("Cannot get database for shard {}: {}", shard_id, e);
                return Err(e);
            }
        };

        // Execute each operation on this shard
        for op in operations {
            if let Err(e) = db.execute_operation(*op).await {
                error!("Failed to execute operation on shard {}: {}", shard_id, e);
                return Err(e);
            }
        }

        debug!(
            "Successfully executed {} operations on shard {}",
            operations.len(),
            shard_id
        );
        Ok(())
    }

    /// Get router statistics
    pub async fn get_stats(&self) -> QueryRouterStats {
        let shard_count = {
            let databases = self.shard_databases.read().await;
            databases.len()
        };

        QueryRouterStats {
            active_connections: shard_count,
            replication_factor: self.config.replication_factor,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct QueryRouterStats {
    pub active_connections: usize,
    pub replication_factor: usize,
}

impl std::fmt::Debug for TaoQueryRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaoQueryRouter")
            .field("config", &self.config)
            .finish()
    }
}
