// Generated Ent trait implementation for EntComment
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use crate::ent_framework::Entity;
use crate::error::AppResult;
use super::entity::EntComment;
use regex;
use crate::infrastructure::tao::TaoOperations;
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
    // Edge traversal methods
    
    /// Get author via TAO edge traversal
    pub async fn get_author(&self) -> AppResult<Vec<EntUser>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "author".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntUser::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count author via TAO edge traversal
    pub async fn count_author(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "author".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Get post via TAO edge traversal
    pub async fn get_post(&self) -> AppResult<Vec<EntPost>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "post".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntPost::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count post via TAO edge traversal
    pub async fn count_post(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "post".to_string()).await?;
        Ok(count as i64)
    }
    
}

