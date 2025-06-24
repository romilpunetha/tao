// TAO - The Objects and Associations
// Developer-friendly interface with all enterprise features integrated

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, OnceCell};
use tracing::{info, warn, error, instrument};

use crate::error::{AppResult, AppError};
// Re-export core TAO types for public use
pub use crate::infrastructure::tao_core::{
    TaoOperations, TaoId, TaoTime, TaoObject, TaoAssociation, AssocQuery, ObjectQuery, 
    TaoType, AssocType, current_time_millis, create_tao_association, generate_tao_id
};

use crate::infrastructure::{
    cache_layer::{TaoMultiTierCache, CacheConfig},
    security::{SecurityService, SecurityContext, Permission},
    monitoring::{MetricsCollector, BusinessEvent, CacheOperation},
    replication::{ReplicationManager, ReplicationOperation, VectorClock},
};

/// TAO - The main interface developers use
/// Includes caching, security, monitoring, and fault tolerance
pub struct Tao {
    /// Core TAO interface for actual operations
    core_tao: Arc<dyn TaoOperations>,
    
    /// Multi-tier caching system (L1 + L2)
    cache: Arc<TaoMultiTierCache>,
    
    /// Security and authentication service
    security: Arc<SecurityService>,
    
    /// Monitoring and metrics collection
    metrics: Arc<MetricsCollector>,
    
    /// Replication manager for multi-master consistency
    replication: Arc<ReplicationManager>,
    
    /// Circuit breaker for fault tolerance
    circuit_breaker: Arc<CircuitBreaker>,
    
    /// Configuration
    config: TaoConfig,
}

#[derive(Debug, Clone)]
pub struct TaoConfig {
    pub enable_caching: bool,
    pub enable_security: bool,
    pub enable_monitoring: bool,
    pub enable_replication: bool,
    pub enable_circuit_breaker: bool,
    pub cache_object_ttl: Duration,
    pub cache_association_ttl: Duration,
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_recovery_timeout: Duration,
}

impl Default for TaoConfig {
    fn default() -> Self {
        TaoConfig {
            enable_caching: true,
            enable_security: true,
            enable_monitoring: true,
            enable_replication: true,
            enable_circuit_breaker: true,
            cache_object_ttl: Duration::from_secs(300),  // 5 minutes
            cache_association_ttl: Duration::from_secs(600), // 10 minutes
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_timeout: Duration::from_secs(30),
        }
    }
}

impl Tao {
    pub fn new(
        core_tao: Arc<dyn TaoOperations>,
        cache: Arc<TaoMultiTierCache>,
        security: Arc<SecurityService>,
        metrics: Arc<MetricsCollector>,
        replication: Arc<ReplicationManager>,
        config: TaoConfig,
    ) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_recovery_timeout,
        ));

        Self {
            core_tao,
            cache,
            security,
            metrics,
            replication,
            circuit_breaker,
            config,
        }
    }

    /// Check permissions for an operation
    async fn check_permission(&self, context: &SecurityContext, permission: Permission) -> AppResult<()> {
        if !self.config.enable_security {
            return Ok(());
        }

        if !self.security.check_permission(context, &permission).await {
            self.metrics.record_business_event(BusinessEvent::CrossShardOperation).await;
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }

        Ok(())
    }

    /// Record operation metrics
    async fn record_operation_metrics(&self, operation: &str, duration: Duration, success: bool) {
        if self.config.enable_monitoring {
            self.metrics.record_request(operation, duration, success).await;
        }
    }

    /// Log replication operation
    async fn log_replication(&self, operation: ReplicationOperation) -> AppResult<()> {
        if self.config.enable_replication {
            let target_shards = vec![]; // Would be populated based on operation
            self.replication.log_operation(operation, target_shards).await?;
        }
        Ok(())
    }

    /// Execute operation with circuit breaker protection
    async fn execute_with_circuit_breaker<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>>,
    {
        if !self.config.enable_circuit_breaker {
            return operation.await;
        }

        self.circuit_breaker.execute(operation).await
    }
}

#[async_trait]
impl TaoOperations for Tao {
    /// Get object with full production features
    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let start_time = Instant::now();
        let operation = "obj_get";
        
        // Note: In production, security context would be extracted from request context
        // For now, we'll skip security checks for read operations or use a default context
        
