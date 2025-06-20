// EntityViewerService - High-level service for viewing entities with associations
// This sits above the TAO interface and provides complex business logic

use serde_json::{Value, json};
use std::sync::Arc;
use base64::Engine;

use crate::{
    database::TaoDatabase,
    entities::{EntUser, EntPost},
    error::{AppError, AppResult},
    thrift_utils::thrift_deserialize,
};

#[derive(Clone)]
pub struct EntityViewerService {
    tao: crate::tao_interface::TaoInterface,
}

impl EntityViewerService {
    pub fn new(db: Arc<TaoDatabase>) -> Self {
        Self {
            tao: crate::tao_interface::TaoInterface::new(db),
        }
    }

    /// Get comprehensive entity information including deserialized data and associations
    pub async fn get_entity_full_details(&self, id: i64) -> AppResult<Value> {
        // Get the raw entity object
        let obj = match self.tao.get_object(id).await? {
            Some(obj) => obj,
            None => return Err(AppError::NotFound(format!("Entity with id {} not found", id))),
        };

        // Deserialize entity data based on type
        let entity_data = self.deserialize_entity_data(&obj.object_type, &obj.data)?;

        // Fetch outgoing edges (where this entity is the source)
        let outgoing_edges = self.get_outgoing_edges(id).await?;

        // Fetch incoming associations (where this entity is the target)
        let incoming_associations = self.get_incoming_associations(id).await?;

        // Get entity type counts for stats
        let stats = self.get_entity_stats(id).await?;

        Ok(json!({
            "entity": {
                "id": obj.id,
                "entity_type": obj.object_type,
                "created": obj.created,
                "updated": obj.updated,
                "data": entity_data
            },
            "relationships": {
                "outgoing_edges": outgoing_edges,
                "incoming_associations": incoming_associations,
                "stats": stats
            }
        }))
    }

    /// Deserialize entity data based on type
    fn deserialize_entity_data(&self, entity_type: &str, data: &[u8]) -> AppResult<Value> {
        match entity_type {
            "ent_user" => {
                let user: EntUser = thrift_deserialize(data)?;
                Ok(json!({
                    "username": user.username,
                    "email": user.email,
                    "full_name": user.full_name,
                    "bio": user.bio,
                    "profile_picture_url": user.profile_picture_url,
                    "created_at": user.created_time,
                    "last_active": user.last_active_time,
                    "is_verified": user.is_verified,
                    "location": user.location
                }))
            }
            "ent_post" => {
                let post: EntPost = thrift_deserialize(data)?;
                Ok(json!({
                    "author_id": post.author_id,
                    "content": post.content,
                    "media_url": post.media_url,
                    "created_at": post.created_time,
                    "updated_at": post.updated_time,
                    "post_type": post.post_type,
                    "visibility": post.visibility,
                    "like_count": post.like_count,
                    "comment_count": post.comment_count,
                    "share_count": post.share_count
                }))
            }
            _ => {
                // For unsupported entity types, return raw data as base64
                Ok(json!({
                    "raw_data": base64::engine::general_purpose::STANDARD.encode(data),
                    "note": format!("Entity type '{}' not yet supported for deserialization", entity_type)
                }))
            }
        }
    }

    /// Get outgoing edges where this entity is the source
    async fn get_outgoing_edges(&self, source_id: i64) -> AppResult<Vec<Value>> {
        let associations = self.tao.entity_context().db.get_outgoing_associations(source_id).await?;
        let mut edges = Vec::new();

        for assoc in associations {
            // Try to get basic info about the target entity
            let target_info = match self.tao.get_object(assoc.target_id).await? {
                Some(target_obj) => json!({
                    "entity_type": target_obj.object_type,
                    "created": target_obj.created
                }),
                None => json!({
                    "entity_type": "unknown",
                    "created": null
                })
            };

            edges.push(json!({
                "edge_id": assoc.edge_id,
                "target_id": assoc.target_id,
                "target_info": target_info,
                "association_type": assoc.association_type,
                "created": assoc.created,
                "updated": assoc.updated,
                "relationship": "outgoing_edge"
            }));
        }

        Ok(edges)
    }

    /// Get incoming associations where this entity is the target
    async fn get_incoming_associations(&self, target_id: i64) -> AppResult<Vec<Value>> {
        let associations = self.tao.entity_context().db.get_incoming_associations(target_id).await?;
        let mut incoming = Vec::new();

        for assoc in associations {
            // Try to get basic info about the source entity
            let source_info = match self.tao.get_object(assoc.source_id).await? {
                Some(source_obj) => json!({
                    "entity_type": source_obj.object_type,
                    "created": source_obj.created
                }),
                None => json!({
                    "entity_type": "unknown",
                    "created": null
                })
            };

            incoming.push(json!({
                "edge_id": assoc.edge_id,
                "source_id": assoc.source_id,
                "source_info": source_info,
                "association_type": assoc.association_type,
                "created": assoc.created,
                "updated": assoc.updated,
                "relationship": "incoming_association"
            }));
        }

        Ok(incoming)
    }

    /// Get entity relationship statistics
    async fn get_entity_stats(&self, entity_id: i64) -> AppResult<Value> {
        let outgoing_count = self.tao.entity_context().db.get_outgoing_associations(entity_id).await?.len();
        let incoming_count = self.tao.entity_context().db.get_incoming_associations(entity_id).await?.len();

        Ok(json!({
            "outgoing_edges_count": outgoing_count,
            "incoming_associations_count": incoming_count,
            "total_relationships": outgoing_count + incoming_count
        }))
    }
}