//! JSON Schema validation
//!
//! This module provides JSON Schema-like validation for values.
//! Supports type checking, constraints, nested validation, and more.
//!
//! # Supported Constraints
//!
//! - **Type validation**: `type: "string" | "number" | "integer" | "boolean" | "array" | "object" | "null"`
//! - **Numeric constraints**: `min`, `max`, `minimum`, `maximum`
//! - **String constraints**: `minLength`, `maxLength`, `pattern`, `format`
//! - **Enum validation**: `enum: [values...]`
//! - **Object validation**: `required: [fields...]`, `properties: {...}`
//! - **Array validation**: `items: {...}`
//!
//! # Examples
//!
//! ```rust
//! use jsonq::validation::validate;
//! use serde_json::json;
//!
//! let value = json!({"name": "Alice", "age": 30});
//! let schema = json!({
//!     "type": "object",
//!     "required": ["name", "age"],
//!     "properties": {
//!         "name": {"type": "string"},
//!         "age": {"type": "integer", "min": 0, "max": 150}
//!     }
//! });
//!
//! let errors = validate(&value, &schema, "user");
//! assert!(errors.is_empty());
//! ```

mod validator;
mod types;
mod constraints;

pub use validator::validate;
pub use types::check_type;
pub use constraints::{
    validate_number_constraints,
    validate_string_constraints,
    validate_enum,
    validate_required_fields,
};
