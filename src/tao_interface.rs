// Unified TAO Interface - Meta's TAO Database unified access layer
// This replaces all individual services (UserService, PostService, etc.)

use std::sync::Arc;
use anyhow::Result;
use axum::{
    extract::{Path as AxumPath, Query, State},
    response::Json,
    routing::{get, post, delete},
    Router,
};
use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    database::TaoDatabase,
    entities::EntityContext,
    models::{EntityType, AssociationType, TaoAssociationQuery},
    error::{AppError, AppResult},
    services::EntityViewerService,
    id_generator::TaoIdGenerator,
    inverse_associations::InverseAssociationMap,
};
use sqlx::Row;

// Unified TAO Interface with caching and sharding
#[derive(Clone)]
pub struct TaoInterface {
    entity_ctx: EntityContext,
    id_generator: std::sync::Arc<TaoIdGenerator>,
    inverse_map: InverseAssociationMap,
}

impl TaoInterface {
    pub fn new(db: Arc<TaoDatabase>) -> Self {
        // For now, use shard 0. In production, this would be configurable
        let id_generator = std::sync::Arc::new(TaoIdGenerator::new(0));
        Self {
            entity_ctx: EntityContext::new(db),
            id_generator,
            inverse_map: InverseAssociationMap::new(),
        }
    }
    
    pub fn new_with_shard(db: Arc<TaoDatabase>, shard_id: u16) -> Self {
        let id_generator = std::sync::Arc::new(TaoIdGenerator::new(shard_id));
        Self {
            entity_ctx: EntityContext::new(db),
            id_generator,
            inverse_map: InverseAssociationMap::new(),
        }
    }

    // Accessor for entity context (for services that need direct access)
    pub fn entity_context(&self) -> &EntityContext {
        &self.entity_ctx
    }

    // Accessor for database (for services that need direct database access)
    pub fn database(&self) -> Arc<TaoDatabase> {
        self.entity_ctx.db.clone()
    }

    // Core TAO operations with caching and transaction support
    pub async fn get_object_by_id_and_type(&self, id: i64, entity_type: &str) -> Result<Option<crate::models::tao_core::TaoObject>> {
        // TAO layer: check cache first, then database
        // TODO: Implement cache lookup before DB
        self.entity_ctx.db.get_object_by_id_and_type(id, entity_type).await
    }

    pub async fn get_object(&self, id: i64) -> Result<Option<crate::models::tao_core::TaoObject>> {
        // TAO layer: check cache first, then database  
        // TODO: Implement cache lookup before DB
        self.entity_ctx.db.get_object(id).await
    }

    pub async fn create_object(&self, entity_type: EntityType, data: &[u8]) -> Result<crate::models::tao_core::TaoObject> {
        // TAO layer: generate shard-aware ID, handle transactions, cache invalidation
        let id = self.id_generator.next_id();
        self.entity_ctx.db.create_object_with_id(id, entity_type, data).await
    }

    pub async fn update_object(&self, id: i64, data: &[u8]) -> Result<()> {
        // TAO layer: handle transactions, cache invalidation
        self.entity_ctx.db.update_object(id, data).await
    }

    pub async fn delete_object(&self, id: i64) -> Result<()> {
        // TAO layer: handle transactions, cache invalidation
        self.entity_ctx.db.delete_object(id).await
    }

    pub async fn get_all_objects_by_type(&self, entity_type: &str, limit: Option<i32>) -> Result<Vec<i64>> {
        // TAO layer: handle caching for common queries
        let limit = limit.unwrap_or(100);
        let entity_ids: Vec<i64> = sqlx::query(
            "SELECT id FROM objects WHERE object_type = ? LIMIT ?"
        )
        .bind(entity_type)
        .bind(limit)
        .fetch_all(&self.entity_ctx.db.pool)
        .await?
        .into_iter()
        .map(|row| row.get::<i64, _>(0))
        .collect();
        Ok(entity_ids)
    }

    // JSON response wrappers for HTTP API
    pub async fn create_entity_binary(&self, entity_type: EntityType, data: &[u8]) -> AppResult<Json<Value>> {
        let obj = self.create_object(entity_type, data).await?;
        Ok(Json(json!({"id": obj.id, "entity_type": obj.object_type, "created": obj.created})))
    }

