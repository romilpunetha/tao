// TAO - Unified TAO Database Interface
// Single entry point for all TAO operations following Meta's TAO architecture
// Framework layer that provides high-level TAO operations

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::OnceCell;
use crate::error::AppResult;
use tracing::{debug, error};

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
    /// In distributed mode, the owner_id is used for shard routing
    async fn obj_add(&self, otype: TaoType, data: Vec<u8>) -> AppResult<TaoId>;


    /// obj_update - Updatce object
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
    async fn begin_transaction(&self) -> AppResult<crate::infrastructure::database::DatabaseTransaction>;
}

/// TaoCore - Core TAO implementation following Meta's architecture
/// Internal TAO layer that handles the actual Meta TAO logic
pub struct TaoCore {
    /// Query router that performs database operations on correct shards
    query_router: Arc<crate::infrastructure::query_router::TaoQueryRouter>,
    /// Write-Ahead Log for operation logging and failure recovery
    wal: Option<Arc<crate::infrastructure::write_ahead_log::TaoWriteAheadLog>>,
}

impl TaoCore {
    /// Create TaoCore with query router (simple, clean architecture)
    pub fn new(query_router: Arc<crate::infrastructure::query_router::TaoQueryRouter>) -> Self {
        Self {
            query_router,
            wal: None,
        }
    }

    /// Create TaoCore with query router and WAL for failure recovery
    pub fn new_with_wal(
        query_router: Arc<crate::infrastructure::query_router::TaoQueryRouter>,
        wal: Arc<crate::infrastructure::write_ahead_log::TaoWriteAheadLog>
    ) -> Self {
        Self {
            query_router,
            wal: Some(wal),
        }
    }

    /// Extract owner ID from object data (internal TAO logic)
    /// This hides sharding complexity from developers
    fn extract_owner_from_data(&self, data: &[u8]) -> Option<TaoId> {
        // Try to extract owner_id from the serialized data
        // For now, this is a simple implementation - in production this would
        // parse the actual data structure based on object type
        if data.len() >= 8 {
            // Try to read first 8 bytes as potential owner_id
            let bytes: [u8; 8] = data[0..8].try_into().ok()?;
            let potential_owner = i64::from_le_bytes(bytes);
            if potential_owner > 0 && potential_owner < i64::MAX / 2 {
                return Some(potential_owner);
            }
        }
        None // Use default shard
    }

    /// Execute operations with proper WAL pattern: WAL logs, TaoCore executes
    /// This follows correct separation of concerns
    async fn execute_with_proper_wal(&self, operations: Vec<crate::infrastructure::write_ahead_log::TaoOperation>) -> AppResult<()> {
        if let Some(ref wal) = self.wal {
            // 1. WAL logs operations for durability (returns transaction ID)
            let txn_id = wal.log_operations(operations.clone()).await?;
            
            // 2. TaoCore executes the operations in parallel on database shards
            let execution_result = self.execute_operations_in_parallel(&operations).await;
            
            // 3. Update WAL with execution result
            match execution_result {
                Ok(_) => {
                    wal.mark_transaction_committed(txn_id).await?;
                    debug!("Successfully executed and committed transaction {}", txn_id);
                }
                Err(ref e) => {
                    wal.mark_transaction_failed(txn_id, e.to_string()).await?;
                    error!("Failed to execute transaction {}: {}", txn_id, e);
                }
            }
            
            execution_result
        } else {
            // No WAL configured, execute operations directly
            self.execute_operations_in_parallel(&operations).await
        }
    }
    
    /// Execute multiple operations in parallel across shards
    async fn execute_operations_in_parallel(&self, operations: &[crate::infrastructure::write_ahead_log::TaoOperation]) -> AppResult<()> {
        use futures::future::try_join_all;
        
        // Execute all operations concurrently
        let futures: Vec<_> = operations.iter()
            .map(|op| self.execute_single_operation(op))
            .collect();
            
        // Wait for all operations to complete
        try_join_all(futures).await?;
        
        debug!("Successfully executed {} operations in parallel", operations.len());
        Ok(())
    }
    
