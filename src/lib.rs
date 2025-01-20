use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use memmap2::Mmap;
use serde_json::{json, Map, Value};
use std::fs::{self, File};
use std::io::Write;
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
    if zval.is_array() {
        if let Some(ht) = zval.array() {
            let mut map = Map::new();
            for (idx, key, val) in ht.iter() {
                let k = key.map(|s| s.to_string()).unwrap_or_else(|| idx.to_string());
                map.insert(k, zval_to_value(val));
            }
            return Value::Object(map);
        }
    }
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
        let meta = fs::metadata(&self.path).map_err(|e| e.to_string())?;
        let flen = meta.len() as usize;
        let file = File::open(&self.path).map_err(|e| e.to_string())?;
        let data: Value = if flen == 0 { Value::Object(Map::new()) }
            else if flen < 64 { serde_json::from_str(&fs::read_to_string(&self.path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())? }
            else { serde_json::from_slice(&unsafe { Mmap::map(&file) }.map_err(|e| e.to_string())?).map_err(|e| e.to_string())? };
        Ok(data)
    }

    fn write(&self, data: &Value) -> Result<(), String> {
        let bytes = serde_json::to_vec(data).map_err(|e| e.to_string())?;
        fs::write(&self.path, bytes).map_err(|e| e.to_string())
    }
}

fn rn<'a>(item: &'a Value, key: &str) -> Option<&'a Value> {
    if let Some(o) = item.as_object() { if let Some(v) = o.get(key) { return Some(v); } }
    rp(item, key)
}

fn mat(item: &Value, cond: &Value) -> bool {
    let co = match cond.as_object() { Some(o) => o, None => return false };
    for (k, c) in co {
        let fv = rn(item, k);
        if fv.unwrap_or(&Value::Null) != c { return false; }
    }
    true
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

struct IndexStore {
    single: HashMap<String, HashMap<String, Vec<usize>>>,
    built_at: u64,
}

impl StoreInner {
    fn new(path: String) -> Self {
        let p = PathBuf::from(&path);
        if !p.exists() { let _ = fs::write(&p, "{}"); }
        Self { path: p }
    }
    // ... existing read/write ...
}

#[php_impl]
impl RjsonStore {
    // ... existing ...
    #[php_method] pub fn createIndex(&self, collection: String, field: String) -> bool { true } // Initial index stub
    #[php_method] pub fn executeQuery(&self, collection: String, query_spec: &Zval) -> Zval {
        let i = self.inner.as_ref().unwrap(); let q = zval_to_value(query_spec);
        let d = i.read().unwrap();
        let arr = match rp(&d, &collection) { Some(Value::Array(a)) => a, _ => return Zval::new() };
        value_to_zval(&Value::Array(arr.clone())) // Fluent placeholder
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { module }
