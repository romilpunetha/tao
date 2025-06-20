// Generated Builder pattern implementation for EntComment
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use async_trait::async_trait;
use crate::ent_framework::{Entity, get_tao};
use crate::infrastructure::tao::TaoOperations;
use crate::error::AppResult;
use crate::infrastructure::tao::current_time_millis;
use super::entity::EntComment;
use thrift::protocol::TSerializable;

#[derive(Debug, Default)]
pub struct EntCommentBuilder {
    author_id: Option<i64>,
    post_id: Option<i64>,
    content: Option<String>,
    created_time: Option<i64>,
}

impl EntCommentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn author_id(mut self, author_id: i64) -> Self {
        self.author_id = Some(author_id);
        self
    }

    pub fn post_id(mut self, post_id: i64) -> Self {
        self.post_id = Some(post_id);
        self
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn created_time(mut self, created_time: i64) -> Self {
        self.created_time = Some(created_time);
        self
    }

    /// Save the entity to database via TAO
    pub async fn save(self) -> AppResult<EntComment> {
        let current_time = current_time_millis();

        let entity = EntComment {
            id: 0, // TAO will generate the actual ID
            author_id: self.author_id.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'author_id' not provided".to_string()
            ))?,
            post_id: self.post_id.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'post_id' not provided".to_string()
            ))?,
            content: self.content.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'content' not provided".to_string()
            ))?,
            created_time: current_time,
        };

        // Validate entity before saving
        let validation_errors = entity.validate()?;
        if !validation_errors.is_empty() {
            return Err(crate::error::AppError::Validation(
                format!("Validation failed: {}", validation_errors.join(", "))
            ));
        }

        // Serialize entity to bytes for TAO storage
        let data = {
            use thrift::protocol::{TCompactOutputProtocol, TOutputProtocol};
            use std::io::Cursor;
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let mut protocol = TCompactOutputProtocol::new(&mut cursor);
            entity.write_to_out_protocol(&mut protocol)
                .map_err(|e| crate::error::AppError::SerializationError(e.to_string()))?;
            buffer
        };

        // Get TAO singleton instance and save
        let tao = get_tao().await?;
        let tao = tao.lock().await;
        let generated_id = tao.obj_add("ent_comment".to_string(), data).await?;

        // Create final entity with generated ID
        let mut final_entity = entity;
        final_entity.id = generated_id;

        println!("✅ Created EntComment with TAO ID: {}", generated_id);

        Ok(final_entity)
    }

}

impl EntComment {
    /// Create a new entity builder
    pub fn create() -> EntCommentBuilder {
        EntCommentBuilder::new()
    }
}

