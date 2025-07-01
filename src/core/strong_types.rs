// Strong Types - Ergonomic type safety following GEMINI.md principles
// Replaces primitive type aliases with proper newtype patterns for compile-time safety

use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly-typed TAO entity ID - prevents confusion with other numeric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaoId(pub i64);

impl TaoId {
    /// Create a new TAO ID
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub fn value(self) -> i64 {
        self.0
    }
    
    /// Check if this is a valid ID (positive)
    pub fn is_valid(self) -> bool {
        self.0 > 0
    }
}

impl fmt::Display for TaoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for TaoId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<TaoId> for i64 {
    fn from(id: TaoId) -> Self {
        id.0
    }
}

/// Strongly-typed timestamp - prevents confusion with IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaoTime(pub i64);

impl TaoTime {
    /// Create a new timestamp
    pub fn new(timestamp: i64) -> Self {
        Self(timestamp)
    }
    
    /// Get current time in milliseconds
    pub fn now() -> Self {
        Self(crate::infrastructure::tao_core::tao_core::current_time_millis())
    }
    
    /// Get the raw timestamp value
    pub fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for TaoTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for TaoTime {
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}

impl From<TaoTime> for i64 {
    fn from(time: TaoTime) -> Self {
        time.0
    }
}

/// Strongly-typed entity type - prevents typos and ensures valid entity types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityType(String);

impl EntityType {
    /// Create a new entity type with validation
    pub fn new(name: &str) -> Result<Self, &'static str> {
        if name.is_empty() {
            return Err("Entity type cannot be empty");
        }
        if !name.chars().all(|c| c.is_lowercase() || c == '_') {
            return Err("Entity type must be lowercase with underscores only");
        }
        Ok(Self(name.to_string()))
    }
    
    /// Create entity type without validation (for internal use)
    pub fn new_unchecked(name: &str) -> Self {
        Self(name.to_string())
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for EntityType {
    fn from(s: &str) -> Self {
        Self::new_unchecked(s)
    }
}

impl From<String> for EntityType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<EntityType> for String {
    fn from(entity_type: EntityType) -> Self {
        entity_type.0
    }
}

/// Strongly-typed association type - ensures type safety for relationships
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociationType(String);

impl AssociationType {
    /// Create a new association type with validation
    pub fn new(name: &str) -> Result<Self, &'static str> {
        if name.is_empty() {
            return Err("Association type cannot be empty");
        }
        Ok(Self(name.to_string()))
    }
    
    /// Create association type without validation (for internal use)
    pub fn new_unchecked(name: &str) -> Self {
        Self(name.to_string())
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
    
    /// Common association types
    pub fn friends() -> Self {
        Self("friends".to_string())
    }
    
    pub fn follows() -> Self {
        Self("follows".to_string())
    }
    
    pub fn likes() -> Self {
        Self("likes".to_string())
    }
    
    pub fn posts() -> Self {
        Self("posts".to_string())
    }
    
    pub fn comments() -> Self {
        Self("comments".to_string())
    }
}

impl fmt::Display for AssociationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for AssociationType {
    fn from(s: &str) -> Self {
        Self::new_unchecked(s)
    }
}

impl From<String> for AssociationType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<AssociationType> for String {
    fn from(assoc_type: AssociationType) -> Self {
        assoc_type.0
    }
}

/// Entity identifier that combines ID and type for maximum safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub id: TaoId,
    pub entity_type: EntityType,
}

impl EntityId {
    /// Create a new entity identifier
    pub fn new(id: TaoId, entity_type: EntityType) -> Self {
        Self { id, entity_type }
    }
    
    /// Create from raw values
    pub fn from_raw(id: i64, entity_type: &str) -> Self {
        Self {
            id: TaoId::new(id),
            entity_type: EntityType::new_unchecked(entity_type),
        }
    }
    
    /// Check if this references a specific entity type
    pub fn is_type(&self, entity_type: &str) -> bool {
        self.entity_type.as_str() == entity_type
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.entity_type, self.id)
    }
}

/// Association identifier for type-safe relationship references
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociationId {
    pub from: EntityId,
    pub to: EntityId,
    pub association_type: AssociationType,
}

impl AssociationId {
    /// Create a new association identifier
    pub fn new(from: EntityId, to: EntityId, association_type: AssociationType) -> Self {
        Self {
            from,
            to,
            association_type,
        }
    }
}

impl fmt::Display for AssociationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -[{}]-> {}", self.from, self.association_type, self.to)
    }
}

/// Ergonomic macros for creating strong types
#[macro_export]
macro_rules! tao_id {
    ($id:expr) => {
        crate::core::strong_types::TaoId::new($id)
    };
}

#[macro_export]
macro_rules! entity_type {
    ($name:expr) => {
        crate::core::strong_types::EntityType::new_unchecked($name)
    };
}

#[macro_export]
macro_rules! association_type {
    ($name:expr) => {
        crate::core::strong_types::AssociationType::new_unchecked($name)
    };
}

// Conversion traits for backward compatibility during migration
impl From<crate::infrastructure::tao_core::tao_core::TaoId> for TaoId {
    fn from(id: crate::infrastructure::tao_core::tao_core::TaoId) -> Self {
        Self(id)
    }
}

impl From<TaoId> for crate::infrastructure::tao_core::tao_core::TaoId {
    fn from(id: TaoId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tao_id_operations() {
        let id = TaoId::new(123);
        assert_eq!(id.value(), 123);
        assert!(id.is_valid());
        
        let invalid_id = TaoId::new(-1);
        assert!(!invalid_id.is_valid());
    }

    #[test]
    fn test_entity_type_validation() {
        assert!(EntityType::new("ent_user").is_ok());
        assert!(EntityType::new("").is_err());
        assert!(EntityType::new("INVALID").is_err());
    }

    #[test]
    fn test_entity_id_formatting() {
        let entity_id = EntityId::from_raw(123, "ent_user");
        assert_eq!(entity_id.to_string(), "ent_user:123");
        assert!(entity_id.is_type("ent_user"));
        assert!(!entity_id.is_type("ent_post"));
    }
}