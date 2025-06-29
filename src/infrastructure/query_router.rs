use rand;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::{AppError, AppResult};
use crate::infrastructure::id_generator::{get_id_generator, TaoIdGenerator};
use crate::infrastructure::shard_topology::{
    ConsistentHashingShardManager, ShardHealth, ShardId, ShardInfo, ShardManager, ShardTopology,
};
use crate::infrastructure::tao_core::TaoId;

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

    /// Generate a new TAO ID with proper shard placement
    /// If owner_id is provided, colocate with the owner; otherwise assign random shard
    pub async fn generate_tao_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        if let Some(owner_id) = owner_id {
            // Extract shard from owner_id for colocation
            let owner_shard_id = TaoIdGenerator::extract_shard_id(owner_id);
            let id_generator = TaoIdGenerator::new(owner_shard_id);
            Ok(id_generator.next_id())
        } else {
            // No owner - assign random shard
            let available_shards = self.shard_manager.get_healthy_shards().await;
            if available_shards.is_empty() {
                return Err(AppError::ShardError(
                    "No healthy shards available".to_string(),
                ));
            }

            // Pick a random shard
            use rand::Rng;
            let mut rng = rand::rng();
            let random_index = rng.random_range(0..available_shards.len());
            let random_shard_id = available_shards[random_index];
            let id_generator = TaoIdGenerator::new(random_shard_id);
            Ok(id_generator.next_id())
        }
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

    // /// Execute operations for a specific shard only
    // /// This is used by the Tao orchestrator to execute operations on the correct shard
    // pub async fn execute_operations_for_shard(
    //     &self,
    //     shard_id: ShardId,
    //     operations: &[TaoOperation],
    // ) -> AppResult<()> {
    //     // Temporarily disabled
    //     todo!("Transaction operations temporarily disabled")
    // }

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
