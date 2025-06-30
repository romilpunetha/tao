// TAO - Main TAO Interface with Decorator Pattern
// This is the primary interface that developers use, wrapping tao_core with decorators
// Architecture: TAO -> Decorators -> TaoCore -> QueryRouter -> Database

use crate::error::AppResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::infrastructure::{
    cache::cache_layer::TaoMultiTierCache,
    database::database::DatabaseTransaction,
    monitoring::monitoring::MetricsCollector,
    tao_core::tao_core::{
        AssocType, TaoAssocQuery, TaoAssociation, TaoCore, TaoId, TaoObject, TaoOperations, TaoType,
    },
    tao_core::tao_decorators::{
        BaseTao, CacheDecorator, CircuitBreakerDecorator, MetricsDecorator, TaoDecorator,
        WalDecorator,
    },
    storage::write_ahead_log::TaoWriteAheadLog,
};

// Re-export core types for convenience
pub use crate::infrastructure::tao_core::tao_core::{
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

// Simple implementation: just forward all calls to decorated_tao
// We can't use Deref because TaoEntityBuilder has generic methods (not dyn compatible)
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

    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool> {
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

    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>> {
        self.decorated_tao.assoc_range(id1, atype, offset, limit).await
    }

    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: i64, low_time: i64, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>> {
        self.decorated_tao.assoc_time_range(id1, atype, high_time, low_time, limit).await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.decorated_tao.assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao.get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao.get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        self.decorated_tao.get_neighbor_ids(id, atype, limit).await
    }

    async fn get_all_objects_of_type(&self, otype: TaoType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        self.decorated_tao.get_all_objects_of_type(otype, limit).await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.decorated_tao.begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.decorated_tao.execute_query(query).await
    }
}

// Blanket implementation for Arc<T> where T implements TaoOperations
#[async_trait]
impl<T: TaoOperations + ?Sized> TaoOperations for Arc<T> {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        (**self).generate_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        (**self).create_object(id, otype, data).await
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        (**self).obj_get(id).await
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        (**self).obj_update(id, data).await
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        (**self).obj_delete(id).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        (**self).obj_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        (**self).obj_exists_by_type(id, otype).await
    }

    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool> {
        (**self).obj_update_by_type(id, otype, data).await
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        (**self).obj_delete_by_type(id, otype).await
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        (**self).assoc_get(query).await
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        (**self).assoc_add(assoc).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        (**self).assoc_delete(id1, atype, id2).await
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        (**self).assoc_count(id1, atype).await
    }

    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>> {
        (**self).assoc_range(id1, atype, offset, limit).await
    }

    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: i64, low_time: i64, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>> {
        (**self).assoc_time_range(id1, atype, high_time, low_time, limit).await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        (**self).assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>> {
        (**self).get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        (**self).get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        (**self).get_neighbor_ids(id, atype, limit).await
    }

    async fn get_all_objects_of_type(&self, otype: TaoType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        (**self).get_all_objects_of_type(otype, limit).await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        (**self).begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        (**self).execute_query(query).await
    }
}