    /// Execute a single operation on the appropriate database shard
    async fn execute_single_operation(&self, operation: &crate::infrastructure::write_ahead_log::TaoOperation) -> AppResult<()> {
        match operation {
            crate::infrastructure::write_ahead_log::TaoOperation::InsertObject { object_id, object_type, data, .. } => {
                let database = self.query_router.get_database_for_object(*object_id).await?;
                database.create_object(*object_id, object_type.clone(), data.clone()).await
            }
            crate::infrastructure::write_ahead_log::TaoOperation::InsertAssociation { assoc, .. } => {
                let database = self.query_router.get_database_for_object(assoc.id1).await?;
                database.create_association(assoc.clone()).await
            }
            crate::infrastructure::write_ahead_log::TaoOperation::DeleteAssociation { id1, atype, id2, .. } => {
                let database = self.query_router.get_database_for_object(*id1).await?;
                database.delete_association(*id1, atype.clone(), *id2).await.map(|_| ())
            }
            crate::infrastructure::write_ahead_log::TaoOperation::UpdateObject { object_id, data, .. } => {
                let database = self.query_router.get_database_for_object(*object_id).await?;
                database.update_object(*object_id, data.clone()).await
            }
            crate::infrastructure::write_ahead_log::TaoOperation::DeleteObject { object_id, .. } => {
                let database = self.query_router.get_database_for_object(*object_id).await?;
                database.delete_object(*object_id).await.map(|_| ())
            }
        }
    }


    /// Determine the inverse association type for bidirectional relationships
    /// As specified by user: when user1 follows user2, create inverse relation user2 followers user1
    fn get_inverse_association_type(&self, atype: &str) -> Option<String> {
        match atype {
            "follows" => Some("followers".to_string()),
            "followers" => Some("follows".to_string()),
            "friends" => Some("friends".to_string()), // Symmetric relationship
            "liked" => Some("liked_by".to_string()),
            "liked_by" => Some("liked".to_string()),
            "member_of" => Some("members".to_string()),
            "members" => Some("member_of".to_string()),
            "owns" => Some("owned_by".to_string()),
            "owned_by" => Some("owns".to_string()),
            _ => None, // Non-bidirectional association
        }
    }
}

