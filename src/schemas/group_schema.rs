// Group entity schema

use crate::ent_framework::{
    EntSchema, FieldDefinition, EdgeDefinition, 
    FieldType, FieldDefault
};
use crate::ent_framework::EntityType;

/// Group entity schema
pub struct GroupSchema;

impl EntSchema for GroupSchema {
    fn entity_type() -> EntityType { EntityType::EntGroup }
    
    fn fields() -> Vec<FieldDefinition> {
        vec![
            FieldDefinition::new("name", FieldType::String),
            FieldDefinition::new("description", FieldType::String).optional(),
            FieldDefinition::new("created_time", FieldType::Time)
                .default_value(FieldDefault::Function("now".to_string())),
        ]
    }
    
    fn edges() -> Vec<EdgeDefinition> {
        vec![
            EdgeDefinition::from("members", EntityType::EntUser, "groups"),
            EdgeDefinition::from("posts", EntityType::EntPost, "shared_in_groups"),
        ]
    }
}