// Database Interface - Low-level database operations for TAO
// This layer handles direct SQL queries for objects, associations, and indexes

use async_trait::async_trait;
use sqlx::{PgPool, Row, Transaction, Postgres};
use std::sync::Arc;
use tokio::sync::OnceCell;
use crate::error::{AppError, AppResult};
use crate::ent_framework::{
    TaoId, TaoType, AssocType, TaoAssociation, TaoObject,
    AssocQuery, ObjectQuery
};

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

/// Transaction wrapper for database operations
pub struct DatabaseTransaction<'a> {
    tx: Transaction<'a, Postgres>,
}

impl<'a> DatabaseTransaction<'a> {
    pub fn new(tx: Transaction<'a, Postgres>) -> Self {
        Self { tx }
    }

    /// Commit the transaction
    pub async fn commit(self) -> AppResult<()> {
        self.tx.commit().await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> AppResult<()> {
        self.tx.rollback().await
            .map_err(|e| AppError::DatabaseError(format!("Failed to rollback transaction: {}", e)))
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
    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()>;
    async fn update_object(&self, id: TaoId, data: Vec<u8>) -> AppResult<()>;
    async fn delete_object(&self, id: TaoId) -> AppResult<bool>;
    async fn object_exists(&self, id: TaoId) -> AppResult<bool>;

    // Association operations - Direct DB queries for association table
    async fn get_associations(&self, query: AssocQuery) -> AppResult<TaoAssocQueryResult>;
    async fn create_association(&self, assoc: TaoAssociation) -> AppResult<()>;
    async fn delete_association(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;
    async fn association_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;
    async fn count_associations(&self, id1: TaoId, atype: AssocType) -> AppResult<u64>;

    // Index operations - Direct DB queries for index table (for performance)
    async fn update_association_count(&self, id: TaoId, atype: AssocType, delta: i64) -> AppResult<()>;
    async fn get_association_count(&self, id: TaoId, atype: AssocType) -> AppResult<u64>;

    // Transactional operations - Execute within existing transaction
    async fn create_object_tx(&self, tx: &mut DatabaseTransaction<'_>, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()>;
    async fn create_association_tx(&self, tx: &mut DatabaseTransaction<'_>, assoc: TaoAssociation) -> AppResult<()>;
    async fn delete_association_tx(&self, tx: &mut DatabaseTransaction<'_>, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool>;
    async fn update_association_count_tx(&self, tx: &mut DatabaseTransaction<'_>, id: TaoId, atype: AssocType, delta: i64) -> AppResult<()>;
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
        // Create objects table partitioned by date (time_created)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tao_objects (
                id BIGINT NOT NULL,
                otype VARCHAR(64) NOT NULL,
                time_created BIGINT NOT NULL,
                time_updated BIGINT NOT NULL,
                data BYTEA,
                version INTEGER DEFAULT 1,
                shard_id INTEGER NOT NULL DEFAULT (id % 4),
                PRIMARY KEY (id, time_created)
            ) PARTITION BY RANGE (time_created)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create objects table: {}", e)))?;

        // Create associations table partitioned by date (time_created)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tao_associations (
                id1 BIGINT NOT NULL,
                atype VARCHAR(64) NOT NULL,
                id2 BIGINT NOT NULL,
                time_created BIGINT NOT NULL,
                data BYTEA,
                shard_id INTEGER NOT NULL DEFAULT (id1 % 4),
                PRIMARY KEY (id1, atype, id2, time_created)
            ) PARTITION BY RANGE (time_created)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create associations table: {}", e)))?;

        // Create association count index table partitioned by date (updated_time)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tao_association_counts (
                id BIGINT NOT NULL,
                atype VARCHAR(64) NOT NULL,
                count BIGINT DEFAULT 0,
                updated_time BIGINT NOT NULL,
                shard_id INTEGER NOT NULL DEFAULT (id % 4),
                PRIMARY KEY (id, atype, updated_time)
            ) PARTITION BY RANGE (updated_time)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association counts table: {}", e)))?;

        // Create monthly partitions for current and next 12 months
        let current_time = crate::infrastructure::tao::current_time_millis();
        let current_month_start = (current_time / (30 * 24 * 60 * 60 * 1000)) * (30 * 24 * 60 * 60 * 1000); // Rough monthly boundaries
        
        for i in 0..13 { // Current month + 12 future months
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

            // Association counts partitions
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS tao_association_counts_m{} PARTITION OF tao_association_counts FOR VALUES FROM ({}) TO ({})",
                i, month_start, month_end
            ))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create association counts monthly partition {}: {}", i, e)))?;
        }

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_objects_otype ON tao_objects(otype)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create objects otype index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_objects_shard ON tao_objects(shard_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create objects shard index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id1_atype ON tao_associations(id1, atype, time_created)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_id2_atype ON tao_associations(id2, atype, time_created)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create reverse associations index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_assoc_shard ON tao_associations(shard_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create associations shard index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tao_counts_shard ON tao_association_counts(shard_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create association counts shard index: {}", e)))?;

        println!("✅ TAO database tables initialized with date partitioning (monthly) and ID-based sharding (4 shards)");
        Ok(())
    }
}

#[async_trait]
impl DatabaseInterface for PostgresDatabase {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        let tx = self.pool.begin().await
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;
        Ok(DatabaseTransaction::new(tx))
    }
    async fn get_object(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let row = sqlx::query(
            "SELECT id, otype, time_created, time_updated, data FROM tao_objects WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get object {}: {}", id, e)))?;

        if let Some(row) = row {
            Ok(Some(TaoObject {
                id: row.get("id"),
                otype: row.get("otype"),
                time: row.get("time_updated"),
                data: row.get("data"),
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

        let objects = rows.into_iter().map(|row| TaoObject {
            id: row.get("id"),
            otype: row.get("otype"),
            time: row.get("time_updated"),
            data: row.get("data"),
        }).collect();

        Ok(TaoObjectQueryResult {
            objects,
            next_cursor: None, // TODO: Implement pagination
        })
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao::current_time_millis();

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
        let now = crate::infrastructure::tao::current_time_millis();

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
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete object {}: {}", id, e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn object_exists(&self, id: TaoId) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM tao_objects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to check if object {} exists: {}", id, e)))?;

        Ok(row.is_some())
    }

    async fn get_associations(&self, query: AssocQuery) -> AppResult<TaoAssocQueryResult> {
        let mut sql = "SELECT id1, atype, id2, time_created, data FROM tao_associations WHERE id1 = $1 AND atype = $2".to_string();

        // Add id2 filter if provided
        if query.id2_set.is_some() {
            sql.push_str(&format!(" AND id2 IN ({:#?})", query.id2_set));
        }

        if let (Some(low_time), Some(high_time)) = (query.low_time, query.high_time) {
            sql.push_str(&format!(" AND time_created BETWEEN ${} AND ${}", low_time, high_time));
        }

        sql.push_str(" ORDER BY time_created DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT ${}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET ${}", offset));
        }

        let mut query_builder = sqlx::query(&sql)
            .bind(query.id1)
            .bind(&query.atype);

        if let (Some(low_time), Some(high_time)) = (query.low_time, query.high_time) {
            query_builder = query_builder.bind(low_time).bind(high_time);
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

        let associations = rows.into_iter().map(|row| TaoAssociation {
            id1: row.get("id1"),
            atype: row.get("atype"),
            id2: row.get("id2"),
            time: row.get("time_created"),
            data: row.get("data"),
        }).collect();

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
        self.update_association_count(assoc.id1, assoc.atype, 1).await?;

        Ok(())
    }

    async fn delete_association(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3"
        )
        .bind(id1)
        .bind(&atype)
        .bind(id2)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete association: {}", e)))?;

        if result.rows_affected() > 0 {
            // Update association count
            self.update_association_count(id1, atype, -1).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn association_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3"
        )
        .bind(id1)
        .bind(&atype)
        .bind(id2)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check association existence: {}", e)))?;

        Ok(row.is_some())
    }

