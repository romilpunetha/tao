// TAO - Unified TAO Database Interface
// Single entry point for all TAO operations following Meta's TAO architecture
// Framework layer that provides high-level TAO operations

use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info};

use crate::infrastructure::{
    association_registry::AssociationRegistry,
    database::DatabaseTransaction,
    query_router::TaoQueryRouter,
    shard_topology::ShardId,
};


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
    pub id1: TaoId,
    pub atype: AssocType,
    pub id2: TaoId,
    pub time: TaoTime,
    pub data: Option<Vec<u8>>,
}

/// TAO Object representing an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoObject {
    pub id: TaoId,
    pub otype: TaoType,
    pub data: Vec<u8>,
    pub created_time: TaoTime,
    pub updated_time: TaoTime,
    pub version: u64,
}

/// Association query parameters for Meta TAO operations
#[derive(Debug, Clone)]
pub struct AssocQuery {
    pub id1: TaoId,
    pub atype: AssocType,
    pub id2_set: Option<Vec<TaoId>>,
    pub high_time: Option<TaoTime>,
    pub low_time: Option<TaoTime>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// Object query parameters
#[derive(Debug, Clone)]
pub struct ObjectQuery {
    pub ids: Vec<TaoId>,
    pub otype: Option<TaoType>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// TAO Operations Interface - Meta's complete TAO API
/// This is the single unified interface for all TAO operations
#[async_trait]
pub trait TaoOperations: Send + Sync + std::fmt::Debug {
    // Object operations
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>>;
    async fn obj_add(
        &self,
        otype: TaoType,
        data: Vec<u8>,
        owner_id: Option<TaoId>,
    ) -> AppResult<TaoId>;
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()>;
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool>;
    async fn obj_exists(&self, id: TaoId) -> AppResult<bool>;
    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;
    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>)
        -> AppResult<bool>;
    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;

    // Association operations
    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>>;
    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()>;
    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;
    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64>;
    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>>;
    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>>;
    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;

    // Batch and utility operations
    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>>;
    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>>;
    async fn get_neighbor_ids(&self, id1: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>>;
    /// Get all objects of a specific type across all shards.
    async fn get_all_objects_of_type(&self, otype: TaoType, limit: Option<u32>) -> AppResult<Vec<TaoObject>>;

    // Graph visualization methods
    /// Get all objects and associations from all shards for graph visualization
    async fn get_graph_data(&self) -> AppResult<(Vec<TaoObject>, Vec<TaoAssociation>)>;

    // Transaction support
    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction>;

    // Custom queries (for advanced use cases)
    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>>;

}

/// TaoCore - Core TAO implementation following Meta's architecture
/// Internal TAO layer that handles the actual Meta TAO logic
#[derive(Debug)]
pub struct TaoCore {
    /// Query router for determining which shard to use
    query_router: Arc<TaoQueryRouter>,
    /// Association registry for inverse type lookups
    association_registry: Arc<AssociationRegistry>,
}

impl TaoCore {
    pub fn new(
        query_router: Arc<TaoQueryRouter>,
        association_registry: Arc<AssociationRegistry>,
    ) -> Self {
        Self {
            query_router,
            association_registry,
        }
    }

}

#[async_trait]
impl TaoOperations for TaoCore {
    async fn obj_add(&self, otype: TaoType, data: Vec<u8>, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        let id = self.query_router.generate_tao_id(owner_id).await?;
        let database = self.query_router.get_database_for_object(id).await?;
         database.create_object(id, otype.clone(), data).await?;  
        Ok(id)
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let query = ObjectQuery {
            ids: vec![id],
            otype: None,
            limit: None,
            offset: None,
        };
        let database = self.query_router.get_database_for_object(id).await?;
        let result = database.get_objects(query).await?;

        if let Some(obj) = result.objects.into_iter().next() {
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let database = self.query_router.get_database_for_object(id).await?;
        database.update_object(id, data).await?;
        info!("obj_update: Object {} updated", id);
        Ok(())
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        let database = self.query_router.get_database_for_object(id).await?;
        let deleted = database.delete_object(id).await?;
        if deleted {
            info!("obj_delete: Deleted object {}", id);
        } else {
            info!("obj_delete: Object {} not found for deletion", id);
        }
        Ok(deleted)
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        let database = self.query_router.get_database_for_object(id).await?;
        database.object_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        Ok(!objects.is_empty())
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false);
        }
        self.obj_update(id, data).await.map(|_| true)
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let objects = self.get_by_id_and_type(vec![id], otype).await?;
        if objects.is_empty() {
            return Ok(false);
        }
        self.obj_delete(id).await
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        let database = self.query_router.get_database_for_object(assoc.id1).await?;
        database.create_association(assoc.clone()).await?;
        info!("assoc_add: Created association {}->{} ({})", assoc.id1, assoc.id2, assoc.atype);
        Ok(())
    }

    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>> {
        let database = self.query_router.get_database_for_object(query.id1).await?;
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let database = self.query_router.get_database_for_object(id1).await?;
        let deleted = database.delete_association(id1, atype.clone(), id2).await?;
        if deleted {
            // Cache removed - handled by decorators now
            info!("assoc_delete: Deleted association {}->{} ({})", id1, id2, atype);
        } else {
            info!("assoc_delete: Association {}->{} ({}) not found for deletion", id1, id2, atype);
        }
        Ok(deleted)
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        let database = self.query_router.get_database_for_object(id1).await?;
        database.count_associations(id1, atype).await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: None,
            low_time: None,
            limit: Some(limit),
            offset: Some(offset),
        };
        let database = self.query_router.get_database_for_object(id1).await?;
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        let query = AssocQuery {
            id1,
            atype,
            id2_set: None,
            high_time: Some(high_time),
            low_time: Some(low_time),
            limit,
            offset: None,
        };
        let database = self.query_router.get_database_for_object(id1).await?;
        let result = database.get_associations(query).await?;
        Ok(result.associations)
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let database = self.query_router.get_database_for_object(id1).await?;
        database.association_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        let mut results = Vec::new();
        let mut shard_groups: HashMap<ShardId, Vec<TaoId>> = HashMap::new();

