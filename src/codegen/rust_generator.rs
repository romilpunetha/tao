// Rust entity struct generation module
// Note: This module generates complementary Rust code that works with thrift-generated structs
use crate::ent_framework::{FieldDefinition, EntityType, SchemaRegistry};
use super::utils; // Import utils for field_type_to_rust

pub struct RustGenerator<'a> {
    _registry: &'a SchemaRegistry,
}

impl<'a> RustGenerator<'a> {
    pub fn new(registry: &'a SchemaRegistry) -> Self {
        Self { _registry: registry }
    }

    /// Generate Rust helper methods and extensions for thrift-generated entities
    pub fn generate_entity_struct(&self, _entity_type: &EntityType, _fields: &[FieldDefinition]) -> Result<(), String> {
        // This module no longer generates new_default.
        // The builder pattern and direct deserialization from TaoObject will handle instantiation.
        Ok(())
    }

    // Removed generate_new_default_method as it's no longer needed.
}
