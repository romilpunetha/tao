use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::infrastructure::tao_core::current_time_millis;
use crate::infrastructure::wal_storage::WalStorage;

/// Unique transaction identifier
pub type TxnId = Uuid;

/// WAL operation types that can be executed atomically
/// Shard routing is handled by tao_core during execution/replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaoOperation {
    InsertObject {
        object_id: i64,
        object_type: String,
        data: Vec<u8>,
    },
    InsertAssociation {
        assoc: TaoAssociation,
    },
    DeleteAssociation {
        id1: i64,
        atype: String,
        id2: i64,
    },
    UpdateObject {
        object_id: i64,
        data: Vec<u8>,
    },
    DeleteObject {
        object_id: i64,
    },
}

// Re-export the TaoAssociation for WAL to use
use crate::infrastructure::tao_core::TaoAssociation;

impl TaoOperation {
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
            max_retry_delay_ms: 30_000,  // 30 seconds
            cleanup_interval_ms: 60_000, // 1 minute
            batch_size: 100,
        }
    }
}

/// Write-Ahead Log for cross-shard atomic operations
/// This is a "dumb" logger with no routing logic or execution capability
#[derive(Debug)]
pub struct TaoWriteAheadLog {
    /// In-memory pending transactions (in production, this would be persisted)
    pending_transactions: Arc<RwLock<HashMap<TxnId, PendingTransaction>>>,
    /// Retry queue for failed operations
    retry_queue: Arc<Mutex<VecDeque<TxnId>>>,
    /// WAL configuration
    config: WalConfig,
    /// Persistent storage for the WAL
    storage: WalStorage,
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
    pub async fn new(config: WalConfig, storage_dir: &str) -> AppResult<Self> {
        let storage = WalStorage::new(storage_dir)?;
        let pending_transactions = storage.load_transactions()?;

        let wal = Self {
            pending_transactions: Arc::new(RwLock::new(pending_transactions)),
            retry_queue: Arc::new(Mutex::new(VecDeque::new())),
            config,
            storage,
            stats: Arc::new(RwLock::new(WalStats::default())),
        };

        info!(
            "TAO Write-Ahead Log initialized with {} pending transactions from storage",
            wal.pending_transactions.read().await.len()
        );
        Ok(wal)
    }

