// TAO Decorators - Pluggable production features using decorator pattern
// Allows composing different features around the core TAO functionality

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::ent_framework::ent_builder::EntBuilder;
use crate::error::{AppError, AppResult};
use crate::infrastructure::{
    cache_layer::TaoMultiTierCache,
    database::DatabaseTransaction,
    monitoring::MetricsCollector,
    tao_core::{
        AssocType, TaoAssocQuery, TaoAssociation, TaoId, TaoObject, TaoOperations, TaoType,
    },
    write_ahead_log::{TaoOperation, TaoWriteAheadLog},
};

/// Base TAO decorator trait - all decorators implement this
#[async_trait]
pub trait TaoDecorator: TaoOperations + Send + Sync + std::fmt::Debug {
    /// Get the name of this decorator for logging
    fn decorator_name(&self) -> &'static str;
}

/// Base TAO wrapper around TaoCore - the foundation for all decorators
#[derive(Debug)]
pub struct BaseTao {
    core: Arc<dyn TaoOperations>,
}

impl BaseTao {
    pub fn new(core: Arc<dyn TaoOperations>) -> Self {
        Self { core }
    }
}

#[async_trait]
impl TaoOperations for BaseTao {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.core.generate_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        self.core.create_object(id, otype, data).await
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        self.core.obj_get(id).await
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        self.core.obj_update(id, data).await
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        self.core.obj_delete(id).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.core.obj_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.core.obj_exists_by_type(id, otype).await
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        self.core.obj_update_by_type(id, otype, data).await
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.core.obj_delete_by_type(id, otype).await
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        self.core.assoc_get(query).await
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        self.core.assoc_add(assoc).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.core.assoc_delete(id1, atype, id2).await
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.core.assoc_count(id1, atype).await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.core.assoc_range(id1, atype, offset, limit).await
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.core
            .assoc_time_range(id1, atype, high_time, low_time, limit)
            .await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.core.assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        self.core.get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.core.get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        self.core.get_neighbor_ids(id, atype, limit).await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.core.begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.core.execute_query(query).await
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.core.get_all_objects_of_type(otype, limit).await
    }
}

#[async_trait]
impl TaoDecorator for BaseTao {
    fn decorator_name(&self) -> &'static str {
        "BaseTao"
    }
}

/// WAL Decorator - Adds Write-Ahead Log functionality for durability and retry
#[derive(Debug)]
pub struct WalDecorator {
    inner: Arc<dyn TaoDecorator>,
    wal: Arc<TaoWriteAheadLog>,
}

impl WalDecorator {
    pub fn new(inner: Arc<dyn TaoDecorator>, wal: Arc<TaoWriteAheadLog>) -> Self {
        Self { inner, wal }
    }

    /// Execute operations with WAL logging and retry on failure
    #[instrument(skip(self, operations))]
    pub async fn execute_transaction_with_wal(
        &self,
        operations: Vec<TaoOperation>,
    ) -> AppResult<Uuid> {
        // 1. Log operations to WAL first for durability
        let txn_id = self.wal.log_operations(operations.clone()).await?;
        info!("Transaction {} logged to WAL", txn_id);

        // 2. Execute operations individually via inner decorator chain
        let mut success = true;
        let mut error_msg = String::new();

        for operation in operations {
            let result = match operation {
                TaoOperation::InsertObject {
                    object_id,
                    object_type,
                    data,
                } => self
                    .inner
                    .create_object(object_id, object_type, data)
                    .await,
                TaoOperation::InsertAssociation { assoc } => self.inner.assoc_add(assoc).await,
                TaoOperation::DeleteAssociation { id1, atype, id2 } => {
                    self.inner.assoc_delete(id1, atype, id2).await.map(|_| ())
                }
                TaoOperation::UpdateObject { object_id, data } => {
                    self.inner.obj_update(object_id, data).await
                }
                TaoOperation::DeleteObject { object_id } => {
                    self.inner.obj_delete(object_id).await.map(|_| ())
                }
            };

            if let Err(e) = result {
                success = false;
                error_msg = e.to_string();
                break;
            }
        }

        if success {
            // Mark as committed in WAL
            self.wal.mark_transaction_committed(txn_id).await?;
            info!("Transaction {} executed and committed successfully", txn_id);
            Ok(txn_id)
        } else {
            // Mark as failed, enabling retry mechanisms
            self.wal
                .mark_transaction_failed(txn_id, error_msg.clone())
                .await?;
            error!("Transaction {} failed: {}", txn_id, error_msg);
            Err(AppError::Internal(error_msg))
        }
    }

