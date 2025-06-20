use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cache::Cache;
use crate::models::{EntityType, AssociationType};
use crate::models::tao_core::{TaoObject, TaoAssociation, TaoIndex, TaoAssociationQuery};

// Modern async TAO Database with SQLx connection pool
pub struct TaoDatabase {
    pub pool: SqlitePool, // Made public for entity framework access
    object_cache: Arc<Mutex<Cache<i64, TaoObject>>>,
    assoc_cache: Arc<Mutex<Cache<String, Vec<TaoAssociation>>>>,
    index_cache: Arc<Mutex<Cache<String, Vec<TaoIndex>>>>,
    count_cache: Arc<Mutex<Cache<String, i64>>>,
}

impl TaoDatabase {
    pub async fn new(database_url: &str, cache_capacity: usize) -> Result<Self> {
        // Create connection pool with proper configuration
        let pool = SqlitePool::connect(database_url).await?;

        Ok(TaoDatabase {
            pool,
            object_cache: Arc::new(Mutex::new(Cache::new(cache_capacity))),
            assoc_cache: Arc::new(Mutex::new(Cache::new(cache_capacity * 2))),
            index_cache: Arc::new(Mutex::new(Cache::new(cache_capacity))),
            count_cache: Arc::new(Mutex::new(Cache::new(cache_capacity / 2))),
        })
    }

    pub async fn init(&self) -> Result<()> {
        // Create TAO objects table - stores all entities
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS objects (
                id INTEGER PRIMARY KEY,
                object_type TEXT NOT NULL,
                data BLOB NOT NULL,
                created INTEGER NOT NULL,
                updated INTEGER NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        // Create TAO associations table - stores all relations/edges
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS associations (
                edge_id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                target_id INTEGER NOT NULL,
                association_type TEXT NOT NULL,
                association_data BLOB,
                created INTEGER NOT NULL,
                updated INTEGER NOT NULL,
                time_field INTEGER NOT NULL,
                UNIQUE(source_id, target_id, association_type)
            )"
        )
        .execute(&self.pool)
        .await?;

        // Create TAO indexes table - for efficient queries
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS indexes (
                entity_id INTEGER NOT NULL,
                edge_id INTEGER NOT NULL,
                target_entity_id INTEGER NOT NULL,
                association_type TEXT NOT NULL,
                created INTEGER NOT NULL,
                updated INTEGER NOT NULL,
                PRIMARY KEY(entity_id, edge_id, association_type)
            )"
        )
        .execute(&self.pool)
        .await?;

        // Create performance indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_objects_id ON objects(id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_objects_type ON objects(object_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_objects_id_type ON objects(id, object_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_assoc_source_type ON associations(source_id, association_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_assoc_target_type ON associations(target_id, association_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_assoc_time ON associations(time_field)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_indexes_entity_type ON indexes(entity_id, association_type)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create_object(&self, entity_type: EntityType, data: &[u8]) -> Result<TaoObject> {
        let now = Utc::now().timestamp();
        let type_str = entity_type.as_str();

        let result = sqlx::query(
            "INSERT INTO objects (object_type, data, created, updated) VALUES (?, ?, ?, ?)"
        )
        .bind(type_str)
        .bind(data)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();

        let obj = TaoObject {
            id,
            object_type: type_str.to_string(),
            data: data.to_vec(),
            created: now,
            updated: now,
        };

        // Cache the new object
        self.object_cache.lock().await.insert(id, obj.clone());

