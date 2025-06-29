// Database Interface - Low-level database operations for TAO
// This layer handles direct SQL queries for objects, associations, and indexes

use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use sqlx::postgres::{PgPool, Postgres};
use sqlx::sqlite::Sqlite;
use sqlx::{Column, Row, Transaction, ValueRef}; // Added Sqlite for generic DatabaseTransaction

// Generic database types - framework agnostic
pub type ObjectId = i64;
pub type ObjectType = String;
pub type AssociationType = String;
pub type Timestamp = i64;

/// Generic Object for database storage - framework agnostic
#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub otype: ObjectType,
    pub data: Vec<u8>,
    pub created_time: Timestamp,
    pub updated_time: Timestamp,
    pub version: u64,
}

/// Generic Association for database storage - framework agnostic
#[derive(Debug, Clone)]
pub struct Association {
    pub id1: ObjectId,
    pub atype: AssociationType,
    pub id2: ObjectId,
    pub time: Timestamp,
    pub data: Option<Vec<u8>>,
}

/// Association query parameters - framework agnostic
#[derive(Debug, Clone)]
pub struct AssocQuery {
    pub id1: ObjectId,
    pub atype: AssociationType,
    pub id2_set: Option<Vec<ObjectId>>,
    pub high_time: Option<Timestamp>,
    pub low_time: Option<Timestamp>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// Object query parameters - framework agnostic
#[derive(Debug, Clone)]
pub struct ObjectQuery {
    pub ids: Vec<ObjectId>,
    pub otype: Option<ObjectType>,
    pub limit: Option<u32>,
    pub offset: Option<u64>,
}

/// Association query result with pagination - framework agnostic
#[derive(Debug, Clone)]
pub struct AssocQueryResult {
    pub associations: Vec<Association>,
    pub next_cursor: Option<String>,
}

/// Object query result with pagination - framework agnostic
#[derive(Debug, Clone)]
pub struct ObjectQueryResult {
    pub objects: Vec<Object>,
    pub next_cursor: Option<String>,
}

/// Unified transaction wrapper for database operations
pub enum DatabaseTransaction {
    Postgres(Transaction<'static, Postgres>),
    Sqlite(Transaction<'static, Sqlite>),
}

impl DatabaseTransaction {
    pub fn new_postgres(tx: Transaction<'static, Postgres>) -> Self {
        Self::Postgres(tx)
    }

    pub fn new_sqlite(tx: Transaction<'static, Sqlite>) -> Self {
        Self::Sqlite(tx)
    }

    /// Commit the transaction
    pub async fn commit(self) -> AppResult<()> {
        match self {
            DatabaseTransaction::Postgres(tx) => tx.commit().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to commit postgres transaction: {}", e))
            }),
            DatabaseTransaction::Sqlite(tx) => tx.commit().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to commit sqlite transaction: {}", e))
            }),
        }
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> AppResult<()> {
        match self {
            DatabaseTransaction::Postgres(tx) => tx.rollback().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to rollback postgres transaction: {}", e))
            }),
            DatabaseTransaction::Sqlite(tx) => tx.rollback().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to rollback sqlite transaction: {}", e))
            }),
        }
    }

    /// Get mutable reference to the underlying transaction for PostgreSQL
    pub fn as_postgres_mut(&mut self) -> AppResult<&mut Transaction<'static, Postgres>> {
        match self {
            DatabaseTransaction::Postgres(tx) => Ok(tx),
            DatabaseTransaction::Sqlite(_) => Err(AppError::DatabaseError(
                "Transaction is not PostgreSQL".to_string(),
            )),
        }
    }

    /// Get mutable reference to the underlying transaction for SQLite
    pub fn as_sqlite_mut(&mut self) -> AppResult<&mut Transaction<'static, Sqlite>> {
        match self {
            DatabaseTransaction::Sqlite(tx) => Ok(tx),
            DatabaseTransaction::Postgres(_) => Err(AppError::DatabaseError(
                "Transaction is not SQLite".to_string(),
            )),
        }
    }
}

/// Database interface trait - completely framework agnostic
/// This layer provides generic object and association storage
#[async_trait]
pub trait DatabaseInterface: Send + Sync {
    /// Allow downcasting to concrete database types
    fn as_any(&self) -> &dyn std::any::Any;
    // Transaction management
    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction>;

    // Object operations - Generic object storage
    async fn get_object(&self, id: ObjectId) -> AppResult<Option<Object>>;
    async fn get_objects(&self, query: ObjectQuery) -> AppResult<ObjectQueryResult>;
    async fn create_object(&self, id: ObjectId, otype: ObjectType, data: Vec<u8>) -> AppResult<()>;
    async fn update_object(&self, id: ObjectId, data: Vec<u8>) -> AppResult<()>;
    async fn delete_object(&self, id: ObjectId) -> AppResult<bool>;
    async fn object_exists(&self, id: ObjectId) -> AppResult<bool>;

