// Event entity schema

use crate::framework::schema::ent_schema::EntityType;
use crate::framework::schema::ent_schema::{
    EdgeDefinition, EntSchema, FieldDefault, FieldDefinition, FieldType,
};

/// Event entity schema
pub struct EventSchema;

impl EntSchema for EventSchema {
    fn entity_type() -> EntityType {
        EntityType::EntEvent
    }

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
