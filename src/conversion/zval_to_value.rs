//! Convert PHP Zval to serde_json::Value
//!
//! This module handles the conversion from PHP's dynamic Zval type system
//! to Rust's strongly-typed JSON values.

use ext_php_rs::types::{Zval, ZendHashTable, ArrayKey};
use serde_json::{Value, Map, Number};

/// Convert a PHP Zval to a serde_json::Value
///
/// Handles all PHP types and converts them to their JSON equivalents:
/// - null → Value::Null
/// - bool → Value::Bool
/// - int → Value::Number (i64)
/// - float → Value::Number (f64)
/// - string → Value::String
/// - array → Value::Array or Value::Object (depending on keys)
///
/// # Array Detection
///
/// PHP arrays can be sequential (like JSON arrays) or associative (like JSON objects).
/// This function automatically detects which type by checking if:
/// 1. All keys are numeric
/// 2. Keys start at 0 and increment sequentially
///
/// # Examples
///
/// ```rust,no_run
/// use jsonq::conversion::zval_to_value;
/// use ext_php_rs::types::Zval;
///
/// let zval = /* some Zval from PHP */;
/// let value = zval_to_value(&zval);
///
/// // Now you can work with it as JSON
/// if let Some(s) = value.as_str() {
///     println!("String value: {}", s);
/// }
/// ```
pub fn zval_to_value(zval: &Zval) -> Value {
    // Check type in order of likelihood for performance
    
    if zval.is_null() {
        return Value::Null;
    }
    
    if zval.is_bool() {
        return Value::Bool(zval.bool().unwrap_or(false));
    }
    
    if zval.is_long() {
        let i = zval.long().unwrap_or(0);
        return Value::Number(Number::from(i));
    }
    
    if zval.is_double() {
        let f = zval.double().unwrap_or(0.0);
        return Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null); // NaN/Inf → null
    }
    
    if zval.is_string() {
        let s = zval.str().unwrap_or("").to_string();
        return Value::String(s);
    }
    
    if let Some(ht) = zval.array() {
        return ht_to_value(ht);
    }
    
    // Unknown type → null
    Value::Null
}

/// Convert a PHP HashTable to either a JSON Array or Object
///
/// # Algorithm
///
/// Determines if the hashtable should be an array or object by checking:
/// 1. If empty → array (for consistency)
/// 2. Iterate through keys:
///    - If any key is a string → object
///    - If all keys are integers starting at 0 and sequential → array
///    - Otherwise → object
///
/// # Performance
///
/// This function iterates through all keys once to determine type,
/// then iterates again to build the result. For large arrays, this
/// is O(2n) which is acceptable.
///
/// # Examples
///
/// ```rust
/// use jsonq::conversion::ht_to_value;
/// use serde_json::json;
///
/// // PHP: [1, 2, 3] → JSON: [1, 2, 3]
/// // PHP: ["a" => 1, "b" => 2] → JSON: {"a": 1, "b": 2}
/// // PHP: [0 => "x", 2 => "y"] → JSON: {"0": "x", "2": "y"} (not sequential)
/// ```
pub fn ht_to_value(ht: &ZendHashTable) -> Value {
    // Empty hashtable → empty array
    if ht.len() == 0 {
        return Value::Array(vec![]);
    }
    
    // Single-pass conversion with on-the-fly detection
    // We build both structures during iteration, then return the appropriate one
    let mut arr = Vec::with_capacity(ht.len());
    let mut map = Map::new();
    let mut expected_index: u64 = 0;
    let mut is_sequential = true;
    
    for (key, val) in ht.iter() {
        // Check if this key maintains sequential array property
        match key {
            ArrayKey::Long(idx) => {
                if idx as u64 != expected_index {
                    is_sequential = false;
                }
                expected_index += 1;
            }
            // Any string key means it's an object
            ArrayKey::String(_) | ArrayKey::Str(_) => {
                is_sequential = false;
            }
        }
        
        // Convert value once
        let value = zval_to_value(val);
        
        // Build both structures (we'll discard one later)
        // This is faster than iterating twice, even with the extra memory
        if is_sequential {
            // Only build array if still sequential
            arr.push(value.clone());
        }
        
        // Always build map for object case
        let key_str = match key {
            ArrayKey::String(s) => s.to_string(),
            ArrayKey::Str(s) => s.to_string(),
            ArrayKey::Long(idx) => idx.to_string(),
        };
        map.insert(key_str, value);
    }
    
    // Return appropriate structure
    if is_sequential {
        Value::Array(arr)
    } else {
        Value::Object(map)
    }
}

/// Detect if a PHP hashtable represents a sequential array
///
/// **Note**: This function is now deprecated in favor of the single-pass
/// implementation in `ht_to_value()`. It's kept for reference but not used.
///
/// Returns true if:
/// - All keys are integers (Long)
/// - Keys start at 0
/// - Keys increment by 1 each time (no gaps)
///
/// # Examples
///
/// ```text
/// [0 => 'a', 1 => 'b', 2 => 'c'] → true (sequential)
/// [0 => 'a', 2 => 'b'] → false (gap at index 1)
/// [1 => 'a', 2 => 'b'] → false (doesn't start at 0)
/// ['x' => 'a', 'y' => 'b'] → false (string keys)
/// [0 => 'a', 'x' => 'b'] → false (mixed keys)
/// ```
#[allow(dead_code)]
fn detect_sequential_array(ht: &ZendHashTable) -> bool {
    let mut expected_index: u64 = 0;
    
    for (key, _) in ht.iter() {
        match key {
            ArrayKey::Long(idx) => {
                // Check if index matches expected sequence
                if idx as u64 != expected_index {
                    return false;
                }
                expected_index += 1;
            }
            // Any string key means it's an object
            ArrayKey::String(_) | ArrayKey::Str(_) => {
                return false;
            }
        }
    }
    
    true
}
