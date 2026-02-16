//! Convert serde_json::Value to PHP Zval
//!
//! This module handles the conversion from Rust's strongly-typed JSON values
//! to PHP's dynamic Zval type system.

//! # Memory Safety
//!
//! According to ext-php-rs 0.15.6 documentation:
//! - `set_string(s, false)` -> Copies string to PHP heap (SAFE)
//! - `set_string(s, true)` -> Uses persistent allocation (for globals)
//!
//! We use `false` because:
//! 1. The string `s` comes from `serde_json::Value` which is owned by Rust
//! 2. The `Zval` will outlive the `Value` only in PHP userland
//! 3. PHP needs its own copy to manage the string's lifecycle
//!
//! This is safe because PHP makes a copy and manages it independently.

use ext_php_rs::types::{Zval, ZendHashTable};
use serde_json::Value;

/// Convert a serde_json::Value to a PHP Zval
///
/// # Safety
///
/// Strings are copied to PHP heap (`persistent = false`) to ensure
/// PHP owns the string data and manages its lifecycle correctly.
/// - Nested structures are recursively converted
/// - Numbers overflow gracefully (saturating to f64 if needed)
///
/// # Examples
///
/// ```rust,no_run
/// use serde_json::json;
/// use jsonq::conversion::value_to_zval;
///
/// let data = json!({
///     "name": "Alice",
///     "age": 30,
///     "active": true
/// });
///
/// let zval = value_to_zval(&data);
/// // zval can now be returned to PHP
/// ```
pub fn value_to_zval(val: &Value) -> Zval {
    let mut z = Zval::new();
    
    match val {
        Value::Null => {
            z.set_null();
        }
        
        Value::Bool(b) => {
            z.set_bool(*b);
        }
        
        Value::Number(n) => {
            // Try integer first for better precision
            if let Some(i) = n.as_i64() {
                z.set_long(i);
            } else if let Some(u) = n.as_u64() {
                // Handle large unsigned integers
                if u <= i64::MAX as u64 {
                    z.set_long(u as i64);
                } else {
                    // Fallback to float for very large numbers
                    z.set_double(u as f64);
                }
            } else {
                // Float
                z.set_double(n.as_f64().unwrap_or(0.0));
            }
        }
        
        Value::String(s) => {
            // ✅ CORRECT: false = copy to PHP heap
            // PHP will own and manage this string
            let _ = z.set_string(s, false);
        }
        
        Value::Array(arr) => {
            let mut ht = ZendHashTable::new();
            
            // Convert each element
            for item in arr {
                let item_zval = value_to_zval(item);
                let _ = ht.push(item_zval);
            }
            
            z.set_hashtable(ht);
        }
        
        Value::Object(map) => {
            let mut ht = ZendHashTable::new();
            
            // Convert each key-value pair
            for (k, v) in map {
                let val_zval = value_to_zval(v);
                let _ = ht.insert(k.as_str(), val_zval);
            }
            
            z.set_hashtable(ht);
        }
    }
    
    z
}