        for id in ids {
            let shard_id = self.query_router.get_shard_for_object(id).await;
            shard_groups
                .entry(shard_id)
                .or_insert_with(Vec::new)
                .push(id);
        }

        for (shard_id, shard_ids) in shard_groups {
            let database = self.query_router.get_database_for_shard(shard_id).await?;
            let query = ObjectQuery {
                ids: shard_ids,
                otype: Some(otype.clone()),
                limit: None,
                offset: None,
            };
            let result = database.get_objects(query).await?;
            results.extend(result.objects);
        }
        Ok(results)
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        let neighbor_ids = self.get_neighbor_ids(id, atype, limit).await?;
        if neighbor_ids.is_empty() {
            return Ok(vec![]);
        }
        self.get_by_id_and_type(neighbor_ids, "".to_string()).await
    }

    async fn get_neighbor_ids(&self, id1: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        let database = self.query_router.get_database_for_object(id1).await?;
        let query = AssocQuery {
            id1,
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

    async fn get_all_objects_of_type(&self, otype: TaoType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        let mut all_objects = Vec::new();
        let all_shard_ids = self.query_router.shard_manager.get_healthy_shards().await;

        for shard_id in all_shard_ids {
            let db = self.query_router.get_database_for_shard(shard_id).await?;
            let query = ObjectQuery {
                ids: vec![],
                otype: Some(otype.clone()),
                limit,
                offset: None,
            };
            let result = db.get_objects(query).await?;
            all_objects.extend(result.objects);
        }
        Ok(all_objects)
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        Err(AppError::Internal(
            "Distributed transactions not supported".to_string(),
        ))
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        let database = self.query_router.get_database_for_object(1).await?;
        database.execute_query(query).await
    }

    async fn get_graph_data(&self) -> AppResult<(Vec<TaoObject>, Vec<TaoAssociation>)> {
        let mut all_objects = Vec::new();
        let mut all_associations = Vec::new();

        // Get all healthy shards
        let all_shard_ids = self.query_router.shard_manager.get_healthy_shards().await;

        // Collect data from each shard
        for shard_id in &all_shard_ids {
            let database = self.query_router.get_database_for_shard(*shard_id).await?;

            // Get all objects from this shard
            let shard_objects = database.get_all_objects_from_shard().await?;
            all_objects.extend(shard_objects);

            // Get all associations from this shard
            let shard_associations = database.get_all_associations_from_shard().await?;
            all_associations.extend(shard_associations);
        }

        info!("get_graph_data: Retrieved {} objects and {} associations from {} shards",
              all_objects.len(), all_associations.len(), all_shard_ids.len());

        Ok((all_objects, all_associations))
    }

}

/// Get current time in milliseconds since epoch
pub fn current_time_millis() -> TaoTime {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as TaoTime
}

/// Create a TAO association
pub fn create_tao_association(
    id1: TaoId,
    atype: AssocType,
    id2: TaoId,
    data: Option<Vec<u8>>,
) -> TaoAssociation {
    TaoAssociation {
        id1,
        atype,
        id2,
        time: current_time_millis(),
        data,
    }
}


// === TAO Singleton Management ===

static TAO_CORE_INSTANCE: OnceCell<Arc<TaoCore>> = OnceCell::new();

/// Initialize the global TaoCore instance with a query router
pub async fn initialize_tao_core(
    query_router: Arc<TaoQueryRouter>,
    association_registry: Arc<AssociationRegistry>,
) -> AppResult<()> {
    let tao_core = TaoCore::new(query_router, association_registry);

    TAO_CORE_INSTANCE.set(Arc::new(tao_core)).map_err(|_| {
        AppError::Internal("TaoCore instance already initialized ".to_string())
    })?;

    println!("✅ TAO Core initialized (Meta architecture: TAO -> Query Router -> Database)");
    Ok(())
}

/// Get the global TaoCore instance (lock-free, thread-safe)
pub async fn get_tao_core() -> AppResult<Arc<TaoCore>> {
    TAO_CORE_INSTANCE
        .get()
        .ok_or_else(|| {
            AppError::Internal(
                "TaoCore instance not initialized. Call initialize_tao_core() first.".to_string(),
            )
        })
        .map(|tao_core| tao_core.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::DatabaseInterface;
    use crate::infrastructure::query_router::QueryRouterConfig;
    use crate::infrastructure::sqlite_database::SqliteDatabase; // Use SqliteDatabase
    use crate::infrastructure::shard_topology::{ShardInfo, ShardHealth}; // Removed unused imports

    async fn setup_tao_core() -> Arc<TaoCore> {
        // Initialize in-memory SQLite database for testing
        let sqlite_db = Arc::new(SqliteDatabase::new_in_memory().await.unwrap());

        // Setup a query router
        let query_router_config = QueryRouterConfig::default();
        let query_router = Arc::new(TaoQueryRouter::new(query_router_config).await);

        // Add a mock shard to the query router
        let shard_info = ShardInfo {
            shard_id: 0,
            health: ShardHealth::Healthy,
            connection_string: "sqlite_in_memory".to_string(),
            region: "test-region".to_string(),
            replicas: vec![],
            last_health_check: crate::infrastructure::tao_core::current_time_millis(),
            load_factor: 0.0,
        };
        query_router.add_shard(shard_info, sqlite_db as Arc<dyn DatabaseInterface>).await.unwrap();

        let association_registry = Arc::new(AssociationRegistry::new());
        Arc::new(TaoCore::new(query_router, association_registry))
    }

    #[tokio::test]
    async fn test_obj_add_get() {
        let tao = setup_tao_core().await;
        let user_data = serde_json::json!({"name": "Test User", "email": "test@example.com"}).to_string().into_bytes();
        let user_id = tao.obj_add("user".to_string(), user_data.clone(), None).await.unwrap();

        let fetched_user = tao.obj_get(user_id).await.unwrap().unwrap();
        assert_eq!(fetched_user.id, user_id);
        assert_eq!(fetched_user.otype, "user");
        assert_eq!(fetched_user.data, user_data);
    }

    #[tokio::test]
    async fn test_assoc_add_get_count() {
        let tao = setup_tao_core().await;
        let user1_id = tao.obj_add("user".to_string(), b"{}".to_vec(), None).await.unwrap();
        let user2_id = tao.obj_add("user".to_string(), b"{}".to_vec(), None).await.unwrap();

        let assoc = create_tao_association(user1_id, "friend".to_string(), user2_id, None);
        tao.assoc_add(assoc.clone()).await.unwrap();

        let fetched_assocs = tao.assoc_get(AssocQuery {
            id1: user1_id,
            atype: "friend".to_string(),
            id2_set: None,
            high_time: None,
            low_time: None,
            limit: None,
            offset: None,
        }).await.unwrap();
        assert_eq!(fetched_assocs.len(), 1);
        assert_eq!(fetched_assocs[0].id2, user2_id);

        let count = tao.assoc_count(user1_id, "friend".to_string()).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_neighbor_ids() {
        let tao = setup_tao_core().await;
        let user1_id = tao.obj_add("user".to_string(), b"{}".to_vec(), None).await.unwrap();
        let user2_id = tao.obj_add("user".to_string(), b"{}".to_vec(), None).await.unwrap();
        let user3_id = tao.obj_add("user".to_string(), b"{}".to_vec(), None).await.unwrap();

        tao.assoc_add(create_tao_association(user1_id, "friend".to_string(), user2_id, None)).await.unwrap();
        tao.assoc_add(create_tao_association(user1_id, "friend".to_string(), user3_id, None)).await.unwrap();

        let neighbors = tao.get_neighbor_ids(user1_id, "friend".to_string(), None).await.unwrap();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&user2_id));
        assert!(neighbors.contains(&user3_id));
    }

    #[tokio::test]
    async fn test_get_all_objects_of_type() {
        let tao = setup_tao_core().await;
        tao.obj_add("user".to_string(), b"user1".to_vec(), None).await.unwrap();
        tao.obj_add("user".to_string(), b"user2".to_vec(), None).await.unwrap();
        tao.obj_add("post".to_string(), b"post1".to_vec(), None).await.unwrap();

        let users = tao.get_all_objects_of_type("user".to_string(), None).await.unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|o| o.data == b"user1"));
        assert!(users.iter().any(|o| o.data == b"user2"));

        let posts = tao.get_all_objects_of_type("post".to_string(), None).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert!(posts.iter().any(|o| o.data == b"post1"));
    }
}
