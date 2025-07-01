// Core infrastructure modules
pub mod association_registry; // Manages association type mappings
pub mod global_tao;
pub mod id_generator; // ID generation system
pub mod query_router; // Query routing
pub mod shard_topology; // Shard management

pub mod cache;
pub mod database;
pub mod middleware;
pub mod monitoring;
pub mod storage;
pub mod tao_core;
pub mod traits;
pub mod viewer;

// Re-export core infrastructure components
pub use database::database::{
    AssocQueryResult, DatabaseInterface, DatabaseTransaction, ObjectQueryResult, PostgresDatabase,
};
pub use id_generator::TaoIdGenerator;
pub use tao_core::tao_core::{
    create_tao_association, current_time_millis, AssocType, TaoAssocQuery, TaoAssociation, TaoId,
    TaoObject, TaoObjectQuery, TaoOperations, TaoTime, TaoType,
};
pub use viewer::viewer::ViewerContext;

pub use association_registry::AssociationRegistry;

// Re-export production components
pub use cache::cache_layer::{
    initialize_cache_default, CacheConfig, CacheEntry, TaoMultiTierCache,
};
pub use monitoring::monitoring::{
    initialize_metrics_default, initialize_monitoring, MetricsCollector,
};

// Re-export new traits
pub use cache::cache::Cache;
pub use database::sqlite_database::SqliteDatabase;
pub use traits::traits::{CacheInterface, MetricsInterface};
