// Simple schema definitions for missing entities

use crate::ent_schema::{
    EntSchema, FieldDefinition, EdgeDefinition, 
    FieldType, FieldDefault
};
use crate::models::EntityType;

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

/// Page entity schema
pub struct PageSchema;

impl EntSchema for PageSchema {
    fn entity_type() -> EntityType { EntityType::EntPage }
    
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
            EdgeDefinition::from("followers", EntityType::EntUser, "followed_pages"),
            EdgeDefinition::from("posts", EntityType::EntPost, "appears_on_pages"),
        ]
    }
}

/// Event entity schema
pub struct EventSchema;

impl EntSchema for EventSchema {
    fn entity_type() -> EntityType { EntityType::EntEvent }
    
    fn fields() -> Vec<FieldDefinition> {
        vec![
            FieldDefinition::new("name", FieldType::String),
            FieldDefinition::new("description", FieldType::String).optional(),
            FieldDefinition::new("event_time", FieldType::Time),
            FieldDefinition::new("created_time", FieldType::Time)
                .default_value(FieldDefault::Function("now".to_string())),
        ]
    }
    
    fn edges() -> Vec<EdgeDefinition> {
        vec![
            EdgeDefinition::from("attendees", EntityType::EntUser, "attending_events"),
            EdgeDefinition::from("related_posts", EntityType::EntPost, "related_events"),
        ]
    }
}