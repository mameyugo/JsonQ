//! Tests for TransactionState

use jsonq::store::transaction::TransactionState;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_transaction_initial_state() {
    let tx = TransactionState::new();
    assert!(!tx.is_active());
    assert!(tx.get_data().is_none());
}

#[test]
fn test_begin_transaction() {
    let tx = TransactionState::new();
    let data = Arc::new(json!({"key": "value"}));

    let result = tx.begin(data.clone());
    assert!(result.is_ok());
    assert!(tx.is_active());
    assert_eq!(*tx.get_data().unwrap(), *data);
}

#[test]
fn test_begin_transaction_twice_fails() {
    let tx = TransactionState::new();
    let data = Arc::new(json!({}));

    tx.begin(data.clone()).unwrap();
    let result = tx.begin(data);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Transaction already in progress");
}

#[test]
fn test_update_transaction_data() {
    let tx = TransactionState::new();
    let data1 = Arc::new(json!({"version": 1}));
    let data2 = Arc::new(json!({"version": 2}));

    tx.begin(data1).unwrap();
    tx.update_data(data2.clone());

    assert_eq!(*tx.get_data().unwrap(), *data2);
}

#[test]
fn test_commit_transaction() {
    let tx = TransactionState::new();
    let data = Arc::new(json!({"committed": true}));

    tx.begin(data.clone()).unwrap();
    let result = tx.commit();

    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), *data);
    assert!(!tx.is_active());
    assert!(tx.get_data().is_none());
}

#[test]
fn test_commit_without_transaction_fails() {
    let tx = TransactionState::new();
    let result = tx.commit();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No active transaction");
}

#[test]
fn test_rollback_transaction() {
    let tx = TransactionState::new();
    let data = Arc::new(json!({"rollback": true}));

    tx.begin(data).unwrap();
    let result = tx.rollback();

    assert!(result.is_ok());
    assert!(!tx.is_active());
    assert!(tx.get_data().is_none());
}

#[test]
fn test_rollback_without_transaction_fails() {
    let tx = TransactionState::new();
    let result = tx.rollback();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No active transaction");
}

#[test]
fn test_transaction_full_cycle() {
    let tx = TransactionState::new();

    // Begin
    let data1 = Arc::new(json!({"value": 1}));
    tx.begin(data1).unwrap();
    assert!(tx.is_active());

    // Update
    let data2 = Arc::new(json!({"value": 2}));
    tx.update_data(data2.clone());

    // Commit
    let committed = tx.commit().unwrap();
    assert_eq!(*committed, *data2);
    assert!(!tx.is_active());

    // Can start new transaction after commit
    let data3 = Arc::new(json!({"value": 3}));
    assert!(tx.begin(data3).is_ok());
}

#[test]
fn test_transaction_rollback_doesnt_persist() {
    let tx = TransactionState::new();

    let data1 = Arc::new(json!({"original": true}));
    tx.begin(data1).unwrap();

    let data2 = Arc::new(json!({"modified": true}));
    tx.update_data(data2);

    tx.rollback().unwrap();

    // After rollback, data is gone
    assert!(tx.get_data().is_none());
}
