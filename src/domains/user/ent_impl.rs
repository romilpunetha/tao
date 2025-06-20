// Generated Ent trait implementation for EntUser
// Generated by TAO Ent Framework - DO NOT EDIT
// Regenerate with: cargo run --bin entc generate

use crate::ent_framework::Entity;
use crate::error::AppResult;
use super::entity::EntUser;
use regex;
use crate::infrastructure::tao::TaoOperations;
use crate::domains::event::EntEvent;
use crate::domains::group::EntGroup;
use crate::domains::post::EntPost;
use crate::domains::page::EntPage;

impl Entity for EntUser {
    const ENTITY_TYPE: &'static str = "ent_user";
    
    fn id(&self) -> i64 {
        self.id
    }

    fn validate(&self) -> AppResult<Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate username (required)
        if self.username.trim().is_empty() {
            errors.push("username cannot be empty".to_string());
        }
        // Validate username min length
        if self.username.len() < 3 {
            errors.push("username must be at least 3 characters".to_string());
        }
        // Validate username max length
        if self.username.len() > 30 {
            errors.push("username cannot exceed 30 characters".to_string());
        }
        // Validate username pattern
        let username_regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !username_regex.is_match(&self.username) {
            errors.push("username format is invalid".to_string());
        }
        
        // Validate email (required)
        if self.email.trim().is_empty() {
            errors.push("email cannot be empty".to_string());
        }
        // Validate email pattern
        let email_regex = regex::Regex::new(r"^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$").unwrap();
        if !email_regex.is_match(&self.email) {
            errors.push("email format is invalid".to_string());
        }
        
        // Validate full name max length
        if let Some(ref val) = self.full_name {
            if val.len() > 100 {
                errors.push("full name cannot exceed 100 characters".to_string());
            }
        }
        
        // Validate bio max length
        if let Some(ref val) = self.bio {
            if val.len() > 500 {
                errors.push("bio cannot exceed 500 characters".to_string());
            }
        }
        
        
        
        
        
        
        Ok(errors)
    }
}

impl EntUser {
    // Edge traversal methods
    
    /// Get friends via TAO edge traversal
    pub async fn get_friends(&self) -> AppResult<Vec<EntUser>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "friends".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntUser::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count friends via TAO edge traversal
    pub async fn count_friends(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "friends".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add friend association via TAO
    pub async fn add_friend(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "friends".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove friend association via TAO
    pub async fn remove_friend(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "friends".to_string(), target_id).await
    }
    
    /// Get following via TAO edge traversal
    pub async fn get_following(&self) -> AppResult<Vec<EntUser>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "following".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntUser::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count following via TAO edge traversal
    pub async fn count_following(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "following".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add following association via TAO
    pub async fn add_following(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "following".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove following association via TAO
    pub async fn remove_following(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "following".to_string(), target_id).await
    }
    
    /// Get followers via TAO edge traversal
    pub async fn get_followers(&self) -> AppResult<Vec<EntUser>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "followers".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntUser::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count followers via TAO edge traversal
    pub async fn count_followers(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "followers".to_string()).await?;
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
    
    /// Get liked posts via TAO edge traversal
    pub async fn get_liked_posts(&self) -> AppResult<Vec<EntPost>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "liked_posts".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntPost::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count liked posts via TAO edge traversal
    pub async fn count_liked_posts(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "liked_posts".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add liked post association via TAO
    pub async fn add_liked_post(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "liked_posts".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove liked post association via TAO
    pub async fn remove_liked_post(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "liked_posts".to_string(), target_id).await
    }
    
    /// Get groups via TAO edge traversal
    pub async fn get_groups(&self) -> AppResult<Vec<EntGroup>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "groups".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntGroup::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count groups via TAO edge traversal
    pub async fn count_groups(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "groups".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add group association via TAO
    pub async fn add_group(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "groups".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove group association via TAO
    pub async fn remove_group(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "groups".to_string(), target_id).await
    }
    
    /// Get followed pages via TAO edge traversal
    pub async fn get_followed_pages(&self) -> AppResult<Vec<EntPage>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "followed_pages".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntPage::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count followed pages via TAO edge traversal
    pub async fn count_followed_pages(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "followed_pages".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add followed page association via TAO
    pub async fn add_followed_page(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "followed_pages".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove followed page association via TAO
    pub async fn remove_followed_page(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "followed_pages".to_string(), target_id).await
    }
    
    /// Get attending events via TAO edge traversal
    pub async fn get_attending_events(&self) -> AppResult<Vec<EntEvent>> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let neighbor_ids = tao.get_neighbor_ids(self.id(), "attending_events".to_string(), Some(100)).await?;
        
        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(entity) = EntEvent::gen_nullable(Some(id)).await? {
                results.push(entity);
            }
        }
        
        Ok(results)
    }
    
    /// Count attending events via TAO edge traversal
    pub async fn count_attending_events(&self) -> AppResult<i64> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        let count = tao.assoc_count(self.id(), "attending_events".to_string()).await?;
        Ok(count as i64)
    }
    
    /// Add attending event association via TAO
    pub async fn add_attending_event(&self, target_id: i64) -> AppResult<()> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        let assoc = crate::infrastructure::tao::create_tao_association(
            self.id(),
            "attending_events".to_string(),
            target_id,
            None // No metadata
        );
        
        tao.assoc_add(assoc).await?;
        Ok(())
    }
    
    /// Remove attending event association via TAO
    pub async fn remove_attending_event(&self, target_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        tao.assoc_delete(self.id(), "attending_events".to_string(), target_id).await
    }
    
}

