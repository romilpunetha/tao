// Schema definitions module - centralized schema registry

pub mod user_schema;
pub mod post_schema;
pub mod simple_schemas;

use crate::ent_schema::SchemaRegistry;

pub use user_schema::UserSchema;
pub use post_schema::PostSchema;
pub use simple_schemas::{CommentSchema, GroupSchema, PageSchema, EventSchema};

/// Initialize and register all schemas
pub fn create_schema_registry() -> SchemaRegistry {
    let mut registry = SchemaRegistry::new();
    
    // Register all entity schemas
    registry.register::<UserSchema>();
    registry.register::<PostSchema>();
    registry.register::<CommentSchema>();
    registry.register::<GroupSchema>();
    registry.register::<PageSchema>();
    registry.register::<EventSchema>();
    
    registry
}

/// Validate all registered schemas
pub fn validate_schemas() -> Result<(), Vec<String>> {
    let registry = create_schema_registry();
    registry.validate()
}