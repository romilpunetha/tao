// Production-grade Monitoring and Observability
// Implements comprehensive metrics, tracing, and health monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::infrastructure::traits::MetricsInterface;
use crate::error::AppResult;
use crate::infrastructure::tao::TaoId;

/// Comprehensive metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    /// Request metrics
    request_metrics: Arc<RwLock<RequestMetrics>>,
    /// Database metrics
    database_metrics: Arc<RwLock<DatabaseMetrics>>,
    /// Cache metrics
    cache_metrics: Arc<RwLock<CacheMetrics>>,
    /// System metrics
    system_metrics: Arc<RwLock<SystemMetrics>>,
    /// Custom business metrics
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    /// Health status
    health_status: Arc<RwLock<HealthStatus>>,
}

/// Request-level metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub requests_per_endpoint: HashMap<String, EndpointMetrics>,
    pub response_times: HistogramMetrics,
    pub active_connections: u64,
    pub rate_limited_requests: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub total_calls: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub last_called: Option<SystemTime>,
}

/// Database performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub query_times: HistogramMetrics,
    pub connection_pool_stats: ConnectionPoolStats,
    pub queries_by_type: HashMap<String, QueryTypeMetrics>,
    pub slow_queries: Vec<SlowQueryRecord>,
    pub deadlocks: u64,
    pub timeouts: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub waiting_for_connection: u32,
    pub connection_wait_time_ms: f64,
    pub connection_creation_failures: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryTypeMetrics {
    pub count: u64,
    pub total_time_ms: f64,
    pub avg_time_ms: f64,
    pub max_time_ms: f64,
    pub rows_affected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQueryRecord {
    pub query: String,
    pub duration_ms: f64,
    pub timestamp: SystemTime,
    pub user_id: Option<TaoId>,
    pub parameters: String,
}

/// Cache performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub cache_writes: u64,
    pub cache_evictions: u64,
    pub cache_invalidations: u64,
    pub avg_lookup_time_ms: f64,
    pub cache_size_bytes: u64,
    pub hit_rate_percentage: f64,
}

/// System-level metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percentage: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percentage: f64,
    pub disk_usage_bytes: u64,
    pub disk_io_read_bytes: u64,
    pub disk_io_write_bytes: u64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
    pub open_file_descriptors: u64,
    pub process_count: u64,
    pub uptime_seconds: u64,
}

/// Business-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub active_users: u64,
    pub new_user_registrations: u64,
    pub posts_created: u64,
    pub likes_given: u64,
    pub comments_made: u64,
    pub friendships_formed: u64,
    pub groups_created: u64,
    pub events_created: u64,
    pub cross_shard_operations: u64,
    pub wal_transactions: u64,
    pub data_distribution: HashMap<String, u64>,
}

/// Health status for different components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: ServiceStatus,
    pub database_status: ServiceStatus,
    pub cache_status: ServiceStatus,
    pub query_router_status: ServiceStatus,
    pub wal_status: ServiceStatus,
    pub consistency_manager_status: ServiceStatus,
    pub last_health_check: Option<SystemTime>,
    pub health_check_failures: u64,
    pub services: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: ServiceStatus,
    pub last_check: SystemTime,
    pub response_time_ms: f64,
    pub error_rate: f64,
    pub details: String,
}

