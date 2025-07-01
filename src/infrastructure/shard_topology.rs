use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::error::{AppError, AppResult};

pub type ShardId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShardHealth {
    Healthy,
    Degraded,
    Failed,
    Recovering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: ShardId,
    pub health: ShardHealth,
    pub connection_string: String,
    pub region: String,
    pub replicas: Vec<ShardId>,
    pub last_health_check: i64,
    pub load_factor: f64, // 0.0 to 1.0
}

/// Consistent hash ring for shard distribution
/// Based on the same principles Meta uses for TAO sharding
#[derive(Debug)]
pub struct ConsistentHashRing {
    /// Virtual nodes per physical shard (increases distribution uniformity)
    virtual_nodes_per_shard: u32,
    /// Hash ring: hash_value -> shard_id
    ring: std::collections::BTreeMap<u64, ShardId>,
    /// Active shards in the ring
    shards: HashMap<ShardId, ShardInfo>,
}

impl ConsistentHashRing {
    pub fn new(virtual_nodes_per_shard: u32) -> Self {
        Self {
            virtual_nodes_per_shard,
            ring: std::collections::BTreeMap::new(),
            shards: HashMap::new(),
        }
    }

    /// Add a shard to the hash ring
    pub fn add_shard(&mut self, shard_info: ShardInfo) {
        let shard_id = shard_info.shard_id;

        // Add virtual nodes for this shard to improve distribution
        for i in 0..self.virtual_nodes_per_shard {
            let virtual_key = format!("shard_{}_vnode_{}", shard_id, i);
            let hash_value = self.hash_key(&virtual_key);
            self.ring.insert(hash_value, shard_id);
        }

        self.shards.insert(shard_id, shard_info);
        info!(
            "Added shard {} to hash ring with {} virtual nodes",
            shard_id, self.virtual_nodes_per_shard
        );
    }

    /// Remove a shard from the hash ring (e.g., during maintenance)
    pub fn remove_shard(&mut self, shard_id: ShardId) {
        // Remove all virtual nodes for this shard
        self.ring.retain(|_, &mut id| id != shard_id);
        self.shards.remove(&shard_id);
        warn!("Removed shard {} from hash ring", shard_id);
    }

    /// Get the primary shard for a given key
    pub fn get_shard(&self, key: &[u8]) -> Option<ShardId> {
        if self.ring.is_empty() {
            return None;
        }

        let hash_value = self.hash_key_bytes(key);

        // Find the first shard at or after this hash value
        if let Some((&_, &shard_id)) = self.ring.range(hash_value..).next() {
            Some(shard_id)
        } else {
            // Wrap around to the beginning of the ring
            self.ring.iter().next().map(|(_, &shard_id)| shard_id)
        }
    }

    /// Get replica shards for fault tolerance
    pub fn get_replica_shards(&self, primary_shard: ShardId, replica_count: usize) -> Vec<ShardId> {
        if replica_count == 0 || self.shards.len() <= 1 {
            return vec![];
        }

        let mut replicas = std::collections::HashSet::new();
        let shard_ids: Vec<ShardId> = self.shards.keys().copied().collect();

        if let Some(primary_index) = shard_ids.iter().position(|&id| id == primary_shard) {
            for i in 1..shard_ids.len() {
                if replicas.len() >= replica_count {
                    break;
                }
                let replica_index = (primary_index + i) % shard_ids.len();
                if let Some(&replica_shard) = shard_ids.get(replica_index) {
                    if replica_shard != primary_shard {
                        replicas.insert(replica_shard);
                    }
                }
            }
        }

        replicas.into_iter().collect()
    }

    /// Get healthy shards only
    pub fn get_healthy_shards(&self) -> Vec<ShardId> {
        self.shards
            .iter()
            .filter(|(_, info)| matches!(info.health, ShardHealth::Healthy))
            .map(|(&shard_id, _)| shard_id)
            .collect()
    }

    /// Update shard health status
    pub fn update_shard_health(&mut self, shard_id: ShardId, health: ShardHealth) {
        if let Some(shard_info) = self.shards.get_mut(&shard_id) {
            let old_health = shard_info.health;
            shard_info.health = health;
            shard_info.last_health_check =
                crate::infrastructure::tao_core::tao_core::current_time_millis();

            info!(
                "Shard {} health changed: {:?} -> {:?}",
                shard_id, old_health, health
            );

            // If shard became unhealthy, log warning
            if matches!(old_health, ShardHealth::Healthy) && !matches!(health, ShardHealth::Healthy)
            {
                warn!("Shard {} is now unhealthy: {:?}", shard_id, health);
            }
        }
    }

