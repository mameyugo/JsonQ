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
        Value::Array(arr) => {
            let mut ht = ext_php_rs::types::ZendHashTable::new();
            for item in arr { let _ = ht.push(value_to_zval(item)); }
            z.set_hashtable(ht);
        }
        Value::Object(map) => {
            let mut ht = ext_php_rs::types::ZendHashTable::new();
            for (k, v) in map { let _ = ht.insert(k, value_to_zval(v)); }
            z.set_hashtable(ht);
        }
    }
    z
}

fn zval_to_value(zval: &Zval) -> Value {
    if zval.is_null() { return Value::Null; }
    if zval.is_bool() { return Value::Bool(zval.bool().unwrap_or(false)); }
    if zval.is_long() { return Value::Number(serde_json::Number::from(zval.long().unwrap_or(0))); }
    if zval.is_double() {
        return serde_json::Number::from_f64(zval.double().unwrap_or(0.0)).map(Value::Number).unwrap_or(Value::Null);
    }
    if zval.is_string() { return Value::String(zval.str().unwrap_or("").to_string()); }
    Value::Null
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

    fn read(&self) -> Result<Value, String> {
        let s = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        serde_json::from_str(&s).map_err(|e| e.to_string())
    }

    fn write(&self, data: &Value) -> Result<(), String> {
        let s = serde_json::to_string(data).map_err(|e| e.to_string())?;
        fs::write(&self.path, s).map_err(|e| e.to_string())
    }
}

fn rp<'a>(root: &'a Value, dp: &str) -> Option<&'a Value> {
    if dp.is_empty() { return Some(root); }
    let mut c = root;
    for k in dp.split('.') { c = match c { Value::Object(m) => m.get(k)?, _ => return None }; }
    Some(c)
}

fn sap(root: &mut Value, dp: &str, value: Value) {
    let keys: Vec<&str> = dp.split('.').collect();
    let mut c = root;
    for (i, k) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            match c { Value::Object(m) => { m.insert(k.to_string(), value); } _ => {} }
            return;
        }
        match c {
            Value::Object(m) => { if !m.contains_key(*k) { m.insert(k.to_string(), Value::Object(Map::new())); } c = m.get_mut(*k).unwrap(); }
            _ => return,
        }
    }
}

#[php_class(name = "Rjson\\Store")]
pub struct RjsonStore { inner: Option<StoreInner> }

#[php_impl]
impl RjsonStore {
    #[php_method] pub fn __construct(path: String) -> RjsonStore { RjsonStore { inner: Some(StoreInner::new(path)) } }
    #[php_method] pub fn get(&self, path: String) -> Zval { let i = self.inner.as_ref().unwrap(); let d = i.read().unwrap(); rp(&d, &path).map(|v| value_to_zval(v)).unwrap_or_else(Zval::new) }
    #[php_method] pub fn set(&self, path: String, value: &Zval) -> bool { let i = self.inner.as_ref().unwrap(); let mut d = i.read().unwrap(); sap(&mut d, &path, zval_to_value(value)); i.write(&d).is_ok() }
    #[php_method] pub fn has(&self, path: String) -> bool { let i = self.inner.as_ref().unwrap(); let d = i.read().unwrap(); rp(&d, &path).is_some() }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { module }
