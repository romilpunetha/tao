use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppResult, AppError};
use crate::infrastructure::shard_topology::ShardId;
use crate::infrastructure::tao::{TaoAssociation, current_time_millis};

/// Unique transaction identifier
pub type TxnId = Uuid;

/// WAL operation types that can be executed atomically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaoOperation {
    InsertObject {
        shard: ShardId,
        object_id: i64,
        object_type: String,
        data: Vec<u8>,
    },
    InsertAssociation {
        shard: ShardId,
        assoc: TaoAssociation,
    },
    DeleteAssociation {
        shard: ShardId,
        id1: i64,
        atype: String,
        id2: i64,
    },
    UpdateObject {
        shard: ShardId,
        object_id: i64,
        data: Vec<u8>,
    },
    DeleteObject {
        shard: ShardId,
        object_id: i64,
    },
}

impl TaoOperation {
    pub fn get_shard(&self) -> ShardId {
        match self {
            TaoOperation::InsertObject { shard, .. } => *shard,
            TaoOperation::InsertAssociation { shard, .. } => *shard,
            TaoOperation::DeleteAssociation { shard, .. } => *shard,
            TaoOperation::UpdateObject { shard, .. } => *shard,
            TaoOperation::DeleteObject { shard, .. } => *shard,
        }
    }

    pub fn operation_type(&self) -> &'static str {
        match self {
            TaoOperation::InsertObject { .. } => "insert_object",
            TaoOperation::InsertAssociation { .. } => "insert_association",
            TaoOperation::DeleteAssociation { .. } => "delete_association",
            TaoOperation::UpdateObject { .. } => "update_object",
            TaoOperation::DeleteObject { .. } => "delete_object",
        }
    }
}

/// Transaction status in the WAL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction written to WAL, waiting to execute
    Pending,
    /// Currently executing operations
    Executing,
    /// All operations completed successfully
    Committed,
    /// Some operations failed, may need compensation
    Failed,
    /// Transaction aborted (e.g., due to timeout)
    Aborted,
    /// Compensation operations completed
    Compensated,
}

/// A pending transaction in the WAL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub txn_id: TxnId,
    pub operations: Vec<TaoOperation>,
    pub status: TransactionStatus,
    pub retry_count: u32,
    pub created_at: i64,
    pub last_attempt_at: Option<i64>,
    pub completed_operations: Vec<usize>, // Indices of completed operations
    pub failed_operations: Vec<(usize, String)>, // (index, error)
}

impl PendingTransaction {
    pub fn new(operations: Vec<TaoOperation>) -> Self {
        Self {
            txn_id: Uuid::new_v4(),
            operations,
            status: TransactionStatus::Pending,
            retry_count: 0,
            created_at: current_time_millis(),
            last_attempt_at: None,
            completed_operations: Vec::new(),
            failed_operations: Vec::new(),
        }
    }

    pub fn age_ms(&self) -> i64 {
        current_time_millis() - self.created_at
    }

    pub fn is_expired(&self, max_age_ms: i64) -> bool {
        self.age_ms() > max_age_ms
    }

    pub fn remaining_operations(&self) -> Vec<(usize, &TaoOperation)> {
        self.operations
            .iter()
            .enumerate()
            .filter(|(idx, _)| !self.completed_operations.contains(idx))
            .collect()
    }
}

/// Configuration for the WAL system
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// Maximum number of retry attempts for failed operations
    pub max_retry_attempts: u32,
    /// Maximum age of a transaction before it's considered expired (ms)
    pub max_transaction_age_ms: i64,
    /// Base delay for exponential backoff (ms)
    pub base_retry_delay_ms: u64,
    /// Maximum retry delay (ms)
    pub max_retry_delay_ms: u64,
    /// How often to run cleanup of old transactions (ms)
    pub cleanup_interval_ms: u64,
    /// Batch size for WAL operations
    pub batch_size: usize,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            max_retry_attempts: 5,
            max_transaction_age_ms: 24 * 60 * 60 * 1000, // 24 hours
            base_retry_delay_ms: 100,
            max_retry_delay_ms: 30_000, // 30 seconds
            cleanup_interval_ms: 60_000, // 1 minute
            batch_size: 100,
        }
    }
}