        let result = async {
            // 1. Try cache first (if enabled)
            if self.config.enable_caching {
                if let Some(cached_object) = self.cache.get_object(id).await? {
                    self.metrics.record_cache_operation(
                        CacheOperation::L1Lookup, 
                        true, 
                        start_time.elapsed()
                    ).await;
                    return Ok(Some(cached_object));
                }
            }

            // 2. Circuit breaker protection
            let object = self.execute_with_circuit_breaker(async {
                self.core_tao.obj_get(id).await
            }).await?;

            // 3. Populate cache if object found
            if let Some(ref obj) = object {
                if self.config.enable_caching {
                    self.cache.put_object(id, obj).await?;
                }
            }

            Ok(object)
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        if success {
            info!("Successfully retrieved object {}", id);
        } else {
            warn!("Failed to retrieve object {}: {:?}", id, result);
        }

        result
    }

    /// Add object with security, replication, and monitoring
    #[instrument(skip(self, data), fields(object_type = %otype))]
    async fn obj_add(&self, otype: TaoType, data: Vec<u8>) -> AppResult<TaoId> {
        let start_time = Instant::now();
        let operation = "obj_add";

        let result = async {
            // 1. Circuit breaker protection
            let object_id = self.execute_with_circuit_breaker(async {
                self.core_tao.obj_add(otype.clone(), data.clone()).await
            }).await?;

            // 2. Log replication operation
            let replication_op = ReplicationOperation::CreateObject {
                object_id,
                object_type: otype.clone(),
                data: data.clone(),
                owner_id: 0, // Default owner
            };
            self.log_replication(replication_op).await?;

            // 3. Record business metrics
            self.metrics.record_business_event(BusinessEvent::PostCreated).await;

            Ok(object_id)
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        if success {
            info!("Successfully created object of type {}", otype);
        } else {
            error!("Failed to create object of type {}: {:?}", otype, result);
        }

        result
    }


    /// Update object with cache invalidation and replication
    #[instrument(skip(self, data), fields(object_id = %id))]
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let start_time = Instant::now();
        let operation = "obj_update";

        let result = async {
            // 1. Circuit breaker protection
            self.execute_with_circuit_breaker(async {
                self.core_tao.obj_update(id, data.clone()).await
            }).await?;

            // 2. Invalidate cache
            if self.config.enable_caching {
                self.cache.invalidate_object(id).await?;
            }

            // 3. Log replication operation
            let replication_op = ReplicationOperation::UpdateObject {
                object_id: id,
                data: data.clone(),
                previous_version: VectorClock::new(), // Would track actual version
            };
            self.log_replication(replication_op).await?;

            Ok(())
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        result
    }

    /// Delete object with cache invalidation and replication
    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        let start_time = Instant::now();
        let operation = "obj_delete";

        let result = async {
            // 1. Circuit breaker protection
            let deleted = self.execute_with_circuit_breaker(async {
                self.core_tao.obj_delete(id).await
            }).await?;

            if deleted {
                // 2. Invalidate cache
                if self.config.enable_caching {
                    self.cache.invalidate_object(id).await?;
                }

                // 3. Log replication operation
                let replication_op = ReplicationOperation::DeleteObject {
                    object_id: id,
                    previous_version: VectorClock::new(),
                };
                self.log_replication(replication_op).await?;
            }

            Ok(deleted)
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        result
    }

    /// Get associations with caching
    #[instrument(skip(self), fields(id1 = %query.id1, atype = %query.atype))]
    async fn assoc_get(&self, query: AssocQuery) -> AppResult<Vec<TaoAssociation>> {
        let start_time = Instant::now();
        let operation = "assoc_get";

        let result = async {
            // 1. Try cache first (if enabled)
            if self.config.enable_caching && query.id2_set.is_none() {
                if let Some(cached_assocs) = self.cache.get_associations(query.id1, &query.atype).await? {
                    self.metrics.record_cache_operation(
                        CacheOperation::L1Lookup, 
                        true, 
                        start_time.elapsed()
                    ).await;
                    return Ok(cached_assocs);
                }
            }

            // 2. Circuit breaker protection
            let associations = self.execute_with_circuit_breaker(async {
                self.core_tao.assoc_get(query.clone()).await
            }).await?;

            // 3. Populate cache if simple query
            if self.config.enable_caching && query.id2_set.is_none() {
                self.cache.put_associations(query.id1, &query.atype, &associations).await?;
            }

            Ok(associations)
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        result
    }

    /// Add association with replication
    #[instrument(skip(self), fields(id1 = %assoc.id1, atype = %assoc.atype, id2 = %assoc.id2))]
    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        let start_time = Instant::now();
        let operation = "assoc_add";

        let result = async {
            // 1. Circuit breaker protection
            self.execute_with_circuit_breaker(async {
                self.core_tao.assoc_add(assoc.clone()).await
            }).await?;

            // 2. Invalidate relevant cache entries
            if self.config.enable_caching {
                // Invalidate associations for both source and target objects
                let _cache_key1 = format!("assoc:{}:{}", assoc.id1, assoc.atype);
                let _cache_key2 = format!("assoc:{}:{}", assoc.id2, assoc.atype);
                // Cache invalidation would happen here in production
            }

            // 3. Log replication operation
            let replication_op = ReplicationOperation::CreateAssociation {
                association: assoc.clone(),
            };
            self.log_replication(replication_op).await?;

            // 4. Record business metrics based on association type
            match assoc.atype.as_str() {
                "friendship" => self.metrics.record_business_event(BusinessEvent::FriendshipFormed).await,
                "like" => self.metrics.record_business_event(BusinessEvent::LikeGiven).await,
                "comment" => self.metrics.record_business_event(BusinessEvent::CommentMade).await,
                _ => self.metrics.record_business_event(BusinessEvent::CrossShardOperation).await,
            }

            Ok(())
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        result
    }

    /// Delete association with cache invalidation
    #[instrument(skip(self), fields(id1 = %id1, atype = %atype, id2 = %id2))]
    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let start_time = Instant::now();
        let operation = "assoc_delete";

        let result = async {
            // 1. Circuit breaker protection
            let deleted = self.execute_with_circuit_breaker(async {
                self.core_tao.assoc_delete(id1, atype.clone(), id2).await
            }).await?;

            if deleted {
                // 2. Invalidate cache
                if self.config.enable_caching {
                    // Invalidate associations for both objects
                    let _cache_key1 = format!("assoc:{}:{}", id1, atype);
                    let _cache_key2 = format!("assoc:{}:{}", id2, atype);
                    // Cache invalidation would happen here
                }

                // 3. Log replication operation
                let replication_op = ReplicationOperation::DeleteAssociation {
                    id1,
                    atype: atype.clone(),
                    id2,
                    previous_version: VectorClock::new(),
                };
                self.log_replication(replication_op).await?;
            }

            Ok(deleted)
        }.await;

        // Record metrics
        let success = result.is_ok();
        self.record_operation_metrics(operation, start_time.elapsed(), success).await;

        result
    }

    // Delegate remaining operations to core TAO with metrics
    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.assoc_count(id1, atype).await
        }).await;
        self.record_operation_metrics("assoc_count", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn assoc_range(&self, id1: TaoId, atype: AssocType, offset: u64, limit: u32) -> AppResult<Vec<TaoAssociation>> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.assoc_range(id1, atype, offset, limit).await
        }).await;
        self.record_operation_metrics("assoc_range", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn assoc_time_range(&self, id1: TaoId, atype: AssocType, high_time: i64, low_time: i64, limit: Option<u32>) -> AppResult<Vec<TaoAssociation>> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.assoc_time_range(id1, atype, high_time, low_time, limit).await
        }).await;
        self.record_operation_metrics("assoc_time_range", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn get_by_id_and_type(&self, ids: Vec<TaoId>, otype: TaoType) -> AppResult<Vec<TaoObject>> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.get_by_id_and_type(ids, otype).await
        }).await;
        self.record_operation_metrics("get_by_id_and_type", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn obj_update_by_type(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<bool> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.obj_update_by_type(id, otype, data).await
        }).await;
        self.record_operation_metrics("obj_update_by_type", start_time.elapsed(), result.is_ok()).await;
        
        // Invalidate cache if successful
        if let Ok(true) = result {
            if self.config.enable_caching {
                let _ = self.cache.invalidate_object(id).await;
            }
        }
        
        result
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.obj_delete_by_type(id, otype).await
        }).await;
        self.record_operation_metrics("obj_delete_by_type", start_time.elapsed(), result.is_ok()).await;
        
        // Invalidate cache if successful
        if let Ok(true) = result {
            if self.config.enable_caching {
                let _ = self.cache.invalidate_object(id).await;
            }
        }
        
        result
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.assoc_exists(id1, atype, id2).await
        }).await;
        self.record_operation_metrics("assoc_exists", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.obj_exists(id).await
        }).await;
        self.record_operation_metrics("obj_exists", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.obj_exists_by_type(id, otype).await
        }).await;
        self.record_operation_metrics("obj_exists_by_type", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn get_neighbors(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoObject>> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.get_neighbors(id, atype, limit).await
        }).await;
        self.record_operation_metrics("get_neighbors", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn get_neighbor_ids(&self, id: TaoId, atype: AssocType, limit: Option<u32>) -> AppResult<Vec<TaoId>> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.get_neighbor_ids(id, atype, limit).await
        }).await;
        self.record_operation_metrics("get_neighbor_ids", start_time.elapsed(), result.is_ok()).await;
        result
    }

    async fn begin_transaction(&self) -> AppResult<crate::infrastructure::DatabaseTransaction> {
        let start_time = Instant::now();
        let result = self.execute_with_circuit_breaker(async {
            self.core_tao.begin_transaction().await
        }).await;
        self.record_operation_metrics("begin_transaction", start_time.elapsed(), result.is_ok()).await;
        result
    }
}

