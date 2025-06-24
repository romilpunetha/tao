use crate::error::AppResult;
use crate::infrastructure::monitoring::{BusinessEvent, CacheOperation};
use crate::infrastructure::replication::ReplicationOperation;
use crate::infrastructure::security::{Permission, SecurityContext};
use crate::infrastructure::shard_topology::ShardId;
use crate::infrastructure::tao_core::{AssocType, TaoAssociation, TaoId, TaoObject, TaoType};
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CacheInterface: Send + Sync {
    async fn get_object(&self, object_id: TaoId) -> AppResult<Option<TaoObject>>;
    async fn put_object(&self, object_id: TaoId, object: &TaoObject) -> AppResult<()>;
    async fn invalidate_object(&self, object_id: TaoId) -> AppResult<()>;
    async fn put_associations(
        &self,
        id1: TaoId,
        atype: &str,
        associations: &[TaoAssociation],
    ) -> AppResult<()>;
    async fn get_associations(
        &self,
        id1: TaoId,
        atype: &str,
    ) -> AppResult<Option<Vec<TaoAssociation>>>;
    // Add other cache methods as needed
}

#[async_trait]
pub trait SecurityInterface: Send + Sync {
    async fn check_permission(&self, context: &SecurityContext, permission: &Permission) -> bool;
    // Add other security methods as needed (auth, register, etc.)
}

#[async_trait]
pub trait MetricsInterface: Send + Sync {
    async fn record_request(&self, operation: &str, duration: Duration, success: bool);
    async fn record_business_event(&self, event: BusinessEvent);
    async fn record_cache_operation(
        &self,
        operation: CacheOperation,
        hit: bool,
        lookup_time: Duration,
    );
    // Add other metrics methods as needed
}

#[async_trait]
pub trait ReplicationInterface: Send + Sync {
    async fn log_operation(
        &self,
        operation: ReplicationOperation,
        target_shards: Vec<ShardId>,
    ) -> AppResult<String>;
    // Add other replication methods as needed
}

#[async_trait]
pub trait CircuitBreakerInterface: Send + Sync {
    async fn execute<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>> + Send + 'static,
        T: Send + 'static;
}
