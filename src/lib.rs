//! JsonQ - High-performance JSON file storage engine for PHP
#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(non_snake_case)]

pub mod conversion;
pub mod store;
pub mod path;
pub mod utils;
pub mod validation;
pub mod index;
pub mod query;
pub mod config;
pub mod security;

#[cfg(test)]
pub mod php;

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use conversion::{value_to_zval, zval_to_value};
use store::StoreInner;
use path::{read_path, read_path_mut, read_nested, write_path, remove_path};
use utils::{value_key, search_in_value, merge_values};
use validation::validate;
use query::{matches, execute_query};
use security::validate_path_depth;
use serde_json::{Map, Value, json};
use std::fs;
use std::sync::Arc;
use std::path::PathBuf;

// ══════════ HELPERS ══════════

// Helper vkey remains temporarily until items that still use it are refactored
pub(crate) fn vkey(v: Option<&Value>) -> String { value_key(v) }

// mat and exec_fluent have been moved to the query module

fn agg(arr: &Vec<Value>, field: &str, op: &str) -> Value {
    let nums: Vec<f64> = arr.iter().filter_map(|i| read_nested(i, field).and_then(|v| v.as_f64())).collect();
    if nums.is_empty() { return Value::Null; }
    match op {
        "sum" => json!(nums.iter().sum::<f64>()), "avg" => json!(nums.iter().sum::<f64>() / nums.len() as f64),
        "min" => json!(nums.iter().fold(f64::INFINITY, |a, &b| a.min(b))), "max" => json!(nums.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
        "count" => json!(arr.len()), _ => Value::Null
    }
}

fn grp(arr: &Vec<Value>, field: &str) -> Value {
    let mut m = Map::new();
    for i in arr { let k = vkey(read_nested(i, field)); m.entry(k).or_insert(Value::Array(vec![])).as_array_mut().unwrap().push(i.clone()); }
    Value::Object(m)
}

fn plk(arr: &Vec<Value>, fields: &[&str]) -> Vec<Value> {
    if fields.len() == 1 {
        let f = fields[0];
        return arr.iter().map(|i| read_nested(i, f).cloned().unwrap_or(Value::Null)).collect();
    }
    arr.iter().map(|i| {
        let mut o = Map::new();
        for &f in fields { if let Some(v) = read_nested(i, f) { o.insert(f.to_string(), v.clone()); } }
        Value::Object(o)
    }).collect()
}



// ══════════ PHP CLASS ══════════

#[php_class]
#[php(name = "JsonQ\\Store")]
pub struct JsonStore { inner: Option<StoreInner> }

#[php_impl]
impl JsonStore {
    pub fn __construct(path: String) -> Self { 
        match StoreInner::new(path) {
            Ok(inner) => Self { inner: Some(inner) },
            Err(e) => {
                let _ = PhpException::new(format!("JsonQ Error: {}", e), 0, ext_php_rs::zend::ce::exception()).throw();
                Self { inner: None }
            }
        }
    }

    #[php(name = "setOption")]
    pub fn set_option(&self, key: String, value: &Zval) -> bool {
        let i = match &self.inner { Some(i) => i, None => return false };
        let mut opts = i.get_opts();
        match key.as_str() {
            "pretty" | "pretty_print" => { opts.pretty = value.bool().unwrap_or(false); i.set_opts(opts); true }
            "fsync" | "sync" => { opts.fsync = value.bool().unwrap_or(false); i.set_opts(opts); true }
            _ => false,
        }
    }
    
    #[php(name = "getOption")]
    pub fn get_option(&self, key: String) -> Zval {
        let i = match &self.inner { Some(i) => i, None => return Zval::new() };
        let opts = i.get_opts();
        let mut z = Zval::new();
        match key.as_str() { "pretty"|"pretty_print" => { z.set_bool(opts.pretty); } "fsync"|"sync" => { z.set_bool(opts.fsync); } _ => {} }
        z
    }

    #[php(name = "beginTransaction")]
    pub fn begin_transaction(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        i.begin_transaction().map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn commit(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        i.commit().map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn rollback(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        i.rollback().map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    #[php(name = "inTransaction")]
    pub fn in_transaction(&self) -> bool { self.inner.as_ref().map(|i| i.in_transaction()).unwrap_or(false) }

    #[php(name = "setMany")]
    pub fn set_many(&self, pairs: &Zval) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let pv = zval_to_value(pairs);
        let po = match pv.as_object() { Some(o) => o, None => return Ok(0) };
        let cd = i.read().map_err(|e: String| PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for (path, value) in po { 
            validate_path_depth(path).map_err(|e| PhpException::from(e))?;
            write_path(&mut data, path, value.clone()); 
            count += 1; 
        }
        i.write(&data).map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(count)
    }
    
    #[php(name = "removeMany")]
    pub fn remove_many(&self, paths: Vec<String>) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e: String| PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for path in &paths { 
            validate_path_depth(path).map_err(|e| PhpException::from(e))?;
            if remove_path(&mut data, path) { count += 1; } 
        }
        i.write(&data).map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(count)
    }

    #[php(name = "toJson")]
    pub fn to_json(&self, pretty: Option<bool>) -> PhpResult<String> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e: String| PhpException::from(e.to_string()))?;
        if pretty.unwrap_or(false) { serde_json::to_string_pretty(&*cd).map_err(|e| e.to_string().into()) } else { serde_json::to_string(&*cd).map_err(|e| e.to_string().into()) }
    }
    
    #[php(name = "fromJson")]
    pub fn from_json(&self, json_str: String) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let data: Value = serde_json::from_str(&json_str).map_err(|e| PhpException::from(e.to_string()))?;
        i.write(&data).map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(true)
    }

    #[php(name = "getAll")]
    pub fn get_all(&self) -> Zval { self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| value_to_zval(&cd)).ok()).unwrap_or_else(Zval::new) }
    
    pub fn clear(&self) -> PhpResult<bool> { let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?; i.write(&Value::Object(Map::new())).map_err(|e: String| PhpException::from(e.to_string()))?; Ok(true) }
    
    pub fn search(&self, collection: String, keyword: String) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() };
        let kw = keyword.to_lowercase();
        i.read().map(|cd: Arc<Value>| {
            let arr = match read_path(&cd, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            let matched: Vec<Value> = arr.iter().filter(|item| search_in_value(item, &kw)).cloned().collect();
            value_to_zval(&Value::Array(matched))
        }).unwrap_or_else(|_| Zval::new())
    }

    pub fn get(&self, path: String) -> PhpResult<Zval> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        let data = i.read().map_err(|e| PhpException::from(e))?;
        Ok(read_path(&data, &path).map(|v| value_to_zval(v)).unwrap_or_else(Zval::new))
    }
    
    pub fn has(&self, path: String) -> bool { 
        if validate_path_depth(&path).is_err() { return false; }
        self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| read_path(&cd, &path).is_some()).ok()).unwrap_or(false) 
    }
    
    pub fn count(&self, path: String) -> i64 { 
        if validate_path_depth(&path).is_err() { return -1; }
        self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| match read_path(&cd, &path) { Some(Value::Array(a)) => a.len() as i64, Some(Value::Object(o)) => o.len() as i64, _ => -1 }).ok()).unwrap_or(-1) 
    }
    
    pub fn keys(&self, path: String) -> Vec<String> { 
        if validate_path_depth(&path).is_err() { return vec![]; }
        self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| match read_path(&cd, &path) { Some(Value::Object(o)) => o.keys().cloned().collect(), _ => vec![] }).ok()).unwrap_or_default() 
    }

    pub fn set(&self, path: String, value: &Zval) -> PhpResult<bool> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        let v = zval_to_value(value); 
        i.mutate(|d: &mut Value| write_path(d, &path, v)).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn remove(&self, path: String) -> PhpResult<bool> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        i.mutate(|d: &mut Value| { remove_path(d, &path); }).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn push(&self, path: String, value: &Zval) -> PhpResult<bool> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        let v = zval_to_value(value); 
        i.mutate(|d: &mut Value| { match read_path_mut(d, &path) { Some(Value::Array(a)) => { a.push(v); } _ => {} } }).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn merge(&self, path: String, value: &Zval) -> PhpResult<bool> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        let nv = zval_to_value(value); 
        i.mutate(|d: &mut Value| { if let Some(e) = read_path_mut(d, &path) { merge_values(e, &nv); } else { write_path(d, &path, nv); } }).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn increment(&self, path: String, amount: Option<f64>) -> PhpResult<bool> { 
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let amt: f64 = amount.unwrap_or(1.0); 
        let i = self.inner.as_ref().ok_or_else(|| PhpException::from("Not init"))?;
        i.mutate(|d: &mut Value| { if let Some(v) = read_path_mut(d, &path) { if let Some(n) = v.as_f64() { *v = json!(n + (amt as f64)); } } }).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }
    
    pub fn decrement(&self, path: String, amount: Option<f64>) -> PhpResult<bool> { self.increment(path, Some(-(amount.unwrap_or(1.0)))) }

    pub fn find(&self, collection: String, conditions: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let cond = zval_to_value(conditions);
        i.read().map(|cd: Arc<Value>| {
            let arr = match read_path(&cd, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            if let Some(co) = cond.as_object() { if co.len() == 1 { if let Some((f, v)) = co.iter().next() { if !f.starts_with('$') && !v.is_object() { if let Some(pos) = i.idx_lookup(&collection, f, v) { return value_to_zval(&Value::Array(pos.iter().filter_map(|&j| arr.get(j).cloned()).collect::<Vec<_>>())); } } } } }
            value_to_zval(&Value::Array(arr.iter().filter(|item| matches(item, &cond)).cloned().collect()))
        }).unwrap_or_else(|_| Zval::new())
    }
    
    #[php(name = "findOne")]
    pub fn find_one(&self, collection: String, conditions: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let c = zval_to_value(conditions); i.read().map(|cd: Arc<Value>| match read_path(&cd, &collection) { Some(Value::Array(a)) => a.iter().find(|item| matches(item, &c)).map(|f| value_to_zval(f)).unwrap_or_else(Zval::new), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "executeQuery")]
    pub fn execute_query(&self, collection: String, query_spec: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let q = zval_to_value(query_spec); i.read().map(|cd: Arc<Value>| match read_path(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(execute_query(a, &q))), _ => value_to_zval(&Value::Array(vec![])) }).unwrap_or_else(|_| Zval::new()) }

    pub fn aggregate(&self, collection: String, field: String, operation: String) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; i.read().map(|cd: Arc<Value>| match read_path(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&agg(a, &field, &operation)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "groupBy")]
    pub fn group_by(&self, collection: String, field: String) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; i.read().map(|cd: Arc<Value>| match read_path(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&grp(a, &field)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    pub fn pluck(&self, collection: String, fields: Vec<String>) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let fr: Vec<&str> = fields.iter().map(|s| s.as_str()).collect(); i.read().map(|cd: Arc<Value>| match read_path(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(plk(a, &fr))), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }

    pub fn validate(&self, path: String, schema: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(schema); i.read().map(|cd: Arc<Value>| { let t = match read_path(&cd,&path){Some(v)=>v,None=>return Zval::new()}; let e = validate(t, &sv, &path); value_to_zval(&json!({"valid":e.is_empty(),"error_count":e.len(),"errors":e})) }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "validateCollection")]
    pub fn validate_collection(&self, path: String, item_schema: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(item_schema);
        i.read().map(|cd: Arc<Value>| {
            let arr = match read_path(&cd, &path) { Some(Value::Array(a)) => a, _ => return Zval::new() };
            let mut ae = Vec::new(); let mut inv = 0usize;
            for (j, item) in arr.iter().enumerate() { let e = validate(item, &sv, &format!("{}.{}",path,j)); if !e.is_empty() { inv += 1; ae.push(json!({"index":j,"errors":e})); } }
            value_to_zval(&json!({"valid":ae.is_empty(),"total_items":arr.len(),"valid_items":arr.len()-inv,"invalid_items":inv,"details":ae}))
        }).unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "createIndex")]
    pub fn create_index(&self, collection: String, field: String) -> bool { self.inner.as_ref().map(|i: &StoreInner| i.build_index(&collection, &field).is_ok()).unwrap_or(false) }
    
    #[php(name = "createCompoundIndex")]
    pub fn create_compound_index(&self, collection: String, fields: Vec<String>) -> bool { self.inner.as_ref().map(|i: &StoreInner| i.build_compound(&collection, &fields).is_ok()).unwrap_or(false) }
    
    #[php(name = "indexLookup")]
    pub fn index_lookup(&self, collection: String, field: String, value: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let v = zval_to_value(value);
        if let Some(pos) = i.idx_lookup(&collection, &field, &v) {
            i.read().map(|cd: Arc<Value>| if let Some(Value::Array(a)) = read_path(&cd, &collection) { value_to_zval(&Value::Array(pos.iter().filter_map(|&j| a.get(j).cloned()).collect::<Vec<_>>())) } else { Zval::new() }).unwrap_or_else(|_| Zval::new())
        } else { Zval::new() }
    }
    
    #[php(name = "listIndexes")]
    pub fn list_indexes(&self) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let idx = i.indexes.read().unwrap();
        let mut r = Vec::new();
        for (c, s) in idx.iter() {
            for (f, im) in &s.single { let uv = im.len(); let te: usize = im.values().map(|v: &Vec<usize>|v.len()).sum(); r.push(json!({"collection":c,"type":"single","field":f,"unique_values":uv,"total_entries":te})); }
            for (f, im) in &s.compound { let uv = im.len(); let te: usize = im.values().map(|v: &Vec<usize>|v.len()).sum(); r.push(json!({"collection":c,"type":"compound","fields":f,"unique_values":uv,"total_entries":te})); }
        }
        value_to_zval(&Value::Array(r))
    }
    
    #[php(name = "dropIndex")]
    pub fn drop_index(&self, collection: String) -> bool { self.inner.as_ref().map(|i: &StoreInner| i.indexes.write().unwrap().remove(&collection).is_some()).unwrap_or(false) }
    
    #[php(name = "dropAllIndexes")]
    pub fn drop_all_indexes(&self) -> i64 { self.inner.as_ref().map(|i: &StoreInner| { let mut idx = i.indexes.write().unwrap(); let c = idx.len() as i64; idx.clear(); c }).unwrap_or(0) }

    pub fn stats(&self) -> Zval {
        let i = match &self.inner { Some(i) => i, None => return Zval::new() };
        let meta = match fs::metadata(&i.path) { Ok(m) => m, Err(_) => return Zval::new() };
        i.read().map(|cd: Arc<Value>| {
            let fs = meta.len(); let fsh = if fs<1024{format!("{} B",fs)}else if fs<1048576{format!("{:.2} KB",fs as f64/1024.0)}else{format!("{:.2} MB",fs as f64/1048576.0)};
            let keys: Vec<Value> = if let Value::Object(o) = cd.as_ref() { o.keys().map(|k| Value::String(k.clone())).collect() } else { vec![] };
            let kc = if let Value::Object(o) = cd.as_ref() { o.len() } else { 0 };
            let ic: usize = i.indexes.read().unwrap().values().map(|s| s.single.len()+s.compound.len()).sum();
            value_to_zval(&json!({"file_path":i.path.to_string_lossy(),"file_size":fs,"file_size_h":fsh,"top_level_keys":keys,"key_count":kc,"active_indexes":ic}))
        }).unwrap_or_else(|_| Zval::new())
    }
    
    pub fn backup(&self, backup_path: Option<String>) -> PhpResult<String> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let t = match backup_path { Some(p) if !p.is_empty() => p, _ => format!("{}.backup.{}",i.path.to_string_lossy(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()) };
        fs::copy(&i.path, &t).map_err(|e| PhpException::from(e.to_string()))?;
        Ok(t)
    }
    
    pub fn restore(&self, backup_path: String) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        fs::copy(&backup_path, &i.path).map_err(|e| PhpException::from(e.to_string()))?;
        *i.cache.write().unwrap() = None; i.indexes.write().unwrap().clear();
        Ok(true)
    }
}