    // Association operations - Generic association storage
    async fn get_associations(&self, query: AssocQuery) -> AppResult<AssocQueryResult>;
    async fn create_association(&self, assoc: Association) -> AppResult<()>;
    async fn delete_association(
        &self,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool>;
    async fn association_exists(
        &self,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool>;
    async fn count_associations(&self, id1: ObjectId, atype: AssociationType) -> AppResult<u64>;

    // Index operations - Generic association counting
    async fn update_association_count(
        &self,
        id: ObjectId,
        atype: AssociationType,
        delta: i64,
    ) -> AppResult<()>;
    async fn get_association_count(&self, id: ObjectId, atype: AssociationType) -> AppResult<u64>;

    // Transactional operations - Execute within existing transaction
    async fn create_object_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: ObjectId,
        otype: ObjectType,
        data: Vec<u8>,
    ) -> AppResult<()>;
    async fn create_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        assoc: Association,
    ) -> AppResult<()>;
    async fn delete_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id1: ObjectId,
        atype: AssociationType,
        id2: ObjectId,
    ) -> AppResult<bool>;
    async fn update_association_count_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: ObjectId,
        atype: AssociationType,
        delta: i64,
    ) -> AppResult<()>;

    /// Execute a raw SQL query and return results as a vector of hashmaps
    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>>;

    // Graph visualization methods
    /// Get all objects from this shard for graph visualization
    async fn get_all_objects_from_shard(&self) -> AppResult<Vec<Object>>;
    /// Get all associations from this shard for graph visualization
    async fn get_all_associations_from_shard(&self) -> AppResult<Vec<Association>>;
}

/// PostgreSQL implementation of database interface
pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Health check to verify database connectivity
    pub async fn health_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Database health check failed: {}", e)))?;
        Ok(())
    }

    /// Get connection pool statistics
    pub fn pool_stats(&self) -> (u32, u32) {
        (self.pool.num_idle() as u32, self.pool.size() as u32)
    }

    /// Initialize TAO database tables with date partitioning and ID sharding
    pub async fn initialize(&self) -> AppResult<()> {
        sqlx::query("DROP TABLE IF EXISTS objects CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to drop objects table: {}", e)))?;
        sqlx::query("DROP TABLE IF EXISTS associations CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to drop associations table: {}", e))
            })?;
        sqlx::query("DROP TABLE IF EXISTS association_counts CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to drop association counts table: {}", e))
            })?;

        // Create objects table partitioned by date (time_created)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                id BIGINT NOT NULL,
                otype VARCHAR(64) NOT NULL,
                time_created BIGINT NOT NULL,
                time_updated BIGINT NOT NULL,
                data BYTEA,
                version INTEGER DEFAULT 1,
                PRIMARY KEY (id, time_created)
            ) PARTITION BY RANGE (time_created)
        "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create objects table: {}", e)))?;

        // Create associations table partitioned by date (time_created)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS associations (
                id1 BIGINT NOT NULL,
                atype VARCHAR(64) NOT NULL,
                id2 BIGINT NOT NULL,
                time_created BIGINT NOT NULL,
                data BYTEA,
                PRIMARY KEY (id1, atype, id2, time_created)
            ) PARTITION BY RANGE (time_created)
        "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create associations table: {}", e))
        })?;

        // Create association count index table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS association_counts (
                id BIGINT NOT NULL,
                atype VARCHAR(64) NOT NULL,
                count BIGINT DEFAULT 0,
                updated_time BIGINT NOT NULL,
                PRIMARY KEY (id, atype)
            )
        "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create association counts table: {}", e))
        })?;

        // Create monthly partitions for current and next 12 months
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let current_month_start =
            (current_time / (30 * 24 * 60 * 60 * 1000)) * (30 * 24 * 60 * 60 * 1000); // Rough monthly boundaries

        for i in 0..13 {
            // Current month + 12 future months
            let month_start = current_month_start + (i * 30 * 24 * 60 * 60 * 1000);
            let month_end = month_start + (30 * 24 * 60 * 60 * 1000);

            // Objects partitions
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS objects_m{} PARTITION OF objects FOR VALUES FROM ({}) TO ({})",
                i, month_start, month_end
            ))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create objects monthly partition {}: {}", i, e)))?;

            // Associations partitions
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS associations_m{} PARTITION OF associations FOR VALUES FROM ({}) TO ({})",
                i, month_start, month_end
            ))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations monthly partition {}: {}", i, e)))?;
        }

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_objects_otype ON objects(otype)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create objects otype index: {}", e))
            })?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id1_atype ON associations(id1, atype, time_created)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id2_atype ON associations(id2, atype, time_created)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create reverse associations index: {}", e)))?;

        println!("✅ TAO database tables initialized with date partitioning (monthly)");
        Ok(())
    }
}

