// Database Interface - Low-level database operations for TAO
// This layer handles direct SQL queries for objects, associations, and indexes

use crate::error::{AppError, AppResult};
use crate::infrastructure::tao_core::{
    AssocQuery, AssocType, ObjectQuery, TaoAssociation, TaoId, TaoObject, TaoType,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;

use sqlx::{Column, Row, Transaction, ValueRef};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::sqlite::Sqlite; // Added Sqlite for generic DatabaseTransaction

/// Association query result with pagination
#[derive(Debug, Clone)]
pub struct TaoAssocQueryResult {
    pub associations: Vec<TaoAssociation>,
    pub next_cursor: Option<String>,
}

/// Object query result with pagination
#[derive(Debug, Clone)]
pub struct TaoObjectQueryResult {
    pub objects: Vec<TaoObject>,
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
            DatabaseTransaction::Postgres(tx) => {
                tx.commit()
                    .await
                    .map_err(|e| AppError::DatabaseError(format!("Failed to commit postgres transaction: {}", e)))
            }
            DatabaseTransaction::Sqlite(tx) => {
                tx.commit()
                    .await
                    .map_err(|e| AppError::DatabaseError(format!("Failed to commit sqlite transaction: {}", e)))
            }
        }
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> AppResult<()> {
        match self {
            DatabaseTransaction::Postgres(tx) => {
                tx.rollback()
                    .await
                    .map_err(|e| AppError::DatabaseError(format!("Failed to rollback postgres transaction: {}", e)))
            }
            DatabaseTransaction::Sqlite(tx) => {
                tx.rollback()
                    .await
                    .map_err(|e| AppError::DatabaseError(format!("Failed to rollback sqlite transaction: {}", e)))
            }
        }
    }

    /// Get mutable reference to the underlying transaction for PostgreSQL
    pub fn as_postgres_mut(&mut self) -> AppResult<&mut Transaction<'static, Postgres>> {
        match self {
            DatabaseTransaction::Postgres(tx) => Ok(tx),
            DatabaseTransaction::Sqlite(_) => Err(AppError::DatabaseError("Transaction is not PostgreSQL".to_string())),
        }
    }

    /// Get mutable reference to the underlying transaction for SQLite
    pub fn as_sqlite_mut(&mut self) -> AppResult<&mut Transaction<'static, Sqlite>> {
        match self {
            DatabaseTransaction::Sqlite(tx) => Ok(tx),
            DatabaseTransaction::Postgres(_) => Err(AppError::DatabaseError("Transaction is not SQLite".to_string())),
        }
    }
}

/// Database interface trait for TAO operations
/// This layer converts TAO operations directly into SQL queries
#[async_trait]
pub trait DatabaseInterface: Send + Sync {
    /// Allow downcasting to concrete database types
    fn as_any(&self) -> &dyn std::any::Any;
    // Transaction management
    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction>;

    // Object operations - Direct DB queries for entity table
    async fn get_object(&self, id: TaoId) -> AppResult<Option<TaoObject>>;
    async fn get_objects(&self, query: ObjectQuery) -> AppResult<TaoObjectQueryResult>;
    /// Create object with pre-generated TAO ID
    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()>;
    async fn update_object(&self, id: TaoId, data: Vec<u8>) -> AppResult<()>;
    async fn delete_object(&self, id: TaoId) -> AppResult<bool>;
    async fn object_exists(&self, id: TaoId) -> AppResult<bool>;

