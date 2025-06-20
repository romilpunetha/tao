// Generated Builder pattern implementation for EntUser
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use async_trait::async_trait;
use crate::ent_framework::{Entity, get_tao};
use crate::infrastructure::tao::TaoOperations;
use crate::error::AppResult;
use crate::infrastructure::tao::current_time_millis;
use super::entity::EntUser;
use thrift::protocol::TSerializable;

#[derive(Debug, Default)]
pub struct EntUserBuilder {
    username: Option<String>,
    email: Option<String>,
    created_time: Option<i64>,
    full_name: Option<String>,
    bio: Option<String>,
    profile_picture_url: Option<String>,
    last_active_time: Option<i64>,
    is_verified: Option<bool>,
    location: Option<String>,
    privacy_settings: Option<String>,
}

impl EntUserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn created_time(mut self, created_time: i64) -> Self {
        self.created_time = Some(created_time);
        self
    }

    pub fn full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    pub fn bio(mut self, bio: String) -> Self {
        self.bio = Some(bio);
        self
    }

    pub fn profile_picture_url(mut self, profile_picture_url: String) -> Self {
        self.profile_picture_url = Some(profile_picture_url);
        self
    }

    pub fn last_active_time(mut self, last_active_time: i64) -> Self {
        self.last_active_time = Some(last_active_time);
        self
    }

    pub fn is_verified(mut self, is_verified: bool) -> Self {
        self.is_verified = Some(is_verified);
        self
    }

    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn privacy_settings(mut self, privacy_settings: String) -> Self {
        self.privacy_settings = Some(privacy_settings);
        self
    }

    /// Save the entity to database via TAO
    pub async fn save(self) -> AppResult<EntUser> {
        let current_time = current_time_millis();

        let entity = EntUser {
            id: 0, // TAO will generate the actual ID
            username: self.username.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'username' not provided".to_string()
            ))?,
            email: self.email.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'email' not provided".to_string()
            ))?,
            created_time: current_time,
            full_name: self.full_name,
            bio: self.bio,
            profile_picture_url: self.profile_picture_url,
            last_active_time: self.last_active_time,
            is_verified: self.is_verified.ok_or_else(|| crate::error::AppError::Validation(
                "Required field 'is_verified' not provided".to_string()
            ))?,
            location: self.location,
            privacy_settings: self.privacy_settings,
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
        let generated_id = tao.obj_add("ent_user".to_string(), data).await?;

        // Create final entity with generated ID
        let mut final_entity = entity;
        final_entity.id = generated_id;

        println!("✅ Created EntUser with TAO ID: {}", generated_id);

        Ok(final_entity)
    }

}

impl EntUser {
    /// Create a new entity builder
    pub fn create() -> EntUserBuilder {
        EntUserBuilder::new()
    }
}