    /// Process pending transactions from WAL
    pub async fn process_pending_transactions(&self) -> AppResult<()> {
        let retry_txns = self.wal.get_pending_retries().await;

        if retry_txns.is_empty() {
            return Ok(());
        }

        info!(
            "Processing {} pending transactions from WAL",
            retry_txns.len()
        );

        for txn_id in retry_txns {
            if let Ok(operations) = self.wal.get_transaction_operations(txn_id).await {
                // Remove from retry queue to prevent re-processing
                self.wal.remove_from_retry_queue(txn_id).await;

                // Increment retry count
                let retry_count = self.wal.increment_retry_count(txn_id).await?;
                info!("Retrying transaction {} (attempt {})", txn_id, retry_count);

                // Execute operations individually via inner decorator chain
                let mut success = true;
                let mut error_msg = String::new();

                for operation in operations {
                    let result = match operation {
                        TaoOperation::InsertObject {
                            object_id,
                            object_type,
                            data,
                        } => self
                            .inner
                            .create_object(object_id, object_type, data)
                            .await,
                        TaoOperation::InsertAssociation { assoc } => {
                            self.inner.assoc_add(assoc).await
                        }
                        TaoOperation::DeleteAssociation { id1, atype, id2 } => {
                            self.inner.assoc_delete(id1, atype, id2).await.map(|_| ())
                        }
                        TaoOperation::UpdateObject { object_id, data } => {
                            self.inner.obj_update(object_id, data).await
                        }
                        TaoOperation::DeleteObject { object_id } => {
                            self.inner.obj_delete(object_id).await.map(|_| ())
                        }
                    };

                    if let Err(e) = result {
                        success = false;
                        error_msg = e.to_string();
                        break;
                    }
                }

                if success {
                    self.wal.mark_transaction_committed(txn_id).await?;
                    info!("Retry of transaction {} succeeded", txn_id);
                } else {
                    self.wal
                        .mark_transaction_failed(txn_id, error_msg.clone())
                        .await?;
                    error!("Retry of transaction {} failed: {}", txn_id, error_msg);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl TaoOperations for WalDecorator {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.inner.generate_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        // Execute the operation first
        self.inner.create_object(id, otype.clone(), data.clone()).await?;

        // Log to WAL for replay capability
        let operation = TaoOperation::InsertObject {
            object_id: id,
            object_type: otype,
            data,
        };

        let txn_id = self.wal.log_operations(vec![operation]).await?;
        // Mark as committed since operation already succeeded
        self.wal.mark_transaction_committed(txn_id).await?;

        debug!(
            "Logged create_object operation {} to WAL as transaction {}",
            id, txn_id
        );
        Ok(())
    }

    // Write operations are enhanced with WAL logging for replay capability
    #[instrument(skip(self, data), fields(object_id = %id))]
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        // Execute the operation first
        self.inner.obj_update(id, data.clone()).await?;

        // Log to WAL for replay capability
        let operation = TaoOperation::UpdateObject {
            object_id: id,
            data,
        };

        let txn_id = self.wal.log_operations(vec![operation]).await?;
        // Mark as committed since operation already succeeded
        self.wal.mark_transaction_committed(txn_id).await?;

        debug!(
            "Logged obj_update operation {} to WAL as transaction {}",
            id, txn_id
        );
        Ok(())
    }

    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        // Execute the operation first
        let result = self.inner.obj_delete(id).await?;

        if result {
            // Log to WAL for replay capability only if deletion succeeded
            let operation = TaoOperation::DeleteObject { object_id: id };

            let txn_id = self.wal.log_operations(vec![operation]).await?;
            // Mark as committed since operation already succeeded
            self.wal.mark_transaction_committed(txn_id).await?;

            debug!(
                "Logged obj_delete operation {} to WAL as transaction {}",
                id, txn_id
            );
        }

        Ok(result)
    }

    #[instrument(skip(self), fields(id1 = %assoc.id1, atype = %assoc.atype, id2 = %assoc.id2))]
    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        // Execute the operation first
        self.inner.assoc_add(assoc.clone()).await?;

        // Log to WAL for replay capability
        let operation = TaoOperation::InsertAssociation { assoc };

        let txn_id = self.wal.log_operations(vec![operation]).await?;
        // Mark as committed since operation already succeeded
        self.wal.mark_transaction_committed(txn_id).await?;

        debug!(
            "Logged assoc_add operation to WAL as transaction {}",
            txn_id
        );
        Ok(())
    }

    #[instrument(skip(self), fields(id1 = %id1, atype = %atype, id2 = %id2))]
    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        // Execute the operation first
        let result = self.inner.assoc_delete(id1, atype.clone(), id2).await?;

        if result {
            // Log to WAL for replay capability only if deletion succeeded
            let operation = TaoOperation::DeleteAssociation { id1, atype, id2 };

            let txn_id = self.wal.log_operations(vec![operation]).await?;
            // Mark as committed since operation already succeeded
            self.wal.mark_transaction_committed(txn_id).await?;

            debug!(
                "Logged assoc_delete operation to WAL as transaction {}",
                txn_id
            );
        }

        Ok(result)
    }