// ══════════ GLOBAL CONFIG API ══════════

/// Get current configuration
#[php_function]
pub fn jsonq_get_config() -> PhpResult<Zval> {
    let config = crate::config::Config::get();
    
    let result = json!({
        "max_file_size": config.max_file_size,
        "max_file_size_mb": config.max_file_size as f64 / (1024.0 * 1024.0),
        "max_validation_depth": config.max_validation_depth,
        "max_path_depth": config.max_path_depth,
        "allowed_extensions": config.allowed_extensions,
        "base_path": config.base_path.map(|p| p.to_string_lossy().to_string()),
    });
    
    Ok(value_to_zval(&result))
}

/// Set maximum file size (e.g. "100M", "1G")
#[php_function]
pub fn jsonq_set_max_file_size(size: String) -> PhpResult<bool> {
    let parsed_size = crate::config::Config::parse_size(&size)
        .map_err(|e| PhpException::from(e))?;
    
    crate::config::Config::update(|cfg| {
        cfg.max_file_size = parsed_size;
    });
    
    Ok(true)
}

/// Set allowed extensions (comma-separated, e.g. "json,db")
#[php_function]
pub fn jsonq_set_allowed_extensions(extensions: String) -> PhpResult<bool> {
    let exts = crate::config::Config::parse_extensions(&extensions);
    if exts.is_empty() {
        return Err(PhpException::from("At least one extension must be allowed"));
    }
    
    crate::config::Config::update(|cfg| {
        cfg.allowed_extensions = exts;
    });
    
    Ok(true)
}

