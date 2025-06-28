//! A registry for managing association types and their inverse relationships.
//!
//! This module provides a centralized place to define and retrieve inverse
//! association types, ensuring consistency across the TAO system. In a
//! production system, this mapping could be loaded from a configuration file
//! or a schema definition.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages the mapping of association types to their inverse types.
#[derive(Debug, Clone)]
pub struct AssociationRegistry {
    /// A map where the key is an association type and the value is its inverse.
    /// For symmetric associations (e.g., "friends"), the inverse is itself.
    inverse_map: Arc<RwLock<HashMap<String, String>>>,
}

impl AssociationRegistry {
    /// Creates a new `AssociationRegistry` with a predefined set of inverse mappings.
    ///
    /// In a real-world scenario, this could be loaded from a configuration
    /// file or dynamically updated.
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("friends".to_string(), "friends".to_string());
        map.insert("follows".to_string(), "followers".to_string());
        map.insert("liked_by".to_string(), "likes".to_string());
        map.insert("likes".to_string(), "liked_by".to_string());
        map.insert("followers".to_string(), "follows".to_string());
        map.insert("parent_of".to_string(), "child_of".to_string());
        map.insert("child_of".to_string(), "parent_of".to_string());
        map.insert("member_of".to_string(), "has_member".to_string());
        map.insert("has_member".to_string(), "member_of".to_string());

        AssociationRegistry {
            inverse_map: Arc::new(RwLock::new(map)),
        }
    }

    /// Retrieves the inverse association type for a given association type.
    ///
    /// Returns `Some(inverse_type)` if an inverse is defined, otherwise `None`.
    pub async fn get_inverse_association_type(&self, atype: &str) -> Option<String> {
        let map = self.inverse_map.read().await;
        map.get(atype).cloned()
    }

    /// Adds or updates an inverse association mapping.
    pub async fn register_inverse_association(&self, atype: String, inverse_atype: String) {
        let mut map = self.inverse_map.write().await;
        map.insert(atype, inverse_atype);
    }
}

impl Default for AssociationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