    // Association operations - Direct DB queries for association table
    async fn get_associations(&self, query: AssocQuery) -> AppResult<TaoAssocQueryResult>;
    async fn create_association(&self, assoc: TaoAssociation) -> AppResult<()>;
    async fn delete_association(
        &self,
        id1: TaoId,
        atype: AssocType,
        id2: TaoId,
    ) -> AppResult<bool>;
    async fn association_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId)
        -> AppResult<bool>;
    async fn count_associations(&self, id1: TaoId, atype: AssocType) -> AppResult<u64>;

    // Index operations - Direct DB queries for index table (for performance)
    async fn update_association_count(
        &self,
        id: TaoId,
        atype: AssocType,
        delta: i64,
    ) -> AppResult<()>;
    async fn get_association_count(&self, id: TaoId, atype: AssocType) -> AppResult<u64>;

    // Transactional operations - Execute within existing transaction
    async fn create_object_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<()>;
    async fn create_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        assoc: TaoAssociation,
    ) -> AppResult<()>;
    async fn delete_association_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id1: TaoId,
        atype: AssocType,
        id2: TaoId,
    ) -> AppResult<bool>;
    async fn update_association_count_tx(
        &self,
        tx: &mut DatabaseTransaction,
        id: TaoId,
        atype: AssocType,
        delta: i64,
    ) -> AppResult<()>;

    /// Execute a raw SQL query and return results as a vector of hashmaps
    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>>;

    // Graph visualization methods
    /// Get all objects from this shard for graph visualization
    async fn get_all_objects_from_shard(&self) -> AppResult<Vec<TaoObject>>;
    /// Get all associations from this shard for graph visualization
    async fn get_all_associations_from_shard(&self) -> AppResult<Vec<TaoAssociation>>;
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
        sqlx::query("DROP TABLE IF EXISTS tao_objects CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to drop objects table: {}", e)))?;
        sqlx::query("DROP TABLE IF EXISTS tao_associations CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to drop associations table: {}", e))
            })?;
        sqlx::query("DROP TABLE IF EXISTS tao_association_counts CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to drop association counts table: {}", e))
            })?;

        // Create objects table partitioned by date (time_created)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tao_objects (
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
            CREATE TABLE IF NOT EXISTS tao_associations (
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
            CREATE TABLE IF NOT EXISTS tao_association_counts (
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
        let current_time = crate::infrastructure::tao_core::current_time_millis();
        let current_month_start =
            (current_time / (30 * 24 * 60 * 60 * 1000)) * (30 * 24 * 60 * 60 * 1000); // Rough monthly boundaries

        for i in 0..13 {
            // Current month + 12 future months
            let month_start = current_month_start + (i * 30 * 24 * 60 * 60 * 1000);
            let month_end = month_start + (30 * 24 * 60 * 60 * 1000);

            // Objects partitions
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS tao_objects_m{} PARTITION OF tao_objects FOR VALUES FROM ({}) TO ({})",
                i, month_start, month_end
            ))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create objects monthly partition {}: {}", i, e)))?;

            // Associations partitions
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS tao_associations_m{} PARTITION OF tao_associations FOR VALUES FROM ({}) TO ({})",
                i, month_start, month_end
            ))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations monthly partition {}: {}", i, e)))?;
        }

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_objects_otype ON tao_objects(otype)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create objects otype index: {}", e))
            })?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id1_atype ON tao_associations(id1, atype, time_created)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id2_atype ON tao_associations(id2, atype, time_created)")
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

    async fn get_object(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let row = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data FROM tao_objects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get object {}: {}", id, e)))?;

        if let Some(row) = row {
            Ok(Some(TaoObject {
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

    async fn get_objects(&self, query: ObjectQuery) -> AppResult<TaoObjectQueryResult> {
        let mut sql = "SELECT id, otype, time_created, time_updated, data FROM tao_objects WHERE id = ANY($1)".to_string();

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
            .map(|row| TaoObject {
                id: row.get("id"),
                otype: row.get("otype"),
                data: row.get("data"),
                created_time: row.get("time_created"),
                updated_time: row.get("time_updated"),
                version: row.try_get::<i32, _>("version").unwrap_or(1) as u64,
            })
            .collect();

        Ok(TaoObjectQueryResult {
            objects,
            next_cursor: None, // TODO: Implement pagination
        })
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();

        sqlx::query(
            "INSERT INTO tao_objects (id, otype, time_created, time_updated, data) VALUES ($1, $2, $3, $4, $5)"
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

    async fn update_object(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();

        let result = sqlx::query(
            "UPDATE tao_objects SET data = $1, time_updated = $2, version = version + 1 WHERE id = $3"
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

    async fn delete_object(&self, id: TaoId) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM tao_objects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete object {}: {}", id, e))
            })?;

        Ok(result.rows_affected() > 0)
    }

    async fn object_exists(&self, id: TaoId) -> AppResult<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM tao_objects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to check association existence: {}", e))
        })?;

        Ok(row.is_some())
    }

    async fn get_associations(&self, query: AssocQuery) -> AppResult<TaoAssocQueryResult> {
        let mut sql = "SELECT id1, atype, id2, time_created, data FROM tao_associations WHERE id1 = $1 AND atype = $2".to_string();
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
            .map(|row| TaoAssociation {
                id1: row.get("id1"),
                atype: row.get("atype"),
                id2: row.get("id2"),
                time: row.get("time_created"),
                data: row.get("data"),
            })
            .collect();

        Ok(TaoAssocQueryResult {
            associations,
            next_cursor: None, // TODO: Implement pagination cursors
        })
    }

    async fn create_association(&self, assoc: TaoAssociation) -> AppResult<()> {
        // Insert association
        sqlx::query(
            "INSERT INTO tao_associations (id1, atype, id2, time_created, data) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
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
        id1: TaoId,
        atype: AssocType,
        id2: TaoId,
    ) -> AppResult<bool> {
        let result =
            sqlx::query("DELETE FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3")
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
        id1: TaoId,
        atype: AssocType,
        id2: TaoId,
    ) -> AppResult<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3",
        )
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

    async fn count_associations(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        // Rely on the pre-calculated index table for performance
        self.get_association_count(id1, atype).await
    }

    async fn update_association_count(
        &self,
        id: TaoId,
        atype: AssocType,
        delta: i64,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();

        sqlx::query(
            "INSERT INTO tao_association_counts (id, atype, count, updated_time) VALUES ($1, $2, $3, $4)
             ON CONFLICT (id, atype) DO UPDATE SET count = tao_association_counts.count + $3, updated_time = $4"
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

    async fn get_association_count(&self, id: TaoId, atype: AssocType) -> AppResult<u64> {
        let row =
            sqlx::query("SELECT count FROM tao_association_counts WHERE id = $1 AND atype = $2")
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
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        let postgres_tx = tx.as_postgres_mut()?;

        sqlx::query(
            "INSERT INTO tao_objects (id, otype, time_created, time_updated, data) VALUES ($1, $2, $3, $4, $5)"
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
        assoc: TaoAssociation,
    ) -> AppResult<()> {
        let postgres_tx = tx.as_postgres_mut()?;

        // Insert association
        sqlx::query(
            "INSERT INTO tao_associations (id1, atype, id2, time_created, data) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
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
        id1: TaoId,
        atype: AssocType,
        id2: TaoId,
    ) -> AppResult<bool> {
        let postgres_tx = tx.as_postgres_mut()?;

        let result =
            sqlx::query("DELETE FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3")
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
        id: TaoId,
        atype: AssocType,
        delta: i64,
    ) -> AppResult<()> {
        let now = crate::infrastructure::tao_core::current_time_millis();
        let postgres_tx = tx.as_postgres_mut()?;

        sqlx::query(
            "INSERT INTO tao_association_counts (id, atype, count, updated_time) VALUES ($1, $2, $3, $4)
             ON CONFLICT (id, atype) DO UPDATE SET count = tao_association_counts.count + $3, updated_time = $4"
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

    async fn get_all_objects_from_shard(&self) -> AppResult<Vec<TaoObject>> {
        let rows = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data, version FROM tao_objects ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get all objects from shard: {}", e)))?;

        let objects = rows
            .into_iter()
            .map(|row| TaoObject {
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

    async fn get_all_associations_from_shard(&self) -> AppResult<Vec<TaoAssociation>> {
        let rows = sqlx::query(
            "SELECT id1, atype, id2, time_created, data FROM tao_associations ORDER BY id1, atype, id2"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get all associations from shard: {}", e)))?;

        let associations = rows
            .into_iter()
            .map(|row| TaoAssociation {
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

// === Database Singleton Management ===

static DATABASE_INSTANCE: OnceCell<Arc<dyn DatabaseInterface>> = OnceCell::const_new();

/// Initialize the global database instance with connection
pub async fn initialize_database(database_url: &str) -> AppResult<()> {
    // Get connection pool configuration from environment variables
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u32>()
        .unwrap_or(20);

    let min_connections = std::env::var("DB_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u32>()
        .unwrap_or(5);

    let acquire_timeout_secs = std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
        .unwrap_or_else(|_| "8".to_string())
        .parse::<u64>()
        .unwrap_or(8);

    // Configure connection pool for production performance
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections) // Maximum concurrent connections
        .min_connections(min_connections) // Keep minimum connections alive
        .acquire_timeout(std::time::Duration::from_secs(acquire_timeout_secs)) // Connection timeout
        .idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes idle timeout
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 minutes max connection lifetime
        .test_before_acquire(true) // Test connections before use
        .connect(database_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    let database = PostgresDatabase::new(pool);

    // Initialize tables and partitions
    database.initialize().await?;

    let db_interface: Arc<dyn DatabaseInterface> = Arc::new(database);

    DATABASE_INSTANCE.set(db_interface).map_err(|_| {
        AppError::DatabaseError("Database instance already initialized".to_string())
    })?;

    println!("✅ Database singleton initialized with connection pool:");
    println!("   • Max connections: {}", max_connections);
    println!("   • Min connections: {}", min_connections);
    println!("   • Acquire timeout: {}s", acquire_timeout_secs);
    println!("   • Tables and partitions created");
    Ok(())
}

/// Get the global database instance
pub async fn get_database() -> AppResult<Arc<dyn DatabaseInterface>> {
    DATABASE_INSTANCE
        .get()
        .ok_or_else(|| {
            AppError::DatabaseError(
                "Database instance not initialized. Call initialize_database() first.".to_string(),
            )
        })
        .map(|db| db.clone())
}

/// Initialize database with default URL for development
pub async fn initialize_database_default() -> AppResult<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/tao_dev".to_string());
    initialize_database(&db_url).await
}

/// Perform database health check
pub async fn database_health_check() -> AppResult<()> {
    let db = get_database().await?;

    // Downcast to PostgresDatabase to access health_check method
    let postgres_db = db
        .as_any()
        .downcast_ref::<PostgresDatabase>()
        .ok_or_else(|| AppError::DatabaseError("Database is not PostgresDatabase".to_string()))?;

    postgres_db.health_check().await
}

/// Get database connection pool statistics
pub async fn database_pool_stats() -> AppResult<(u32, u32)> {
    let db = get_database().await?;

    // Downcast to PostgresDatabase to access pool_stats method
    let postgres_db = db
        .as_any()
        .downcast_ref::<PostgresDatabase>()
        .ok_or_else(|| AppError::DatabaseError("Database is not PostgresDatabase".to_string()))?;

    Ok(postgres_db.pool_stats())
}