    // Additional write operations with WAL logging

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        // Execute the operation first
        let result = self
            .inner
            .obj_update_by_type(id, otype.clone(), data.clone())
            .await?;

        if result {
            // Log to WAL for replay capability only if update succeeded
            let operation = TaoOperation::UpdateObject {
                object_id: id,
                data,
            };

            let txn_id = self.wal.log_operations(vec![operation]).await?;
            // Mark as committed since operation already succeeded
            self.wal.mark_transaction_committed(txn_id).await?;

            debug!(
                "Logged obj_update_by_type operation {} to WAL as transaction {}",
                id, txn_id
            );
        }

        Ok(result)
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        // Execute the operation first
        let result = self.inner.obj_delete_by_type(id, otype).await?;

        if result {
            // Log to WAL for replay capability only if deletion succeeded
            let operation = TaoOperation::DeleteObject { object_id: id };

            let txn_id = self.wal.log_operations(vec![operation]).await?;
            // Mark as committed since operation already succeeded
            self.wal.mark_transaction_committed(txn_id).await?;

            debug!(
                "Logged obj_delete_by_type operation {} to WAL as transaction {}",
                id, txn_id
            );
        }

        Ok(result)
    }

    // Read operations delegate to inner
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        self.inner.obj_get(id).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.inner.obj_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.inner.obj_exists_by_type(id, otype).await
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        self.inner.assoc_get(query).await
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.inner.assoc_count(id1, atype).await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.inner.assoc_range(id1, atype, offset, limit).await
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.inner
            .assoc_time_range(id1, atype, high_time, low_time, limit)
            .await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.inner.assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        self.inner.get_neighbor_ids(id, atype, limit).await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.inner.begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.inner.execute_query(query).await
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_all_objects_of_type(otype, limit).await
    }
}

#[async_trait]
impl TaoDecorator for WalDecorator {
    fn decorator_name(&self) -> &'static str {
        "WalDecorator"
    }
}

/// Metrics Decorator - Adds comprehensive monitoring and metrics collection
#[derive(Debug)]
pub struct MetricsDecorator {
    inner: Arc<dyn TaoDecorator>,
    metrics: Arc<MetricsCollector>,
}

impl MetricsDecorator {
    pub fn new(inner: Arc<dyn TaoDecorator>, metrics: Arc<MetricsCollector>) -> Self {
        Self { inner, metrics }
    }

    async fn record_operation(&self, operation: &str, start_time: Instant, success: bool) {
        self.metrics
            .record_request(operation, start_time.elapsed(), success)
            .await;
    }

    async fn record_business_event(&self, event: &str) {
        self.metrics.record_business_event(event).await;
    }
}

