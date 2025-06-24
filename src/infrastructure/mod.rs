// Core infrastructure modules
pub mod database;              // Database interface and implementations
pub mod cache;                 // Basic caching functionality
pub mod id_generator;          // ID generation system
pub mod viewer;                // Viewer context
pub mod tao_core;              // Core TAO operations and types
pub mod tao;                   // Main TAO interface
pub mod shard_topology;        // Shard management
pub mod query_router;          // Query routing
pub mod write_ahead_log;       // Write-ahead log
pub mod eventual_consistency;  // Eventual consistency management
pub mod distributed_tao_factory; // Distributed TAO factory
pub mod monitoring;            // Metrics and monitoring
pub mod security;              // Security and authentication
pub mod replication;           // Multi-master replication with conflict resolution
pub mod service_discovery;     // Service discovery and load balancing
pub mod cache_layer;           // Multi-tier caching
pub mod traits;                // Infrastructure traits

// Re-export core infrastructure components
pub use database::{DatabaseInterface, PostgresDatabase, DatabaseTransaction, TaoAssocQueryResult, TaoObjectQueryResult, initialize_database, get_database, initialize_database_default, database_health_check, database_pool_stats};
pub use cache::*;
pub use id_generator::TaoIdGenerator;
pub use viewer::ViewerContext;
// Core TAO types and interfaces (developers can use these types)
pub use tao_core::{TaoOperations, TaoId, TaoTime, TaoType, AssocType, TaoAssociation, TaoObject, AssocQuery, ObjectQuery, generate_tao_id, current_time_millis, create_tao_association, create_tao_object};
// Main TAO interface for developers (this is what developers should use)
pub use tao::{Tao, TaoConfig, TaoFactory, initialize_tao, initialize_tao_with_config, get_tao, CircuitBreaker};

// Re-export production components
pub use cache_layer::{TaoMultiTierCache, CacheConfig, CacheEntry, DistributedCache};
pub use security::{SecurityService, SecurityContext, Permission, Resource, Action, Scope, UserRole, SecurityConfig};
pub use monitoring::{MetricsCollector, initialize_monitoring, HealthStatus, ServiceStatus, BusinessEvent, CacheOperation};
pub use replication::{ReplicationManager, VectorClock, ReplicationOperation, ReplicationConfig, ConsistencyLevel};
pub use service_discovery::{ServiceRegistry, TaoServiceDiscovery, ServiceInstance, LoadBalancingStrategy, ServiceDiscoveryConfig};

// Re-export new traits
pub use traits::{CacheInterface, SecurityInterface, MetricsInterface, ReplicationInterface, CircuitBreakerInterface};