#[async_trait]
impl DatabaseInterface for PostgresDatabase {
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
                    // Simplified approach - for now, just return a placeholder
                    // In production, you'd want proper type handling based on column type
                    "<value>".to_string()
                };
                row_map.insert(col_name, value_str);
            }
            results.push(row_map);
        }
        Ok(results)
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;
        Ok(DatabaseTransaction::new_postgres(tx))
    }

    async fn get_object(&self, id: ObjectId) -> AppResult<Option<Object>> {
        let row = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data FROM objects WHERE id = $1",
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
                version: row.try_get::<i32, _>("version").unwrap_or(1) as u64,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_objects(&self, query: ObjectQuery) -> AppResult<ObjectQueryResult> {
        let mut sql =
            "SELECT id, otype, time_created, time_updated, data FROM objects WHERE id = ANY($1)"
                .to_string();

        if query.otype.is_some() {
            sql.push_str(" AND otype = $2");
        }

        sql.push_str(" ORDER BY id");

        let mut query_builder = sqlx::query(&sql).bind(&query.ids);

        if let Some(ref otype) = query.otype {
            query_builder = query_builder.bind(otype);
        }

        let rows = query_builder
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
                version: row.try_get::<i32, _>("version").unwrap_or(1) as u64,
            })
            .collect();

        Ok(ObjectQueryResult {
            objects,
            next_cursor: None, // TODO: Implement pagination
        })
    }

    async fn create_object(&self, id: ObjectId, otype: ObjectType, data: Vec<u8>) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            "INSERT INTO objects (id, otype, time_created, time_updated, data) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(&otype)
        .bind(now)
        .bind(now)
        .bind(&data)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create object with ID {}: {}", id, e)))?;

        Ok(())
    }

    async fn update_object(&self, id: ObjectId, data: Vec<u8>) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let result = sqlx::query(
            "UPDATE objects SET data = $1, time_updated = $2, version = version + 1 WHERE id = $3",
        )
        .bind(&data)
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
        let result = sqlx::query("DELETE FROM objects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete object {}: {}", id, e))
            })?;

        Ok(result.rows_affected() > 0)
    }

    async fn object_exists(&self, id: ObjectId) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM objects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to check association existence: {}", e))
            })?;

        Ok(row.is_some())
    }

    async fn get_associations(&self, query: AssocQuery) -> AppResult<AssocQueryResult> {
        let mut sql = "SELECT id1, atype, id2, time_created, data FROM associations WHERE id1 = $1 AND atype = $2".to_string();
        let mut param_index = 2;

        // Add id2_set clause if present
        if let Some(ref _id2_set) = query.id2_set {
            param_index += 1;
            sql.push_str(&format!(" AND id2 = ANY(${})", param_index));
        }

        if query.low_time.is_some() {
            param_index += 1;
            sql.push_str(&format!(" AND time_created >= ${}", param_index));
        }

        if query.high_time.is_some() {
            param_index += 1;
            sql.push_str(&format!(" AND time_created <= ${}", param_index));
        }

        sql.push_str(" ORDER BY time_created DESC");

        if query.limit.is_some() {
            param_index += 1;
            sql.push_str(&format!(" LIMIT ${}", param_index));
        }

        if query.offset.is_some() {
            param_index += 1;
            sql.push_str(&format!(" OFFSET ${}", param_index));
        }

        let mut query_builder = sqlx::query(&sql).bind(query.id1).bind(&query.atype);

        // Bind id2_set if present
        if let Some(ref id2_set) = query.id2_set {
            query_builder = query_builder.bind(id2_set);
        }
        if let Some(low_time) = query.low_time {
            query_builder = query_builder.bind(low_time);
        }
        if let Some(high_time) = query.high_time {
            query_builder = query_builder.bind(high_time);
        }
        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit as i64);
        }
        if let Some(offset) = query.offset {
            query_builder = query_builder.bind(offset as i64);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to get associations: {}", e)))?;

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
            next_cursor: None, // TODO: Implement pagination cursors
        })
    }

    async fn create_association(&self, assoc: Association) -> AppResult<()> {
        // Insert association
        sqlx::query(
            "INSERT INTO associations (id1, atype, id2, time_created, data) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
        )
        .bind(assoc.id1)
        .bind(&assoc.atype)
        .bind(assoc.id2)
        .bind(assoc.time)
        .bind(&assoc.data)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association: {}", e)))?;

        // Update association count
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
            sqlx::query("DELETE FROM associations WHERE id1 = $1 AND atype = $2 AND id2 = $3")
                .bind(id1)
                .bind(&atype)
                .bind(id2)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to delete association: {}", e))
                })?;

        if result.rows_affected() > 0 {
            // Update association count
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
            sqlx::query("SELECT 1 FROM associations WHERE id1 = $1 AND atype = $2 AND id2 = $3")
                .bind(id1)
                .bind(&atype)
                .bind(id2)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check association existence: {}", e))
                })?;

        Ok(row.is_some())
    }

    async fn count_associations(&self, id1: ObjectId, atype: AssociationType) -> AppResult<u64> {
        // Rely on the pre-calculated index table for performance
        self.get_association_count(id1, atype).await
    }

    async fn update_association_count(
        &self,
        id: ObjectId,
        atype: AssociationType,
        delta: i64,
    ) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            "INSERT INTO association_counts (id, atype, count, updated_time) VALUES ($1, $2, $3, $4)
             ON CONFLICT (id, atype) DO UPDATE SET count = association_counts.count + $3, updated_time = $4"
        )
        .bind(id)
        .bind(&atype)
        .bind(delta)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update association count: {}", e)))?;

        Ok(())
    }

    async fn get_association_count(&self, id: ObjectId, atype: AssociationType) -> AppResult<u64> {
        let row = sqlx::query("SELECT count FROM association_counts WHERE id = $1 AND atype = $2")
            .bind(id)
            .bind(&atype)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to get association count: {}", e))
            })?;

        if let Some(row) = row {
            let count: i64 = row.get("count");
            Ok(count as u64)
        } else {
            Ok(0)
        }
    }

    // Transactional operations - Execute within existing transaction
    async fn create_object_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: ObjectId,
        otype: ObjectType,
        data: Vec<u8>,
    ) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let postgres_tx = tx.as_postgres_mut()?;

        sqlx::query(
            "INSERT INTO objects (id, otype, time_created, time_updated, data) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(&otype)
        .bind(now)
        .bind(now)
        .bind(&data)
        .execute(&mut **postgres_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create object with ID {} in transaction: {}", id, e)))?;

        Ok(())
    }

    async fn create_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        assoc: Association,
    ) -> AppResult<()> {
        let postgres_tx = tx.as_postgres_mut()?;

        // Insert association
        sqlx::query(
            "INSERT INTO associations (id1, atype, id2, time_created, data) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
        )
        .bind(assoc.id1)
        .bind(&assoc.atype)
        .bind(assoc.id2)
        .bind(assoc.time)
        .bind(&assoc.data)
        .execute(&mut **postgres_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association in transaction: {}", e)))?;

        // Update association count
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
        let postgres_tx = tx.as_postgres_mut()?;

        let result =
            sqlx::query("DELETE FROM associations WHERE id1 = $1 AND atype = $2 AND id2 = $3")
                .bind(id1)
                .bind(&atype)
                .bind(id2)
                .execute(&mut **postgres_tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to delete association in transaction: {}",
                        e
                    ))
                })?;

        if result.rows_affected() > 0 {
            // Update association count
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let postgres_tx = tx.as_postgres_mut()?;

        sqlx::query(
            "INSERT INTO association_counts (id, atype, count, updated_time) VALUES ($1, $2, $3, $4)
             ON CONFLICT (id, atype) DO UPDATE SET count = association_counts.count + $3, updated_time = $4"
        )
        .bind(id)
        .bind(&atype)
        .bind(delta)
        .bind(now)
        .execute(&mut **postgres_tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update association count in transaction: {}", e)))?;

        Ok(())
    }

    async fn get_all_objects_from_shard(&self) -> AppResult<Vec<Object>> {
        let rows = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data, version FROM objects ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to get all objects from shard: {}", e))
        })?;

        let objects = rows
            .into_iter()
            .map(|row| Object {
                id: row.get("id"),
                otype: row.get("otype"),
                data: row.get("data"),
                created_time: row.get("time_created"),
                updated_time: row.get("time_updated"),
                version: row.try_get::<i32, _>("version").unwrap_or(1) as u64,
            })
            .collect();

        Ok(objects)
    }

    async fn get_all_associations_from_shard(&self) -> AppResult<Vec<Association>> {
        let rows = sqlx::query(
            "SELECT id1, atype, id2, time_created, data FROM associations ORDER BY id1, atype, id2",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to get all associations from shard: {}", e))
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

        Ok(associations)
    }
}
