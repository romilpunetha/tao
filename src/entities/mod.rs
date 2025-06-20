// Meta's Entity Framework - Auto-generated database methods for entities

use std::sync::Arc;
use thrift::protocol::TSerializable;
use crate::models::{EntityType, AssociationType};
use crate::thrift_utils::{thrift_serialize, thrift_deserialize};
use anyhow::Result;

// Entity trait that all generated entities implement
pub trait Entity: TSerializable + Clone + Send + Sync {
    fn entity_type() -> EntityType;
    fn entity_type_str() -> &'static str {
        Self::entity_type().as_str()
    }
}

// Generic TAO entity operations trait - uses TaoInterface instead of direct DB access
#[allow(async_fn_in_trait)]
pub trait TaoEntity: Entity + Sized {
    // Meta TAO pattern: Entity::genNullable(id)
    async fn gen_nullable(tao: &crate::tao_interface::TaoInterface, id: i64) -> Result<Option<(i64, Self)>> {
        if let Some(obj) = tao.get_object_by_id_and_type(id, Self::entity_type_str()).await? {
            let entity: Self = thrift_deserialize(&obj.data)?;
            return Ok(Some((obj.id, entity)));
        }
        Ok(None)
    }

    // Meta TAO pattern: Entity::genMulti(ids)
    async fn gen_multi(tao: &crate::tao_interface::TaoInterface, ids: Vec<i64>) -> Result<Vec<(i64, Self)>> {
        let mut entities = Vec::new();
        for id in ids {
            if let Some(entity) = Self::gen_nullable(tao, id).await? {
                entities.push(entity);
            }
        }
        Ok(entities)
    }

    // Meta TAO pattern: Entity::genAll(limit)
    async fn gen_all(tao: &crate::tao_interface::TaoInterface, limit: Option<i32>) -> Result<Vec<(i64, Self)>> {
        let entity_ids = tao.get_all_objects_by_type(Self::entity_type_str(), limit).await?;
        Self::gen_multi(tao, entity_ids).await
    }

    // Meta TAO pattern: Entity::gen_enforce(id) - throws if not found
    async fn gen_enforce(tao: &crate::tao_interface::TaoInterface, id: i64) -> Result<(i64, Self)> {
        Self::gen_nullable(tao, id).await?
            .ok_or_else(|| anyhow::anyhow!("{} with id {} not found", Self::entity_type_str(), id))
    }

    // Create new entity
    async fn create(tao: &crate::tao_interface::TaoInterface, entity: &Self) -> Result<i64> {
        let data = thrift_serialize(entity)?;
        let obj = tao.create_object(Self::entity_type(), &data).await?;
        Ok(obj.id)
    }

    // Update entity
    async fn update(tao: &crate::tao_interface::TaoInterface, id: i64, entity: &Self) -> Result<()> {
        let data = thrift_serialize(entity)?;
        tao.update_object(id, &data).await
    }

    // Delete entity
    async fn delete(tao: &crate::tao_interface::TaoInterface, id: i64) -> Result<()> {
        tao.delete_object(id).await
    }

    // TAO Association helpers
    async fn get_associations(
        tao: &crate::tao_interface::TaoInterface, 
        id: i64, 
        assoc_type: AssociationType
    ) -> Result<Vec<i64>> {
        tao.entity_context().db.get_entity_associations_by_index(id, assoc_type, None).await
    }

    // Batch operations
    async fn create_many(tao: &crate::tao_interface::TaoInterface, entities: &[Self]) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        for entity in entities {
            let id = Self::create(tao, entity).await?;
            ids.push(id);
        }
        Ok(ids)
    }
}

// Database context for entity operations (kept for backward compatibility)
#[derive(Clone)]
pub struct EntityContext {
    pub db: Arc<crate::database::TaoDatabase>,
}

impl EntityContext {
    pub fn new(db: Arc<crate::database::TaoDatabase>) -> Self {
        Self { db }
    }
}

// Re-export entity implementations
pub mod ent_user;
pub mod ent_event;
pub mod ent_post;
pub mod ent_comment;
pub mod ent_group;
pub mod ent_page;

// Re-export the structs with their methods 
pub use crate::models::{EntUser, EntPost, EntComment, EntGroup, EntPage, EntEvent,};