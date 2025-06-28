use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::{AppError, AppResult};
use super::write_ahead_log::{PendingTransaction, TransactionStatus, TxnId};

/// File-based storage for the Write-Ahead Log
/// Provides durable persistence for transaction logs
#[derive(Debug)]
pub struct WalStorage {
    /// Directory where WAL files are stored
    storage_dir: PathBuf,
    /// Transaction log file
    log_file: Arc<Mutex<BufWriter<File>>>,
    /// Index file for quick transaction lookups
    index_file: Arc<Mutex<BufWriter<File>>>,
}

/// Entry in the WAL log file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalLogEntry {
    txn_id: TxnId,
    entry_type: WalEntryType,
    timestamp: i64,
    data: Vec<u8>,
}

/// Type of WAL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WalEntryType {
    /// New transaction logged
    Transaction,
    /// Status update for existing transaction
    StatusUpdate(TransactionStatus),
}

/// Index entry for quick lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    txn_id: TxnId,
    file_offset: u64,
    status: TransactionStatus,
    timestamp: i64,
}

impl WalStorage {
    /// Create a new WAL storage instance
    pub fn new(storage_dir: &str) -> AppResult<Self> {
        let storage_path = PathBuf::from(storage_dir);

        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&storage_path).map_err(|e| {
            AppError::StorageError(format!("Failed to create WAL storage directory: {}", e))
        })?;

        let log_path = storage_path.join("wal.log");
        let index_path = storage_path.join("wal.index");

