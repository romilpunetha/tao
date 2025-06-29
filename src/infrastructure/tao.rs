// TAO - Main TAO Interface with Decorator Pattern
// This is the primary interface that developers use, wrapping tao_core with decorators
// Architecture: TAO -> Decorators -> TaoCore -> QueryRouter -> Database

use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ent_framework::ent_builder::EntBuilder;
use crate::infrastructure::{
    cache_layer::TaoMultiTierCache,
    database::DatabaseTransaction, // Keep AssocQuery for now, will remove once all Tao uses TaoAssocQuery
    monitoring::MetricsCollector,
    tao_core::{
        AssocType, TaoAssocQuery, TaoAssociation, TaoCore, TaoId, TaoObject, TaoOperations, TaoType,
    },
    tao_decorators::{
        BaseTao, CacheDecorator, CircuitBreakerDecorator, MetricsDecorator, TaoDecorator,
        WalDecorator,
    },
    write_ahead_log::TaoWriteAheadLog,
};

// Re-export core types for convenience
pub use crate::infrastructure::tao_core::{
    create_tao_association, current_time_millis, TaoAssociation as Association, TaoId as Id,
    TaoObject as Object, TaoType as Type,
};

/// TAO - The main entry point for all TAO operations
/// This provides a fully decorated TAO instance with all production features
#[derive(Debug)]
pub struct Tao {
    /// Fully decorated TAO implementation chain
    decorated_tao: Arc<dyn TaoDecorator>,
}

impl Tao {
    /// Create a new TAO instance with all decorators enabled
    pub fn new(
        tao_core: Arc<TaoCore>,
        wal: Arc<TaoWriteAheadLog>,
        cache: Arc<TaoMultiTierCache>,
        metrics: Arc<MetricsCollector>,
        enable_caching: bool,
        enable_circuit_breaker: bool,
    ) -> Self {
        // Build the decorator chain: CircuitBreaker -> Metrics -> WAL -> Cache -> BaseTao -> TaoCore
        let base_tao = Arc::new(BaseTao::new(tao_core));

        let cache_decorator = Arc::new(CacheDecorator::new(base_tao, cache, enable_caching));

        let wal_decorator = Arc::new(WalDecorator::new(cache_decorator, wal));

        let metrics_decorator = Arc::new(MetricsDecorator::new(wal_decorator, metrics));

        let circuit_breaker_decorator = Arc::new(CircuitBreakerDecorator::new(
            metrics_decorator,
            5,                       // failure threshold
            Duration::from_secs(30), // recovery timeout
            enable_circuit_breaker,
        ));

        Self {
            decorated_tao: circuit_breaker_decorator,
        }
    }

    /// Create a minimal TAO instance with only basic functionality
    pub fn minimal(tao_core: Arc<TaoCore>) -> Self {
        let base_tao = Arc::new(BaseTao::new(tao_core));
        Self {
            decorated_tao: base_tao,
        }
    }
}

