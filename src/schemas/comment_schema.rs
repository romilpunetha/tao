// Comment entity schema

use crate::framework::{
    EntSchema, FieldDefinition, EdgeDefinition, 
    FieldType, FieldDefault
};
use crate::framework::EntityType;

/// Comment entity schema
pub struct CommentSchema;

impl EntSchema for CommentSchema {
    fn entity_type() -> EntityType { EntityType::EntComment }
    
    fn fields() -> Vec<FieldDefinition> {
        vec![
            FieldDefinition::new("author_id", FieldType::Int64),
            FieldDefinition::new("post_id", FieldType::Int64),
            FieldDefinition::new("content", FieldType::String),
            FieldDefinition::new("created_time", FieldType::Time)
                .default_value(FieldDefault::Function("now".to_string())),
        ]
    }
    
    fn edges() -> Vec<EdgeDefinition> {
        vec![
            EdgeDefinition::from("author", EntityType::EntUser, "comments"),
            EdgeDefinition::from("post", EntityType::EntPost, "comments"),
        ]
    }
}