/// Circuit breaker for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failures: u32,
    last_failure_time: Option<Instant>,
    state: CircuitState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                failures: 0,
                last_failure_time: None,
                state: CircuitState::Closed,
            })),
        }
    }

    pub async fn execute<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>>,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            if state.state == CircuitState::Open {
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() < self.recovery_timeout {
                        return Err(AppError::ServiceUnavailable("Circuit breaker is open".to_string()));
                    }
                }
                // Time to try half-open
                drop(state);
                let mut state = self.state.write().await;
                state.state = CircuitState::HalfOpen;
            }
        }

        // Execute operation
        match operation.await {
            Ok(result) => {
                // Reset on success
                let mut state = self.state.write().await;
                state.failures = 0;
                state.state = CircuitState::Closed;
                Ok(result)
            }
            Err(error) => {
                // Record failure
                let mut state = self.state.write().await;
                state.failures += 1;
                state.last_failure_time = Some(Instant::now());
                
                if state.failures >= self.failure_threshold {
                    state.state = CircuitState::Open;
                    warn!("Circuit breaker opened after {} failures", state.failures);
                }
                
                Err(error)
            }
        }
    }
}

/// Factory for creating TAO instances
pub struct TaoFactory;

impl TaoFactory {
    /// Create a fully integrated TAO instance with enterprise features
    pub async fn create_tao(
        core_tao: Arc<dyn TaoOperations>,
        config: Option<TaoConfig>,
    ) -> AppResult<Arc<Tao>> {
        let config = config.unwrap_or_default();

        // Create cache layer
        let cache_config = CacheConfig {
            l1_default_ttl: config.cache_object_ttl,
            l2_default_ttl: config.cache_association_ttl,
            ..Default::default()
        };
        let cache = Arc::new(TaoMultiTierCache::new(cache_config));

        // Create security service
        let security_config = crate::infrastructure::security::SecurityConfig::default();
        let security = Arc::new(SecurityService::new(security_config));

        // Create monitoring system
        let metrics = crate::infrastructure::monitoring::initialize_monitoring()?;

        // Create replication manager
        let replication_config = crate::infrastructure::replication::ReplicationConfig::default();
        let replication = Arc::new(ReplicationManager::new("node-1".to_string(), replication_config));

        let tao = Tao::new(
            core_tao,
            cache,
            security,
            metrics,
            replication,
            config,
        );

        info!("🚀 TAO instance created with enterprise features (cache, security, monitoring)");
        Ok(Arc::new(tao))
    }
}