    /// Hash a string key
    fn hash_key(&self, key: &str) -> u64 {
        self.hash_key_bytes(key.as_bytes())
    }

    /// Hash byte array key using same algorithm Meta uses
    fn hash_key_bytes(&self, key: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_shard_info(&self, shard_id: ShardId) -> Option<&ShardInfo> {
        self.shards.get(&shard_id)
    }

    pub fn total_shards(&self) -> usize {
        self.shards.len()
    }
}

/// Main shard topology manager
/// This is the brain of TAO's sharding system
#[derive(Debug)]
pub struct ShardTopology {
    hash_ring: ConsistentHashRing,
    replication_factor: usize,
    /// Cache for owner_id -> shard mappings (performance optimization)
    owner_shard_cache: lru::LruCache<i64, ShardId>,
}

impl ShardTopology {
    pub fn new(replication_factor: usize) -> Self {
        Self {
            hash_ring: ConsistentHashRing::new(150), // 150 virtual nodes per shard
            replication_factor,
            owner_shard_cache: lru::LruCache::new(std::num::NonZeroUsize::new(10000).unwrap()),
        }
    }

    /// Core routing logic: determine which shard owns objects for a given user/entity
    /// This is the MOST CRITICAL function - it determines data placement
    pub fn get_shard_for_owner(&mut self, owner_id: i64) -> Option<ShardId> {
        // Check cache first
        if let Some(&cached_shard) = self.owner_shard_cache.get(&owner_id) {
            return Some(cached_shard);
        }

        // Hash the owner_id to determine shard placement
        let owner_key = owner_id.to_be_bytes(); // Big-endian for consistent hashing
        let shard_id = self.hash_ring.get_shard(&owner_key)?;

        // Cache the result
        self.owner_shard_cache.put(owner_id, shard_id);

        Some(shard_id)
    }

    /// Extract shard information from an existing object ID
    /// Meta embeds shard info in the object ID itself
    pub fn get_shard_for_object(&self, object_id: i64) -> ShardId {
        // Extract shard bits from object ID (bits 12-21)
        ((object_id as u64) >> 12 & 0x3FF) as u16
    }

    /// Get replicas for fault tolerance
    pub fn get_replica_shards(&self, primary_shard: ShardId) -> Vec<ShardId> {
        self.hash_ring
            .get_replica_shards(primary_shard, self.replication_factor)
    }

    /// Add a new shard to the topology
    pub fn add_shard(&mut self, shard_info: ShardInfo) {
        info!("Adding shard {} to topology", shard_info.shard_id);
        self.hash_ring.add_shard(shard_info);
        // Clear cache when topology changes
        self.owner_shard_cache.clear();
    }

    /// Remove a shard (e.g., for maintenance)
    pub fn remove_shard(&mut self, shard_id: ShardId) {
        warn!("Removing shard {} from topology", shard_id);
        self.hash_ring.remove_shard(shard_id);
        self.owner_shard_cache.clear();
    }

    /// Update shard health and handle failures
    pub fn update_shard_health(&mut self, shard_id: ShardId, health: ShardHealth) {
        self.hash_ring.update_shard_health(shard_id, health);

        // If shard failed, we might need to redirect traffic
        if matches!(health, ShardHealth::Failed) {
            self.handle_shard_failure(shard_id);
        }
    }

    /// Handle shard failures by redirecting to replicas
    fn handle_shard_failure(&mut self, failed_shard: ShardId) {
        error!("Handling failure of shard {}", failed_shard);

        // In a real system, this would:
        // 1. Redirect reads to replica shards
        // 2. Queue writes for when shard recovers
        // 3. Trigger alerting/monitoring
        // 4. Potentially initiate shard recovery

        let replicas = self.get_replica_shards(failed_shard);
        if replicas.is_empty() {
            error!(
                "CRITICAL: Shard {} failed with no available replicas!",
                failed_shard
            );
        } else {
            info!(
                "Shard {} failure - traffic can be redirected to replicas: {:?}",
                failed_shard, replicas
            );
        }
    }

    /// Get healthy shards for load balancing
    pub fn get_healthy_shards(&self) -> Vec<ShardId> {
        self.hash_ring.get_healthy_shards()
    }

    /// Get shard information
    pub fn get_shard_info(&self, shard_id: ShardId) -> Option<&ShardInfo> {
        self.hash_ring.get_shard_info(shard_id)
    }

