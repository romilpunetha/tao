// Ent Privacy System - Access control and data protection
// Equivalent to Meta's Ent privacy policies for query and mutation control

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use crate::{
    ent_framework::EntityType,
    error::AppResult,
};

/// Privacy rule context for access control decisions
#[derive(Debug, Clone)]
pub struct PrivacyContext {
    pub entity_type: EntityType,
    pub entity_id: Option<i64>,
    pub operation: PrivacyOperation,
    pub user_id: Option<i64>,
    pub user_roles: Vec<String>,
    pub data: Option<Value>,
    pub metadata: HashMap<String, Value>,
}

/// Operations that can be controlled by privacy policies
#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyOperation {
    Create,
    Read,
    Update,
    Delete,
    Query,
}

/// Privacy rule result
#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyResult {
    Allow,
    Deny,
    Skip,    // Skip this rule, continue to next
    Filter,  // Allow but apply data filtering
}

/// Trait for implementing privacy rules
#[async_trait]
pub trait PrivacyRule: Send + Sync {
    /// Evaluate the privacy rule
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult>;
    
    /// Get rule name for debugging
    fn name(&self) -> &str;
    
    /// Get supported operations
    fn operations(&self) -> Vec<PrivacyOperation>;
    
    /// Get rule priority (higher = evaluated first)
    fn priority(&self) -> i32;
}

/// Privacy policy registry
#[derive(Default)]
pub struct PrivacyRegistry {
    rules: HashMap<EntityType, Vec<Box<dyn PrivacyRule>>>,
}

impl PrivacyRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a privacy rule for an entity type
    pub fn register_rule(&mut self, entity_type: EntityType, rule: Box<dyn PrivacyRule>) {
        let rules = self.rules.entry(entity_type).or_default();
        rules.push(rule);
        // Sort by priority (highest first)
        rules.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }
    
    /// Evaluate privacy rules for an operation
    pub async fn evaluate_access(
        &self,
        entity_type: &EntityType,
        operation: PrivacyOperation,
        ctx: &PrivacyContext,
    ) -> AppResult<PrivacyResult> {
        if let Some(rules) = self.rules.get(entity_type) {
            for rule in rules {
                if rule.operations().contains(&operation) {
                    match rule.evaluate(ctx).await? {
                        PrivacyResult::Allow => return Ok(PrivacyResult::Allow),
                        PrivacyResult::Deny => return Ok(PrivacyResult::Deny),
                        PrivacyResult::Filter => return Ok(PrivacyResult::Filter),
                        PrivacyResult::Skip => continue,
                    }
                }
            }
        }
        
        // Default to deny if no rules explicitly allow
        Ok(PrivacyResult::Deny)
    }
}

/// Built-in privacy rules

/// Public access rule - allows public read access
pub struct PublicReadRule;

#[async_trait]
impl PrivacyRule for PublicReadRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        match ctx.operation {
            PrivacyOperation::Read | PrivacyOperation::Query => Ok(PrivacyResult::Allow),
            _ => Ok(PrivacyResult::Skip),
        }
    }
    
    fn name(&self) -> &str {
        "public_read"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![PrivacyOperation::Read, PrivacyOperation::Query]
    }
    
    fn priority(&self) -> i32 {
        100
    }
}

/// Owner-only rule - only entity owner can modify
pub struct OwnerOnlyRule;

#[async_trait]
impl PrivacyRule for OwnerOnlyRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        match ctx.operation {
            PrivacyOperation::Update | PrivacyOperation::Delete => {
                if let (Some(user_id), Some(data)) = (ctx.user_id, &ctx.data) {
                    // Check if user owns the entity (simplified - would need actual ownership check)
                    if let Some(owner_id) = data.get("author_id").or_else(|| data.get("user_id")) {
                        if owner_id.as_i64() == Some(user_id) {
                            return Ok(PrivacyResult::Allow);
                        }
                    }
                }
                Ok(PrivacyResult::Deny)
            },
            _ => Ok(PrivacyResult::Skip),
        }
    }
    
    fn name(&self) -> &str {
        "owner_only"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![PrivacyOperation::Update, PrivacyOperation::Delete]
    }
    
    fn priority(&self) -> i32 {
        200
    }
}

/// Admin access rule - admins can do anything
pub struct AdminAccessRule;

#[async_trait]
impl PrivacyRule for AdminAccessRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        if ctx.user_roles.contains(&"admin".to_string()) {
            Ok(PrivacyResult::Allow)
        } else {
            Ok(PrivacyResult::Skip)
        }
    }
    
    fn name(&self) -> &str {
        "admin_access"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![
            PrivacyOperation::Create,
            PrivacyOperation::Read,
            PrivacyOperation::Update,
            PrivacyOperation::Delete,
            PrivacyOperation::Query,
        ]
    }
    
    fn priority(&self) -> i32 {
        1000 // Highest priority
    }
}

