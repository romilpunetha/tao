// Page entity schema

use crate::framework::schema::ent_schema::EntityType;
use crate::framework::schema::ent_schema::{
    EdgeDefinition, EntSchema, FieldDefault, FieldDefinition, FieldType,
};

/// Page entity schema
pub struct PageSchema;

impl EntSchema for PageSchema {
    fn entity_type() -> EntityType {
        EntityType::EntPage
    }

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