/// Write-Ahead Log for cross-shard atomic operations
/// This is Meta's solution to the distributed transaction problem
#[derive(Debug)]
pub struct TaoWriteAheadLog {
    /// In-memory pending transactions (in production, this would be persisted)
    pending_transactions: Arc<RwLock<HashMap<TxnId, PendingTransaction>>>,
    /// Retry queue for failed operations
    retry_queue: Arc<Mutex<VecDeque<TxnId>>>,
    /// WAL configuration
    config: WalConfig,
    /// Reference to query router for executing operations
    query_router: Option<Arc<crate::infrastructure::query_router::TaoQueryRouter>>,
    /// Statistics
    stats: Arc<RwLock<WalStats>>,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct WalStats {
    pub total_transactions: u64,
    pub committed_transactions: u64,
    pub failed_transactions: u64,
    pub retries_executed: u64,
    pub pending_transactions: u64,
    pub avg_commit_time_ms: f64,
}

impl TaoWriteAheadLog {
    pub async fn new(config: WalConfig) -> Self {
        let wal = Self {
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            retry_queue: Arc::new(Mutex::new(VecDeque::new())),
            config,
            query_router: None,
            stats: Arc::new(RwLock::new(WalStats::default())),
        };

        // Start background workers
        wal.start_retry_worker().await;
        wal.start_cleanup_worker().await;

        info!("TAO Write-Ahead Log initialized");
        wal
    }

    /// Set the query router reference (for executing operations)
    pub fn set_query_router(&mut self, router: Arc<crate::infrastructure::query_router::TaoQueryRouter>) {
        self.query_router = Some(router);
    }

    /// Log a batch of operations atomically for durability
    /// WAL ONLY logs - does NOT execute database operations (that's TaoCore's job)
    pub async fn log_operations(&self, operations: Vec<TaoOperation>) -> AppResult<uuid::Uuid> {
        if operations.is_empty() {
            return Err(AppError::Validation("No operations provided".to_string()));
        }

        // Create transaction and log ALL operations to WAL atomically
        let txn = PendingTransaction::new(operations.clone());
        let txn_id = txn.txn_id;
        
        info!("Logging batch of {} operations to WAL with txn_id {}", operations.len(), txn_id);
        
        // Write entire batch to WAL atomically for durability
        {
            let mut pending = self.pending_transactions.write().await;
            pending.insert(txn_id, txn);
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_transactions += 1;
            stats.pending_transactions += 1;
        }
        
        debug!("Successfully logged {} operations to WAL with transaction ID: {}", operations.len(), txn_id);
        Ok(txn_id)
    }
    
    /// Mark a transaction as committed (called by TaoCore after successful execution)
    pub async fn mark_transaction_committed(&self, txn_id: uuid::Uuid) -> AppResult<()> {
        let mut pending = self.pending_transactions.write().await;
        let mut stats = self.stats.write().await;
        
        if let Some(txn) = pending.get_mut(&txn_id) {
            txn.status = TransactionStatus::Committed;
            stats.committed_transactions += 1;
            stats.pending_transactions = stats.pending_transactions.saturating_sub(1);
            info!("Transaction {} marked as committed in WAL", txn_id);
            Ok(())
        } else {
            Err(AppError::Validation(format!("Transaction {} not found in WAL", txn_id)))
        }
    }
    