    /// Start the background cleanup worker
    pub async fn start_cleanup_worker(&self) {
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
                            warn!(
                                "Cleaned up expired transaction {} (age: {}ms, status: {:?})",
                                txn_id,
                                txn.age_ms(),
                                txn.status
                            );
                        }
                    }
                }
            }
        });
    }

    /// Log a batch of operations atomically for durability
    /// WAL ONLY logs - does NOT execute database operations
    pub async fn log_operations(&self, operations: Vec<TaoOperation>) -> AppResult<uuid::Uuid> {
        if operations.is_empty() {
            return Err(AppError::Validation("No operations provided".to_string()));
        }

        // Create transaction and log ALL operations to WAL atomically
        let txn = PendingTransaction::new(operations.clone());
        let txn_id = txn.txn_id;

        info!(
            "Logging batch of {} operations to WAL with txn_id {}",
            operations.len(),
            txn_id
        );

        // Write to persistent storage first
        self.storage.append_transaction(&txn).await?;

        // Then, update in-memory state
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

        debug!(
            "Successfully logged {} operations to WAL with transaction ID: {}",
            operations.len(),
            txn_id
        );
        Ok(txn_id)
    }

    /// Mark a transaction as committed
    pub async fn mark_transaction_committed(&self, txn_id: uuid::Uuid) -> AppResult<()> {
        // Update persistent storage first
        self.storage
            .update_transaction_status(txn_id, TransactionStatus::Committed)
            .await?;

        let mut pending = self.pending_transactions.write().await;
        let mut stats = self.stats.write().await;

        if let Some(txn) = pending.get_mut(&txn_id) {
            txn.status = TransactionStatus::Committed;
            stats.committed_transactions += 1;
            stats.pending_transactions = stats.pending_transactions.saturating_sub(1);
            info!("Transaction {} marked as committed in WAL", txn_id);
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Transaction {} not found in WAL",
                txn_id
            )))
        }
    }

    /// Mark a transaction as failed
    pub async fn mark_transaction_failed(
        &self,
        txn_id: uuid::Uuid,
        error_msg: String,
    ) -> AppResult<()> {
        // Update persistent storage first
        self.storage
            .update_transaction_status(txn_id, TransactionStatus::Failed)
            .await?;

        let mut pending = self.pending_transactions.write().await;
        let mut stats = self.stats.write().await;

        if let Some(txn) = pending.get_mut(&txn_id) {
            txn.status = TransactionStatus::Failed;
            stats.failed_transactions += 1;
            stats.pending_transactions = stats.pending_transactions.saturating_sub(1);
            error!(
                "Transaction {} marked as failed in WAL: {}",
                txn_id, error_msg
            );

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
            Err(AppError::Validation(format!(
                "Transaction {} not found in WAL",
                txn_id
            )))
        }
    }

    /// Get pending transactions that need to be retried
    pub async fn get_pending_retries(&self) -> Vec<TxnId> {
        let retry_queue = self.retry_queue.lock().await;
        retry_queue.iter().copied().collect()
    }

    /// Get a pending transaction by ID
    pub async fn get_transaction(&self, txn_id: TxnId) -> Option<PendingTransaction> {
        let pending = self.pending_transactions.read().await;
        pending.get(&txn_id).cloned()
    }

    /// Remove a transaction from the retry queue
    pub async fn remove_from_retry_queue(&self, txn_id: TxnId) -> bool {
        let mut retry_queue = self.retry_queue.lock().await;
        let original_len = retry_queue.len();
        retry_queue.retain(|id| *id != txn_id);
        retry_queue.len() < original_len
    }

    /// Update transaction retry count
    pub async fn increment_retry_count(&self, txn_id: TxnId) -> AppResult<u32> {
        let mut pending = self.pending_transactions.write().await;

        if let Some(txn) = pending.get_mut(&txn_id) {
            txn.retry_count += 1;
            txn.status = TransactionStatus::Pending;
            txn.last_attempt_at = Some(current_time_millis());
            txn.failed_operations.clear(); // Clear previous failures

            // Update in storage
            self.storage.update_transaction(txn).await?;

            Ok(txn.retry_count)
        } else {
            Err(AppError::Validation(format!(
                "Transaction {} not found in WAL",
                txn_id
            )))
        }
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, txn_id: TxnId) -> Option<TransactionStatus> {
        let pending = self.pending_transactions.read().await;
        pending.get(&txn_id).map(|txn| txn.status)
    }

    /// Wait for transaction completion
    pub async fn wait_for_transaction(
        &self,
        txn_id: TxnId,
        timeout: Duration,
    ) -> AppResult<TransactionStatus> {
        let start = SystemTime::now();

        loop {
            if let Some(status) = self.get_transaction_status(txn_id).await {
                match status {
                    TransactionStatus::Committed
                    | TransactionStatus::Failed
                    | TransactionStatus::Aborted => {
                        return Ok(status);
                    }
                    _ => {
                        // Still in progress
                        if start.elapsed().unwrap_or(Duration::ZERO) > timeout {
                            return Err(AppError::TimeoutError(
                                "Transaction wait timeout".to_string(),
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            } else {
                return Err(AppError::Validation("Transaction not found".to_string()));
            }
        }
    }

    /// Get operations for a transaction
    pub async fn get_transaction_operations(
        &self,
        txn_id: uuid::Uuid,
    ) -> AppResult<Vec<TaoOperation>> {
        let pending = self.pending_transactions.read().await;

        if let Some(txn) = pending.get(&txn_id) {
            Ok(txn.operations.clone())
        } else {
            Err(AppError::Validation(format!(
                "Transaction {} not found in WAL",
                txn_id
            )))
        }
    }

    /// Get stats for the WAL
    pub async fn get_stats(&self) -> WalStats {
        *self.stats.read().await
    }

    /// Get pending transaction count
    pub async fn get_pending_transaction_count(&self) -> usize {
        self.pending_transactions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_wal_creation() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();
        let config = WalConfig::default();
        let wal = TaoWriteAheadLog::new(config, storage_dir).await.unwrap();

        let stats = wal.get_stats().await;
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.pending_transactions, 0);
    }

    #[tokio::test]
    async fn test_log_and_commit() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();
        let config = WalConfig::default();
        let wal = TaoWriteAheadLog::new(config, storage_dir).await.unwrap();

        let operations = vec![TaoOperation::InsertAssociation {
            assoc: TaoAssociation {
                id1: 123,
                atype: "test".to_string(),
                id2: 456,
                time: current_time_millis(),
                data: None,
            },
        }];

        // Log the transaction
        let txn_id = wal.log_operations(operations).await.unwrap();

        // Check state after logging
        assert_eq!(wal.get_pending_transaction_count().await, 1);
        assert_eq!(wal.get_stats().await.pending_transactions, 1);
        let status = wal.get_transaction_status(txn_id).await.unwrap();
        assert_eq!(status, TransactionStatus::Pending);

        // Mark as committed
        wal.mark_transaction_committed(txn_id).await.unwrap();

        // Check state after committing
        assert_eq!(wal.get_pending_transaction_count().await, 1); // Still in map
        let status_after_commit = wal.get_transaction_status(txn_id).await.unwrap();
        assert_eq!(status_after_commit, TransactionStatus::Committed);

        // Check stats
        let stats = wal.get_stats().await;
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.committed_transactions, 1);
        assert_eq!(stats.pending_transactions, 0); // This counter is correct
    }

    #[tokio::test]
    async fn test_wal_persistence_and_reload() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();
        let config = WalConfig::default();

        // Create a WAL and log a transaction
        {
            let wal = TaoWriteAheadLog::new(config.clone(), storage_dir)
                .await
                .unwrap();
            let operations = vec![TaoOperation::InsertObject {
                object_id: 1,
                object_type: "persistent_object".to_string(),
                data: vec![1, 2, 3],
            }];
            wal.log_operations(operations).await.unwrap();
            assert_eq!(wal.get_pending_transaction_count().await, 1);
        } // wal is dropped here, its background tasks are stopped.

        // Create a new WAL instance from the same directory
        let wal2 = TaoWriteAheadLog::new(config, storage_dir).await.unwrap();

        // It should have loaded the pending transaction from storage
        assert_eq!(wal2.get_pending_transaction_count().await, 1);
        let pending_txns = wal2.pending_transactions.read().await;
        let txn = pending_txns.values().next().unwrap();
        assert_eq!(txn.operations[0].operation_type(), "insert_object");
        assert_eq!(txn.status, TransactionStatus::Pending);
    }
}
