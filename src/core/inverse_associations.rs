// Inverse Association Management - Automatic bidirectional relationship handling
// Following Meta's TAO pattern for inverse associations

use std::collections::HashMap;
use crate::models::AssociationType;

/// Inverse association mapping - defines bidirectional relationships
/// This follows Meta's TAO pattern where associations can have automatic inverses
#[derive(Clone, Debug)]
pub struct InverseAssociationMap {
    /// Maps association type to its inverse type
    inverse_map: HashMap<AssociationType, AssociationType>,
}

impl InverseAssociationMap {
    pub fn new() -> Self {
        let mut inverse_map = HashMap::new();
        
        // Define bidirectional relationships following social graph patterns
        inverse_map.insert(AssociationType::Friendship, AssociationType::Friendship); // symmetric
        inverse_map.insert(AssociationType::Follow, AssociationType::FollowedBy);
        inverse_map.insert(AssociationType::FollowedBy, AssociationType::Follow);
        inverse_map.insert(AssociationType::Like, AssociationType::LikedBy);
        inverse_map.insert(AssociationType::LikedBy, AssociationType::Like);
        // PostAuthor is unidirectional - no inverse
        
        Self { inverse_map }
    }

    /// Get the inverse association type for a given type
    pub fn get_inverse(&self, assoc_type: &AssociationType) -> Option<&AssociationType> {
        self.inverse_map.get(assoc_type)
    }

    /// Check if an association type has an inverse
    pub fn has_inverse(&self, assoc_type: &AssociationType) -> bool {
        self.inverse_map.contains_key(assoc_type)
    }

    /// Check if an association is symmetric (its own inverse)
    pub fn is_symmetric(&self, assoc_type: &AssociationType) -> bool {
        if let Some(inverse) = self.get_inverse(assoc_type) {
            inverse == assoc_type
        } else {
            false
        }
    }
}

impl Default for InverseAssociationMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Add new association types with inverses
/// This would be part of the schema definition in a full Ent framework
impl AssociationType {
    /// Get the inverse type if it exists
    pub fn inverse(&self) -> Option<AssociationType> {
        let map = InverseAssociationMap::new();
        map.get_inverse(self).copied()
    }

    /// Check if this association type is symmetric
    pub fn is_symmetric(&self) -> bool {
        let map = InverseAssociationMap::new();
        map.is_symmetric(self)
    }
}

// Add missing association types that would be in Meta's system
impl AssociationType {
    /// Get display name for association type
    pub fn display_name(&self) -> &'static str {
        match self {
            AssociationType::Friendship => "friendship",
            AssociationType::Follow => "follows",
            AssociationType::Like => "likes",
            AssociationType::PostAuthor => "authored",
            AssociationType::FollowedBy => "followed_by",
            AssociationType::LikedBy => "liked_by",
            AssociationType::Membership => "membership",
            AssociationType::EventAttendance => "attendance",
            AssociationType::CommentParent => "comment_parent",
            AssociationType::Comments => "comments",
            AssociationType::MentionedUsers => "mentions",
        }
    }
}

// Add the missing association types to the enum
// Note: These would be added to the actual AssociationType enum in models/mod.rs
pub enum ExtendedAssociationType {
    // Existing types
    Friendship,
    Follow,
    Like,
    PostAuthor,
    
    // Inverse types
    FollowedBy,
    LikedBy,
    
    // Additional social graph types (for future)
    Blocked,
    BlockedBy,
    Tagged,
    TaggedBy,
    Mentioned,
    MentionedBy,
    GroupMember,
    GroupAdmin,
    EventAttending,
    EventHost,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse_mapping() {
        let map = InverseAssociationMap::new();
        
        // Test symmetric relationship
        assert_eq!(map.get_inverse(&AssociationType::Friendship), Some(&AssociationType::Friendship));
        assert!(map.is_symmetric(&AssociationType::Friendship));
        
        // Test asymmetric relationship
        assert_eq!(map.get_inverse(&AssociationType::Follow), Some(&AssociationType::FollowedBy));
        assert_eq!(map.get_inverse(&AssociationType::FollowedBy), Some(&AssociationType::Follow));
        assert!(!map.is_symmetric(&AssociationType::Follow));
        
        // Test unidirectional relationship
        assert_eq!(map.get_inverse(&AssociationType::PostAuthor), None);
        assert!(!map.has_inverse(&AssociationType::PostAuthor));
    }
}