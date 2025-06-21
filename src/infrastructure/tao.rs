// TAO - Unified TAO Database Interface
// Single entry point for all TAO operations following Meta's TAO architecture
// Framework layer that provides high-level TAO operations

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{OnceCell, Mutex};
use crate::error::AppResult;
use crate::infrastructure::{DatabaseInterface, DatabaseTransaction, get_database};

/// TAO ID type for entity and association IDs
pub type TaoId = i64;

/// TAO timestamp type
pub type TaoTime = i64;

/// TAO object type (e.g., "user", "post")
pub type TaoType = String;

/// Association type (e.g., "friendship", "like")
pub type AssocType = String;

/// TAO Association representing edge relationships between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoAssociation {
    pub id1: TaoId,          // Source entity ID
    pub atype: AssocType,    // Association type (e.g., "friendship", "like")
    pub id2: TaoId,          // Target entity ID
    pub time: TaoTime,       // Association timestamp
    pub data: Option<Vec<u8>>, // Optional metadata
}

/// TAO Object representing an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoObject {
    pub id: TaoId,           // Entity ID
    pub otype: TaoType,      // Object type (e.g., "user", "post")
    pub time: TaoTime,       // Creation/update timestamp
    pub data: Option<Vec<u8>>, // Serialized entity data
}

/// Association query parameters for Meta TAO operations
#[derive(Debug, Clone)]
pub struct AssocQuery {
    pub id1: TaoId,
    pub atype: AssocType,
    pub id2_set: Option<Vec<TaoId>>,  // Optional target ID filtering
    pub high_time: Option<TaoTime>,   // Time range high bound
    pub low_time: Option<TaoTime>,    // Time range low bound
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// Object query parameters
#[derive(Debug, Clone)]
pub struct ObjectQuery {
    pub ids: Vec<TaoId>,
    pub otype: Option<TaoType>,
}

/// TAO Operations Interface - Meta's complete TAO API
/// This is the single unified interface for all TAO operations
#[async_trait]
pub trait TaoOperations: Send + Sync {
    // === Core Meta TAO API Methods ===

    /// assoc_get - Get associations with optional filtering
    /// Equivalent to Meta's assoc_get(id1, atype, id2_set?, high?, low?)
    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>>;

    /// assoc_count - Count associations of given type
    /// Equivalent to Meta's assoc_count(id1, atype)
    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64>;

    /// assoc_range - Get paginated associations
    /// Equivalent to Meta's assoc_range(id1, atype, pos, limit)
    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>>;

    /// assoc_time_range - Get associations within time range
    /// Equivalent to Meta's assoc_time_range(id1, atype, high_time, low_time, limit?)
    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: TaoTime, low_time: TaoTime, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>>;

    /// obj_get - Get object by ID
    /// Equivalent to Meta's obj_get(id)
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>>;

    /// get_by_id_and_type - Get objects by IDs and type
    /// Equivalent to Meta's get_by_id_and_type(ids, otype)
    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>>;

    // === Write Operations ===

    /// assoc_add - Add association
    /// Equivalent to Meta's assoc_add(id1, atype, id2, time, data?)
    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()>;

    /// assoc_delete - Delete association
    /// Equivalent to Meta's assoc_delete(id1, atype, id2)
    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;

    /// obj_add - Add object
    /// Equivalent to Meta's obj_add(otype, data)
    async fn obj_add(&self, otype: TaoType, data: Vec<u8>) -> AppResult<TaoId>;

    /// obj_update - Update object
    /// Equivalent to Meta's obj_update(id, data)
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()>;

    /// obj_update_by_type - Update object with type verification
    /// Ensures only objects of the specified type are updated
    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool>;

    /// obj_delete - Delete object
    /// Equivalent to Meta's obj_delete(id)
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool>;

    /// obj_delete_by_type - Delete object with type verification
    /// Ensures only objects of the specified type are deleted
    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;

    // === Convenience Methods ===

    /// Check if association exists
    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;

    /// Check if object exists
    async fn obj_exists(&self, id: TaoId) -> AppResult<bool>;

    /// Check if object exists with type verification
    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;

    /// Get neighbors (objects connected via associations)
    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>>;

    /// Get neighbor IDs only (more efficient)
    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>>;

    // === Transaction Support ===
    
    /// Begin a transaction for atomic operations
    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction>;
}

/// TAO - Unified TAO Database Interface
/// Single entry point for all TAO operations in the framework
pub struct Tao {
    database: Arc<dyn DatabaseInterface>,
}

impl Tao {
    pub fn new(database: Arc<dyn DatabaseInterface>) -> Self {
        Self {
            database,
        }
    }

