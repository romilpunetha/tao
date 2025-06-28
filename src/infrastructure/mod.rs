// Core infrastructure modules
pub mod association_registry; // Manages association type mappings
pub mod cache; // Basic caching functionality
pub mod cache_layer; // Multi-tier caching
pub mod database; // Database interface and implementations
pub mod id_generator; // ID generation system
pub mod monitoring; // Metrics and monitoring
pub mod query_router; // Query routing
pub mod shard_topology; // Shard management
pub mod tao; // Main TAO interface
pub mod tao_core; // Core TAO operations and types
pub mod traits;
pub mod viewer; // Viewer context
pub mod sqlite_database; // SQLite database for testing
pub mod wal_storage; // WAL file-based storage
pub mod write_ahead_log; // Write-ahead log // Infrastructure traits
pub mod tao_decorators; // TAO decorator pattern implementations

// Re-export core infrastructure components
pub use cache::*;
pub use database::{
    DatabaseInterface, PostgresDatabase,
    AssocQueryResult, ObjectQueryResult, DatabaseTransaction
};
pub use id_generator::TaoIdGenerator;
pub use viewer::ViewerContext;
pub use sqlite_database::SqliteDatabase; // Also adding SqliteDatabase to mod.rs
// Core TAO types and interfaces (developers can use these types)
pub use tao_core::{
    create_tao_association, current_time_millis, TaoAssocQuery,
    AssocType, TaoObjectQuery, TaoAssociation, TaoId, TaoObject, TaoOperations, TaoTime, TaoType,
};

pub use association_registry::AssociationRegistry;

// Re-export production components
pub use cache_layer::{CacheConfig, CacheEntry, TaoMultiTierCache, initialize_cache_default};
pub use monitoring::{
    initialize_monitoring, initialize_metrics_default, MetricsCollector,
};

// Re-export new traits
pub use traits::{
    CacheInterface, MetricsInterface,
};
