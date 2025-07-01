// TAO - Unified TAO Database Interface
// Single entry point for all TAO operations following Meta's TAO architecture
// Framework layer that provides high-level TAO operations

use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::framework::builder::ent_builder::EntBuilder;
use crate::framework::entity::ent_trait::Entity;
use crate::infrastructure::association_registry::AssociationRegistry;
use crate::infrastructure::database::database::{DatabaseInterface, PostgresDatabase, AssocQuery, Association, Object, ObjectQuery, DatabaseTransaction};
use crate::infrastructure::query_router::{QueryRouterConfig, TaoQueryRouter};
use crate::infrastructure::shard_topology::{ShardHealth, ShardId, ShardInfo};
use sqlx::postgres::PgPoolOptions;

/// Current time in milliseconds since Unix epoch
pub fn current_time_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// TAO ID type for entity and association IDs
pub type TaoId = i64;

/// TAO timestamp type
pub type TaoTime = i64;

/// TAO object type (e.g., "user", "post")
pub type TaoType = String;

/// Association type (e.g., "friendship", "like")
pub type AssocType = String;

/// Configuration for database shards
#[derive(Debug, Clone)]
pub struct DatabaseShardConfig {
    pub shard_id: u16,
    pub connection_string: String,
    pub region: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
}

/// Configuration for TAO initialization
#[derive(Debug, Clone)]
pub struct TaoConfig {
    pub database_shards: Vec<DatabaseShardConfig>,
    pub query_router_config: QueryRouterConfig,
}

impl Default for TaoConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TaoConfig {
    pub fn new() -> Self {
        Self {
            database_shards: Vec::new(),
            query_router_config: QueryRouterConfig::default(),
        }
    }

    pub fn add_shard(&mut self, shard_config: DatabaseShardConfig) {
        self.database_shards.push(shard_config);
    }
}

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

/// Conversion functions between TAO types and database types
impl From<Object> for TaoObject {
    fn from(obj: Object) -> Self {
        TaoObject {
            id: obj.id,
            otype: obj.otype,
            data: obj.data,
            created_time: obj.created_time,
            updated_time: obj.updated_time,
            version: obj.version,
        }
    }
}

impl From<TaoObject> for Object {
    fn from(tao_obj: TaoObject) -> Self {
        Object {
            id: tao_obj.id,
            otype: tao_obj.otype,
            data: tao_obj.data,
            created_time: tao_obj.created_time,
            updated_time: tao_obj.updated_time,
            version: tao_obj.version,
        }
    }
}

impl From<Association> for TaoAssociation {
    fn from(assoc: Association) -> Self {
        TaoAssociation {
            id1: assoc.id1,
            atype: assoc.atype,
            id2: assoc.id2,
            time: assoc.time,
            data: assoc.data,
        }
    }
}

impl From<TaoAssociation> for Association {
    fn from(tao_assoc: TaoAssociation) -> Self {
        Association {
            id1: tao_assoc.id1,
            atype: tao_assoc.atype,
            id2: tao_assoc.id2,
            time: tao_assoc.time,
            data: tao_assoc.data,
        }
    }
}

/// TAO-specific query parameters (converted to database queries)
#[derive(Debug, Clone)]
pub struct TaoAssocQuery {
    pub id1: TaoId,
    pub atype: AssocType,
    pub id2_set: Option<Vec<TaoId>>,
    pub high_time: Option<TaoTime>,
    pub low_time: Option<TaoTime>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// TAO object query parameters
#[derive(Debug, Clone)]
pub struct TaoObjectQuery {
    pub ids: Vec<TaoId>,
    pub otype: Option<TaoType>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// Convert TAO queries to database queries
impl From<TaoAssocQuery> for AssocQuery {
    fn from(tao_query: TaoAssocQuery) -> Self {
        AssocQuery {
            id1: tao_query.id1,
            atype: tao_query.atype,
            id2_set: tao_query.id2_set,
            high_time: tao_query.high_time,
            low_time: tao_query.low_time,
            limit: tao_query.limit,
            offset: tao_query.offset,
        }
    }
}

impl From<TaoObjectQuery> for ObjectQuery {
    fn from(tao_query: TaoObjectQuery) -> Self {
        ObjectQuery {
            ids: tao_query.ids,
            otype: tao_query.otype,
            limit: tao_query.limit,
            offset: tao_query.offset,
        }
    }
}

/// TAO Operations Interface - Meta's complete TAO API
/// This is the single unified interface for all TAO operations
#[async_trait]
pub trait TaoOperations: Send + Sync + std::fmt::Debug {
    // Object operations
    async fn create<B: EntBuilder + Send>(
        &self,
        state: B::BuilderState,
        owner_id: Option<TaoId>,
    ) -> AppResult<B>
    where
        Self: Sized,
        B::BuilderState: Send + Sync,
    {
        let id = self.generate_id(owner_id).await?;
        let entity = B::build(state, id)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let validation_errors = entity.validate()?;
        if !validation_errors.is_empty() {
            return Err(AppError::Validation(format!(
                "Validation failed: {}",
                validation_errors.join(", ")
            )));
        }

        let data = entity.serialize_to_bytes()?;
        let otype = <B as EntBuilder>::entity_type().to_string();

        self.create_object(id, otype, data).await?;

        Ok(entity)
    }


    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId>;
    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()>;
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>>;
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()>;
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool>;
    async fn obj_exists(&self, id: TaoId) -> AppResult<bool>;
    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;
    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>)
        -> AppResult<bool>;
    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool>;

