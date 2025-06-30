// Post Entity Schema - Demonstrates complex relationships and constraints
// Shows unidirectional vs bidirectional edge configurations

use crate::framework::schema::ent_schema::{
    AnnotationDefinition, EdgeDefinition, EntSchema, EntityType, FieldDefault, FieldDefinition, FieldType,
    FieldValidator, IndexDefinition,
};

/// Post entity schema demonstrating various edge types and constraints
pub struct PostSchema;

impl EntSchema for PostSchema {
    fn entity_type() -> EntityType {
        EntityType::EntPost
    }

    fn fields() -> Vec<FieldDefinition> {
        vec![
            // Author reference (foreign key)
            FieldDefinition::new("author_id", FieldType::Int64),
            // Post content
            FieldDefinition::new("content", FieldType::String)
                .validate(FieldValidator::MinLength(1))
                .validate(FieldValidator::MaxLength(10000)),
            // Optional media
            FieldDefinition::new("media_url", FieldType::String).optional(),
            // Timestamps
            FieldDefinition::new("created_time", FieldType::Time)
                .immutable()
                .default_value(FieldDefault::Function("now".to_string())),
            FieldDefinition::new("updated_time", FieldType::Time).optional(),
            // Post metadata
            FieldDefinition::new("post_type", FieldType::String)
                .default_value(FieldDefault::String("text".to_string())),
            FieldDefinition::new("visibility", FieldType::String)
                .optional()
                .default_value(FieldDefault::String("public".to_string())),
            // Engagement metrics
            FieldDefinition::new("like_count", FieldType::Int).default_value(FieldDefault::Int(0)),
            FieldDefinition::new("comment_count", FieldType::Int)
                .default_value(FieldDefault::Int(0)),
            FieldDefinition::new("share_count", FieldType::Int).default_value(FieldDefault::Int(0)),
            // SEO and discovery
            FieldDefinition::new("tags", FieldType::JSON).optional(),
            FieldDefinition::new("mentions", FieldType::JSON).optional(),
        ]
    }

    fn edges() -> Vec<EdgeDefinition> {
        vec![
            // Author relationship (many-to-one, unidirectional from post perspective)
            EdgeDefinition::from("author", EntityType::EntUser, "posts").required(),
            // Comments on this post (one-to-many)
            EdgeDefinition::to("comments", EntityType::EntComment),
            // Users who liked this post (many-to-many, bidirectional)
            EdgeDefinition::from("liked_by", EntityType::EntUser, "liked_posts"),
            // Users mentioned in this post (many-to-many, unidirectional)
            // Note: Users don't automatically have a "mentioned_in_posts" edge
            EdgeDefinition::to("mentioned_users", EntityType::EntUser),
            // Pages where this post appears (many-to-many)
            EdgeDefinition::to("appears_on_pages", EntityType::EntPage)
                .bidirectional()
                .inverse("posts"),
            // Groups where this post is shared (many-to-many)
            EdgeDefinition::to("shared_in_groups", EntityType::EntGroup)
                .bidirectional()
                .inverse("posts"),
            // Events this post is associated with (many-to-many, optional)
            EdgeDefinition::to("related_events", EntityType::EntEvent)
                .bidirectional()
                .inverse("related_posts"),
        ]
    }

    fn indexes() -> Vec<IndexDefinition> {
        vec![
            IndexDefinition::new("idx_author", vec!["author_id"]),
            IndexDefinition::new("idx_created_time", vec!["created_time"]),
            IndexDefinition::new("idx_post_type", vec!["post_type"]),
            IndexDefinition::new("idx_visibility", vec!["visibility"]),
            IndexDefinition::new("idx_like_count", vec!["like_count"]),
            IndexDefinition::new("idx_author_created", vec!["author_id", "created_time"]),
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
                value: "posts".to_string(),
            },
            AnnotationDefinition {
                name: "cache_ttl".to_string(),
                value: "3600".to_string(), // 1 hour cache
            },
        ]
    }
}
