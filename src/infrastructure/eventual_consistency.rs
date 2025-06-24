use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};
use serde::Serialize;

use crate::error::AppResult;
use crate::infrastructure::write_ahead_log::{TaoWriteAheadLog, TaoOperation, TxnId};
use crate::infrastructure::tao::{TaoAssociation, current_time_millis};
use crate::infrastructure::shard_topology::ShardId;

/// Eventual consistency manager for cross-shard operations
/// This handles the complex scenario where operations span multiple shards
/// and we need to guarantee eventual consistency despite partial failures
#[derive(Debug)]
pub struct EventualConsistencyManager {
    /// Reference to the WAL system
    wal: Arc<TaoWriteAheadLog>,
    /// Compensation operations for failed transactions
    compensation_queue: Arc<Mutex<VecDeque<CompensationTask>>>,
    /// Configuration
    config: ConsistencyConfig,
    /// Statistics
    stats: Arc<RwLock<ConsistencyStats>>,
}

#[derive(Debug, Clone)]
pub struct ConsistencyConfig {
    /// How long to wait before considering a cross-shard operation failed
    pub cross_shard_timeout_ms: u64,
    /// Maximum number of compensation attempts
    pub max_compensation_attempts: u32,
    /// Base delay for compensation retry (ms)
    pub compensation_retry_delay_ms: u64,
    /// How often to check for operations needing compensation
    pub compensation_check_interval_ms: u64,
}