#[async_trait]
impl TaoOperations for MetricsDecorator {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        let start = Instant::now();
        let result = self.inner.generate_id(owner_id).await;
        self.record_operation("generate_id", start, result.is_ok())
            .await;
        result
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let start = Instant::now();
        let result = self.inner.create_object(id, otype, data).await;
        self.record_operation("create_object", start, result.is_ok())
            .await;
        if result.is_ok() {
            self.record_business_event("create_object").await;
        }
        result
    }

    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        let start = Instant::now();
        let result = self.inner.obj_get(id).await;
        self.record_operation("obj_get", start, result.is_ok())
            .await;
        result
    }

    #[instrument(skip(self, data), fields(object_id = %id))]
    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let start = Instant::now();
        let result = self.inner.obj_update(id, data).await;
        self.record_operation("obj_update", start, result.is_ok())
            .await;
        result
    }

    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.obj_delete(id).await;
        self.record_operation("obj_delete", start, result.is_ok())
            .await;
        result
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.obj_exists(id).await;
        self.record_operation("obj_exists", start, result.is_ok())
            .await;
        result
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.obj_exists_by_type(id, otype).await;
        self.record_operation("obj_exists_by_type", start, result.is_ok())
            .await;
        result
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.obj_update_by_type(id, otype, data).await;
        self.record_operation("obj_update_by_type", start, result.is_ok())
            .await;
        result
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.obj_delete_by_type(id, otype).await;
        self.record_operation("obj_delete_by_type", start, result.is_ok())
            .await;
        result
    }

    #[instrument(skip(self), fields(id1 = %query.id1, atype = %query.atype))]
    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        let start = Instant::now();
        let result = self.inner.assoc_get(query).await;
        self.record_operation("assoc_get", start, result.is_ok())
            .await;
        result
    }

    #[instrument(skip(self), fields(id1 = %assoc.id1, atype = %assoc.atype, id2 = %assoc.id2))]
    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        let start = Instant::now();
        let result = self.inner.assoc_add(assoc).await;
        self.record_operation("assoc_add", start, result.is_ok())
            .await;
        if result.is_ok() {
            self.record_business_event("assoc_add").await;
        }
        result
    }

    #[instrument(skip(self), fields(id1 = %id1, atype = %atype, id2 = %id2))]
    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.assoc_delete(id1, atype, id2).await;
        self.record_operation("assoc_delete", start, result.is_ok())
            .await;
        result
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        let start = Instant::now();
        let result = self.inner.assoc_count(id1, atype).await;
        self.record_operation("assoc_count", start, result.is_ok())
            .await;
        result
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        let start = Instant::now();
        let result = self.inner.assoc_range(id1, atype, offset, limit).await;
        self.record_operation("assoc_range", start, result.is_ok())
            .await;
        result
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        let start = Instant::now();
        let result = self
            .inner
            .assoc_time_range(id1, atype, high_time, low_time, limit)
            .await;
        self.record_operation("assoc_time_range", start, result.is_ok())
            .await;
        result
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let start = Instant::now();
        let result = self.inner.assoc_exists(id1, atype, id2).await;
        self.record_operation("assoc_exists", start, result.is_ok())
            .await;
        result
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        let start = Instant::now();
        let result = self.inner.get_by_id_and_type(ids, otype).await;
        self.record_operation("get_by_id_and_type", start, result.is_ok())
            .await;
        result
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        let start = Instant::now();
        let result = self.inner.get_neighbors(id, atype, limit).await;
        self.record_operation("get_neighbors", start, result.is_ok())
            .await;
        result
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        let start = Instant::now();
        let result = self.inner.get_neighbor_ids(id, atype, limit).await;
        self.record_operation("get_neighbor_ids", start, result.is_ok())
            .await;
        result
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        let start = Instant::now();
        let result = self.inner.begin_transaction().await;
        self.record_operation("begin_transaction", start, result.is_ok())
            .await;
        result
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        let start = Instant::now();
        let result = self.inner.execute_query(query).await;
        self.record_operation("execute_query", start, result.is_ok())
            .await;
        result
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        let start = Instant::now();
        let result = self.inner.get_all_objects_of_type(otype, limit).await;
        self.record_operation("get_all_objects_of_type", start, result.is_ok())
            .await;
        result
    }
}

#[async_trait]
impl TaoDecorator for MetricsDecorator {
    fn decorator_name(&self) -> &'static str {
        "MetricsDecorator"
    }
}

/// Cache Decorator - Adds caching functionality for read operations
#[derive(Debug)]
pub struct CacheDecorator {
    inner: Arc<dyn TaoDecorator>,
    cache: Arc<TaoMultiTierCache>,
    enable_caching: bool,
}

impl CacheDecorator {
    pub fn new(
        inner: Arc<dyn TaoDecorator>,
        cache: Arc<TaoMultiTierCache>,
        enable_caching: bool,
    ) -> Self {
        Self {
            inner,
            cache,
            enable_caching,
        }
    }
}

#[async_trait]
impl TaoOperations for CacheDecorator {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.inner.generate_id(owner_id).await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        let result = self.inner.create_object(id, otype, data).await;

        // Invalidate cache on successful creation
        if result.is_ok() && self.enable_caching {
            let _ = self.cache.invalidate_object(id).await;
        }