/// Set base path restriction
#[php_function]
pub fn jsonq_set_base_path(path: String) -> PhpResult<bool> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() || !path_buf.is_dir() {
        return Err(PhpException::from(format!("Invalid base path: {}", path)));
    }
    
    let canonical = path_buf.canonicalize()
        .map_err(|e| PhpException::from(format!("Failed to canonicalize base path: {}", e)))?;
    
    crate::config::Config::update(|cfg| {
        cfg.base_path = Some(canonical);
    });
    
    Ok(true)
}

/// Clear base path restriction
#[php_function]
pub fn jsonq_clear_base_path() -> bool {
    crate::config::Config::update(|cfg| {
        cfg.base_path = None;
    });
    true
}

#[php_function] 
pub fn jsonq_version() -> String { 
    env!("CARGO_PKG_VERSION").to_string() 
}

#[php_module] 
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { 
    // ✅ Initialize global configuration
    crate::config::Config::init();
    crate::config::php_ini::load_from_ini();

    module
        .function(wrap_function!(jsonq_version))
        .function(wrap_function!(jsonq_get_config))
        .function(wrap_function!(jsonq_set_max_file_size))
        .function(wrap_function!(jsonq_set_allowed_extensions))
        .function(wrap_function!(jsonq_set_base_path))
        .function(wrap_function!(jsonq_clear_base_path))
        .class::<JsonStore>()
}
