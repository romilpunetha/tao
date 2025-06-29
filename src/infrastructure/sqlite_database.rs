use async_trait::async_trait;
use sqlx::{sqlite::Sqlite, sqlite::SqlitePool, Column, QueryBuilder, Row, ValueRef};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::infrastructure::database::{
    AssocQuery, AssocQueryResult, Association, AssociationType, DatabaseInterface,
    DatabaseTransaction, Object, ObjectId, ObjectQuery, ObjectQueryResult, ObjectType, Timestamp,
};

/// SQLite implementation of database interface for in-memory testing
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new_in_memory() -> AppResult<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await.map_err(|e| {
            AppError::DatabaseError(format!("Failed to connect to in-memory SQLite: {}", e))
        })?;

        let db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }

    /// Initialize TAO database tables for SQLite
    pub async fn initialize(&self) -> AppResult<()> {
        sqlx::query("DROP TABLE IF EXISTS tao_objects")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS tao_associations")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS tao_association_counts")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query(
            r#"
            CREATE TABLE tao_objects (
                id INTEGER PRIMARY KEY,
                otype TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL,
                data BLOB,
                version INTEGER DEFAULT 1
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create objects table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE tao_associations (
                id1 INTEGER NOT NULL,
                atype TEXT NOT NULL,
                id2 INTEGER NOT NULL,
                time_created INTEGER NOT NULL,
                data BLOB,
                PRIMARY KEY (id1, atype, id2)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create associations table: {}", e))
        })?;

        sqlx::query(
            r#"
            CREATE TABLE tao_association_counts (
                id INTEGER NOT NULL,
                atype TEXT NOT NULL,
                count INTEGER DEFAULT 0,
                updated_time INTEGER NOT NULL,
                PRIMARY KEY (id, atype)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create association counts table: {}", e))
        })?;

        sqlx::query("CREATE INDEX idx_tao_objects_otype ON tao_objects(otype)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create objects otype index: {}", e))
            })?;

        sqlx::query("CREATE INDEX idx_tao_assoc_id1_atype ON tao_associations(id1, atype, time_created DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations index: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl DatabaseInterface for SqliteDatabase {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;
        Ok(DatabaseTransaction::new_sqlite(tx))
    }

    async fn get_object(&self, id: ObjectId) -> AppResult<Option<Object>> {
        let row = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data, version FROM tao_objects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get object {}: {}", id, e)))?;

        if let Some(row) = row {
            Ok(Some(Object {
                id: row.get("id"),
                otype: row.get("otype"),
                data: row.get("data"),
                created_time: row.get("time_created"),
                updated_time: row.get("time_updated"),
                version: row.get::<i64, _>("version") as u64, // Cast to u64
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_objects(&self, query: ObjectQuery) -> AppResult<ObjectQueryResult> {
        let mut qb = QueryBuilder::<Sqlite>::new(
            "SELECT id, otype, time_created, time_updated, data, version FROM tao_objects WHERE id IN ("
        );
        let mut separated = qb.separated(",");
        for id in query.ids {
            separated.push_bind(id);
        }
        qb.push(")");

        if query.otype.is_some() {
            qb.push(" AND otype = ");
            qb.push_bind(query.otype);
        }

        qb.push(" ORDER BY id");

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to get objects: {}", e)))?;

        let objects = rows
            .into_iter()
            .map(|row| Object {
                id: row.get("id"),
                otype: row.get("otype"),
                data: row.get("data"),
                created_time: row.get("time_created"),
                updated_time: row.get("time_updated"),
                version: row.get::<i64, _>("version") as u64, // Cast to u64
            })
            .collect();

        Ok(ObjectQueryResult {
            objects,
            next_cursor: None,
        })
    }

    async fn create_object(&self, id: ObjectId, otype: ObjectType, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        sqlx::query(
            "INSERT INTO tao_objects (id, otype, time_created, time_updated, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(otype)
        .bind(now)
        .bind(now)
        .bind(data)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create object with ID {}: {}", id, e)))?;
        Ok(())
    }

    async fn update_object(&self, id: ObjectId, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        let result = sqlx::query(
            "UPDATE tao_objects SET data = ?, time_updated = ?, version = version + 1 WHERE id = ?",
        )
        .bind(data)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update object {}: {}", id, e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Object {} not found", id)));
        }
        Ok(())
    }

    async fn delete_object(&self, id: ObjectId) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM tao_objects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete object {}: {}", id, e))
            })?;
        Ok(result.rows_affected() > 0)
    }

    async fn object_exists(&self, id: ObjectId) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM tao_objects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to check if object {} exists: {}", id, e))
            })?;
        Ok(row.is_some())
    }

    async fn get_associations(&self, query: AssocQuery) -> AppResult<AssocQueryResult> {
        let mut qb = QueryBuilder::<Sqlite>::new(
            "SELECT id1, atype, id2, time_created, data FROM tao_associations WHERE id1 = ",
        );
        qb.push_bind(query.id1);
        qb.push(" AND atype = ");
        qb.push_bind(query.atype.clone());

        if let Some(id2_set) = query.id2_set {
            qb.push(" AND id2 IN (");
            let mut separated = qb.separated(",");
            for id2 in id2_set {
                separated.push_bind(id2);
            }
            qb.push(")");
        }
        if let Some(low_time) = query.low_time {
            qb.push(" AND time_created >= ");
            qb.push_bind(low_time);
        }
        if let Some(high_time) = query.high_time {
            qb.push(" AND time_created <= ");
            qb.push_bind(high_time);
        }

        qb.push(" ORDER BY time_created DESC");

        if let Some(limit) = query.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }
        if let Some(offset) = query.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows =
            qb.build().fetch_all(&self.pool).await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to get associations: {}", e))
            })?;

        let associations = rows
            .into_iter()
            .map(|row| Association {
                id1: row.get("id1"),
                atype: row.get("atype"),
                id2: row.get("id2"),
                time: row.get("time_created"),
                data: row.get("data"),
            })
            .collect();

        Ok(AssocQueryResult {
            associations,
            next_cursor: None,
        })
    }

    async fn create_association(&self, assoc: Association) -> AppResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO tao_associations (id1, atype, id2, time_created, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(assoc.id1)
        .bind(assoc.atype.clone())
        .bind(assoc.id2)
        .bind(assoc.time)
        .bind(assoc.data)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association: {}", e)))?;

        self.update_association_count(assoc.id1, assoc.atype, 1)
            .await?;
        Ok(())
    }

    async fn delete_association(
        &self,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool> {
        let result =
            sqlx::query("DELETE FROM tao_associations WHERE id1 = ? AND atype = ? AND id2 = ?")
                .bind(id1)
                .bind(atype.clone())
                .bind(id2)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to delete association: {}", e))
                })?;

        if result.rows_affected() > 0 {
            self.update_association_count(id1, atype, -1).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn association_exists(
        &self,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool> {
        let row =
            sqlx::query("SELECT 1 FROM tao_associations WHERE id1 = ? AND atype = ? AND id2 = ?")
                .bind(id1)
                .bind(atype)
                .bind(id2)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check association existence: {}", e))
                })?;
        Ok(row.is_some())
    }

    async fn count_associations(&self, id1: ObjectId, atype: AssociationType) -> AppResult<u64> {
        self.get_association_count(id1, atype).await
    }

    async fn update_association_count(
        &self,
        id: ObjectId,
        atype: AssociationType,
        delta: i64,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        sqlx::query(
            "INSERT OR REPLACE INTO tao_association_counts (id, atype, count, updated_time) VALUES (?, ?, COALESCE((SELECT count FROM tao_association_counts WHERE id = ? AND atype = ?), 0) + ?, ?)",
        )
        .bind(id)
        .bind(atype.clone())
        .bind(id)
        .bind(atype)
        .bind(delta)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update association count: {}", e)))?;
        Ok(())
    }

    async fn get_association_count(&self, id: ObjectId, atype: AssociationType) -> AppResult<u64> {
        let row =
            sqlx::query("SELECT count FROM tao_association_counts WHERE id = ? AND atype = ?")
                .bind(id)
                .bind(atype)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to get association count: {}", e))
                })?;
        Ok(row.map_or(0, |r| r.get::<i64, _>("count") as u64)) // Cast to u64
    }

    async fn create_object_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: ObjectId,
        otype: ObjectType,
        data: Vec<u8>,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        let sqlite_tx = tx.as_sqlite_mut()?;

        sqlx::query(
            "INSERT INTO tao_objects (id, otype, time_created, time_updated, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(otype)
        .bind(now)
        .bind(now)
        .bind(data)
        .execute(&mut **sqlite_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create object with ID {} in transaction: {}", id, e)))?;
        Ok(())
    }

    async fn create_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        assoc: Association,
    ) -> AppResult<()> {
        let sqlite_tx = tx.as_sqlite_mut()?;

        sqlx::query(
            "INSERT OR IGNORE INTO tao_associations (id1, atype, id2, time_created, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(assoc.id1)
        .bind(assoc.atype.clone())
        .bind(assoc.id2)
        .bind(assoc.time)
        .bind(assoc.data)
        .execute(&mut **sqlite_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association in transaction: {}", e)))?;

        self.update_association_count_tx(tx, assoc.id1, assoc.atype, 1)
            .await?;
        Ok(())
    }

    async fn delete_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool> {
        let sqlite_tx = tx.as_sqlite_mut()?;

        let result =
            sqlx::query("DELETE FROM tao_associations WHERE id1 = ? AND atype = ? AND id2 = ?")
                .bind(id1)
                .bind(atype.clone())
                .bind(id2)
                .execute(&mut **sqlite_tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to delete association in transaction: {}",
                        e
                    ))
                })?;

        if result.rows_affected() > 0 {
            self.update_association_count_tx(tx, id1, atype, -1).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_association_count_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: ObjectId,
        atype: AssociationType,
        delta: i64,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        let sqlite_tx = tx.as_sqlite_mut()?;

        sqlx::query(
            "INSERT OR REPLACE INTO tao_association_counts (id, atype, count, updated_time) VALUES (?, ?, COALESCE((SELECT count FROM tao_association_counts WHERE id = ? AND atype = ?), 0) + ?, ?)",
        )
        .bind(id)
        .bind(atype.clone())
        .bind(id)
        .bind(atype)
        .bind(delta)
        .bind(now)
        .execute(&mut **sqlite_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update association count in transaction: {}", e)))?;
        Ok(())
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let mut row_map = HashMap::new();
            for column in row.columns() {
                let col_name = column.name().to_string();
                let value_ref = row.try_get_raw(column.ordinal()).map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to get raw value for column {}: {}",
                        col_name, e
                    ))
                })?;

                let value_str = if value_ref.is_null() {
                    "NULL".to_string()
                } else {
                    // For SQLite, we'll use a simplified approach
                    "<value>".to_string()
                };
                row_map.insert(col_name, value_str);
            }
            results.push(row_map);
        }
        Ok(results)
    }

    async fn get_all_objects_from_shard(&self) -> AppResult<Vec<Object>> {
        let rows = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data, version FROM tao_objects ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get all objects from shard: {}", e)))?;

        let objects = rows
            .into_iter()
            .map(|row| Object {
                id: row.get("id"),
                otype: row.get("otype"),
                data: row.get("data"),
                created_time: row.get("time_created"),
                updated_time: row.get("time_updated"),
                version: row.get::<i64, _>("version") as u64, // Cast to u64
            })
            .collect();

        Ok(objects)
    }

    async fn get_all_associations_from_shard(&self) -> AppResult<Vec<Association>> {
        let rows = sqlx::query(
            "SELECT id1, atype, id2, time_created, data FROM tao_associations ORDER BY id1, atype, id2"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get all associations from shard: {}", e)))?;

        let associations = rows
            .into_iter()
            .map(|row| Association {
                id1: row.get("id1"),
                atype: row.get("atype"),
                id2: row.get("id2"),
                time: row.get("time_created"),
                data: row.get("data"),
            })
            .collect();

        Ok(associations)
    }
}