    pub async fn get_entity_binary(&self, id: i64) -> AppResult<Json<Value>> {
        match self.get_object(id).await? {
            Some(obj) => Ok(Json(json!({
                "id": obj.id,
                "entity_type": obj.object_type,
                "data": base64::engine::general_purpose::STANDARD.encode(&obj.data),
                "created": obj.created,
                "updated": obj.updated
            }))),
            None => Err(AppError::NotFound(format!("Entity with id {} not found", id))),
        }
    }

    pub async fn update_entity_binary(&self, id: i64, data: &[u8]) -> AppResult<Json<Value>> {
        self.update_object(id, data).await?;
        Ok(Json(json!({"id": id, "updated": true})))
    }

    pub async fn delete_entity(&self, id: i64) -> AppResult<Json<Value>> {
        self.delete_object(id).await?;
        Ok(Json(json!({"id": id, "deleted": true})))
    }


    // Association operations with automatic inverse handling
    pub async fn create_association(
        &self,
        source_id: i64,
        target_id: i64,
        assoc_type: AssociationType,
        data: Option<&[u8]>,
    ) -> AppResult<Json<Value>> {
        // Create the primary association
        let association = self.entity_ctx.db.create_association(
            source_id,
            target_id,
            assoc_type,
            data,
        ).await?;

        // Create inverse association if one exists and it's not symmetric
        if let Some(inverse_type) = self.inverse_map.get_inverse(&assoc_type) {
            if !self.inverse_map.is_symmetric(&assoc_type) {
                // Create inverse association (target -> source)
                let _inverse_assoc = self.entity_ctx.db.create_association(
                    target_id,
                    source_id,
                    *inverse_type,
                    data,
                ).await?;
            }
        }

        Ok(Json(json!({
            "edge_id": association.edge_id,
            "source_id": association.source_id,
            "target_id": association.target_id,
            "association_type": association.association_type,
            "created": association.created
        })))
    }

    pub async fn get_associations(&self, query: TaoAssociationQuery) -> AppResult<Json<Value>> {
        let associations = self.entity_ctx.db.get_associations(&query).await?;
        let mut assoc_list = Vec::new();
        
        for assoc in associations {
            assoc_list.push(json!({
                "edge_id": assoc.edge_id,
                "source_id": assoc.source_id,
                "target_id": assoc.target_id,
                "association_type": assoc.association_type,
                "created": assoc.created,
                "updated": assoc.updated
            }));
        }
        
        Ok(Json(json!({"associations": assoc_list})))
    }

    pub async fn delete_association(
        &self,
        source_id: i64,
        target_id: i64,
        assoc_type: AssociationType,
    ) -> AppResult<Json<Value>> {
        self.entity_ctx.db.delete_association(source_id, target_id, assoc_type).await?;
        Ok(Json(json!({"deleted": true})))
    }

    // Edge traversal methods for generated entity code
    pub async fn get_outgoing_associations(
        &self,
        source_id: i64,
    ) -> Result<Vec<crate::models::tao_core::TaoAssociation>> {
        self.entity_ctx.db.get_outgoing_associations(source_id).await
    }

    pub async fn get_incoming_associations(
        &self,
        target_id: i64,
    ) -> Result<Vec<crate::models::tao_core::TaoAssociation>> {
        self.entity_ctx.db.get_incoming_associations(target_id).await
    }

    pub async fn get_associations_by_type(
        &self,
        source_id: i64,
        assoc_type: AssociationType,
        limit: Option<i32>,
    ) -> Result<Vec<i64>> {
        self.entity_ctx.db.get_entity_associations_by_index(source_id, assoc_type, limit).await
    }

    pub async fn get_associated_entities<T>(
        &self,
        source_id: i64,
        assoc_type: AssociationType,
        target_entity_type: &str,
        limit: Option<i32>,
    ) -> Result<Vec<(i64, T)>>
    where
        T: crate::entities::TaoEntity + for<'de> serde::Deserialize<'de>,
    {
        // First get target IDs using efficient index queries
        let target_ids = self.get_associations_by_type(source_id, assoc_type, limit).await?;
        let mut results = Vec::new();
        
        for target_id in target_ids {
            if let Some(obj) = self.get_object_by_id_and_type(target_id, target_entity_type).await? {
                match crate::thrift_utils::thrift_deserialize::<T>(&obj.data) {
                    Ok(entity) => results.push((obj.id, entity)),
                    Err(e) => {
                        // Log error but continue
                        eprintln!("Failed to deserialize entity {}: {}", obj.id, e);
                    }
                }
            }
        }
        
        Ok(results)
    }

