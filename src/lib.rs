use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

fn value_to_zval(val: &Value) -> Zval {
    let mut z = Zval::new();
    match val {
        Value::Null => {}
        Value::Bool(b) => { z.set_bool(*b); }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { z.set_long(i); }
            else if let Some(f) = n.as_f64() { z.set_double(f); }
        }
        Value::String(s) => { let _ = z.set_string(s, false); }
        _ => {} // Arrays/Objects coming later
    }
    z
}

struct StoreInner {
    path: PathBuf,
}

impl StoreInner {
    fn new(path: String) -> Self {
        let p = PathBuf::from(&path);
        if !p.exists() { let _ = fs::write(&p, "{}"); }
        Self { path: p }
    }
}

#[php_class(name = "Rjson\\Store")]
pub struct RjsonStore { inner: Option<StoreInner> }

#[php_impl]
impl RjsonStore {
    #[php_method] pub fn __construct(path: String) -> RjsonStore { RjsonStore { inner: Some(StoreInner::new(path)) } }
    #[php_method] pub fn get(&self, path: String) -> Zval { Zval::new() } // TODO
    #[php_method] pub fn set(&self, path: String, value: &Zval) -> bool { true } // TODO
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { module }