    pub fn new_with_shard(database: Arc<dyn DatabaseInterface>) -> Self {
        Self {
            database,
        }
    }
}

#[async_trait]
impl TaoOperations for Tao {
    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>> {
        let db_query = AssocQuery {
            id1: query.id1,
            atype: query.atype,
            id2_set: query.id2_set.clone(),
            high_time: query.high_time,
            low_time: query.low_time,
            limit: query.limit,
            offset: query.offset,
        };

        let result = self.database.get_associations(db_query).await?;

        // Filter by id2_set if provided
        if let Some(id2_set) = query.id2_set {
            let filtered = result.associations.into_iter()
                .filter(|assoc| id2_set.contains(&assoc.id2))
                .collect();
            Ok(filtered)
        } else {
            Ok(result.associations)
        }
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.database.count_associations(id1, atype).await
    }

    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>> {
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: None,
            low_time: None,
            limit: Some(limit),
            offset: Some(offset),
        };

        let result = self.database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: TaoTime, low_time: TaoTime, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>> {
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: Some(high_time),
            low_time: Some(low_time),
            limit,
            offset: None,
        };

        let result = self.database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        self.database.get_object(id).await
    }

    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>> {
        let query = ObjectQuery {
            ids,
            otype: Some(otype),
        };

        let result = self.database.get_objects(query).await?;
        Ok(result.objects)
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        self.database.create_association(assoc).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.database.delete_association(id1, atype, id2).await
    }

    async fn obj_add(&self, otype: TaoType, data: Vec<u8>) -> AppResult<TaoId> {
        // Generate unique ID using TAO ID generator
        let id = crate::infrastructure::id_generator::get_id_generator().next_id();

        // Create object with generated ID
        self.database.create_object(id, otype, data).await?;

        Ok(id)
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        self.database.update_object(id, data).await
    }

    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool> {
        // First verify the object exists and is of the correct type
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false); // Object doesn't exist or wrong type
        }
        
        // Update the object
        self.database.update_object(id, data).await?;
        Ok(true)
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        self.database.delete_object(id).await
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        // First verify the object exists and is of the correct type
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false); // Object doesn't exist or wrong type
        }
        
        // Delete the object
        self.database.delete_object(id).await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.database.association_exists(id1, atype, id2).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.database.object_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        Ok(!objects.is_empty())
    }

    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        let neighbor_ids = self.get_neighbor_ids(id, atype, limit).await?;
        let query = ObjectQuery {
            ids: neighbor_ids,
            otype: None,
        };
        let result = self.database.get_objects(query).await?;
        Ok(result.objects)
    }

    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        let query = AssocQuery {
            id1: id,
            atype,
            id2_set: None,
            high_time: None,
            low_time: None,
            limit,
            offset: None,
        };

        let result = self.database.get_associations(query).await?;
        Ok(result.associations.into_iter().map(|a| a.id2).collect())
    }

    // === Transaction Support ===

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.database.begin_transaction().await
    }
}

/// Helper functions for TAO operations
pub fn current_time_millis() -> TaoTime {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as TaoTime
}

/// Create a TAO association
pub fn create_tao_association(id1: TaoId, atype: AssocType, id2: TaoId, data: Option<Vec<u8>>) -> TaoAssociation {
    TaoAssociation {
        id1,
        atype,
        id2,
        time: current_time_millis(),
        data,
    }
}

/// Create a TAO object
pub fn create_tao_object(otype: TaoType, data: Vec<u8>) -> TaoObject {
    TaoObject {
        id: 0, // Will be set by database
        otype,
        time: current_time_millis(),
        data: Some(data),
    }
}

/// Generate a unique TAO ID using the infrastructure ID generator
pub fn generate_tao_id() -> TaoId {
    crate::infrastructure::id_generator::get_id_generator().next_id()
}

// === TAO Singleton Management ===

static TAO_INSTANCE: OnceCell<Arc<Mutex<Tao>>> = OnceCell::const_new();

/// Initialize the global TAO instance using the database singleton
pub async fn initialize_tao() -> AppResult<()> {
    let database = get_database().await?;
    let tao = Tao::new(database);

    TAO_INSTANCE.set(Arc::new(Mutex::new(tao)))
        .map_err(|_| crate::error::AppError::Internal("TAO instance already initialized".to_string()))?;

    println!("✅ TAO singleton initialized with database");
    Ok(())
}

/// Get the global TAO instance
pub async fn get_tao() -> AppResult<Arc<Mutex<Tao>>> {
    TAO_INSTANCE.get()
        .ok_or_else(|| crate::error::AppError::Internal("TAO instance not initialized. Call initialize_tao() first.".to_string()))
        .map(|tao| tao.clone())
}