impl Default for ConsistencyConfig {
    fn default() -> Self {
        Self {
            cross_shard_timeout_ms: 30_000, // 30 seconds
            max_compensation_attempts: 3,
            compensation_retry_delay_ms: 1_000, // 1 second
            compensation_check_interval_ms: 5_000, // 5 seconds
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsistencyStats {
    pub cross_shard_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub compensations_attempted: u64,
    pub compensations_successful: u64,
    pub pending_compensations: u64,
}

impl Default for ConsistencyStats {
    fn default() -> Self {
        Self {
            cross_shard_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            compensations_attempted: 0,
            compensations_successful: 0,
            pending_compensations: 0,
        }
    }
}

/// A compensation task for failed cross-shard operations
#[derive(Debug, Clone)]
pub struct CompensationTask {
    pub original_txn_id: TxnId,
    pub compensation_operations: Vec<TaoOperation>,
    pub attempt_count: u32,
    pub created_at: i64,
    pub last_attempt_at: Option<i64>,
    pub error_reason: String,
}

impl EventualConsistencyManager {
    pub async fn new(wal: Arc<TaoWriteAheadLog>, config: ConsistencyConfig) -> Self {
        let manager = Self {
            wal,
            compensation_queue: Arc::new(Mutex::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(ConsistencyStats::default())),
        };

        // Start background compensation worker
        manager.start_compensation_worker().await;

        info!("Eventual Consistency Manager initialized");
        manager
    }

    /// =========================================================================
    /// HIGH-LEVEL CONSISTENCY OPERATIONS
    /// =========================================================================

    /// Handle a follow relationship (classic cross-shard scenario)
    /// User A follows User B - this creates associations on both users' shards
    pub async fn handle_follow_relationship(&self, follower_id: i64, followee_id: i64) -> AppResult<TxnId> {
        info!("Creating follow relationship: {} -> {}", follower_id, followee_id);

        let operations = vec![
            // Add "following" association on follower's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_user(follower_id),
                assoc: TaoAssociation {
                    id1: follower_id,
                    atype: "following".to_string(),
                    id2: followee_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
            // Add "followers" association on followee's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_user(followee_id),
                assoc: TaoAssociation {
                    id1: followee_id,
                    atype: "followers".to_string(),
                    id2: follower_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
        ];

        self.execute_cross_shard_operation("follow_relationship", operations).await
    }

    /// Handle a like operation (user likes a post on different shard)
    pub async fn handle_like_operation(&self, user_id: i64, post_id: i64) -> AppResult<TxnId> {
        info!("Creating like operation: user {} likes post {}", user_id, post_id);

        let operations = vec![
            // Add "liked" association on user's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_user(user_id),
                assoc: TaoAssociation {
                    id1: user_id,
                    atype: "liked".to_string(),
                    id2: post_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
            // Add "liked_by" association on post's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_object(post_id),
                assoc: TaoAssociation {
                    id1: post_id,
                    atype: "liked_by".to_string(),
                    id2: user_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
        ];

        self.execute_cross_shard_operation("like_operation", operations).await
    }

    /// Handle group membership (user joins a group)
    pub async fn handle_group_membership(&self, user_id: i64, group_id: i64) -> AppResult<TxnId> {
        info!("Creating group membership: user {} joins group {}", user_id, group_id);

        let operations = vec![
            // Add "member_of" association on user's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_user(user_id),
                assoc: TaoAssociation {
                    id1: user_id,
                    atype: "member_of".to_string(),
                    id2: group_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
            // Add "members" association on group's shard
            TaoOperation::InsertAssociation {
                shard: self.get_shard_for_object(group_id),
                assoc: TaoAssociation {
                    id1: group_id,
                    atype: "members".to_string(),
                    id2: user_id,
                    time: current_time_millis(),
                    data: None,
                }
            },
        ];

        self.execute_cross_shard_operation("group_membership", operations).await
    }

    /// =========================================================================
    /// CORE CONSISTENCY LOGIC
    /// =========================================================================

    async fn execute_cross_shard_operation(&self, operation_name: &str, operations: Vec<TaoOperation>) -> AppResult<TxnId> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.cross_shard_operations += 1;
        }

        // Check if this is actually cross-shard
        let shards_involved = self.get_shards_involved(&operations);
        if shards_involved.len() == 1 {
            // Single shard - use fast path (still log for consistency)
            debug!("{} is single-shard, using fast path", operation_name);
            let txn_id = self.wal.log_single_shard_operations(operations).await?;
            
            let mut stats = self.stats.write().await;
            stats.successful_operations += 1;
            
            // Note: In a full implementation, we'd execute the operations here
            // and then mark the transaction as committed in WAL
            return Ok(txn_id);
        }

        info!("{} is cross-shard (shards: {:?}), using WAL", operation_name, shards_involved);

        // Execute via WAL
        let txn_id = self.wal.execute_cross_shard_transaction(operations.clone()).await?;

        // Monitor the transaction asynchronously
        self.monitor_transaction(txn_id, operation_name.to_string(), operations).await;

        Ok(txn_id)
    }

    async fn monitor_transaction(&self, txn_id: TxnId, operation_name: String, original_operations: Vec<TaoOperation>) {
        let wal = Arc::clone(&self.wal);
        let compensation_queue = Arc::clone(&self.compensation_queue);
        let stats = Arc::clone(&self.stats);
        let timeout = self.config.cross_shard_timeout_ms;

        tokio::spawn(async move {
            // Wait for transaction completion or timeout
            let timeout_duration = Duration::from_millis(timeout);
            
            match wal.wait_for_transaction(txn_id, timeout_duration).await {
                Ok(status) => {
                    match status {
                        crate::infrastructure::write_ahead_log::TransactionStatus::Committed => {
                            info!("{} transaction {} completed successfully", operation_name, txn_id);
                            let mut stats_guard = stats.write().await;
                            stats_guard.successful_operations += 1;
                        }
                        _ => {
                            error!("{} transaction {} failed with status {:?}", operation_name, txn_id, status);
                            let mut stats_guard = stats.write().await;
                            stats_guard.failed_operations += 1;

                            // Create compensation task
                            let compensation_ops = Self::create_compensation_operations(&original_operations);
                            if !compensation_ops.is_empty() {
                                let compensation_task = CompensationTask {
                                    original_txn_id: txn_id,
                                    compensation_operations: compensation_ops,
                                    attempt_count: 0,
                                    created_at: current_time_millis(),
                                    last_attempt_at: None,
                                    error_reason: format!("Transaction failed with status {:?}", status),
                                };

                                let mut queue = compensation_queue.lock().await;
                                queue.push_back(compensation_task);
                                stats_guard.pending_compensations += 1;
                                
                                warn!("Added compensation task for failed {} transaction {}", operation_name, txn_id);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("{} transaction {} monitoring failed: {}", operation_name, txn_id, e);
                    let mut stats_guard = stats.write().await;
                    stats_guard.failed_operations += 1;
                }
            }
        });
    }

    /// =========================================================================
    /// COMPENSATION OPERATIONS
    /// =========================================================================

    /// Create compensation operations for failed transactions
    /// This is where we implement "undo" logic for partially completed operations
    fn create_compensation_operations(failed_operations: &[TaoOperation]) -> Vec<TaoOperation> {
        let mut compensation_ops = Vec::new();

        for operation in failed_operations {
            match operation {
                TaoOperation::InsertAssociation { shard, assoc } => {
                    // Compensate by deleting the association
                    compensation_ops.push(TaoOperation::DeleteAssociation {
                        shard: *shard,
                        id1: assoc.id1,
                        atype: assoc.atype.clone(),
                        id2: assoc.id2,
                    });
                }
                TaoOperation::DeleteAssociation { shard, id1, atype, id2 } => {
                    // Compensate by re-inserting the association
                    // Note: We don't have the original data, so this is a limitation
                    compensation_ops.push(TaoOperation::InsertAssociation {
                        shard: *shard,
                        assoc: TaoAssociation {
                            id1: *id1,
                            atype: atype.clone(),
                            id2: *id2,
                            time: current_time_millis(),
                            data: None, // Lost original data - this is a known limitation
                        },
                    });
                }
                TaoOperation::InsertObject { .. } => {
                    // Object insertions are harder to compensate
                    // In practice, Meta might leave orphaned objects for cleanup
                    warn!("Cannot compensate object insertion - leaving orphaned object");
                }
                TaoOperation::UpdateObject { .. } => {
                    // Updates are very hard to compensate without version history
                    warn!("Cannot compensate object update - data may be inconsistent");
                }
                TaoOperation::DeleteObject { .. } => {
                    // Object deletions are hard to compensate without backup data
                    warn!("Cannot compensate object deletion - data may be lost");
                }
            }
        }

        compensation_ops
    }

    async fn start_compensation_worker(&self) {
        let compensation_queue = Arc::clone(&self.compensation_queue);
        let wal = Arc::clone(&self.wal);
        let stats = Arc::clone(&self.stats);
        let max_attempts = self.config.max_compensation_attempts;
        let retry_delay = self.config.compensation_retry_delay_ms;
        let check_interval = self.config.compensation_check_interval_ms;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(check_interval));

            loop {
                interval.tick().await;

                // Process compensation tasks
                let task_to_process = {
                    let mut queue = compensation_queue.lock().await;
                    queue.pop_front()
                };

                if let Some(mut task) = task_to_process {
                    if task.attempt_count >= max_attempts {
                        error!("Compensation task for transaction {} exceeded max attempts", task.original_txn_id);
                        let mut stats_guard = stats.write().await;
                        stats_guard.pending_compensations = stats_guard.pending_compensations.saturating_sub(1);
                        continue;
                    }

                    info!("Processing compensation task for transaction {} (attempt {})", 
                          task.original_txn_id, task.attempt_count + 1);

                    task.attempt_count += 1;
                    task.last_attempt_at = Some(current_time_millis());

                    // Update stats
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.compensations_attempted += 1;
                    }

                    // Execute compensation operations
                    match wal.execute_cross_shard_transaction(task.compensation_operations.clone()).await {
                        Ok(compensation_txn_id) => {
                            info!("Compensation transaction {} started for original transaction {}", 
                                  compensation_txn_id, task.original_txn_id);

                            // Wait for compensation to complete
                            let timeout = Duration::from_millis(retry_delay * 2);
                            match wal.wait_for_transaction(compensation_txn_id, timeout).await {
                                Ok(status) if matches!(status, crate::infrastructure::write_ahead_log::TransactionStatus::Committed) => {
                                    info!("Compensation successful for transaction {}", task.original_txn_id);
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.compensations_successful += 1;
                                    stats_guard.pending_compensations = stats_guard.pending_compensations.saturating_sub(1);
                                }
                                _ => {
                                    warn!("Compensation failed for transaction {}, will retry", task.original_txn_id);
                                    // Add back to queue for retry
                                    let mut queue = compensation_queue.lock().await;
                                    queue.push_back(task);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to start compensation for transaction {}: {}", task.original_txn_id, e);
                            // Add back to queue for retry
                            let mut queue = compensation_queue.lock().await;
                            queue.push_back(task);
                        }
                    }

                    // Add delay between compensation attempts
                    tokio::time::sleep(Duration::from_millis(retry_delay)).await;
                }
            }
        });
    }

    /// =========================================================================
    /// UTILITY METHODS
    /// =========================================================================

    fn get_shards_involved(&self, operations: &[TaoOperation]) -> Vec<ShardId> {
        let mut shards = std::collections::HashSet::new();
        for op in operations {
            shards.insert(op.get_shard());
        }
        shards.into_iter().collect()
    }

    // These would be implemented using the actual shard topology
    fn get_shard_for_user(&self, user_id: i64) -> ShardId {
        // Simplified - in reality this would use the shard topology
        (user_id % 16) as u16
    }

    fn get_shard_for_object(&self, object_id: i64) -> ShardId {
        // Extract shard from object ID (Meta's approach)
        ((object_id as u64) >> 12 & 0x3FF) as u16
    }

    pub async fn get_stats(&self) -> ConsistencyStats {
        self.stats.read().await.clone()
    }

    pub async fn get_pending_compensation_count(&self) -> usize {
        self.compensation_queue.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::write_ahead_log::WalConfig;

    #[tokio::test]
    async fn test_consistency_manager_creation() {
        let wal_config = WalConfig::default();
        let wal = Arc::new(TaoWriteAheadLog::new(wal_config).await);
        
        let consistency_config = ConsistencyConfig::default();
        let manager = EventualConsistencyManager::new(wal, consistency_config).await;
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.cross_shard_operations, 0);
        assert_eq!(stats.pending_compensations, 0);
    }

    #[test]
    fn test_compensation_operations() {
        let failed_ops = vec![
            TaoOperation::InsertAssociation {
                shard: 1,
                assoc: TaoAssociation {
                    id1: 123,
                    atype: "following".to_string(),
                    id2: 456,
                    time: current_time_millis(),
                    data: None,
                },
            }
        ];

        let compensation_ops = EventualConsistencyManager::create_compensation_operations(&failed_ops);
        assert_eq!(compensation_ops.len(), 1);
        
        match &compensation_ops[0] {
            TaoOperation::DeleteAssociation { id1, atype, id2, .. } => {
                assert_eq!(*id1, 123);
                assert_eq!(atype, "following");
                assert_eq!(*id2, 456);
            }
            _ => panic!("Expected DeleteAssociation compensation"),
        }
    }
}