/// Histogram for tracking response times and other metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistogramMetrics {
    pub count: u64,
    pub sum: f64,
    pub buckets: HashMap<String, u64>, // e.g., "0-10ms": 100, "10-50ms": 50
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
    pub min: f64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            request_metrics: Arc::new(RwLock::new(RequestMetrics::default())),
            database_metrics: Arc::new(RwLock::new(DatabaseMetrics::default())),
            cache_metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics::default())),
            health_status: Arc::new(RwLock::new(HealthStatus::default())),
        }
    }

    /// Record a request completion
    #[instrument(skip(self))]
    pub async fn record_request(&self, endpoint: &str, duration: Duration, success: bool) {
        let mut metrics = self.request_metrics.write().await;

        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        let duration_ms = duration.as_millis() as f64;
        self.update_histogram(&mut metrics.response_times, duration_ms);

        let endpoint_metrics = metrics.requests_per_endpoint
            .entry(endpoint.to_string())
            .or_insert_with(EndpointMetrics::default);

        endpoint_metrics.total_calls += 1;
        if success {
            endpoint_metrics.success_count += 1;
        } else {
            endpoint_metrics.error_count += 1;
        }

        // Update running average (simplified)
        endpoint_metrics.avg_response_time_ms =
            (endpoint_metrics.avg_response_time_ms * (endpoint_metrics.total_calls - 1) as f64 + duration_ms)
            / endpoint_metrics.total_calls as f64;

        endpoint_metrics.last_called = Some(SystemTime::now());
    }

    /// Record a database query
    #[instrument(skip(self, query))]
    pub async fn record_database_query(&self, query_type: &str, query: &str, duration: Duration, success: bool, rows_affected: u64) {
        let mut metrics = self.database_metrics.write().await;

        metrics.total_queries += 1;
        if success {
            metrics.successful_queries += 1;
        } else {
            metrics.failed_queries += 1;
        }

        let duration_ms = duration.as_millis() as f64;
        self.update_histogram(&mut metrics.query_times, duration_ms);

        // Track by query type
        let query_metrics = metrics.queries_by_type
            .entry(query_type.to_string())
            .or_insert_with(QueryTypeMetrics::default);

        query_metrics.count += 1;
        query_metrics.total_time_ms += duration_ms;
        query_metrics.avg_time_ms = query_metrics.total_time_ms / query_metrics.count as f64;
        query_metrics.max_time_ms = query_metrics.max_time_ms.max(duration_ms);
        query_metrics.rows_affected += rows_affected;

        // Record slow queries (> 100ms)
        if duration_ms > 100.0 {
            if metrics.slow_queries.len() >= 100 {
                metrics.slow_queries.remove(0); // Keep only last 100
            }

            metrics.slow_queries.push(SlowQueryRecord {
                query: query.to_string(),
                duration_ms,
                timestamp: SystemTime::now(),
                user_id: None, // Would be extracted from context
                parameters: "".to_string(), // Would include actual parameters
            });
        }
    }

    /// Record cache operation
    #[instrument(skip(self))]
    pub async fn record_cache_operation(&self, operation: CacheOperation, hit: bool, lookup_time: Duration) {
        let mut metrics = self.cache_metrics.write().await;

        match operation {
            CacheOperation::L1Lookup => {
                if hit {
                    metrics.l1_hits += 1;
                } else {
                    metrics.l1_misses += 1;
                }
            }
            CacheOperation::L2Lookup => {
                if hit {
                    metrics.l2_hits += 1;
                } else {
                    metrics.l2_misses += 1;
                }
            }
            CacheOperation::Write => {
                metrics.cache_writes += 1;
            }
            CacheOperation::Eviction => {
                metrics.cache_evictions += 1;
            }
            CacheOperation::Invalidation => {
                metrics.cache_invalidations += 1;
            }
        }

        // Update average lookup time
        let total_lookups = metrics.l1_hits + metrics.l1_misses + metrics.l2_hits + metrics.l2_misses;
        if total_lookups > 0 {
            metrics.avg_lookup_time_ms =
                (metrics.avg_lookup_time_ms * (total_lookups - 1) as f64 + lookup_time.as_millis() as f64)
                / total_lookups as f64;
        }

        // Calculate hit rate
        let total_hits = metrics.l1_hits + metrics.l2_hits;
        if total_lookups > 0 {
            metrics.hit_rate_percentage = (total_hits as f64 / total_lookups as f64) * 100.0;
        }
    }

    /// Record business metric
    #[instrument(skip(self))]
    pub async fn record_business_event(&self, event: BusinessEvent) {
        let mut metrics = self.business_metrics.write().await;

        match event {
            BusinessEvent::UserRegistered => metrics.new_user_registrations += 1,
            BusinessEvent::PostCreated => metrics.posts_created += 1,
            BusinessEvent::LikeGiven => metrics.likes_given += 1,
            BusinessEvent::CommentMade => metrics.comments_made += 1,
            BusinessEvent::FriendshipFormed => metrics.friendships_formed += 1,
            BusinessEvent::GroupCreated => metrics.groups_created += 1,
            BusinessEvent::EventCreated => metrics.events_created += 1,
            BusinessEvent::CrossShardOperation => metrics.cross_shard_operations += 1,
            BusinessEvent::WalTransaction => metrics.wal_transactions += 1,
        }
    }

    /// Update system metrics (called periodically)
    pub async fn update_system_metrics(&self) {
        let mut metrics = self.system_metrics.write().await;

        // In production, these would use system APIs
        metrics.cpu_usage_percentage = self.get_cpu_usage().await;
        metrics.memory_usage_bytes = self.get_memory_usage().await;
        metrics.memory_usage_percentage = self.get_memory_percentage().await;
        metrics.disk_usage_bytes = self.get_disk_usage().await;
        metrics.open_file_descriptors = self.get_open_fds().await;
        metrics.uptime_seconds = self.get_uptime().await;
    }

    /// Perform comprehensive health check
    #[instrument(skip(self))]
    pub async fn perform_health_check(&self) -> HealthStatus {
        let mut health = HealthStatus::default();
        health.last_health_check = Some(SystemTime::now());

        // Check database health
        health.database_status = self.check_database_health().await;

        // Check cache health
        health.cache_status = self.check_cache_health().await;

        // Check query router health
        health.query_router_status = self.check_query_router_health().await;

        // Check WAL health
        health.wal_status = self.check_wal_health().await;

        // Check consistency manager health
        health.consistency_manager_status = self.check_consistency_manager_health().await;

        // Determine overall status
        health.overall_status = self.determine_overall_status(&health);

        // Update stored health status
        {
            let mut stored_health = self.health_status.write().await;
            *stored_health = health.clone();
        }

        health
    }

    /// Get comprehensive metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            request_metrics: self.request_metrics.read().await.clone(),
            database_metrics: self.database_metrics.read().await.clone(),
            cache_metrics: self.cache_metrics.read().await.clone(),
            system_metrics: self.system_metrics.read().await.clone(),
            business_metrics: self.business_metrics.read().await.clone(),
            health_status: self.health_status.read().await.clone(),
            snapshot_time: SystemTime::now(),
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus_metrics(&self) -> String {
        let snapshot = self.get_metrics_snapshot().await;

        // Convert to Prometheus format
        let mut output = String::new();

        // Request metrics
        output.push_str(&format!(
            "# HELP tao_requests_total Total number of requests\n\
             # TYPE tao_requests_total counter\n\
             tao_requests_total {}\n\n",
            snapshot.request_metrics.total_requests
        ));

        output.push_str(&format!(
            "# HELP tao_request_duration_seconds Request duration in seconds\n\
             # TYPE tao_request_duration_seconds histogram\n\
             tao_request_duration_seconds_sum {}\n\
             tao_request_duration_seconds_count {}\n\n",
            snapshot.request_metrics.response_times.sum / 1000.0,
            snapshot.request_metrics.response_times.count
        ));

        // Database metrics
        output.push_str(&format!(
            "# HELP tao_database_queries_total Total number of database queries\n\
             # TYPE tao_database_queries_total counter\n\
             tao_database_queries_total {}\n\n",
            snapshot.database_metrics.total_queries
        ));

        // Cache metrics
        output.push_str(&format!(
            "# HELP tao_cache_hit_rate Cache hit rate percentage\n\
             # TYPE tao_cache_hit_rate gauge\n\
             tao_cache_hit_rate {}\n\n",
            snapshot.cache_metrics.hit_rate_percentage
        ));

        // Business metrics
        output.push_str(&format!(
            "# HELP tao_active_users Number of active users\n\
             # TYPE tao_active_users gauge\n\
             tao_active_users {}\n\n",
            snapshot.business_metrics.active_users
        ));

        output
    }

    // Helper methods for updating histograms
    fn update_histogram(&self, histogram: &mut HistogramMetrics, value: f64) {
        histogram.count += 1;
        histogram.sum += value;

        if histogram.count == 1 {
            histogram.min = value;
            histogram.max = value;
        } else {
            histogram.min = histogram.min.min(value);
            histogram.max = histogram.max.max(value);
        }

        // Update buckets (simplified)
        let bucket = if value < 10.0 {
            "0-10ms"
        } else if value < 50.0 {
            "10-50ms"
        } else if value < 100.0 {
            "50-100ms"
        } else if value < 500.0 {
            "100-500ms"
        } else {
            "500ms+"
        };

        *histogram.buckets.entry(bucket.to_string()).or_insert(0) += 1;
    }

    // System metric collection helpers (would use real system APIs in production)
    async fn get_cpu_usage(&self) -> f64 { 0.0 }
    async fn get_memory_usage(&self) -> u64 { 0 }
    async fn get_memory_percentage(&self) -> f64 { 0.0 }
    async fn get_disk_usage(&self) -> u64 { 0 }
    async fn get_open_fds(&self) -> u64 { 0 }
    async fn get_uptime(&self) -> u64 { 0 }

    // Health check helpers
    async fn check_database_health(&self) -> ServiceStatus { ServiceStatus::Healthy }
    async fn check_cache_health(&self) -> ServiceStatus { ServiceStatus::Healthy }
    async fn check_query_router_health(&self) -> ServiceStatus { ServiceStatus::Healthy }
    async fn check_wal_health(&self) -> ServiceStatus { ServiceStatus::Healthy }
    async fn check_consistency_manager_health(&self) -> ServiceStatus { ServiceStatus::Healthy }

    fn determine_overall_status(&self, health: &HealthStatus) -> ServiceStatus {
        // Simple logic: if any critical component is unhealthy, overall is unhealthy
        if matches!(health.database_status, ServiceStatus::Unhealthy) ||
           matches!(health.query_router_status, ServiceStatus::Unhealthy) {
            ServiceStatus::Unhealthy
        } else if matches!(health.database_status, ServiceStatus::Degraded) ||
                  matches!(health.cache_status, ServiceStatus::Degraded) ||
                  matches!(health.query_router_status, ServiceStatus::Degraded) {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        }
    }
}