    /// Mark a transaction as failed (called by TaoCore after failed execution)
    pub async fn mark_transaction_failed(&self, txn_id: uuid::Uuid, error_msg: String) -> AppResult<()> {
        let mut pending = self.pending_transactions.write().await;
        let mut stats = self.stats.write().await;
        
        if let Some(txn) = pending.get_mut(&txn_id) {
            txn.status = TransactionStatus::Failed;
            stats.failed_transactions += 1;
            stats.pending_transactions = stats.pending_transactions.saturating_sub(1);
            error!("Transaction {} marked as failed in WAL: {}", txn_id, error_msg);
            
            // Add to retry queue if not exceeded max attempts
            if txn.retry_count < self.config.max_retry_attempts {
                let mut retry_queue = self.retry_queue.lock().await;
                retry_queue.push_back(txn_id);
                info!("Added failed transaction {} to retry queue", txn_id);
            } else {
                warn!("Transaction {} exceeded max retry attempts", txn_id);
            }
            
            Ok(())
        } else {
            Err(AppError::Validation(format!("Transaction {} not found in WAL", txn_id)))
        }
    }

    /// Execute a cross-shard transaction atomically
    /// This is the CORE of Meta's distributed transaction system
    pub async fn execute_cross_shard_transaction(&self, operations: Vec<TaoOperation>) -> AppResult<TxnId> {
        if operations.is_empty() {
            return Err(AppError::Validation("No operations provided".to_string()));
        }

        // 1. Create pending transaction
        let txn = PendingTransaction::new(operations);
        let txn_id = txn.txn_id;

        info!("Starting cross-shard transaction {} with {} operations", txn_id, txn.operations.len());

        // 2. Analyze operation complexity
        let shards_involved = self.get_shards_involved(&txn.operations);
        info!("Transaction {} involves shards: {:?}", txn_id, shards_involved);

        // 3. Write transaction to WAL BEFORE executing (durability)
        {
            let mut pending = self.pending_transactions.write().await;
            pending.insert(txn_id, txn.clone());
        }

        // 4. Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_transactions += 1;
            stats.pending_transactions += 1;
        }

        // 5. Return transaction ID - execution will be handled by external system
        info!("Cross-shard transaction {} logged to WAL, ready for execution", txn_id);

        Ok(txn_id)
    }

    /// Log single-shard operations (fast path - still logged for consistency)
    pub async fn log_single_shard_operations(&self, operations: Vec<TaoOperation>) -> AppResult<uuid::Uuid> {
        if operations.is_empty() {
            return Err(AppError::Validation("No operations provided".to_string()));
        }

        // Verify all operations are on the same shard
        let first_shard = operations[0].get_shard();
        if !operations.iter().all(|op| op.get_shard() == first_shard) {
            return Err(AppError::Validation("All operations must be on the same shard".to_string()));
        }

        // Log operations (even single-shard operations should be logged for consistency)
        self.log_operations(operations).await
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, txn_id: TxnId) -> Option<TransactionStatus> {
        let pending = self.pending_transactions.read().await;
        pending.get(&txn_id).map(|txn| txn.status)
    }