        // Open or create log file
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| AppError::StorageError(format!("Failed to open WAL log file: {}", e)))?;

        // Open or create index file
        let index_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&index_path)
            .map_err(|e| AppError::StorageError(format!("Failed to open WAL index file: {}", e)))?;

        let storage = Self {
            storage_dir: storage_path,
            log_file: Arc::new(Mutex::new(BufWriter::new(log_file))),
            index_file: Arc::new(Mutex::new(BufWriter::new(index_file))),
        };

        info!("WAL storage initialized at: {}", storage_dir);
        Ok(storage)
    }

    /// Load all pending transactions from storage
    pub fn load_transactions(&self) -> AppResult<HashMap<TxnId, PendingTransaction>> {
        let index_path = self.storage_dir.join("wal.index");
        let log_path = self.storage_dir.join("wal.log");

        if !index_path.exists() || !log_path.exists() {
            debug!("WAL files don't exist, starting with empty transaction set");
            return Ok(HashMap::new());
        }

        // Load index to get transaction metadata
        let mut index_entries = Vec::new();
        let index_file = File::open(&index_path).map_err(|e| {
            AppError::StorageError(format!("Failed to open index file for reading: {}", e))
        })?;

        let reader = BufReader::new(index_file);
        for line in reader.lines() {
            let line = line.map_err(|e| {
                AppError::StorageError(format!("Failed to read index line: {}", e))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let entry: IndexEntry = serde_json::from_str(&line).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize index entry: {}", e))
            })?;

            index_entries.push(entry);
        }

        // Group index entries by transaction ID and get the latest status
        let mut latest_status: HashMap<TxnId, (TransactionStatus, u64)> = HashMap::new();
        for entry in index_entries {
            match latest_status.get(&entry.txn_id) {
                Some((_, existing_offset)) if *existing_offset > entry.file_offset => {
                    // Skip older entries
                    continue;
                }
                _ => {
                    latest_status.insert(entry.txn_id, (entry.status, entry.file_offset));
                }
            }
        }

        // Load transactions that are still pending or need processing
        let mut transactions = HashMap::new();
        let log_file = File::open(&log_path).map_err(|e| {
            AppError::StorageError(format!("Failed to open log file for reading: {}", e))
        })?;

        let reader = BufReader::new(log_file);
        for line in reader.lines() {
            let line = line.map_err(|e| {
                AppError::StorageError(format!("Failed to read log line: {}", e))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let entry: WalLogEntry = serde_json::from_str(&line).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize log entry: {}", e))
            })?;

            // Only load transactions that are still active
            if let Some((status, _)) = latest_status.get(&entry.txn_id) {
                match status {
                    TransactionStatus::Committed => continue, // Skip committed
                    _ => {
                        // Load the transaction data
                        if let WalEntryType::Transaction = entry.entry_type {
                            let mut txn: PendingTransaction = serde_json::from_slice(&entry.data)
                                .map_err(|e| {
                                    AppError::DeserializationError(format!(
                                        "Failed to deserialize transaction: {}", e
                                    ))
                                })?;

                            // Update with latest status
                            txn.status = *status;
                            transactions.insert(entry.txn_id, txn);
                        }
                    }
                }
            }
        }

        info!("Loaded {} pending transactions from WAL storage", transactions.len());
        Ok(transactions)
    }

    /// Append a new transaction to the WAL
    pub async fn append_transaction(&self, txn: &PendingTransaction) -> AppResult<()> {
        let current_time = crate::infrastructure::tao_core::current_time_millis();

        // Serialize transaction data
        let txn_data = serde_json::to_vec(txn).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize transaction: {}", e))
        })?;

        // Create log entry
        let log_entry = WalLogEntry {
            txn_id: txn.txn_id,
            entry_type: WalEntryType::Transaction,
            timestamp: current_time,
            data: txn_data,
        };

        // Write to log file
        let log_line = serde_json::to_string(&log_entry).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize log entry: {}", e))
        })?;

        let file_offset = {
            let mut log_file = self.log_file.lock().await;
            let offset = log_file.seek(SeekFrom::End(0)).map_err(|e| {
                AppError::StorageError(format!("Failed to seek to end of log file: {}", e))
            })?;

            writeln!(log_file, "{}", log_line).map_err(|e| {
                AppError::StorageError(format!("Failed to write to log file: {}", e))
            })?;

            log_file.flush().map_err(|e| {
                AppError::StorageError(format!("Failed to flush log file: {}", e))
            })?;

            offset
        };

        // Write to index file
        let index_entry = IndexEntry {
            txn_id: txn.txn_id,
            file_offset,
            status: txn.status,
            timestamp: current_time,
        };

        let index_line = serde_json::to_string(&index_entry).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize index entry: {}", e))
        })?;

        {
            let mut index_file = self.index_file.lock().await;
            writeln!(index_file, "{}", index_line).map_err(|e| {
                AppError::StorageError(format!("Failed to write to index file: {}", e))
            })?;

            index_file.flush().map_err(|e| {
                AppError::StorageError(format!("Failed to flush index file: {}", e))
            })?;
        }

        debug!("Appended transaction {} to WAL storage", txn.txn_id);
        Ok(())
    }

    /// Update the status of a transaction
    pub async fn update_transaction_status(
        &self,
        txn_id: TxnId,
        status: TransactionStatus,
    ) -> AppResult<()> {
        let current_time = crate::infrastructure::tao_core::current_time_millis();

        // Create status update entry
        let log_entry = WalLogEntry {
            txn_id,
            entry_type: WalEntryType::StatusUpdate(status),
            timestamp: current_time,
            data: Vec::new(), // No data needed for status updates
        };

        // Write to log file
        let log_line = serde_json::to_string(&log_entry).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize status update: {}", e))
        })?;

        let file_offset = {
            let mut log_file = self.log_file.lock().await;
            let offset = log_file.seek(SeekFrom::End(0)).map_err(|e| {
                AppError::StorageError(format!("Failed to seek to end of log file: {}", e))
            })?;

            writeln!(log_file, "{}", log_line).map_err(|e| {
                AppError::StorageError(format!("Failed to write status update to log file: {}", e))
            })?;

            log_file.flush().map_err(|e| {
                AppError::StorageError(format!("Failed to flush log file: {}", e))
            })?;

            offset
        };

        // Update index
        let index_entry = IndexEntry {
            txn_id,
            file_offset,
            status,
            timestamp: current_time,
        };

        let index_line = serde_json::to_string(&index_entry).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize index entry: {}", e))
        })?;

        {
            let mut index_file = self.index_file.lock().await;
            writeln!(index_file, "{}", index_line).map_err(|e| {
                AppError::StorageError(format!("Failed to write to index file: {}", e))
            })?;

            index_file.flush().map_err(|e| {
                AppError::StorageError(format!("Failed to flush index file: {}", e))
            })?;
        }

        debug!("Updated transaction {} status to {:?}", txn_id, status);
        Ok(())
    }

    /// Update a complete transaction record
    pub async fn update_transaction(&self, txn: &PendingTransaction) -> AppResult<()> {
        // For now, we'll just update the status
        // In a more sophisticated implementation, we might want to store
        // the full transaction update
        self.update_transaction_status(txn.txn_id, txn.status).await
    }

    /// Compact the WAL files by removing committed transactions
    /// This is a maintenance operation that should be run periodically
    pub async fn compact(&self) -> AppResult<()> {
        warn!("WAL compaction not yet implemented - this is a TODO for production systems");
        // TODO: Implement compaction logic
        // 1. Read all transactions
        // 2. Filter out committed/expired ones
        // 3. Write remaining transactions to new files
        // 4. Atomically replace old files with new ones
        Ok(())
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> AppResult<WalStorageStats> {
        let log_path = self.storage_dir.join("wal.log");
        let index_path = self.storage_dir.join("wal.index");

        let log_size = if log_path.exists() {
            std::fs::metadata(&log_path)
                .map_err(|e| AppError::StorageError(format!("Failed to get log file metadata: {}", e)))?
                .len()
        } else {
            0
        };

        let index_size = if index_path.exists() {
            std::fs::metadata(&index_path)
                .map_err(|e| AppError::StorageError(format!("Failed to get index file metadata: {}", e)))?
                .len()
        } else {
            0
        };

        Ok(WalStorageStats {
            log_file_size_bytes: log_size,
            index_file_size_bytes: index_size,
            total_size_bytes: log_size + index_size,
            storage_dir: self.storage_dir.to_string_lossy().to_string(),
        })
    }
}

