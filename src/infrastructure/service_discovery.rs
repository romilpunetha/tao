// Service Discovery and Advanced Load Balancing
// Production-grade service discovery with health-aware load balancing

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, instrument};

use crate::error::{AppResult, AppError};
use crate::infrastructure::shard_topology::ShardId;

/// Service registry for managing distributed TAO nodes
#[derive(Debug)]
pub struct ServiceRegistry {
    /// Registered services by service ID
    services: Arc<RwLock<HashMap<String, ServiceInstance>>>,
    /// Services grouped by type (e.g., "tao-shard", "tao-cache")
    services_by_type: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Health checker for monitoring service health
    health_checker: Arc<HealthChecker>,
    /// Load balancer for distributing requests
    load_balancer: Arc<LoadBalancer>,
    /// Service discovery configuration
    config: ServiceDiscoveryConfig,
}

/// Individual service instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub service_id: String,
    pub service_type: String,
    pub endpoint: String,
    pub metadata: HashMap<String, String>,
    pub health_status: HealthStatus,
    pub last_heartbeat: SystemTime,
    pub registration_time: SystemTime,
    pub load_metrics: LoadMetrics,
    pub shard_id: Option<ShardId>,
}

/// Health status of a service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Load metrics for intelligent load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_connections: u32,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub last_updated: SystemTime,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_connections: 0,
            requests_per_second: 0.0,
            average_response_time_ms: 0.0,
            error_rate: 0.0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Service discovery configuration
#[derive(Debug, Clone)]
pub struct ServiceDiscoveryConfig {
    pub heartbeat_interval: Duration,
    pub health_check_timeout: Duration,
    pub service_ttl: Duration,
    pub cleanup_interval: Duration,
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(5),
            service_ttl: Duration::from_secs(120),
            cleanup_interval: Duration::from_secs(60),
            load_balancing_strategy: LoadBalancingStrategy::WeightedRoundRobin,
        }
    }
}

impl ServiceRegistry {
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        let health_checker = Arc::new(HealthChecker::new(
            config.health_check_timeout,
            config.heartbeat_interval,
        ));
        
        let load_balancer = Arc::new(LoadBalancer::new(config.load_balancing_strategy.clone()));

        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            services_by_type: Arc::new(RwLock::new(HashMap::new())),
            health_checker,
            load_balancer,
            config,
        }
    }

    /// Register a new service instance
    #[instrument(skip(self))]
    pub async fn register_service(&self, service: ServiceInstance) -> AppResult<()> {
        let service_id = service.service_id.clone();
        let service_type = service.service_type.clone();

        // Store service instance
        {
            let mut services = self.services.write().await;
            services.insert(service_id.clone(), service);
        }

        // Update services by type index
        {
            let mut services_by_type = self.services_by_type.write().await;
            services_by_type
                .entry(service_type.clone())
                .or_insert_with(Vec::new)
                .push(service_id.clone());
        }

        info!("Registered service {} of type {}", service_id, service_type);
        Ok(())
    }

    /// Deregister a service instance
    #[instrument(skip(self))]
    pub async fn deregister_service(&self, service_id: &str) -> AppResult<()> {
        let service_type = {
            let mut services = self.services.write().await;
            if let Some(service) = services.remove(service_id) {
                service.service_type
            } else {
                return Err(AppError::NotFound(format!("Service {} not found", service_id)));
            }
        };

        // Update services by type index
        {
            let mut services_by_type = self.services_by_type.write().await;
            if let Some(service_list) = services_by_type.get_mut(&service_type) {
                service_list.retain(|id| id != service_id);
                if service_list.is_empty() {
                    services_by_type.remove(&service_type);
                }
            }
        }

        info!("Deregistered service {}", service_id);
        Ok(())
    }

    /// Update service health and load metrics
    #[instrument(skip(self))]
    pub async fn update_service_health(&self, service_id: &str, health: HealthStatus, load_metrics: LoadMetrics) -> AppResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.health_status = health;
            service.load_metrics = load_metrics;
            service.last_heartbeat = SystemTime::now();
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Service {} not found", service_id)))
        }
    }

    /// Discover services by type with load balancing
    #[instrument(skip(self))]
    pub async fn discover_service(&self, service_type: &str) -> AppResult<Option<ServiceInstance>> {
        let healthy_services = self.get_healthy_services_by_type(service_type).await?;
        
        if healthy_services.is_empty() {
            return Ok(None);
        }

        // Use load balancer to select best service
        let selected_service = self.load_balancer.select_service(healthy_services).await?;
        Ok(Some(selected_service))
    }

    /// Get all healthy services of a specific type
    pub async fn get_healthy_services_by_type(&self, service_type: &str) -> AppResult<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        let services_by_type = self.services_by_type.read().await;

        let mut healthy_services = Vec::new();

        if let Some(service_ids) = services_by_type.get(service_type) {
            for service_id in service_ids {
                if let Some(service) = services.get(service_id) {
                    if service.health_status == HealthStatus::Healthy {
                        healthy_services.push(service.clone());
                    }
                }
            }
        }

        Ok(healthy_services)
    }

    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> AppResult<Option<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.get(service_id).cloned())
    }

    /// List all services
    pub async fn list_services(&self) -> AppResult<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    /// Start background processes for service discovery
    pub async fn start_background_processes(&self) {
        self.start_health_monitoring().await;
        self.start_cleanup_process().await;
    }

    /// Start health monitoring background task
    async fn start_health_monitoring(&self) {
        let health_checker = Arc::clone(&self.health_checker);
        let services = Arc::clone(&self.services);
        let interval = self.config.heartbeat_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                let service_list = {
                    let services_guard = services.read().await;
                    services_guard.values().cloned().collect::<Vec<_>>()
                };

                for service in service_list {
                    health_checker.check_service_health(&service).await;
                }
            }
        });
    }

    /// Start cleanup process for expired services
    async fn start_cleanup_process(&self) {
        let services = Arc::clone(&self.services);
        let services_by_type = Arc::clone(&self.services_by_type);
        let service_ttl = self.config.service_ttl;
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(cleanup_interval);
            loop {
                interval_timer.tick().await;

                let expired_services = {
                    let services_guard = services.read().await;
                    let now = SystemTime::now();
                    
                    services_guard
                        .iter()
                        .filter_map(|(id, service)| {
                            if now.duration_since(service.last_heartbeat).unwrap_or_default() > service_ttl {
                                Some((id.clone(), service.service_type.clone()))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                };

                for (service_id, service_type) in expired_services {
                    // Remove expired service
                    {
                        let mut services_guard = services.write().await;
                        services_guard.remove(&service_id);
                    }

                    // Update services by type index
                    {
                        let mut services_by_type_guard = services_by_type.write().await;
                        if let Some(service_list) = services_by_type_guard.get_mut(&service_type) {
                            service_list.retain(|id| id != &service_id);
                            if service_list.is_empty() {
                                services_by_type_guard.remove(&service_type);
                            }
                        }
                    }

                    warn!("Removed expired service: {}", service_id);
                }
            }
        });
    }
}

