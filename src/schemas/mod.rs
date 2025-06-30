// Schema definitions module - centralized schema registry

pub mod comment_schema;
pub mod event_schema;
pub mod group_schema;
pub mod page_schema;
pub mod post_schema;
pub mod user_schema;

use crate::framework::schema::ent_schema::SchemaRegistry;

pub use comment_schema::CommentSchema;
pub use event_schema::EventSchema;
pub use group_schema::GroupSchema;
pub use page_schema::PageSchema;
pub use post_schema::PostSchema;
pub use user_schema::UserSchema;

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