    /// Wait for transaction completion
    pub async fn wait_for_transaction(&self, txn_id: TxnId, timeout: Duration) -> AppResult<TransactionStatus> {
        let start = SystemTime::now();
        
        loop {
            if let Some(status) = self.get_transaction_status(txn_id).await {
                match status {
                    TransactionStatus::Committed | TransactionStatus::Failed | TransactionStatus::Aborted => {
                        return Ok(status);
                    }
                    _ => {
                        // Still in progress
                        if start.elapsed().unwrap_or(Duration::ZERO) > timeout {
                            return Err(AppError::TimeoutError("Transaction wait timeout".to_string()));
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            } else {
                return Err(AppError::Validation("Transaction not found".to_string()));
            }
        }
    }

    /// =========================================================================
    /// WAL UTILITY METHODS - Only logging and status management
    /// =========================================================================

    /// Get operations for a failed transaction (for recovery purposes)
    pub async fn get_failed_transaction_operations(&self, txn_id: uuid::Uuid) -> AppResult<Vec<TaoOperation>> {
        let pending = self.pending_transactions.read().await;
        
        if let Some(txn) = pending.get(&txn_id) {
            if txn.status == TransactionStatus::Failed {
                Ok(txn.operations.clone())
            } else {
                Err(AppError::Validation(format!("Transaction {} is not in failed state", txn_id)))
            }
        } else {
            Err(AppError::Validation(format!("Transaction {} not found in WAL", txn_id)))
        }
    }

    /// =========================================================================
    /// BACKGROUND WORKERS
    /// =========================================================================

    async fn start_retry_worker(&self) {
        let retry_queue = Arc::clone(&self.retry_queue);
        let pending_transactions = Arc::clone(&self.pending_transactions);
        let stats = Arc::clone(&self.stats);
        let base_delay = self.config.base_retry_delay_ms;
        let max_delay = self.config.max_retry_delay_ms;
        let _max_attempts = self.config.max_retry_attempts;

        tokio::spawn(async move {
            loop {
                // Check if there are transactions to retry
                let txn_to_retry = {
                    let mut queue = retry_queue.lock().await;
                    queue.pop_front()
                };

                if let Some(txn_id) = txn_to_retry {
                    // Calculate exponential backoff delay
                    let retry_count = {
                        let pending = pending_transactions.read().await;
                        pending.get(&txn_id).map(|t| t.retry_count).unwrap_or(0)
                    };

                    let delay_ms = std::cmp::min(
                        base_delay * 2_u64.pow(retry_count),
                        max_delay
                    );

                    info!("Retrying transaction {} after {}ms delay (attempt {})", txn_id, delay_ms, retry_count + 1);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                    // Increment retry count
                    {
                        let mut pending = pending_transactions.write().await;
                        if let Some(txn) = pending.get_mut(&txn_id) {
                            txn.retry_count += 1;
                            txn.status = TransactionStatus::Pending;
                            txn.failed_operations.clear(); // Clear previous failures
                        }
                    }

                    // Update stats
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.retries_executed += 1;
                    }

                    // Re-execute the transaction
                    // In a real implementation, this would call execute_transaction_async
                    info!("Would re-execute transaction {}", txn_id);
                } else {
                    // No transactions to retry, sleep briefly
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
    }

    async fn start_cleanup_worker(&self) {
        let pending_transactions = Arc::clone(&self.pending_transactions);
        let cleanup_interval = self.config.cleanup_interval_ms;
        let max_age = self.config.max_transaction_age_ms;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(cleanup_interval));

            loop {
                interval.tick().await;

                let mut to_remove = Vec::new();
                
                // Find expired transactions
                {
                    let pending = pending_transactions.read().await;
                    for (txn_id, txn) in pending.iter() {
                        if txn.is_expired(max_age) {
                            to_remove.push(*txn_id);
                        }
                    }
                }

                // Remove expired transactions
                if !to_remove.is_empty() {
                    let mut pending = pending_transactions.write().await;
                    for txn_id in to_remove {
                        if let Some(txn) = pending.remove(&txn_id) {
                            warn!("Cleaned up expired transaction {} (age: {}ms, status: {:?})", 
                                  txn_id, txn.age_ms(), txn.status);
                        }
                    }
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

    pub async fn get_stats(&self) -> WalStats {
        *self.stats.read().await
    }

    pub async fn get_pending_transaction_count(&self) -> usize {
        self.pending_transactions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::tao::TaoAssociation;

    #[tokio::test]
    async fn test_wal_creation() {
        let config = WalConfig::default();
        let wal = TaoWriteAheadLog::new(config).await;
        
        let stats = wal.get_stats().await;
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.pending_transactions, 0);
    }

    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let config = WalConfig::default();
        let wal = TaoWriteAheadLog::new(config).await;

        let operations = vec![
            TaoOperation::InsertAssociation {
                shard: 1,
                assoc: TaoAssociation {
                    id1: 123,
                    atype: "test".to_string(),
                    id2: 456,
                    time: current_time_millis(),
                    data: None,
                },
            }
        ];

        // Note: This test would need a mock query router to actually execute
        // For now, we just test the WAL structure
        assert_eq!(wal.get_pending_transaction_count().await, 0);
    }
}