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
        let mut in_tx = self.in_transaction.write().unwrap();
        
        if *in_tx {
            return Err("Transaction already in progress".to_string());
        }
        
        *self.tx_data.write().unwrap() = Some(initial_data);
        *in_tx = true;
        
        Ok(())
    }
    
    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        *self.in_transaction.read().unwrap()
    }
    
    /// Get current transaction data (if any)
    pub fn get_data(&self) -> Option<Arc<Value>> {
        self.tx_data.read().unwrap().clone()
    }
    
    /// Update transaction data
    pub fn update_data(&self, data: Arc<Value>) {
        *self.tx_data.write().unwrap() = Some(data);
    }
    
    /// Commit transaction and return final data
    pub fn commit(&self) -> Result<Arc<Value>, String> {
        let mut in_tx = self.in_transaction.write().unwrap();
        
        if !*in_tx {
            return Err("No active transaction".to_string());
        }
        
        let data = self.tx_data.write().unwrap().take()
            .ok_or("No transaction data")?;
        
        *in_tx = false;
        
        Ok(data)
    }
    
    /// Rollback transaction and discard changes
    pub fn rollback(&self) -> Result<(), String> {
        let mut in_tx = self.in_transaction.write().unwrap();
        
        if !*in_tx {
            return Err("No active transaction".to_string());
        }
        
        *self.tx_data.write().unwrap() = None;
        *in_tx = false;
        
        Ok(())
    }
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::new()
    }
}
