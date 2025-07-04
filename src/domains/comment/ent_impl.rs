// Generated Ent trait implementation for EntComment
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use std::sync::Arc;
use crate::framework::entity::ent_trait::Entity;
use crate::error::AppResult;
use super::entity::EntComment;
use crate::infrastructure::tao_core::tao_core::{TaoOperations, TaoObject};
use crate::infrastructure::tao_core::tao::Tao;
use thrift::protocol::{TCompactInputProtocol, TSerializable};
use crate::infrastructure::global_tao::get_global_tao;
use std::io::Cursor;
use regex;
use crate::domains::post::EntPost;
use crate::domains::user::EntUser;

impl Entity for EntComment {
    const ENTITY_TYPE: &'static str = "ent_comment";
    
    fn id(&self) -> i64 {
        self.id
    }

    fn validate(&self) -> AppResult<Vec<String>> {
        let mut errors = Vec::new();
        
        
        
        // Validate content (required)
        if self.content.trim().is_empty() {
            errors.push("content cannot be empty".to_string());
        }
        
        Ok(errors)
    }
}

impl EntComment {
    /// Create an entity from a TaoObject
    pub(crate) async fn from_tao_object(tao_obj: TaoObject) -> AppResult<Option<EntComment>> {
        if tao_obj.otype != EntComment::ENTITY_TYPE {
            return Ok(None);
        }
        
        let mut cursor = Cursor::new(&tao_obj.data);
        let mut protocol = TCompactInputProtocol::new(&mut cursor);
        let mut entity = EntComment::read_from_in_protocol(&mut protocol)
            .map_err(|e| crate::error::AppError::SerializationError(e.to_string()))?;
        
        Ok(Some(entity))
    }

    // Edge traversal methods
    
    /// Get author via TAO edge traversal
    pub async fn get_author(&self) -> AppResult<Vec<EntUser>> {
        let tao = get_global_tao()?.clone();
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "author".to_string(), Some(100)).await?;

        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(tao_obj) = tao.obj_get(id).await? {
                if let Some(entity) = EntUser::from_tao_object(tao_obj).await? {
                    results.push(entity);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Count author via TAO edge traversal
    pub async fn count_author(&self) -> AppResult<i64> {
        let tao = get_global_tao()?.clone();
        let count = tao.assoc_count(self.id(), "author".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Get post via TAO edge traversal
    pub async fn get_post(&self) -> AppResult<Vec<EntPost>> {
        let tao = get_global_tao()?.clone();
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "post".to_string(), Some(100)).await?;

        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(tao_obj) = tao.obj_get(id).await? {
                if let Some(entity) = EntPost::from_tao_object(tao_obj).await? {
                    results.push(entity);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Count post via TAO edge traversal
    pub async fn count_post(&self) -> AppResult<i64> {
        let tao = get_global_tao()?.clone();
        let count = tao.assoc_count(self.id(), "post".to_string()).await?;
        Ok(count as i64)
    }
    
}