    pub async fn get_reverse_associated_entities<T>(
        &self,
        target_id: i64,
        assoc_type: AssociationType,
        source_entity_type: &str,
        limit: Option<i32>,
    ) -> Result<Vec<(i64, T)>>
    where
        T: crate::entities::TaoEntity + for<'de> serde::Deserialize<'de>,
    {
        // Get all incoming associations and filter by type
        let associations = self.get_incoming_associations(target_id).await?;
        let mut results = Vec::new();
        let mut count = 0;
        
        for assoc in associations {
            if assoc.association_type == assoc_type.as_str() {
                if let Some(obj) = self.get_object_by_id_and_type(assoc.source_id, source_entity_type).await? {
                    match crate::thrift_utils::thrift_deserialize::<T>(&obj.data) {
                        Ok(entity) => {
                            results.push((obj.id, entity));
                            count += 1;
                            if let Some(lim) = limit {
                                if count >= lim {
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to deserialize entity {}: {}", obj.id, e);
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}

// HTTP Request/Response types
#[derive(Deserialize)]
pub struct CreateAssociationRequest {
    pub source_id: i64,
    pub target_id: i64,
    pub assoc_type: String,
}


#[derive(Deserialize)]
pub struct GetAssociationsQuery {
    pub id1: i64,
    pub id2: Option<i64>,
    pub assoc_type: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i64>,
}

// Helper function to parse association type
fn parse_association_type(type_str: &str) -> AppResult<AssociationType> {
    match type_str.to_lowercase().as_str() {
        "friendship" => Ok(AssociationType::Friendship),
        "follow" => Ok(AssociationType::Follow),
        "like" => Ok(AssociationType::Like),
        "post_author" | "postauthor" => Ok(AssociationType::PostAuthor),
        "followed_by" | "followedby" => Ok(AssociationType::FollowedBy),
        "liked_by" | "likedby" => Ok(AssociationType::LikedBy),
        _ => Err(AppError::Validation(format!("Unknown association type: {}", type_str))),
    }
}

// HTTP Handlers

pub async fn create_association_handler(
    State(tao): State<TaoInterface>,
    Json(req): Json<CreateAssociationRequest>,
) -> Result<Json<Value>, AppError> {
    let assoc_type = parse_association_type(&req.assoc_type)?;
    tao.create_association(req.source_id, req.target_id, assoc_type, None).await
}

pub async fn get_associations_handler(
    State(tao): State<TaoInterface>,
    Query(params): Query<GetAssociationsQuery>,
) -> Result<Json<Value>, AppError> {
    let query = TaoAssociationQuery {
        id1: params.id1,
        id2: params.id2,
        assoc_type: params.assoc_type,
        start_time: params.start_time,
        end_time: params.end_time,
        limit: params.limit,
        offset: params.offset,
    };
    tao.get_associations(query).await
}

pub async fn delete_association_handler(
    State(tao): State<TaoInterface>,
    AxumPath((source_id, target_id, assoc_type)): AxumPath<(i64, i64, String)>,
) -> Result<Json<Value>, AppError> {
    let assoc_type = parse_association_type(&assoc_type)?;
    tao.delete_association(source_id, target_id, assoc_type).await
}

pub async fn get_entity_handler(
    State(tao): State<TaoInterface>,
    AxumPath(id): AxumPath<i64>,
) -> Result<Json<Value>, AppError> {
    tao.get_entity_binary(id).await
}

pub async fn get_entity_viewer_handler(
    State(tao): State<TaoInterface>,
    AxumPath(id): AxumPath<i64>,
) -> Result<Json<Value>, AppError> {
    let viewer_service = EntityViewerService::new(tao.database());
    let result = viewer_service.get_entity_full_details(id).await?;
    Ok(Json(result))
}


pub async fn delete_entity_handler(
    State(tao): State<TaoInterface>,
    AxumPath(id): AxumPath<i64>,
) -> Result<Json<Value>, AppError> {
    tao.delete_entity(id).await
}

// Create unified router
pub fn create_tao_router(tao: TaoInterface) -> Router {
    Router::new()
        
        // Generic entity operations
        .route("/entities/{id}", get(get_entity_handler))
        .route("/entities/{id}", delete(delete_entity_handler))
        
        // Entity viewer operations
        .route("/viewer/entity/{id}", get(get_entity_viewer_handler))
        
        // Association operations
        .route("/associations", post(create_association_handler))
        .route("/associations", get(get_associations_handler))
        .route("/associations/{source_id}/{target_id}/{assoc_type}", delete(delete_association_handler))
        
        .with_state(tao)
}