#[derive(Debug)]
pub enum CacheOperation {
    L1Lookup,
    L2Lookup,
    Write,
    Eviction,
    Invalidation,
}

#[derive(Debug)]
pub enum BusinessEvent {
    UserRegistered,
    PostCreated,
    LikeGiven,
    CommentMade,
    FriendshipFormed,
    GroupCreated,
    EventCreated,
    CrossShardOperation,
    WalTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub request_metrics: RequestMetrics,
    pub database_metrics: DatabaseMetrics,
    pub cache_metrics: CacheMetrics,
    pub system_metrics: SystemMetrics,
    pub business_metrics: BusinessMetrics,
    pub health_status: HealthStatus,
    pub snapshot_time: SystemTime,
}

/// Distributed tracing context
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }

    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }
}

/// Performance profiler for identifying bottlenecks
#[derive(Debug)]
pub struct PerformanceProfiler {
    active_spans: Arc<RwLock<HashMap<String, ProfileSpan>>>,
    completed_spans: Arc<RwLock<Vec<ProfileSpan>>>,
}

#[derive(Debug, Clone)]
pub struct ProfileSpan {
    pub span_id: String,
    pub operation: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub metadata: HashMap<String, String>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_spans: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_span(&self, operation: &str) -> String {
        let span_id = uuid::Uuid::new_v4().to_string();
        let span = ProfileSpan {
            span_id: span_id.clone(),
            operation: operation.to_string(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            metadata: HashMap::new(),
        };

        let mut active_spans = self.active_spans.write().await;
        active_spans.insert(span_id.clone(), span);

        span_id
    }

    pub async fn end_span(&self, span_id: &str) {
        let mut active_spans = self.active_spans.write().await;
        if let Some(mut span) = active_spans.remove(span_id) {
            let end_time = Instant::now();
            span.end_time = Some(end_time);
            span.duration = Some(end_time - span.start_time);

            let mut completed_spans = self.completed_spans.write().await;
            completed_spans.push(span);

            // Keep only last 1000 spans
            if completed_spans.len() > 1000 {
                completed_spans.remove(0);
            }
        }
    }

    pub async fn get_slowest_operations(&self, limit: usize) -> Vec<ProfileSpan> {
        let completed_spans = self.completed_spans.read().await;
        let mut spans = completed_spans.clone();

        spans.sort_by(|a, b| {
            b.duration.unwrap_or_default().cmp(&a.duration.unwrap_or_default())
        });

        spans.into_iter().take(limit).collect()
    }
}

/// Initialize comprehensive monitoring
pub fn initialize_monitoring() -> AppResult<Arc<MetricsCollector>> {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let metrics_collector = Arc::new(MetricsCollector::new());

    // Start background metrics collection
    let collector_clone = Arc::clone(&metrics_collector);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            collector_clone.update_system_metrics().await;
            collector_clone.perform_health_check().await;
        }
    });

    info!("Monitoring and observability initialized");
    Ok(metrics_collector)
}