        result
    }

    #[instrument(skip(self), fields(object_id = %id))]
    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        if !self.enable_caching {
            return self.inner.obj_get(id).await;
        }

        // Try cache first
        if let Ok(Some(cached)) = self.cache.get_object(id).await {
            debug!("Cache hit for object {}", id);
            return Ok(Some(cached));
        }

        // Cache miss, fetch from inner
        let result = self.inner.obj_get(id).await?;

        // Populate cache if object found
        if let Some(ref obj) = result {
            let _ = self.cache.put_object(id, obj).await;
        }

        Ok(result)
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        let result = self.inner.obj_update(id, data).await;

        // Invalidate cache on successful update
        if result.is_ok() && self.enable_caching {
            let _ = self.cache.invalidate_object(id).await;
        }

        result
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        let result = self.inner.obj_delete(id).await;

        // Invalidate cache on successful deletion
        if let Ok(true) = result {
            if self.enable_caching {
                let _ = self.cache.invalidate_object(id).await;
            }
        }

        result
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        if !self.enable_caching || query.id2_set.is_some() {
            // Skip cache for complex queries
            return self.inner.assoc_get(query).await;
        }

        // Try cache for simple queries
        if let Ok(Some(cached_assocs)) = self.cache.get_associations(query.id1, &query.atype).await
        {
            debug!(
                "Cache hit for associations {} -> {}",
                query.id1, query.atype
            );
            return Ok(cached_assocs);
        }

        // Cache miss, fetch from inner
        let associations = self.inner.assoc_get(query.clone()).await?;

        // Populate cache
        let _ = self
            .cache
            .put_associations(query.id1, &query.atype, &associations)
            .await;

        Ok(associations)
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        let result = self.inner.assoc_add(assoc.clone()).await;

        // Invalidate cache for both objects
        if result.is_ok() && self.enable_caching {
            let _ = self.cache.invalidate_object(assoc.id1).await;
            let _ = self.cache.invalidate_object(assoc.id2).await;
        }

        result
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        let result = self.inner.assoc_delete(id1, atype, id2).await;

        // Invalidate cache for both objects on successful deletion
        if let Ok(true) = result {
            if self.enable_caching {
                let _ = self.cache.invalidate_object(id1).await;
                let _ = self.cache.invalidate_object(id2).await;
            }
        }

        result
    }

    // Delegate other operations without caching
    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.inner.obj_exists(id).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.inner.obj_exists_by_type(id, otype).await
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        let result = self.inner.obj_update_by_type(id, otype, data).await;
        if let Ok(true) = result {
            if self.enable_caching {
                let _ = self.cache.invalidate_object(id).await;
            }
        }
        result
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        let result = self.inner.obj_delete_by_type(id, otype).await;
        if let Ok(true) = result {
            if self.enable_caching {
                let _ = self.cache.invalidate_object(id).await;
            }
        }
        result
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.inner.assoc_count(id1, atype).await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.inner.assoc_range(id1, atype, offset, limit).await
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.inner
            .assoc_time_range(id1, atype, high_time, low_time, limit)
            .await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.inner.assoc_exists(id1, atype, id2).await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_by_id_and_type(ids, otype).await
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_neighbors(id, atype, limit).await
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        self.inner.get_neighbor_ids(id, atype, limit).await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.inner.begin_transaction().await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.inner.execute_query(query).await
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.inner.get_all_objects_of_type(otype, limit).await
    }
}

#[async_trait]
impl TaoDecorator for CacheDecorator {
    fn decorator_name(&self) -> &'static str {
        "CacheDecorator"
    }
}

/// Circuit Breaker Decorator - Adds fault tolerance
#[derive(Debug)]
pub struct CircuitBreakerDecorator {
    inner: Arc<dyn TaoDecorator>,
    circuit_breaker: Arc<CircuitBreaker>,
    enable_circuit_breaker: bool,
}

impl CircuitBreakerDecorator {
    pub fn new(
        inner: Arc<dyn TaoDecorator>,
        failure_threshold: u32,
        recovery_timeout: Duration,
        enable_circuit_breaker: bool,
    ) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(failure_threshold, recovery_timeout));
        Self {
            inner,
            circuit_breaker,
            enable_circuit_breaker,
        }
    }

    async fn execute_with_breaker<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>>,
    {
        if !self.enable_circuit_breaker {
            return operation.await;
        }
        self.circuit_breaker.execute(operation).await
    }
}