/// Health checker for monitoring service health
#[derive(Debug)]
pub struct HealthChecker {
    timeout: Duration,
    check_interval: Duration,
    ongoing_checks: Arc<RwLock<HashMap<String, Instant>>>,
}

impl HealthChecker {
    pub fn new(timeout: Duration, check_interval: Duration) -> Self {
        Self {
            timeout,
            check_interval,
            ongoing_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check health of a specific service
    #[instrument(skip(self, service))]
    pub async fn check_service_health(&self, service: &ServiceInstance) -> HealthStatus {
        // Avoid concurrent health checks for the same service
        {
            let ongoing_checks = self.ongoing_checks.read().await;
            if let Some(last_check) = ongoing_checks.get(&service.service_id) {
                if last_check.elapsed() < self.check_interval {
                    return service.health_status.clone();
                }
            }
        }

        // Mark check as ongoing
        {
            let mut ongoing_checks = self.ongoing_checks.write().await;
            ongoing_checks.insert(service.service_id.clone(), Instant::now());
        }

        // Perform actual health check
        let health_status = self.perform_health_check(service).await;

        // Remove from ongoing checks
        {
            let mut ongoing_checks = self.ongoing_checks.write().await;
            ongoing_checks.remove(&service.service_id);
        }

        health_status
    }

    async fn perform_health_check(&self, service: &ServiceInstance) -> HealthStatus {
        // In production, this would make HTTP health check requests
        // For now, simulate based on load metrics and heartbeat
        
        let now = SystemTime::now();
        let heartbeat_age = now.duration_since(service.last_heartbeat).unwrap_or_default();
        
        if heartbeat_age > Duration::from_secs(60) {
            return HealthStatus::Unhealthy;
        }

        // Check load metrics for degraded performance
        if service.load_metrics.cpu_usage > 90.0 || 
           service.load_metrics.memory_usage > 95.0 ||
           service.load_metrics.error_rate > 10.0 {
            return HealthStatus::Degraded;
        }

        HealthStatus::Healthy
    }
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LeastResponseTime,
    HealthAware,
    ResourceAware,
}

/// Advanced load balancer with multiple strategies
#[derive(Debug)]
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    round_robin_state: Arc<RwLock<HashMap<String, usize>>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            round_robin_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Select the best service instance using the configured strategy
    #[instrument(skip(self, services))]
    pub async fn select_service(&self, services: Vec<ServiceInstance>) -> AppResult<ServiceInstance> {
        if services.is_empty() {
            return Err(AppError::ServiceUnavailable("No healthy services available".to_string()));
        }

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.round_robin_selection(services).await
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin_selection(services).await
            }
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections_selection(services).await
            }
            LoadBalancingStrategy::LeastResponseTime => {
                self.least_response_time_selection(services).await
            }
            LoadBalancingStrategy::HealthAware => {
                self.health_aware_selection(services).await
            }
            LoadBalancingStrategy::ResourceAware => {
                self.resource_aware_selection(services).await
            }
        };

        selected.ok_or_else(|| AppError::ServiceUnavailable("No suitable service found".to_string()))
    }

    async fn round_robin_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        if services.is_empty() {
            return None;
        }

        let service_type = &services[0].service_type;
        let mut state = self.round_robin_state.write().await;
        let current_index = state.entry(service_type.clone()).or_insert(0);
        
        let selected = services.get(*current_index).cloned();
        *current_index = (*current_index + 1) % services.len();
        
        selected
    }

    async fn weighted_round_robin_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        // Use inverse of load metrics as weights
        let mut weighted_services = Vec::new();
        
        for service in &services {
            let load_score = self.calculate_load_score(&service.load_metrics);
            let weight = if load_score > 0.0 { 1.0 / load_score } else { 1.0 };
            
            // Add service multiple times based on weight
            let repetitions = (weight * 10.0) as usize + 1;
            for _ in 0..repetitions {
                weighted_services.push(service.clone());
            }
        }

        self.round_robin_selection(weighted_services).await
    }

    async fn least_connections_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        services.into_iter()
            .min_by_key(|service| service.load_metrics.active_connections)
    }

    async fn least_response_time_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        services.into_iter()
            .min_by(|a, b| a.load_metrics.average_response_time_ms
                .partial_cmp(&b.load_metrics.average_response_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    async fn health_aware_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        // Prefer healthy services, then degraded, avoid unhealthy
        let healthy: Vec<_> = services.iter()
            .filter(|s| s.health_status == HealthStatus::Healthy)
            .cloned()
            .collect();

        if !healthy.is_empty() {
            return self.weighted_round_robin_selection(healthy).await;
        }

        let degraded: Vec<_> = services.into_iter()
            .filter(|s| s.health_status == HealthStatus::Degraded)
            .collect();

        if !degraded.is_empty() {
            return self.weighted_round_robin_selection(degraded).await;
        }

        None
    }

    async fn resource_aware_selection(&self, services: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        // Select based on comprehensive resource utilization
        services.into_iter()
            .min_by(|a, b| {
                let score_a = self.calculate_load_score(&a.load_metrics);
                let score_b = self.calculate_load_score(&b.load_metrics);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn calculate_load_score(&self, metrics: &LoadMetrics) -> f64 {
        // Weighted composite score of various metrics
        let cpu_weight = 0.3;
        let memory_weight = 0.2;
        let connection_weight = 0.2;
        let response_time_weight = 0.2;
        let error_weight = 0.1;

        (metrics.cpu_usage / 100.0) * cpu_weight +
        (metrics.memory_usage / 100.0) * memory_weight +
        (metrics.active_connections as f64 / 1000.0) * connection_weight +
        (metrics.average_response_time_ms / 1000.0) * response_time_weight +
        (metrics.error_rate / 100.0) * error_weight
    }
}

/// Service discovery client for TAO shards
pub struct TaoServiceDiscovery {
    registry: Arc<ServiceRegistry>,
}

impl TaoServiceDiscovery {
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        let registry = Arc::new(ServiceRegistry::new(config));
        Self { registry }
    }

    /// Register a TAO shard service
    pub async fn register_shard(&self, shard_id: ShardId, endpoint: String, metadata: HashMap<String, String>) -> AppResult<()> {
        let service = ServiceInstance {
            service_id: format!("tao-shard-{}", shard_id),
            service_type: "tao-shard".to_string(),
            endpoint,
            metadata,
            health_status: HealthStatus::Unknown,
            last_heartbeat: SystemTime::now(),
            registration_time: SystemTime::now(),
            load_metrics: LoadMetrics::default(),
            shard_id: Some(shard_id),
        };

        self.registry.register_service(service).await
    }

    /// Discover healthy shard for a given shard ID
    pub async fn discover_shard(&self, shard_id: ShardId) -> AppResult<Option<ServiceInstance>> {
        let services = self.registry.get_healthy_services_by_type("tao-shard").await?;
        
        for service in services {
            if service.shard_id == Some(shard_id) {
                return Ok(Some(service));
            }
        }

        Ok(None)
    }

    /// Get all available shards
    pub async fn list_available_shards(&self) -> AppResult<Vec<ServiceInstance>> {
        self.registry.get_healthy_services_by_type("tao-shard").await
    }

    /// Update shard health and metrics
    pub async fn update_shard_metrics(&self, shard_id: ShardId, health: HealthStatus, metrics: LoadMetrics) -> AppResult<()> {
        let service_id = format!("tao-shard-{}", shard_id);
        self.registry.update_service_health(&service_id, health, metrics).await
    }

    /// Start background service discovery processes
    pub async fn start(&self) {
        self.registry.start_background_processes().await;
        info!("TAO service discovery started");
    }
}

/// Configuration for TAO service discovery
impl ServiceDiscoveryConfig {
    pub fn for_production() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(15),
            health_check_timeout: Duration::from_secs(3),
            service_ttl: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(30),
            load_balancing_strategy: LoadBalancingStrategy::ResourceAware,
        }
    }

    pub fn for_development() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(10),
            service_ttl: Duration::from_secs(120),
            cleanup_interval: Duration::from_secs(60),
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}