    /// Get topology statistics
    pub fn get_stats(&self) -> TopologyStats {
        let total_shards = self.hash_ring.total_shards();
        let healthy_shards = self.get_healthy_shards().len();
        let cache_hit_rate = self.owner_shard_cache.len() as f64 / 10000.0;

        TopologyStats {
            total_shards,
            healthy_shards,
            failed_shards: total_shards - healthy_shards,
            replication_factor: self.replication_factor,
            cache_hit_rate,
        }
    }
}

/// Trait for managing shard topology and routing
#[async_trait]
pub trait ShardManager {
    async fn get_shard_for_owner(&self, owner_id: i64) -> AppResult<ShardId>;
    async fn get_shard_for_object(&self, object_id: i64) -> ShardId;
    async fn get_shard_info(&self, shard_id: ShardId) -> Option<ShardInfo>;
    async fn add_shard(&self, shard_info: ShardInfo);
    async fn remove_shard(&self, shard_id: ShardId);
    async fn get_healthy_shards(&self) -> Vec<ShardId>;
}

/// Implementation of ShardManager using consistent hashing
pub struct ConsistentHashingShardManager {
    topology: Arc<RwLock<ShardTopology>>,
}

impl ConsistentHashingShardManager {
    pub fn new(topology: Arc<RwLock<ShardTopology>>) -> Self {
        Self { topology }
    }
}

#[async_trait]
impl ShardManager for ConsistentHashingShardManager {
    async fn get_shard_for_owner(&self, owner_id: i64) -> AppResult<ShardId> {
        let mut topology = self.topology.write().await;
        topology
            .get_shard_for_owner(owner_id)
            .ok_or_else(|| AppError::Validation("No healthy shards available".to_string()))
    }

    async fn get_shard_for_object(&self, object_id: i64) -> ShardId {
        let topology = self.topology.read().await;
        topology.get_shard_for_object(object_id)
    }

    async fn get_shard_info(&self, shard_id: ShardId) -> Option<ShardInfo> {
        let topology = self.topology.read().await;
        topology.get_shard_info(shard_id).cloned()
    }

    async fn add_shard(&self, shard_info: ShardInfo) {
        let mut topology = self.topology.write().await;
        topology.add_shard(shard_info);
    }

    async fn remove_shard(&self, shard_id: ShardId) {
        let mut topology = self.topology.write().await;
        topology.remove_shard(shard_id);
    }

    async fn get_healthy_shards(&self) -> Vec<ShardId> {
        let topology = self.topology.read().await;
        topology.get_healthy_shards()
    }
}

#[derive(Debug, Serialize)]
pub struct TopologyStats {
    pub total_shards: usize,
    pub healthy_shards: usize,
    pub failed_shards: usize,
    pub replication_factor: usize,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hashing() {
        let mut ring = ConsistentHashRing::new(100);

        // Add some shards
        for i in 0..5 {
            let shard_info = ShardInfo {
                shard_id: i,
                health: ShardHealth::Healthy,
                connection_string: format!("shard_{}", i),
                region: "us-east-1".to_string(),
                replicas: vec![],
                last_health_check: 0,
                load_factor: 0.5,
            };
            ring.add_shard(shard_info);
        }

        // Test that same key always maps to same shard
        let key = b"user_12345";
        let shard1 = ring.get_shard(key);
        let shard2 = ring.get_shard(key);
        assert_eq!(shard1, shard2);

        // Test that different keys distribute across shards
        let mut shard_distribution = HashMap::new();
        for i in 0..1000 {
            let key = format!("user_{}", i);
            if let Some(shard) = ring.get_shard(key.as_bytes()) {
                *shard_distribution.entry(shard).or_insert(0) += 1;
            }
        }

        // Should have decent distribution (no shard should have >60% of keys)
        for (shard, count) in shard_distribution {
            assert!(count < 600, "Shard {} has too many keys: {}", shard, count);
        }
    }

    #[test]
    fn test_shard_topology() {
        let mut topology = ShardTopology::new(2);

        // Add test shards
        for i in 0..3 {
            let shard_info = ShardInfo {
                shard_id: i,
                health: ShardHealth::Healthy,
                connection_string: format!("postgresql://shard_{}", i),
                region: "us-east-1".to_string(),
                replicas: vec![],
                last_health_check: 0,
                load_factor: 0.3,
            };
            topology.add_shard(shard_info);
        }

        // Test owner-to-shard mapping
        let owner_id = 12345_i64;
        let shard1 = topology.get_shard_for_owner(owner_id);
        let shard2 = topology.get_shard_for_owner(owner_id);
        assert_eq!(shard1, shard2); // Should be consistent

        // Test object ID shard extraction
        let object_id = (42_u64 << 12) | 0x123; // Shard 42 embedded
        let extracted_shard = topology.get_shard_for_object(object_id as i64);
        assert_eq!(extracted_shard, 42);
    }
}