    async fn count_associations(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        // Try to get from index table first (faster)
        if let Ok(count) = self.get_association_count(id1, atype.clone()).await {
            return Ok(count);
        }

        // Fallback to direct count
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM tao_associations WHERE id1 = $1 AND atype = $2"
        )
        .bind(id1)
        .bind(&atype)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count associations: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn update_association_count(&self, id: TaoId, atype: AssocType, delta: i64) -> AppResult<()> {
        let now = crate::infrastructure::tao::current_time_millis();

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
        let row = sqlx::query(
            "SELECT count FROM tao_association_counts WHERE id = $1 AND atype = $2"
        )
        .bind(id)
        .bind(&atype)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get association count: {}", e)))?;

        if let Some(row) = row {
            let count: i64 = row.get("count");
            Ok(count as u64)
        } else {
            Ok(0)
        }
    }

    // === Transactional Operations ===

    async fn create_object_tx(&self, tx: &mut DatabaseTransaction<'_>, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let now = crate::infrastructure::tao::current_time_millis();

        sqlx::query(
            "INSERT INTO tao_objects (id, otype, time_created, time_updated, data) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(&otype)
        .bind(now)
        .bind(now)
        .bind(&data)
        .execute(&mut *tx.tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create object with ID {} in transaction: {}", id, e)))?;

        Ok(())
    }

    async fn create_association_tx(&self, tx: &mut DatabaseTransaction<'_>, assoc: TaoAssociation) -> AppResult<()> {
        // Insert association
        sqlx::query(
            "INSERT INTO tao_associations (id1, atype, id2, time_created, data) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
        )
        .bind(assoc.id1)
        .bind(&assoc.atype)
        .bind(assoc.id2)
        .bind(assoc.time)
        .bind(&assoc.data)
        .execute(&mut *tx.tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create association in transaction: {}", e)))?;

        // Update association count
        self.update_association_count_tx(tx, assoc.id1, assoc.atype, 1).await?;

        Ok(())
    }

    async fn delete_association_tx(&self, tx: &mut DatabaseTransaction<'_>, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM tao_associations WHERE id1 = $1 AND atype = $2 AND id2 = $3"
        )
        .bind(id1)
        .bind(&atype)
        .bind(id2)
        .execute(&mut *tx.tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete association in transaction: {}", e)))?;

        if result.rows_affected() > 0 {
            // Update association count
            self.update_association_count_tx(tx, id1, atype, -1).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_association_count_tx(&self, tx: &mut DatabaseTransaction<'_>, id: TaoId, atype: AssocType, delta: i64) -> AppResult<()> {
        let now = crate::infrastructure::tao::current_time_millis();

        sqlx::query(
            "INSERT INTO tao_association_counts (id, atype, count, updated_time) VALUES ($1, $2, $3, $4)
             ON CONFLICT (id, atype) DO UPDATE SET count = tao_association_counts.count + $3, updated_time = $4"
        )
        .bind(id)
        .bind(&atype)
        .bind(delta)
        .bind(now)
        .execute(&mut *tx.tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update association count in transaction: {}", e)))?;

        Ok(())
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
        .max_connections(max_connections)                    // Maximum concurrent connections
        .min_connections(min_connections)                     // Keep minimum connections alive
        .acquire_timeout(std::time::Duration::from_secs(acquire_timeout_secs))  // Connection timeout
        .idle_timeout(std::time::Duration::from_secs(600))   // 10 minutes idle timeout
        .max_lifetime(std::time::Duration::from_secs(1800))  // 30 minutes max connection lifetime
        .test_before_acquire(true)              // Test connections before use
        .connect(database_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    let database = PostgresDatabase::new(pool);
    
    // Initialize tables and partitions
    database.initialize().await?;

    let db_interface: Arc<dyn DatabaseInterface> = Arc::new(database);
    
    DATABASE_INSTANCE.set(db_interface)
        .map_err(|_| AppError::Internal("Database instance already initialized".to_string()))?;

    println!("✅ Database singleton initialized with connection pool:");
    println!("   • Max connections: {}", max_connections);
    println!("   • Min connections: {}", min_connections);
    println!("   • Acquire timeout: {}s", acquire_timeout_secs);
    println!("   • Tables and partitions created");
    Ok(())
}

/// Get the global database instance
pub async fn get_database() -> AppResult<Arc<dyn DatabaseInterface>> {
    DATABASE_INSTANCE.get()
        .ok_or_else(|| AppError::Internal("Database instance not initialized. Call initialize_database() first.".to_string()))
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
    let postgres_db = db.as_any()
        .downcast_ref::<PostgresDatabase>()
        .ok_or_else(|| AppError::Internal("Database is not PostgresDatabase".to_string()))?;
    
    postgres_db.health_check().await
}

/// Get database connection pool statistics
pub async fn database_pool_stats() -> AppResult<(u32, u32)> {
    let db = get_database().await?;
    
    // Downcast to PostgresDatabase to access pool_stats method
    let postgres_db = db.as_any()
        .downcast_ref::<PostgresDatabase>()
        .ok_or_else(|| AppError::Internal("Database is not PostgresDatabase".to_string()))?;
    
    Ok(postgres_db.pool_stats())
}