#[async_trait]
impl TaoOperations for CircuitBreakerDecorator {
    async fn generate_id(&self, owner_id: Option<TaoId>) -> AppResult<TaoId> {
        self.execute_with_breaker(self.inner.generate_id(owner_id))
            .await
    }

    async fn create_object(&self, id: TaoId, otype: TaoType, data: Vec<u8>) -> AppResult<()> {
        self.execute_with_breaker(self.inner.create_object(id, otype, data))
            .await
    }

    async fn obj_get(&self, id: TaoId) -> AppResult<Option<TaoObject>> {
        self.execute_with_breaker(self.inner.obj_get(id)).await
    }

    async fn obj_update(&self, id: TaoId, data: Vec<u8>) -> AppResult<()> {
        self.execute_with_breaker(self.inner.obj_update(id, data))
            .await
    }

    async fn obj_delete(&self, id: TaoId) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.obj_delete(id)).await
    }

    async fn obj_exists(&self, id: TaoId) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.obj_exists(id)).await
    }

    async fn obj_exists_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.obj_exists_by_type(id, otype))
            .await
    }

    async fn obj_update_by_type(
        &self,
        id: TaoId,
        otype: TaoType,
        data: Vec<u8>,
    ) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.obj_update_by_type(id, otype, data))
            .await
    }

    async fn obj_delete_by_type(&self, id: TaoId, otype: TaoType) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.obj_delete_by_type(id, otype))
            .await
    }

    async fn assoc_get(&self, query: TaoAssocQuery) -> AppResult<Vec<TaoAssociation>> {
        self.execute_with_breaker(self.inner.assoc_get(query)).await
    }

    async fn assoc_add(&self, assoc: TaoAssociation) -> AppResult<()> {
        self.execute_with_breaker(self.inner.assoc_add(assoc)).await
    }

    async fn assoc_delete(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.assoc_delete(id1, atype, id2))
            .await
    }

    async fn assoc_count(&self, id1: TaoId, atype: AssocType) -> AppResult<u64> {
        self.execute_with_breaker(self.inner.assoc_count(id1, atype))
            .await
    }

    async fn assoc_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        offset: u64,
        limit: u32,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.execute_with_breaker(self.inner.assoc_range(id1, atype, offset, limit))
            .await
    }

    async fn assoc_time_range(
        &self,
        id1: TaoId,
        atype: AssocType,
        high_time: i64,
        low_time: i64,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoAssociation>> {
        self.execute_with_breaker(
            self.inner
                .assoc_time_range(id1, atype, high_time, low_time, limit),
        )
        .await
    }

    async fn assoc_exists(&self, id1: TaoId, atype: AssocType, id2: TaoId) -> AppResult<bool> {
        self.execute_with_breaker(self.inner.assoc_exists(id1, atype, id2))
            .await
    }

    async fn get_by_id_and_type(
        &self,
        ids: Vec<TaoId>,
        otype: TaoType,
    ) -> AppResult<Vec<TaoObject>> {
        self.execute_with_breaker(self.inner.get_by_id_and_type(ids, otype))
            .await
    }

    async fn get_neighbors(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.execute_with_breaker(self.inner.get_neighbors(id, atype, limit))
            .await
    }

    async fn get_neighbor_ids(
        &self,
        id: TaoId,
        atype: AssocType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoId>> {
        self.execute_with_breaker(self.inner.get_neighbor_ids(id, atype, limit))
            .await
    }

    async fn begin_transaction(&self) -> AppResult<DatabaseTransaction> {
        self.execute_with_breaker(self.inner.begin_transaction())
            .await
    }

    async fn execute_query(&self, query: String) -> AppResult<Vec<HashMap<String, String>>> {
        self.execute_with_breaker(self.inner.execute_query(query))
            .await
    }

    async fn get_all_objects_of_type(
        &self,
        otype: TaoType,
        limit: Option<u32>,
    ) -> AppResult<Vec<TaoObject>> {
        self.execute_with_breaker(self.inner.get_all_objects_of_type(otype, limit))
            .await
    }
}

#[async_trait]
impl TaoDecorator for CircuitBreakerDecorator {
    fn decorator_name(&self) -> &'static str {
        "CircuitBreakerDecorator"
    }
}

/// Circuit breaker implementation for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<tokio::sync::RwLock<CircuitBreakerState>>,
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
            state: Arc::new(tokio::sync::RwLock::new(CircuitBreakerState {
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
                        return Err(AppError::ServiceUnavailable(
                            "Circuit breaker is open".to_string(),
                        ));
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