        Ok(obj)
    }

    // TAO-compliant object creation with specific ID (for sharding)
    pub async fn create_object_with_id(&self, id: i64, entity_type: EntityType, data: &[u8]) -> Result<TaoObject> {
        let now = Utc::now().timestamp();
        let type_str = entity_type.as_str();

        let result = sqlx::query(
            "INSERT INTO objects (id, object_type, data, created, updated) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(type_str)
        .bind(data)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Failed to create object with ID {}", id));
        }

        let obj = TaoObject {
            id,
            object_type: type_str.to_string(),
            data: data.to_vec(),
            created: now,
            updated: now,
        };

        // Cache the new object
        self.object_cache.lock().await.insert(id, obj.clone());

        Ok(obj)
    }

    pub async fn get_object(&self, id: i64) -> Result<Option<TaoObject>> {
        // Check cache first
        {
            let mut cache = self.object_cache.lock().await;
            if let Some(obj) = cache.get(&id).cloned() {
                return Ok(Some(obj));
            }
        }

        // Query database using SQLx
        let row = sqlx::query(
            "SELECT id, object_type, data, created, updated FROM objects WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let obj = TaoObject {
                id: row.get("id"),
                object_type: row.get("object_type"),
                data: row.get("data"),
                created: row.get("created"),
                updated: row.get("updated"),
            };
            self.object_cache.lock().await.insert(id, obj.clone());
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    pub async fn get_object_by_id_and_type(&self, id: i64, object_type: &str) -> Result<Option<TaoObject>> {
        // Check cache first
        {
            let mut cache = self.object_cache.lock().await;
            if let Some(obj) = cache.get(&id).cloned() {
                if obj.object_type == object_type {
                    return Ok(Some(obj));
                }
            }
        }

        // Query database with both id and type for efficiency
        let row = sqlx::query(
            "SELECT id, object_type, data, created, updated FROM objects WHERE id = ? AND object_type = ?"
        )
        .bind(id)
        .bind(object_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let obj = TaoObject {
                id: row.get("id"),
                object_type: row.get("object_type"),
                data: row.get("data"),
                created: row.get("created"),
                updated: row.get("updated"),
            };
            self.object_cache.lock().await.insert(id, obj.clone());
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    pub async fn update_object(&self, id: i64, data: &[u8]) -> Result<()> {
        let now = Utc::now().timestamp();

        sqlx::query("UPDATE objects SET data = ?, updated = ? WHERE id = ?")
            .bind(data)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Invalidate cache
        self.object_cache.lock().await.remove(&id);

        Ok(())
    }

    // TAO-compliant delete: atomic delete with transaction
    pub async fn delete_object(&self, id: i64) -> Result<()> {
        // Start transaction for atomic operations
        let mut tx = self.pool.begin().await?;

        // Delete all associations involving this object
        sqlx::query("DELETE FROM associations WHERE source_id = ? OR target_id = ?")
            .bind(id)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Delete corresponding index entries
        sqlx::query("DELETE FROM indexes WHERE entity_id = ? OR target_entity_id = ?")
            .bind(id)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Delete the object
        sqlx::query("DELETE FROM objects WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Commit transaction - all operations succeed or all fail
        tx.commit().await?;

        // Invalidate caches only after successful commit
        self.object_cache.lock().await.remove(&id);
        self.assoc_cache.lock().await.clear();
        self.index_cache.lock().await.clear();
        self.count_cache.lock().await.clear();

        Ok(())
    }

    // TAO-compliant association creation with atomic index table updates
    pub async fn create_association(
        &self,
        source_id: i64,
        target_id: i64,
        assoc_type: AssociationType,
        data: Option<&[u8]>,
    ) -> Result<TaoAssociation> {
        let now = Utc::now().timestamp();
        let type_str = assoc_type.as_str();

        // Start transaction for atomic operations
        let mut tx = self.pool.begin().await?;

        // Insert into associations table
        let result = sqlx::query(
            "INSERT OR REPLACE INTO associations (source_id, target_id, association_type, association_data, created, updated, time_field)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(source_id)
        .bind(target_id)
        .bind(type_str)
        .bind(data)
        .bind(now)
        .bind(now)
        .bind(now) // time_field for creation-time locality
        .execute(&mut *tx)
        .await?;

        let edge_id = result.last_insert_rowid();

        // Insert into indexes table for efficient queries
        sqlx::query(
            "INSERT OR REPLACE INTO indexes (entity_id, edge_id, target_entity_id, association_type, created, updated)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(source_id)
        .bind(edge_id)
        .bind(target_id)
        .bind(type_str)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Commit transaction
        tx.commit().await?;

        let assoc = TaoAssociation {
            edge_id,
            source_id,
            target_id,
            association_type: type_str.to_string(),
            association_data: data.map(|d| d.to_vec()),
            created: now,
            updated: now,
            time_field: now,
        };

        // Invalidate relevant caches only after successful commit
        let cache_key = format!("{}:{}", source_id, type_str);
        self.assoc_cache.lock().await.remove(&cache_key);
        let count_key = format!("count:{}:{}", source_id, type_str);
        self.count_cache.lock().await.remove(&count_key);
        self.index_cache.lock().await.clear(); // Clear index cache

        Ok(assoc)
    }

    // Raw association query for complex filtering (used by TAO operations)
    pub async fn get_associations_raw(&self, query: &str, source_id: i64, assoc_type: &str) -> Result<Vec<TaoAssociation>> {
        let rows = sqlx::query(query)
            .bind(source_id)
            .bind(assoc_type)
            .fetch_all(&self.pool)
            .await?;

        let mut assocs = Vec::new();
        for row in rows {
            let assoc = TaoAssociation {
                edge_id: row.get("edge_id"),
                source_id: row.get("source_id"),
                target_id: row.get("target_id"),
                association_type: row.get("association_type"),
                association_data: row.get("association_data"),
                created: row.get("created"),
                updated: row.get("updated"),
                time_field: row.get("time_field"),
            };
            assocs.push(assoc);
        }

        Ok(assocs)
    }

    // TAO-compliant association queries using both associations and indexes tables
    pub async fn get_associations(&self, query: &TaoAssociationQuery) -> Result<Vec<TaoAssociation>> {
        let cache_key = format!("{}:{}", query.id1, query.assoc_type);

        // Check cache first for simple queries
        if query.id2.is_none() && query.start_time.is_none() && query.end_time.is_none() {
            let mut cache = self.assoc_cache.lock().await;
            if let Some(assocs) = cache.get(&cache_key).cloned() {
                return Ok(self.apply_limit_offset(assocs, query.limit, query.offset));
            }
        }

        // Query database using SQLx with new field names
        let rows = sqlx::query(
            "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field
             FROM associations WHERE source_id = ? AND association_type = ? ORDER BY time_field DESC"
        )
        .bind(query.id1)
        .bind(&query.assoc_type)
        .fetch_all(&self.pool)
        .await?;

        let mut assocs = Vec::new();
        for row in rows {
            let assoc = TaoAssociation {
                edge_id: row.get("edge_id"),
                source_id: row.get("source_id"),
                target_id: row.get("target_id"),
                association_type: row.get("association_type"),
                association_data: row.get("association_data"),
                created: row.get("created"),
                updated: row.get("updated"),
                time_field: row.get("time_field"),
            };
            assocs.push(assoc);
        }

        // Cache simple queries
        if query.id2.is_none() && query.start_time.is_none() && query.end_time.is_none()
           && query.limit.is_none() && query.offset.is_none() {
            self.assoc_cache.lock().await.insert(cache_key, assocs.clone());
        }

        Ok(assocs)
    }

    // Compatibility method for old AssociationQuery
    pub async fn get_associations_legacy(&self, query: &crate::models::AssociationQuery) -> Result<Vec<TaoAssociation>> {
        let tao_query = TaoAssociationQuery {
            id1: query.id1,
            id2: query.id2,
            assoc_type: query.assoc_type.clone(),
            start_time: query.start_time,
            end_time: query.end_time,
            limit: query.limit,
            offset: query.offset,
        };
        self.get_associations(&tao_query).await
    }

    // TAO-compliant association deletion with atomic index cleanup
    pub async fn delete_association(&self, source_id: i64, target_id: i64, assoc_type: AssociationType) -> Result<()> {
        let type_str = assoc_type.as_str();

        // Start transaction for atomic operations
        let mut tx = self.pool.begin().await?;

        // Get edge_id before deletion for index cleanup
        let edge_row = sqlx::query(
            "SELECT edge_id FROM associations WHERE source_id = ? AND target_id = ? AND association_type = ?"
        )
        .bind(source_id)
        .bind(target_id)
        .bind(type_str)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = edge_row {
            let edge_id: i64 = row.get("edge_id");

            // Delete from indexes table
            sqlx::query("DELETE FROM indexes WHERE edge_id = ?")
                .bind(edge_id)
                .execute(&mut *tx)
                .await?;
        }

        // Delete from associations table
        sqlx::query("DELETE FROM associations WHERE source_id = ? AND target_id = ? AND association_type = ?")
            .bind(source_id)
            .bind(target_id)
            .bind(type_str)
            .execute(&mut *tx)
            .await?;

        // Commit transaction
        tx.commit().await?;

        // Invalidate caches only after successful commit
        let cache_key = format!("{}:{}", source_id, type_str);
        self.assoc_cache.lock().await.remove(&cache_key);
        let count_key = format!("count:{}:{}", source_id, type_str);
        self.count_cache.lock().await.remove(&count_key);
        self.index_cache.lock().await.clear();

        Ok(())
    }

    // TAO-compliant association count using indexes table for efficiency
    pub async fn get_association_count(&self, source_id: i64, assoc_type: AssociationType) -> Result<i64> {
        let type_str = assoc_type.as_str();
        let cache_key = format!("count:{}:{}", source_id, type_str);

        // Check cache first
        {
            let mut cache = self.count_cache.lock().await;
            if let Some(count) = cache.get(&cache_key).cloned() {
                return Ok(count);
            }
        }

        // Use indexes table for faster counting
        let row = sqlx::query("SELECT COUNT(*) FROM indexes WHERE entity_id = ? AND association_type = ?")
            .bind(source_id)
            .bind(type_str)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get(0);

        self.count_cache.lock().await.insert(cache_key, count);
        Ok(count)
    }

    // TAO-specific index queries for efficient relationship lookups
    pub async fn get_entity_associations_by_index(&self, entity_id: i64, assoc_type: AssociationType, limit: Option<i32>) -> Result<Vec<i64>> {
        let type_str = assoc_type.as_str();
        let limit = limit.unwrap_or(100);

        let target_ids: Vec<i64> = sqlx::query(
            "SELECT target_entity_id FROM indexes WHERE entity_id = ? AND association_type = ? ORDER BY created DESC LIMIT ?"
        )
        .bind(entity_id)
        .bind(type_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.get::<i64, _>(0))
        .collect();

        Ok(target_ids)
    }

    // Get all associations where this entity is the source (outgoing edges)
    pub async fn get_outgoing_associations(&self, source_id: i64) -> Result<Vec<TaoAssociation>> {
        let rows = sqlx::query(
            "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field 
             FROM associations WHERE source_id = ? ORDER BY created DESC"
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        let mut associations = Vec::new();
        for row in rows {
            associations.push(TaoAssociation {
                edge_id: row.get("edge_id"),
                source_id: row.get("source_id"),
                target_id: row.get("target_id"),
                association_type: row.get("association_type"),
                association_data: row.get("association_data"),
                created: row.get("created"),
                updated: row.get("updated"),
                time_field: row.get("time_field"),
            });
        }

        Ok(associations)
    }

    // Get all associations where this entity is the target (incoming associations)
    pub async fn get_incoming_associations(&self, target_id: i64) -> Result<Vec<TaoAssociation>> {
        let rows = sqlx::query(
            "SELECT edge_id, source_id, target_id, association_type, association_data, created, updated, time_field 
             FROM associations WHERE target_id = ? ORDER BY created DESC"
        )
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;

        let mut associations = Vec::new();
        for row in rows {
            associations.push(TaoAssociation {
                edge_id: row.get("edge_id"),
                source_id: row.get("source_id"),
                target_id: row.get("target_id"),
                association_type: row.get("association_type"),
                association_data: row.get("association_data"),
                created: row.get("created"),
                updated: row.get("updated"),
                time_field: row.get("time_field"),
            });
        }

        Ok(associations)
    }

    // Begin a transaction - caller is responsible for commit/rollback
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
        Ok(self.pool.begin().await?)
    }

    // Utility methods
    fn apply_limit_offset(&self, mut items: Vec<TaoAssociation>, limit: Option<i32>, offset: Option<i64>) -> Vec<TaoAssociation> {
        if let Some(offset) = offset {
            if offset > 0 {
                let offset_usize = offset as usize;
                if offset_usize < items.len() {
                    items = items.into_iter().skip(offset_usize).collect();
                } else {
                    return Vec::new();
                }
            }
        }

        if let Some(limit) = limit {
            if limit > 0 {
                items.truncate(limit as usize);
            }
        }

        items
    }

}