#[async_trait]
impl TaoOperations for TaoCore {
    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(query.id1).await?;
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id1).await?;
        database.count_associations(id1, atype).await
    }

    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id1).await?;
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: None,
            low_time: None,
            limit: Some(limit),
            offset: Some(offset),
        };
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: TaoTime, low_time: TaoTime, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id1).await?;
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: Some(high_time),
            low_time: Some(low_time),
            limit,
            offset: None,
        };
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id).await?;
        database.get_object(id).await
    }

    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>> {
        // Meta TAO architecture: TAO handles cross-shard operations using Query Router for routing
        let mut results = Vec::new();
        let mut shard_groups: HashMap<crate::infrastructure::shard_topology::ShardId, Vec<TaoId>> = HashMap::new();

        // Group IDs by shard using Query Router
        for id in ids {
            let shard_id = self.query_router.get_shard_for_object(id).await;
            shard_groups.entry(shard_id).or_insert_with(Vec::new).push(id);
        }

        // Query each shard using database instances from Query Router
        for (shard_id, shard_ids) in shard_groups {
            let database = self.query_router.get_database_for_shard(shard_id).await?;
            let query = ObjectQuery {
                ids: shard_ids,
                otype: Some(otype.clone()),
            };
            let result = database.get_objects(query).await?;
            results.extend(result.objects);
        }

        Ok(results)
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let inverse_atype = self.get_inverse_association_type(&assoc.atype);
        let mut operations = Vec::new();

        if let Some(inverse_type) = inverse_atype {
            // Bidirectional association - batch both operations together
            let inverse_assoc = TaoAssociation {
                id1: assoc.id2,    // Flipped: user2 -> followers -> user1
                atype: inverse_type.clone(),
                id2: assoc.id1,    // Flipped source becomes target
                time: assoc.time,
                data: assoc.data.clone(),
            };

            let source_shard = self.query_router.get_shard_for_object(assoc.id1).await;
            let target_shard = self.query_router.get_shard_for_object(assoc.id2).await;

            // Add both operations to the batch
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::InsertAssociation {
                shard: source_shard,
                assoc: assoc.clone(),
            });
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::InsertAssociation {
                shard: target_shard,
                assoc: inverse_assoc,
            });
        } else {
            // Non-bidirectional association - single operation
            let shard = self.query_router.get_shard_for_object(assoc.id1).await;
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::InsertAssociation {
                shard,
                assoc: assoc.clone(),
            });
        }

        // Execute all operations as a batch (WAL logs all first, then executes all)
        self.execute_with_proper_wal(operations).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let inverse_atype = self.get_inverse_association_type(&atype);
        let mut operations = Vec::new();

        if let Some(inverse_type) = inverse_atype {
            // Bidirectional association - batch both deletions together
            let source_shard = self.query_router.get_shard_for_object(id1).await;
            let target_shard = self.query_router.get_shard_for_object(id2).await;

            // Add both operations to the batch
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::DeleteAssociation {
                shard: source_shard,
                id1,
                atype: atype.clone(),
                id2,
            });
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::DeleteAssociation {
                shard: target_shard,
                id1: id2,    // Flipped for inverse
                atype: inverse_type,
                id2: id1,    // Flipped for inverse
            });
        } else {
            // Non-bidirectional association - single deletion
            let shard = self.query_router.get_shard_for_object(id1).await;
            operations.push(crate::infrastructure::write_ahead_log::TaoOperation::DeleteAssociation {
                shard,
                id1,
                atype: atype.clone(),
                id2,
            });
        }

        // Execute all operations as a batch (WAL logs all first, then executes all)
        self.execute_with_proper_wal(operations).await?;
        
        // For deletions, we assume success if no error occurred
        // In a real implementation, we'd need to collect the actual results
        Ok(true)
    }

    async fn obj_add(&self, otype: TaoType, data: Vec<u8>) -> AppResult<TaoId> {
        // TAO automatically determines the best shard for the object
        // For now, we'll use a simple strategy - extract owner from data or use default
        let owner_id = self.extract_owner_from_data(&data).unwrap_or(0);

        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let shard_id = self.query_router.get_shard_for_owner(owner_id).await?;

        // Generate shard-aware ID
        let id_generator = crate::infrastructure::id_generator::TaoIdGenerator::new(shard_id);
        let object_id = id_generator.next_id();

        // Create batch with single operation
        let operations = vec![crate::infrastructure::write_ahead_log::TaoOperation::InsertObject {
            shard: shard_id,
            object_id,
            object_type: otype.clone(),
            data: data.clone(),
        }];

        // Execute as batch (WAL logs first, then executes)
        self.execute_with_proper_wal(operations).await?;

        Ok(object_id)
    }


    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let shard_id = self.query_router.get_shard_for_object(id).await;

        // Create batch with single operation
        let operations = vec![crate::infrastructure::write_ahead_log::TaoOperation::UpdateObject {
            shard: shard_id,
            object_id: id,
            data: data.clone(),
        }];

        // Execute as batch (WAL logs first, then executes)
        self.execute_with_proper_wal(operations).await
    }

    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool> {
        // Check type first using Meta TAO architecture
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false); // Object doesn't exist or wrong type
        }

        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let shard_id = self.query_router.get_shard_for_object(id).await;

        // Create batch with single operation
        let operations = vec![crate::infrastructure::write_ahead_log::TaoOperation::UpdateObject {
            shard: shard_id,
            object_id: id,
            data: data.clone(),
        }];

        // Execute as batch (WAL logs first, then executes)
        self.execute_with_proper_wal(operations).await?;

        Ok(true)
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let shard_id = self.query_router.get_shard_for_object(id).await;

        // Create batch with single operation
        let operations = vec![crate::infrastructure::write_ahead_log::TaoOperation::DeleteObject {
            shard: shard_id,
            object_id: id,
        }];

        // Execute as batch (WAL logs first, then executes)
        self.execute_with_proper_wal(operations).await?;
        
        // For deletions, we assume success if no error occurred
        // In a real implementation, we'd need to collect the actual results
        Ok(true)
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        // Check type first using Meta TAO architecture
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false); // Object doesn't exist or wrong type
        }

        // Meta TAO architecture: TAO -> Query Router (get database) -> TAO performs operation with batched WAL
        let shard_id = self.query_router.get_shard_for_object(id).await;

        // Create batch with single operation
        let operations = vec![crate::infrastructure::write_ahead_log::TaoOperation::DeleteObject {
            shard: shard_id,
            object_id: id,
        }];

        // Execute as batch (WAL logs first, then executes)
        self.execute_with_proper_wal(operations).await?;
        
        // For deletions, we assume success if no error occurred
        // In a real implementation, we'd need to collect the actual results
        Ok(true)
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id1).await?;
        database.association_exists(id1, atype, id2).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id).await?;
        database.object_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        Ok(!objects.is_empty())
    }

    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        // Simple flow: Get neighbor IDs then fetch objects
        let neighbor_ids = self.get_neighbor_ids(id, atype, limit).await?;
        if neighbor_ids.is_empty() {
            return Ok(vec![]);
        }

        // Use get_by_id_and_type with no type filter
        self.get_by_id_and_type(neighbor_ids, "".to_string()).await
    }

    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        // Meta TAO architecture: TAO → Query Router (get database) → TAO performs operation
        let database = self.query_router.get_database_for_object(id).await?;
        let query = AssocQuery {
            id1: id,
            atype,
            id2_set: None,
            high_time: None,
            low_time: None,
            limit,
            offset: None,
        };
        let result = database.get_associations(query).await?;
        Ok(result.associations.into_iter().map(|a| a.id2).collect())
    }

    // === Transaction Support ===

    async fn begin_transaction(&self) -> AppResult<crate::infrastructure::database::DatabaseTransaction> {
        // For now, transactions are not supported across shards
        // This would require distributed transaction coordination
        Err(crate::error::AppError::Internal("Distributed transactions not yet supported".to_string()))
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

static TAO_CORE_INSTANCE: OnceCell<Arc<TaoCore>> = OnceCell::const_new();

/// Initialize the global TaoCore instance with a query router
pub async fn initialize_tao_core(query_router: Arc<crate::infrastructure::query_router::TaoQueryRouter>) -> AppResult<()> {
    let tao_core = TaoCore::new(query_router);

    TAO_CORE_INSTANCE.set(Arc::new(tao_core))
        .map_err(|_| crate::error::AppError::Internal("TaoCore instance already initialized".to_string()))?;

    println!("✅ TAO Core initialized (Meta architecture: TAO -> Query Router -> Database + parallel WAL)");
    Ok(())
}

/// Initialize the global TaoCore instance with query router and WAL for failure recovery
pub async fn initialize_tao_core_with_wal(
    query_router: Arc<crate::infrastructure::query_router::TaoQueryRouter>,
    wal: Arc<crate::infrastructure::write_ahead_log::TaoWriteAheadLog>
) -> AppResult<()> {
    let tao_core = TaoCore::new_with_wal(query_router, wal);

    TAO_CORE_INSTANCE.set(Arc::new(tao_core))
        .map_err(|_| crate::error::AppError::Internal("TaoCore instance already initialized".to_string()))?;

    println!("✅ TAO Core with WAL initialized (Meta architecture: TAO -> Query Router -> Database + parallel WAL logging)");
    Ok(())
}

/// Get the global TaoCore instance (lock-free, thread-safe)
pub async fn get_tao_core() -> AppResult<Arc<TaoCore>> {
    TAO_CORE_INSTANCE.get()
        .ok_or_else(|| crate::error::AppError::Internal("TaoCore instance not initialized. Call initialize_tao_core() first.".to_string()))
        .map(|tao_core| tao_core.clone())
}