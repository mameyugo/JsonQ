//! Transaction management
//!
//! Provides ACID transaction support with in-memory buffering

use serde_json::Value;
use std::sync::{Arc, RwLock};

/// Transaction state manager
///
/// Manages transaction lifecycle and buffered data
#[derive(Debug)]
pub struct TransactionState {
    /// Whether a transaction is currently active
    in_transaction: RwLock<bool>,

    /// Buffered data during transaction (not yet flushed to disk)
    tx_data: RwLock<Option<Arc<Value>>>,
}

impl TransactionState {
    /// Create a new transaction state (no active transaction)
    pub fn new() -> Self {
        Self {
            in_transaction: RwLock::new(false),
            tx_data: RwLock::new(None),
        }
    }

    /// Begin a new transaction with initial data
    pub fn begin(&self, initial_data: Arc<Value>) -> Result<(), String> {
        let mut in_tx = self
            .in_transaction
            .write()
            .map_err(|e| format!("Transaction lock poisoned: {}", e))?;

        if *in_tx {
            return Err("Transaction already in progress".to_string());
        }

        let mut tx_data = self
            .tx_data
            .write()
            .map_err(|e| format!("Transaction data lock poisoned: {}", e))?;

        *tx_data = Some(initial_data);
        *in_tx = true;

        Ok(())
    }

    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        *self
            .in_transaction
            .read()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Get current transaction data (if any)
    pub fn get_data(&self) -> Option<Arc<Value>> {
        self.tx_data
            .read()
            .map(|d| d.clone())
            .unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Update transaction data
    pub fn update_data(&self, data: Arc<Value>) {
        if let Ok(mut tx_data) = self.tx_data.write() {
            *tx_data = Some(data);
        } else if let Err(e) = self.tx_data.write() {
            *e.into_inner() = Some(data);
        }
    }

    /// Commit transaction and return final data
    pub fn commit(&self) -> Result<Arc<Value>, String> {
        let mut in_tx = self
            .in_transaction
            .write()
            .map_err(|e| format!("Transaction lock poisoned: {}", e))?;

        if !*in_tx {
            return Err("No active transaction".to_string());
        }

        let mut tx_data_lock = self
            .tx_data
            .write()
            .map_err(|e| format!("Transaction data lock poisoned: {}", e))?;

        let data = tx_data_lock.take().ok_or("No transaction data")?;

        *in_tx = false;

        Ok(data)
    }

    /// Rollback transaction and discard changes
    pub fn rollback(&self) -> Result<(), String> {
        let mut in_tx = self
            .in_transaction
            .write()
            .map_err(|e| format!("Transaction lock poisoned: {}", e))?;

        if !*in_tx {
            return Err("No active transaction".to_string());
        }

        if let Ok(mut tx_data) = self.tx_data.write() {
            *tx_data = None;
        } else if let Err(e) = self.tx_data.write() {
            *e.into_inner() = None;
        }

        *in_tx = false;

        Ok(())
    }
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::new()
    }
}
