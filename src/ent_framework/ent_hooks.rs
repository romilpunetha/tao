// Ent Hooks System - Middleware pattern for entity mutations
// Equivalent to Meta's Ent hooks for pre/post operation logic

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use crate::{
    ent_framework::EntityType,
    error::{AppError, AppResult},
};

/// Hook context containing mutation information
#[derive(Debug, Clone)]
pub struct HookContext {
    pub entity_type: EntityType,
    pub entity_id: Option<i64>,
    pub operation: HookOperation,
    pub data: Option<Value>,
    pub user_id: Option<i64>, // For access control
    pub metadata: HashMap<String, Value>,
}

/// Types of operations that can trigger hooks
#[derive(Debug, Clone, PartialEq)]
pub enum HookOperation {
    Create,
    Update,
    Delete,
    Query,
}

/// Hook execution timing
#[derive(Debug, Clone, PartialEq)]
pub enum HookTiming {
    Before,
    After,
}

/// Trait for implementing entity hooks
#[async_trait]
pub trait EntHook: Send + Sync {
    /// Execute the hook logic
    async fn execute(&self, ctx: &mut HookContext) -> AppResult<()>;
    
    /// Get hook name for debugging
    fn name(&self) -> &str;
    
    /// Get supported operations
    fn operations(&self) -> Vec<HookOperation>;
    
    /// Get hook timing
    fn timing(&self) -> HookTiming;
}

/// Hook registry for managing entity hooks
#[derive(Default)]
pub struct HookRegistry {
    hooks: HashMap<EntityType, Vec<Box<dyn EntHook>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a hook for an entity type
    pub fn register_hook(&mut self, entity_type: EntityType, hook: Box<dyn EntHook>) {
        self.hooks.entry(entity_type).or_default().push(hook);
    }
    
    /// Execute all applicable hooks for an operation
    pub async fn execute_hooks(
        &self,
        entity_type: &EntityType,
        operation: HookOperation,
        timing: HookTiming,
        ctx: &mut HookContext,
    ) -> AppResult<()> {
        if let Some(hooks) = self.hooks.get(entity_type) {
            for hook in hooks {
                if hook.operations().contains(&operation) && hook.timing() == timing {
                    hook.execute(ctx).await.map_err(|e| {
                        AppError::Validation(format!("Hook '{}' failed: {}", hook.name(), e))
                    })?;
                }
            }
        }
        Ok(())
    }
}

/// Built-in hooks for common patterns

/// Timestamp hook - automatically sets created/updated timestamps
pub struct TimestampHook;

#[async_trait]
impl EntHook for TimestampHook {
    async fn execute(&self, ctx: &mut HookContext) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp();
        
        if let Some(data) = &mut ctx.data {
            match ctx.operation {
                HookOperation::Create => {
                    data["created_time"] = Value::Number(now.into());
                    data["updated_time"] = Value::Number(now.into());
                },
                HookOperation::Update => {
                    data["updated_time"] = Value::Number(now.into());
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "timestamp_hook"
    }
    
    fn operations(&self) -> Vec<HookOperation> {
        vec![HookOperation::Create, HookOperation::Update]
    }
    
    fn timing(&self) -> HookTiming {
        HookTiming::Before
    }
}

/// Validation hook - validates entity data before mutations
pub struct ValidationHook;

#[async_trait]
impl EntHook for ValidationHook {
    async fn execute(&self, ctx: &mut HookContext) -> AppResult<()> {
        if let Some(data) = &ctx.data {
            // Perform entity-specific validation based on schema
            match ctx.entity_type {
                EntityType::EntUser => {
                    self.validate_user(data)?;
                },
                EntityType::EntPost => {
                    self.validate_post(data)?;
                },
                _ => {}
            }
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "validation_hook"
    }
    
    fn operations(&self) -> Vec<HookOperation> {
        vec![HookOperation::Create, HookOperation::Update]
    }
    
    fn timing(&self) -> HookTiming {
        HookTiming::Before
    }
}

impl ValidationHook {
    fn validate_user(&self, data: &Value) -> AppResult<()> {
        if let Some(username) = data.get("username").and_then(|v| v.as_str()) {
            if username.len() < 3 {
                return Err(AppError::Validation("Username must be at least 3 characters".to_string()));
            }
            if username.len() > 30 {
                return Err(AppError::Validation("Username must be at most 30 characters".to_string()));
            }
        }
        
        if let Some(email) = data.get("email").and_then(|v| v.as_str()) {
            if !email.contains('@') {
                return Err(AppError::Validation("Invalid email format".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn validate_post(&self, data: &Value) -> AppResult<()> {
        if let Some(content) = data.get("content").and_then(|v| v.as_str()) {
            if content.is_empty() {
                return Err(AppError::Validation("Post content cannot be empty".to_string()));
            }
            if content.len() > 10000 {
                return Err(AppError::Validation("Post content too long".to_string()));
            }
        }
        
        Ok(())
    }
}

/// Audit log hook - logs all mutations for compliance
pub struct AuditLogHook;

#[async_trait]
impl EntHook for AuditLogHook {
    async fn execute(&self, ctx: &mut HookContext) -> AppResult<()> {
        // TODO: Implement actual audit logging to database/file
        println!(
            "AUDIT: {:?} {:?} on {:?} by user {:?}",
            ctx.operation, ctx.entity_type, ctx.entity_id, ctx.user_id
        );
        Ok(())
    }
    
    fn name(&self) -> &str {
        "audit_log_hook"
    }
    
    fn operations(&self) -> Vec<HookOperation> {
        vec![HookOperation::Create, HookOperation::Update, HookOperation::Delete]
    }
    
    fn timing(&self) -> HookTiming {
        HookTiming::After
    }
}

/// Cache invalidation hook - invalidates cached data after mutations
pub struct CacheInvalidationHook;

#[async_trait]
impl EntHook for CacheInvalidationHook {
    async fn execute(&self, ctx: &mut HookContext) -> AppResult<()> {
        // TODO: Implement actual cache invalidation
        if let Some(entity_id) = ctx.entity_id {
            println!(
                "CACHE: Invalidating cache for {:?} id {}",
                ctx.entity_type, entity_id
            );
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "cache_invalidation_hook"
    }
    
    fn operations(&self) -> Vec<HookOperation> {
        vec![HookOperation::Update, HookOperation::Delete]
    }
    
    fn timing(&self) -> HookTiming {
        HookTiming::After
    }
}

/// Create default hook registry with common hooks
pub fn create_default_hook_registry() -> HookRegistry {
    let mut registry = HookRegistry::new();
    
    // Register common hooks for all entity types
    let entity_types = [
        EntityType::EntUser,
        EntityType::EntPost,
        EntityType::EntComment,
        EntityType::EntGroup,
        EntityType::EntPage,
        EntityType::EntEvent,
    ];
    
    for entity_type in entity_types {
        registry.register_hook(entity_type.clone(), Box::new(TimestampHook));
        registry.register_hook(entity_type.clone(), Box::new(ValidationHook));
        registry.register_hook(entity_type.clone(), Box::new(AuditLogHook));
        registry.register_hook(entity_type.clone(), Box::new(CacheInvalidationHook));
    }
    
    registry
}