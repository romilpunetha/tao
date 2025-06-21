// Entity Trait - Simplified Meta's Entity Framework Interface
// Single trait that provides both entity identity and common CRUD operations

use async_trait::async_trait;
use crate::error::AppResult;
use crate::infrastructure::tao::TaoOperations;
use thrift::protocol::TSerializable;

/// Entity trait that all generated entities implement
/// Provides both entity identity and common CRUD operations templated for all entity types
#[async_trait]
pub trait Entity: Send + Sync + Clone + Sized + TSerializable {
    /// Entity type name for TAO operations (entity-specific)
    const ENTITY_TYPE: &'static str;
    
    /// Get entity ID (entity-specific implementation)
    fn id(&self) -> i64;
    
    /// Validate entity according to schema constraints (entity-specific implementation)
    fn validate(&self) -> AppResult<Vec<String>>;
    
    // --- Common CRUD Operations (templated for all entities) ---
    
    /// Serialize entity to bytes using Thrift
    fn serialize_to_bytes(&self) -> AppResult<Vec<u8>> {
        use thrift::protocol::TCompactOutputProtocol;
        use std::io::Cursor;

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let mut protocol = TCompactOutputProtocol::new(&mut cursor);

        self.write_to_out_protocol(&mut protocol)
            .map_err(|e| crate::error::AppError::SerializationError(e.to_string()))?;

        Ok(buffer)
    }

    /// Deserialize entity from bytes using Thrift
    fn deserialize_from_bytes(data: &[u8]) -> AppResult<Self> {
        use thrift::protocol::TCompactInputProtocol;
        use std::io::Cursor;

        let mut cursor = Cursor::new(data);
        let mut protocol = TCompactInputProtocol::new(&mut cursor);

        Self::read_from_in_protocol(&mut protocol)
            .map_err(|e| crate::error::AppError::DeserializationError(e.to_string()))
    }
    
    /// Load entity with nullable ID - returns None if not found (TYPE-SAFE)
    /// Only returns entities of the correct type, ensuring EntUser::gen_nullable(post_id) returns None
    async fn gen_nullable(entity_id: Option<i64>) -> AppResult<Option<Self>> {
        match entity_id {
            Some(id) => {
                let tao = crate::infrastructure::tao::get_tao().await?;
                let tao = tao.lock().await;
                
                // Use type-aware query to ensure we only get entities of the correct type
                let objects = tao.get_by_id_and_type(vec![id], Self::ENTITY_TYPE.to_string()).await?;
                
                if let Some(obj) = objects.into_iter().next() {
                    if let Some(data) = obj.data {
                        let entity = Self::deserialize_from_bytes(&data)?;
                        Ok(Some(entity))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None) // No entity of this type with this ID
                }
            },
            None => Ok(None),
        }
    }
    
    /// Load entity with enforcement - errors if not found (TYPE-SAFE)
    /// Only loads entities of the correct type, ensuring type safety across the database layer
    async fn gen_enforce(entity_id: i64) -> AppResult<Self> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        // Use type-aware query to ensure we only get entities of the correct type
        let objects = tao.get_by_id_and_type(vec![entity_id], Self::ENTITY_TYPE.to_string()).await?;
        
        if let Some(obj) = objects.into_iter().next() {
            if let Some(data) = obj.data {
                Self::deserialize_from_bytes(&data)
            } else {
                Err(crate::error::AppError::Validation(format!("Entity {} of type {} has no data", entity_id, Self::ENTITY_TYPE)))
            }
        } else {
            Err(crate::error::AppError::Validation(format!("Entity {} of type {} not found", entity_id, Self::ENTITY_TYPE)))
        }
    }
    
    /// Update existing entity (TYPE-SAFE)
    /// Only updates entities of the correct type, ensuring type safety
    async fn update(&mut self) -> AppResult<()> {
        let validation_errors = self.validate()?;
        if !validation_errors.is_empty() {
            return Err(crate::error::AppError::Validation(
                format!("Validation failed: {}", validation_errors.join(", "))
            ));
        }

        let data = self.serialize_to_bytes()?;
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        // Use type-aware update to ensure we only update entities of the correct type
        let updated = tao.obj_update_by_type(self.id(), Self::ENTITY_TYPE.to_string(), data).await?;
        if !updated {
            return Err(crate::error::AppError::Validation(
                format!("Cannot update: entity {} is not of type {}", self.id(), Self::ENTITY_TYPE)
            ));
        }
        
        Ok(())
    }
    
    /// Delete entity by ID (TYPE-SAFE)
    /// Only deletes entities of the correct type, ensuring EntUser::delete(post_id) returns false
    async fn delete(entity_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        // Use type-aware delete to ensure we only delete entities of the correct type
        tao.obj_delete_by_type(entity_id, Self::ENTITY_TYPE.to_string()).await
    }
    
    /// Check if entity exists (TYPE-SAFE)
    /// Only checks for entities of the correct type, ensuring type safety
    async fn exists(entity_id: i64) -> AppResult<bool> {
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        // Use type-aware exists to ensure we only check for entities of the correct type
        tao.obj_exists_by_type(entity_id, Self::ENTITY_TYPE.to_string()).await
    }
    
    /// Batch load multiple entities (TYPE-SAFE)
    /// Efficiently loads multiple entities of the correct type in a single database query
    async fn load_many(entity_ids: Vec<i64>) -> AppResult<Vec<Option<Self>>> {
        if entity_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let tao = crate::infrastructure::tao::get_tao().await?;
        let tao = tao.lock().await;
        
        // Use type-aware batch query for efficiency
        let objects = tao.get_by_id_and_type(entity_ids.clone(), Self::ENTITY_TYPE.to_string()).await?;
        
        // Create a map of found objects by ID
        let mut object_map = std::collections::HashMap::new();
        for obj in objects {
            object_map.insert(obj.id, obj);
        }
        
        // Build results in the same order as requested IDs
        let mut results = Vec::with_capacity(entity_ids.len());
        for id in entity_ids {
            if let Some(obj) = object_map.get(&id) {
                if let Some(data) = &obj.data {
                    let entity = Self::deserialize_from_bytes(data)?;
                    results.push(Some(entity));
                } else {
                    results.push(None);
                }
            } else {
                results.push(None); // No entity of this type with this ID
            }
        }
        
        Ok(results)
    }
    
    /// Get entity type name
    fn entity_type() -> &'static str {
        Self::ENTITY_TYPE
    }
}