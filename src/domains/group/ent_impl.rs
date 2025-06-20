// Generated Ent trait implementation for EntGroup
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use crate::ent_framework::Entity;
use crate::error::AppResult;
use super::entity::EntGroup;
use regex;
use crate::infrastructure::tao::TaoOperations;
use crate::domains::user::EntUser;
use crate::domains::post::EntPost;

impl Entity for EntGroup {
    const ENTITY_TYPE: &'static str = "ent_group";
    
    fn id(&self) -> i64 {
        self.id
    }

    fn validate(&self) -> AppResult<Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate name (required)
        if self.name.trim().is_empty() {
            errors.push("name cannot be empty".to_string());
        }
        
        
        Ok(errors)
    }
}

impl EntGroup {
    // Edge traversal methods
    
    /// Get members via TAO edge traversal
    pub async fn get_members(&self) -> AppResult<Vec<EntUser>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "members".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntUser::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count members via TAO edge traversal
    pub async fn count_members(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "members".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Get posts via TAO edge traversal
    pub async fn get_posts(&self) -> AppResult<Vec<EntPost>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "posts".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntPost::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count posts via TAO edge traversal
    pub async fn count_posts(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "posts".to_string()).await?;
        Ok(count as i64)
    }
    
}

