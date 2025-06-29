// User Entity Schema - Example of Meta's Ent framework schema definition
// Demonstrates bidirectional edges, field validation, and constraints

use crate::ent_framework::{
    AnnotationDefinition, EdgeDefinition, EntSchema, EntityType, FieldDefault, FieldDefinition,
    FieldType, FieldValidator, IndexDefinition,
};

/// User entity schema with comprehensive field and edge definitions
pub struct UserSchema;

impl EntSchema for UserSchema {
    fn entity_type() -> EntityType {
        EntityType::EntUser
    }

    fn fields() -> Vec<FieldDefinition> {
        vec![
            // Required fields
            FieldDefinition::new("username", FieldType::String)
                .unique()
                .validate(FieldValidator::MinLength(3))
                .validate(FieldValidator::MaxLength(30))
                .validate(FieldValidator::Pattern("^[a-zA-Z0-9_]+$".to_string())),
            FieldDefinition::new("email", FieldType::String)
                .unique()
                .validate(FieldValidator::Pattern(
                    r"^[^\s@]+@[^\s@]+\.[^\s@]+$".to_string(),
                )),
            FieldDefinition::new("created_time", FieldType::Time)
                .immutable()
                .default_value(FieldDefault::Function("now".to_string())),
            // Optional fields
            FieldDefinition::new("full_name", FieldType::String)
                .optional()
                .validate(FieldValidator::MaxLength(100)),
            FieldDefinition::new("bio", FieldType::String)
                .optional()
                .validate(FieldValidator::MaxLength(500)),
            FieldDefinition::new("profile_picture_url", FieldType::String).optional(),
            FieldDefinition::new("last_active_time", FieldType::Time).optional(),
            FieldDefinition::new("is_verified", FieldType::Bool)
                .default_value(FieldDefault::Bool(false)),
            FieldDefinition::new("location", FieldType::String).optional(),
            FieldDefinition::new("privacy_settings", FieldType::JSON).optional(),
        ]
    }

    fn edges() -> Vec<EdgeDefinition> {
        vec![
            // Bidirectional friendship edge (symmetric)
            EdgeDefinition::to("friends", EntityType::EntUser)
                .bidirectional()
                .inverse("friends"), // Same name for symmetric relationship
            // Following relationship (asymmetric)
            EdgeDefinition::to("following", EntityType::EntUser)
                .bidirectional()
                .inverse("followers"),
            // Followers (back-reference to following)
            EdgeDefinition::from("followers", EntityType::EntUser, "following"),
            // Posts authored by this user (one-to-many)
            EdgeDefinition::to("posts", EntityType::EntPost),
            // Liked posts (many-to-many)
            EdgeDefinition::to("liked_posts", EntityType::EntPost)
                .bidirectional()
                .inverse("liked_by"),
            // Groups the user is a member of
            EdgeDefinition::to("groups", EntityType::EntGroup)
                .bidirectional()
                .inverse("members"),
            // Pages the user follows
            EdgeDefinition::to("followed_pages", EntityType::EntPage)
                .bidirectional()
                .inverse("followers"),
            // Events the user is attending
            EdgeDefinition::to("attending_events", EntityType::EntEvent)
                .bidirectional()
                .inverse("attendees"),
        ]
    }

    fn indexes() -> Vec<IndexDefinition> {
        vec![
            IndexDefinition::new("idx_username", vec!["username"]).unique(),
            IndexDefinition::new("idx_email", vec!["email"]).unique(),
            IndexDefinition::new("idx_created_time", vec!["created_time"]),
            IndexDefinition::new("idx_last_active", vec!["last_active_time"]),
            IndexDefinition::new("idx_location", vec!["location"]),
        ]
    }

    fn annotations() -> Vec<AnnotationDefinition> {
        vec![
            AnnotationDefinition {
                name: "graphql".to_string(),
                value: "enabled".to_string(),
            },
            AnnotationDefinition {
                name: "table_name".to_string(),
                value: "users".to_string(),
            },
        ]
    }
}
