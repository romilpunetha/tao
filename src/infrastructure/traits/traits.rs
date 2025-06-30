use crate::error::AppResult;
use crate::infrastructure::tao_core::tao_core::{TaoAssociation, TaoId, TaoObject};
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
pub trait MetricsInterface: Send + Sync {
    async fn record_request(&self, operation: &str, duration: Duration, success: bool);
    async fn record_business_event(&self, event: &str);
    async fn record_cache_hit(&self, cache_type: &str);
    async fn record_cache_miss(&self, cache_type: &str);
}