/// Statistics about WAL storage
#[derive(Debug, Clone, Serialize)]
pub struct WalStorageStats {
    pub log_file_size_bytes: u64,
    pub index_file_size_bytes: u64,
    pub total_size_bytes: u64,
    pub storage_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::infrastructure::write_ahead_log::TaoOperation;
    use crate::infrastructure::tao_core::TaoAssociation;

    #[tokio::test]
    async fn test_storage_creation() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();

        let storage = WalStorage::new(storage_dir).unwrap();
        let stats = storage.get_storage_stats().unwrap();

        assert_eq!(stats.log_file_size_bytes, 0);
        assert_eq!(stats.index_file_size_bytes, 0);
    }

    #[tokio::test]
    async fn test_transaction_persistence() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();

        let storage = WalStorage::new(storage_dir).unwrap();

        // Create a test transaction
        let operations = vec![TaoOperation::InsertAssociation {
            assoc: TaoAssociation {
                id1: 123,
                atype: "test".to_string(),
                id2: 456,
                time: crate::infrastructure::tao_core::current_time_millis(),
                data: None,
            },
        }];

        let txn = PendingTransaction::new(operations);
        let txn_id = txn.txn_id;

        // Store the transaction
        storage.append_transaction(&txn).await.unwrap();

        // Update its status
        storage.update_transaction_status(txn_id, TransactionStatus::Committed)
            .await.unwrap();

        // Verify files were created and have content
        let stats = storage.get_storage_stats().unwrap();
        assert!(stats.log_file_size_bytes > 0);
        assert!(stats.index_file_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_transaction_loading() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();

        // Create storage and add a transaction
        {
            let storage = WalStorage::new(storage_dir).unwrap();
            let operations = vec![TaoOperation::InsertObject {
                object_id: 1,
                object_type: "test_object".to_string(),
                data: vec![1, 2, 3],
            }];

            let txn = PendingTransaction::new(operations);
            storage.append_transaction(&txn).await.unwrap();
        }

        // Create new storage instance and load transactions
        let storage2 = WalStorage::new(storage_dir).unwrap();
        let loaded_txns = storage2.load_transactions().unwrap();

        assert_eq!(loaded_txns.len(), 1);
        let txn = loaded_txns.values().next().unwrap();
        assert_eq!(txn.operations.len(), 1);
        assert_eq!(txn.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_committed_transactions_not_loaded() {
        let dir = tempdir().unwrap();
        let storage_dir = dir.path().to_str().unwrap();

        let storage = WalStorage::new(storage_dir).unwrap();

        // Create and commit a transaction
        let operations = vec![TaoOperation::InsertObject {
            object_id: 1,
            object_type: "test_object".to_string(),
            data: vec![1, 2, 3],
        }];

        let txn = PendingTransaction::new(operations);
        let txn_id = txn.txn_id;

        storage.append_transaction(&txn).await.unwrap();
        storage.update_transaction_status(txn_id, TransactionStatus::Committed)
            .await.unwrap();

        // Load transactions - committed ones should not be loaded
        let loaded_txns = storage.load_transactions().unwrap();
        assert_eq!(loaded_txns.len(), 0);
    }
}