    // Association operations
    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>>;
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
    async fn get_neighbor_ids(
        &self,
        id1: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>>;
    /// Get all objects of a specific type across all shards.
    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>>;

    // Transaction support
    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction>;

    // Custom queries (for advanced use cases)
    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>>;
}

/// Extension trait for unified builder operations
/// Separate trait to avoid trait object compatibility issues with generics
#[async_trait]
pub trait TaoEntityBuilder: TaoOperations {
    /// Create entity using unified builder pattern
    async fn create_entity<E: EntBuilder + Send + Sync>(
        &self,
        state: E::BuilderState,
    ) -> AppResult<E>
    where
        E::BuilderState: Send + Sync,
    {
        let id = self.generate_id(None).await?;
        let entity = E::build(state, id)
            .map_err(AppError::Validation)?;

        // Validate entity
        let validation_errors = entity.validate()?;
        if !validation_errors.is_empty() {
            return Err(AppError::Validation(format!(
                "Validation failed: {}",
                validation_errors.join(", ")
            )));
        }

        // Serialize and store
        let data = entity.serialize_to_bytes()?;
        let otype = <E as EntBuilder>::entity_type().to_string();

        self.create_object(id, otype, data).await?;

        Ok(entity)
    }
}

// Blanket implementation for all TaoOperations
impl<T: TaoOperations> TaoEntityBuilder for T {}

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

    /// Initialize TaoCore with configuration
    pub async fn from_config(
        mut config: TaoConfig,
        association_registry: Arc<AssociationRegistry>,
    ) -> AppResult<Self> {
        let query_router = Arc::new(TaoQueryRouter::new(config.query_router_config).await);

        // Initialize database shards from config
        for shard_config in config.database_shards.drain(..) {
            info!(
                "Initializing shard {} at {}",
                shard_config.shard_id, shard_config.connection_string
            );

            let pool = PgPoolOptions::new()
                .max_connections(shard_config.max_connections)
                .min_connections(shard_config.min_connections)
                .acquire_timeout(std::time::Duration::from_secs(
                    shard_config.acquire_timeout_secs,
                ))
                .connect(&shard_config.connection_string)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to connect to database for shard {}: {}",
                        shard_config.shard_id, e
                    ))
                })?;

            let database = PostgresDatabase::new(pool);
            database.initialize().await?;

            let db_interface: Arc<dyn DatabaseInterface> = Arc::new(database);

            let shard_info = ShardInfo {
                shard_id: shard_config.shard_id,
                connection_string: shard_config.connection_string.clone(),
                region: shard_config.region,
                health: ShardHealth::Healthy,
                replicas: vec![],
                last_health_check: current_time_millis(),
                load_factor: 0.0,
            };

            query_router.add_shard(shard_info, db_interface).await?;
            info!("✅ Shard {} configured", shard_config.shard_id);
        }

        info!("✅ All {} shards configured", config.database_shards.len());

        Ok(Self::new(query_router, association_registry))
    }
}

#[async_trait]
impl TaoOperations for TaoCore {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.query_router.generate_tao_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let database = self.query_router.get_database_for_object(id).await?;
        database.create_object(id, otype, data).await
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let database = self.query_router.get_database_for_object(id).await?;
        let result = database.get_object(id).await?;

        if let Some(obj) = result {
            Ok(Some(TaoObject {
                id: obj.id,
                otype: obj.otype,
                data: obj.data, // Data is already in raw bytes (Thrift)
                created_time: obj.created_time,
                updated_time: obj.updated_time,
                version: obj.version,
            }))
        } else {
            Ok(None)
        }
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let database = self.query_router.get_database_for_object(id).await?;
        database.update_object(id, data).await?; // Data is already in raw bytes (Thrift)
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
        let db_assoc: Association = assoc.clone().into(); // Convert TaoAssociation to Association
        database.create_association(db_assoc).await?;
        info!(
            "assoc_add: Created association {}->{} ({})",
            assoc.id1, assoc.id2, assoc.atype
        );
        Ok(())
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        let database = self.query_router.get_database_for_object(query.id1).await?;
        let db_query: AssocQuery = query.into();
        let result = database.get_associations(db_query).await?;
        // Convert database associations back to TAO associations
        Ok(result
            .associations
            .into_iter()
            .map(|assoc| assoc.into())
            .collect())
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let database = self.query_router.get_database_for_object(id1).await?;
        let deleted = database.delete_association(id1, atype.clone(), id2).await?;
        if deleted {
            // Cache removed - handled by decorators now
            info!(
                "assoc_delete: Deleted association {}->{} ({})",
                id1, id2, atype
            );
        } else {
            info!(
                "assoc_delete: Association {}->{} ({}) not found for deletion",
                id1, id2, atype
            );
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
        // Convert database associations back to TAO associations
        Ok(result
            .associations
            .into_iter()
            .map(|assoc| assoc.into())
            .collect())
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
        // Convert database associations back to TAO associations
        Ok(result
            .associations
            .into_iter()
            .map(|assoc| assoc.into())
            .collect())
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
                .or_default()
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
            // Convert database objects back to TAO objects
            results.extend(result.objects.into_iter().map(|obj| TaoObject {
                id: obj.id,
                otype: obj.otype,
                data: obj.data, // Data is already in raw bytes (Thrift)
                created_time: obj.created_time,
                updated_time: obj.updated_time,
                version: obj.version,
            }));
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

    async fn get_neighbor_ids(
        &self,
        id1: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
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

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
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
            // Convert database objects back to TAO objects
            all_objects.extend(result.objects.into_iter().map(|obj| TaoObject {
                id: obj.id,
                otype: obj.otype,
                data: obj.data, // Data is already in raw bytes (Thrift)
                created_time: obj.created_time,
                updated_time: obj.updated_time,
                version: obj.version,
            }));
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