/// Friends-only rule - only friends can see private content
pub struct FriendsOnlyRule;

#[async_trait]
impl PrivacyRule for FriendsOnlyRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        if ctx.operation == PrivacyOperation::Read || ctx.operation == PrivacyOperation::Query {
            if let (Some(user_id), Some(data)) = (ctx.user_id, &ctx.data) {
                // Check privacy settings
                if let Some(visibility) = data.get("visibility").and_then(|v| v.as_str()) {
                    match visibility {
                        "public" => return Ok(PrivacyResult::Allow),
                        "friends" => {
                            // TODO: Check if users are friends
                            // For now, simplified check
                            return Ok(PrivacyResult::Filter);
                        },
                        "private" => {
                            // Only owner can see
                            if let Some(owner_id) = data.get("author_id").or_else(|| data.get("user_id")) {
                                if owner_id.as_i64() == Some(user_id) {
                                    return Ok(PrivacyResult::Allow);
                                }
                            }
                            return Ok(PrivacyResult::Deny);
                        },
                        _ => return Ok(PrivacyResult::Deny),
                    }
                }
            }
        }
        Ok(PrivacyResult::Skip)
    }
    
    fn name(&self) -> &str {
        "friends_only"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![PrivacyOperation::Read, PrivacyOperation::Query]
    }
    
    fn priority(&self) -> i32 {
        300
    }
}

/// Rate limiting rule - prevents spam/abuse
pub struct RateLimitRule {
    max_requests: u32,
    time_window: u64, // seconds
}

impl RateLimitRule {
    pub fn new(max_requests: u32, time_window: u64) -> Self {
        Self { max_requests, time_window }
    }
}

#[async_trait]
impl PrivacyRule for RateLimitRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        if ctx.operation == PrivacyOperation::Create {
            if let Some(user_id) = ctx.user_id {
                // TODO: Check actual rate limiting store (Redis, etc.)
                // For now, simplified check
                println!("Rate limit check for user {} (max: {}/{}s)", 
                    user_id, self.max_requests, self.time_window);
                Ok(PrivacyResult::Allow)
            } else {
                Ok(PrivacyResult::Deny)
            }
        } else {
            Ok(PrivacyResult::Skip)
        }
    }
    
    fn name(&self) -> &str {
        "rate_limit"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![PrivacyOperation::Create]
    }
    
    fn priority(&self) -> i32 {
        500
    }
}

/// Data sanitization rule - filters sensitive data
pub struct DataSanitizationRule;

#[async_trait]
impl PrivacyRule for DataSanitizationRule {
    async fn evaluate(&self, ctx: &PrivacyContext) -> AppResult<PrivacyResult> {
        if ctx.operation == PrivacyOperation::Read || ctx.operation == PrivacyOperation::Query {
            // Always apply filtering for reads
            Ok(PrivacyResult::Filter)
        } else {
            Ok(PrivacyResult::Skip)
        }
    }
    
    fn name(&self) -> &str {
        "data_sanitization"
    }
    
    fn operations(&self) -> Vec<PrivacyOperation> {
        vec![PrivacyOperation::Read, PrivacyOperation::Query]
    }
    
    fn priority(&self) -> i32 {
        50 // Low priority - last filter
    }
}

/// Create default privacy registry with common rules
pub fn create_default_privacy_registry() -> PrivacyRegistry {
    let mut registry = PrivacyRegistry::new();
    
    // Register rules for all entity types
    let entity_types = [
        EntityType::EntUser,
        EntityType::EntPost,
        EntityType::EntComment,
        EntityType::EntGroup,
        EntityType::EntPage,
        EntityType::EntEvent,
    ];
    
    for entity_type in entity_types {
        // Admin access (highest priority)
        registry.register_rule(entity_type.clone(), Box::new(AdminAccessRule));
        
        // Owner access for modifications
        registry.register_rule(entity_type.clone(), Box::new(OwnerOnlyRule));
        
        // Friends-only access for private content
        registry.register_rule(entity_type.clone(), Box::new(FriendsOnlyRule));
        
        // Rate limiting for creates
        registry.register_rule(entity_type.clone(), Box::new(RateLimitRule::new(100, 3600)));
        
        // Public read access (if no other rules apply)
        registry.register_rule(entity_type.clone(), Box::new(PublicReadRule));
        
        // Data sanitization (lowest priority)
        registry.register_rule(entity_type.clone(), Box::new(DataSanitizationRule));
    }
    
    registry
}