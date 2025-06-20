// Core Ent Trait - Meta's Entity Framework Interface
// This trait defines the standard interface that all generated entities implement

use async_trait::async_trait;
use crate::error::AppResult;

/// Core Ent trait that all generated entities implement
/// This provides the standard Meta Ent framework functions
#[async_trait]
pub trait Ent: Send + Sync + Clone {
    /// Entity type name
    const ENTITY_TYPE: &'static str;
    
    /// Get entity ID
    fn id(&self) -> i64;
    
    /// Load entity with nullable ID - returns None if not found
    async fn gen_nullable(entity_id: Option<i64>) -> AppResult<Option<Self>>;
    
    /// Load entity with enforcement - panics if not found
    async fn gen_enforce(entity_id: i64) -> AppResult<Self>;
    
    /// Create new entity and return the saved entity with generated ID
    async fn gen_create(entity: Self) -> AppResult<Self>;
    
    /// Update existing entity
    async fn gen_update(&mut self) -> AppResult<()>;
    
    /// Delete entity by ID
    async fn gen_delete(entity_id: i64) -> AppResult<bool>;
    
    /// Check if entity exists
    async fn gen_exists(entity_id: i64) -> AppResult<bool>;
    
    /// Get entity type name
    fn gen_type() -> &'static str {
        Self::ENTITY_TYPE
    }
    
    /// Batch operations for performance
    async fn gen_load_many(entity_ids: Vec<i64>) -> AppResult<Vec<Option<Self>>>;
    
    /// Create multiple entities
    async fn gen_create_many(entities: Vec<Self>) -> AppResult<Vec<Self>>;
    
    /// Delete multiple entities  
    async fn gen_delete_many(entity_ids: Vec<i64>) -> AppResult<Vec<bool>>;
    
    /// Privacy and permissions
    async fn gen_can_view(&self, viewer_id: Option<i64>) -> AppResult<bool>;
    
    /// Check edit permissions
    async fn gen_can_edit(&self, viewer_id: Option<i64>) -> AppResult<bool>;
    
    /// Validate entity according to schema constraints
    fn validate(&self) -> AppResult<Vec<String>>;
}