#[async_trait]
impl TaoOperations for Tao {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.decorated_tao.generate_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        self.decorated_tao.create_object(id, otype, data).await
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        self.decorated_tao.obj_get(id).await
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        self.decorated_tao.obj_update(id, data).await
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        self.decorated_tao.obj_delete(id).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.decorated_tao.obj_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.decorated_tao.obj_exists_by_type(id, otype).await
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        self.decorated_tao.obj_update_by_type(id, otype, data).await
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.decorated_tao.obj_delete_by_type(id, otype).await
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        self.decorated_tao.assoc_get(query).await
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        self.decorated_tao.assoc_add(assoc).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.decorated_tao.assoc_delete(id1, atype, id2).await
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.decorated_tao.assoc_count(id1, atype).await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.decorated_tao
            .assoc_range(id1, atype, offset, limit)
            .await
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.decorated_tao
            .assoc_time_range(id1, atype, high_time, low_time, limit)
            .await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.decorated_tao.assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao.get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao.get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        self.decorated_tao.get_neighbor_ids(id, atype, limit).await
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao
            .get_all_objects_of_type(otype, limit)
            .await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.decorated_tao.begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.decorated_tao.execute_query(query).await
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::infrastructure::{
//         cache_layer::initialize_cache_default,
//         database::DatabaseInterface, // Import DatabaseInterface for tests
//         // database::initialize_database_default, // Removed singleton initialization
//         monitoring::initialize_metrics_default,
//         query_router::{QueryRouterConfig, TaoQueryRouter},
//         sqlite_database::SqliteDatabase, // Import SqliteDatabase for tests
//         write_ahead_log::WalConfig,
//         AssociationRegistry,
//     };
//     use sqlx::sqlite::SqlitePoolOptions; // Import for creating SQLite pool

//     async fn setup_minimal_tao() -> Arc<Tao> {
//         // Initialize an in-memory SQLite database for testing
//         let sqlite_db = SqliteDatabase::new_in_memory().await.unwrap();
//         sqlite_db.initialize().await.unwrap();
//         let db_interface: Arc<dyn DatabaseInterface> = Arc::new(sqlite_db);

//         // Setup a query router with the in-memory database
//         let query_router = Arc::new(TaoQueryRouter::new(QueryRouterConfig::default()).await);
//         query_router
//             .add_shard(
//                 crate::infrastructure::shard_topology::ShardInfo {
//                     shard_id: 0,
//                     connection_string: "sqlite_in_memory".to_string(),
//                     region: "test-region".to_string(),
//                     health: crate::infrastructure::shard_topology::ShardHealth::Healthy,
//                     replicas: vec![],
//                     last_health_check: crate::infrastructure::tao_core::current_time_millis(),
//                     load_factor: 0.0,
//                 },
//                 db_interface,
//             )
//             .await
//             .unwrap();

//         let association_registry = Arc::new(AssociationRegistry::new());
//         let tao_core = Arc::new(TaoCore::new(query_router, association_registry));
//         Arc::new(Tao::minimal(tao_core))
//     }

//     #[tokio::test]
//     async fn test_obj_add_get() {
//         let tao = setup_minimal_tao().await;
//         let user_data = serde_json::json!({"name": "Test User", "email": "test@example.com"})
//             .to_string()
//             .into_bytes();
//         let user_id = tao
//             .obj_add("user".to_string(), user_data.clone(), None)
//             .await
//             .unwrap();

//         let fetched_user = tao.obj_get(user_id).await.unwrap().unwrap();
//         assert_eq!(fetched_user.id, user_id);
//         assert_eq!(fetched_user.otype, "user");
//         assert_eq!(fetched_user.data, user_data);
//     }

//     #[tokio::test]
//     async fn test_decorated_tao_initialization() {
//         // Initialize an in-memory SQLite database for testing
//         let sqlite_db = SqliteDatabase::new_in_memory().await.unwrap();
//         sqlite_db.initialize().await.unwrap();
//         let db_interface: Arc<dyn DatabaseInterface> = Arc::new(sqlite_db);

//         let query_router = Arc::new(TaoQueryRouter::new(QueryRouterConfig::default()).await);
//         query_router
//             .add_shard(
//                 crate::infrastructure::shard_topology::ShardInfo {
//                     shard_id: 0,
//                     connection_string: "sqlite_in_memory".to_string(),
//                     region: "test-region".to_string(),
//                     health: crate::infrastructure::shard_topology::ShardHealth::Healthy,
//                     replicas: vec![],
//                     last_health_check: crate::infrastructure::tao_core::current_time_millis(),
//                     load_factor: 0.0,
//                 },
//                 db_interface,
//             )
//             .await
//             .unwrap();

//         let association_registry = Arc::new(AssociationRegistry::new());
//         let tao_core = Arc::new(TaoCore::new(query_router, association_registry));

//         // Create WAL with default config
//         let wal_config = WalConfig::default();
//         let wal = Arc::new(
//             TaoWriteAheadLog::new(wal_config, "/tmp/test_wal")
//                 .await
//                 .unwrap(),
//         );
//         let cache = initialize_cache_default().await.unwrap();
//         let metrics = initialize_metrics_default().await.unwrap();

//         let tao = Tao::new(tao_core, wal, cache, metrics, true, true);

//         // Test basic operations work through the decorator chain
//         let user_data = b"test user".to_vec();
//         let user_id = tao
//             .obj_add("user".to_string(), user_data.clone(), None)
//             .await
//             .unwrap();
//         let fetched_user = tao.obj_get(user_id).await.unwrap().unwrap();
//         assert_eq!(fetched_user.data, user_data);
//     }

//     #[tokio::test]
//     async fn test_assoc_operations() {
//         let tao = setup_minimal_tao().await;
//         let user1_id = tao
//             .obj_add("user".to_string(), b"{}".to_vec(), None)
//             .await
//             .unwrap();
//         let user2_id = tao
//             .obj_add("user".to_string(), b"{}".to_vec(), None)
//             .await
//             .unwrap();

//         let assoc = create_tao_association(user1_id, "friend".to_string(), user2_id, None);
//         tao.assoc_add(assoc.clone()).await.unwrap();

//         let fetched_assocs = tao
//             .assoc_get(TaoAssocQuery {
//                 // Changed from AssocQuery
//                 id1: user1_id,
//                 atype: "friend".to_string(),
//                 id2_set: None,
//                 high_time: None,
//                 low_time: None,
//                 limit: None,
//                 offset: None,
//             })
//             .await
//             .unwrap();
//         assert_eq!(fetched_assocs.len(), 1);
//         assert_eq!(fetched_assocs[0].id2, user2_id);

//         let count = tao
//             .assoc_count(user1_id, "friend".to_string())
//             .await
//             .unwrap();
//         assert_eq!(count, 1);
//     }
// }
