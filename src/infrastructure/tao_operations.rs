// TAO Operations - Core TAO API operations following Meta's patterns
// Implements the essential TAO query patterns with proper semantics

use anyhow::Result;
use crate::{
    models::{AssociationType, tao_core::{TaoObject, TaoAssociation}},
    tao_interface::TaoInterface,
};

impl TaoInterface {
    /// TAO assoc_get - Get associations with optional range filtering
    /// Equivalent to Meta's assoc_get(id1, atype, id2_set, high?, low?)
    pub async fn assoc_get(
        &self,
        source_id: i64,
        assoc_type: AssociationType,
        target_ids: Option<Vec<i64>>,
        high_time: Option<i64>,
        low_time: Option<i64>,
    ) -> Result<Vec<TaoAssociation>> {
        let mut query = String::from(
            "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field 
             FROM associations WHERE source_id = ? AND association_type = ?"
        );
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + '_>> = vec![];
        params.push(Box::new(source_id));
        params.push(Box::new(assoc_type.as_str()));

        // Add target_id filtering if specified
        if let Some(targets) = &target_ids {
            if !targets.is_empty() {
                let placeholders = targets.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                query.push_str(&format!(" AND target_id IN ({})", placeholders));
                for target_id in targets {
                    params.push(Box::new(*target_id));
                }
            }
        }

        // Add time range filtering
        if let Some(high) = high_time {
            query.push_str(" AND time_field <= ?");
            params.push(Box::new(high));
        }
        if let Some(low) = low_time {
            query.push_str(" AND time_field >= ?");
            params.push(Box::new(low));
        }

        // Order by time_field for chronological ordering (TAO's creation time locality)
        query.push_str(" ORDER BY time_field DESC");

        // Execute query and collect results
        // Note: This is a simplified version - production would use proper parameter binding
        let associations = self.database().get_associations_raw(&query, source_id, assoc_type.as_str()).await?;
        Ok(associations)
    }

    /// TAO assoc_count - Count associations of given type
    /// Equivalent to Meta's assoc_count(id1, atype)
    pub async fn assoc_count(&self, source_id: i64, assoc_type: AssociationType) -> Result<i64> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM associations WHERE source_id = ? AND association_type = ?"
        )
        .bind(source_id)
        .bind(assoc_type.as_str())
        .fetch_one(&self.database().pool)
        .await?;

        Ok(count)
    }

    /// TAO assoc_range - Get paginated associations
    /// Equivalent to Meta's assoc_range(id1, atype, pos, limit)
    pub async fn assoc_range(
        &self,
        source_id: i64,
        assoc_type: AssociationType,
        offset: i64,
        limit: i32,
    ) -> Result<Vec<TaoAssociation>> {
        let associations = sqlx::query_as::<_, (i64, i64, i64, String, Option<Vec<u8>>, i64, i64, i64)>(
            "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field 
             FROM associations 
             WHERE source_id = ? AND association_type = ? 
             ORDER BY time_field DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(source_id)
        .bind(assoc_type.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.database().pool)
        .await?
        .into_iter()
        .map(|(edge_id, source_id, target_id, association_type, association_data, created, updated, time_field)| {
            TaoAssociation {
                edge_id,
                source_id,
                target_id,
                association_type,
                association_data,
                created,
                updated,
                time_field,
            }
        })
        .collect();

        Ok(associations)
    }

    /// TAO assoc_time_range - Get associations within time range
    /// Equivalent to Meta's assoc_time_range(id1, atype, high_time, low_time)
    pub async fn assoc_time_range(
        &self,
        source_id: i64,
        assoc_type: AssociationType,
        high_time: i64,
        low_time: i64,
        limit: Option<i32>,
    ) -> Result<Vec<TaoAssociation>> {
        let limit_clause = if let Some(l) = limit {
            format!(" LIMIT {}", l)
        } else {
            String::new()
        };

        let associations = sqlx::query_as::<_, (i64, i64, i64, String, Option<Vec<u8>>, i64, i64, i64)>(
            &format!(
                "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field 
                 FROM associations 
                 WHERE source_id = ? AND association_type = ? AND time_field BETWEEN ? AND ? 
                 ORDER BY time_field DESC{}", 
                limit_clause
            )
        )
        .bind(source_id)
        .bind(assoc_type.as_str())
        .bind(low_time)
        .bind(high_time)
        .fetch_all(&self.database().pool)
        .await?
        .into_iter()
        .map(|(edge_id, source_id, target_id, association_type, association_data, created, updated, time_field)| {
            TaoAssociation {
                edge_id,
                source_id,
                target_id,
                association_type,
                association_data,
                created,
                updated,
                time_field,
            }
        })
        .collect();

        Ok(associations)
    }

    /// TAO obj_update_or_create - Upsert semantics for objects
    pub async fn obj_update_or_create(
        &self,
        id: Option<i64>,
        entity_type: crate::models::EntityType,
        data: &[u8],
    ) -> Result<TaoObject> {
        if let Some(existing_id) = id {
            // Try to update existing object
            if let Some(_) = self.get_object(existing_id).await? {
                self.update_object(existing_id, data).await?;
                // Return updated object
                return Ok(self.get_object(existing_id).await?.unwrap());
            }
        }

        // Create new object
        self.create_object(entity_type, data).await
    }

    /// Get multiple objects efficiently (batch operation)
    pub async fn get_objects_batch(&self, ids: Vec<i64>) -> Result<Vec<Option<TaoObject>>> {
        let mut results = Vec::with_capacity(ids.len());
        
        // For now, sequential fetching. Production would use parallel fetching
        for id in ids {
            let obj = self.get_object(id).await?;
            results.push(obj);
        }
        
        Ok(results)
    }
}