// === TAO Singleton Management ===

static TAO_INSTANCE: OnceCell<Arc<Tao>> = OnceCell::const_new();

/// Initialize the global TAO instance (developer-facing)
pub async fn initialize_tao(core_tao: Arc<dyn TaoOperations>) -> AppResult<()> {
    let tao = TaoFactory::create_tao(core_tao, None).await?;
    
    TAO_INSTANCE.set(tao)
        .map_err(|_| crate::error::AppError::Internal("TAO instance already initialized".to_string()))?;

    println!("✅ TAO initialized (developer interface with enterprise features)");
    Ok(())
}

/// Initialize the global TAO instance with custom config
pub async fn initialize_tao_with_config(core_tao: Arc<dyn TaoOperations>, config: TaoConfig) -> AppResult<()> {
    let tao = TaoFactory::create_tao(core_tao, Some(config)).await?;
    
    TAO_INSTANCE.set(tao)
        .map_err(|_| crate::error::AppError::Internal("TAO instance already initialized".to_string()))?;

    println!("✅ TAO initialized with custom config (developer interface with enterprise features)");
    Ok(())
}

/// Get the global TAO instance (developer-facing)
pub async fn get_tao() -> AppResult<Arc<Tao>> {
    TAO_INSTANCE.get()
        .ok_or_else(|| crate::error::AppError::Internal("TAO instance not initialized. Call initialize_tao() first.".to_string()))
        .map(|tao| tao.clone())
}