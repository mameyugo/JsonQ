//! JsonQ - High-performance JSON file storage engine for PHP
#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(non_snake_case)]

pub mod config;
pub mod conversion;
pub mod error;
pub mod index;
pub mod metrics;
pub mod path;
pub mod query;
pub mod security;
pub mod store;
pub mod utils;
pub mod validation;
pub mod stream;

#[cfg(test)]
pub mod php;

use crate::query::executor::QueryExecutor;
use crate::query::path::PathSegment;
use conversion::{value_to_zval, zval_to_value};
use ext_php_rs::exception::PhpException;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use path::{read_nested, read_path, read_path_mut, remove_path, write_path};
use query::{execute_query, matches};
use security::validate_path_depth;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use store::options::CompressionMethod;
use store::StoreInner;
use utils::{merge_values, search_in_value, value_key};
use validation::validate;
use crate::stream::{StreamReader, StreamFilter, FilteredStream};

// ══════════ HELPERS ══════════

// Helper vkey remains temporarily until items that still use it are refactored
pub(crate) fn vkey(v: Option<&Value>) -> String {
    value_key(v)
}

// mat and exec_fluent have been moved to the query module

fn agg(arr: &Vec<Value>, field: &str, op: &str) -> Value {
    let nums: Vec<f64> = arr
        .iter()
        .filter_map(|i| read_nested(i, field).and_then(|v| v.as_f64()))
        .collect();
    if nums.is_empty() {
        return Value::Null;
    }
    match op {
        "sum" => json!(nums.iter().sum::<f64>()),
        "avg" => json!(nums.iter().sum::<f64>() / nums.len() as f64),
        "min" => json!(nums.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
        "max" => json!(nums.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
        "count" => json!(arr.len()),
        _ => Value::Null,
    }
}

fn grp(arr: &Vec<Value>, field: &str) -> Value {
    let mut m = Map::new();
    for i in arr {
        let k = vkey(read_nested(i, field));
        let entry = m.entry(k).or_insert(Value::Array(vec![]));
        if let Some(arr_mut) = entry.as_array_mut() {
            arr_mut.push(i.clone());
        }
    }
    Value::Object(m)
}

fn plk(arr: &Vec<Value>, fields: &[&str]) -> Vec<Value> {
    if fields.len() == 1 {
        let f = fields[0];
        return arr
            .iter()
            .map(|i| read_nested(i, f).cloned().unwrap_or(Value::Null))
            .collect();
    }
    arr.iter()
        .map(|i| {
            let mut o = Map::new();
            for &f in fields {
                if let Some(v) = read_nested(i, f) {
                    o.insert(f.to_string(), v.clone());
                }
            }
            Value::Object(o)
        })
        .collect()
}

// ══════════ PHP CLASS ══════════

#[php_class]
#[php(name = "JsonQ\\Store")]
pub struct JsonStore {
    inner: Option<StoreInner>,
}

#[php_impl]
impl JsonStore {
    pub fn __construct(path: String) -> Self {
        match StoreInner::new(path) {
            Ok(inner) => {
                // If file exists, trigger a read to validate content (e.g. UTF-8)
                if inner.path.exists() {
                     match inner.read() {
                        Ok(_) => {},
                        Err(e) => {
                            let _ = PhpException::new(
                                format!("JsonQ Open Error: {}", e),
                                0,
                                ext_php_rs::zend::ce::exception(),
                            )
                            .throw();
                            return Self { inner: None };
                        }
                    }
                }
                Self { inner: Some(inner) }
            },
            Err(e) => {
                let _ = PhpException::new(
                    format!("JsonQ Error: {}", e),
                    0,
                    ext_php_rs::zend::ce::exception(),
                )
                .throw();
                Self { inner: None }
            }
        }
    }

    #[php(name = "getOption")]
    pub fn get_option(&self, key: String) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let opts = i.get_opts();
        let mut z = Zval::new();
        match key.as_str() {
            "pretty" | "pretty_print" => {
                z.set_bool(opts.pretty);
            }
            "fsync" | "sync" => {
                z.set_bool(opts.fsync);
            }
            "compression" => {
                z.set_string(&format!("{:?}", opts.compression), false)?;
            }
            _ => return Err(format!("Unknown option: {}", key).into()),
        }
        Ok(z)
    }

    #[php(name = "setOption")]
    pub fn set_option(&self, key: String, value: &Zval) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let mut opts = i.get_opts();
        match key.as_str() {
            "pretty" | "pretty_print" => {
                opts.pretty = value.bool().unwrap_or(false);
            }
            "fsync" | "sync" => {
                opts.fsync = value.bool().unwrap_or(false);
            }
            "compression" => {
                let s = value.str().unwrap_or("none").to_lowercase();
                opts.compression = match s.as_str() {
                    "gzip" => CompressionMethod::Gzip,
                    "zstd" => CompressionMethod::Zstd,
                    _ => CompressionMethod::None,
                };
            }
            _ => return Err(format!("Unknown option: {}", key).into()),
        }
        i.set_opts(opts);
        Ok(true)
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
    pub fn in_transaction(&self) -> bool {
        self.inner
            .as_ref()
            .map(|i| i.in_transaction())
            .unwrap_or(false)
    }

    #[php(name = "setMany")]
    pub fn set_many(&self, pairs: &Zval) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let pv = zval_to_value(pairs);
        let po = match pv.as_object() {
            Some(o) => o,
            None => return Ok(0),
        };
        let cd = i
            .read()
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for (path, value) in po {
            validate_path_depth(path).map_err(|e| PhpException::from(e))?;
            write_path(&mut data, path, value.clone());
            count += 1;
        }
        i.write(Arc::new(data))
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(count)
    }

    #[php(name = "removeMany")]
    pub fn remove_many(&self, paths: Vec<String>) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i
            .read()
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for path in &paths {
            validate_path_depth(path).map_err(|e| PhpException::from(e))?;
            if remove_path(&mut data, path) {
                count += 1;
            }
        }
        i.write(Arc::new(data))
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(count)
    }

    #[php(name = "toJson")]
    pub fn to_json(&self, pretty: Option<bool>) -> PhpResult<String> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i
            .read()
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        if pretty.unwrap_or(false) {
            serde_json::to_string_pretty(&*cd).map_err(|e| e.to_string().into())
        } else {
            serde_json::to_string(&*cd).map_err(|e| e.to_string().into())
        }
    }

    #[php(name = "fromJson")]
    pub fn from_json(&self, json_str: String) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let data: Value =
            serde_json::from_str(&json_str).map_err(|e| PhpException::from(e.to_string()))?;
        i.write(Arc::new(data))
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(true)
    }

    #[php(name = "getAll")]
    pub fn get_all(&self) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let cd = i.read()?;
        Ok(value_to_zval(&cd))
    }

    pub fn clear(&self) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        i.write(Arc::new(Value::Object(Map::new())))
            .map_err(|e: String| PhpException::from(e.to_string()))?;
        Ok(true)
    }

    pub fn search(&self, collection: String, keyword: String) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let kw = keyword.to_lowercase();
        let cd = i.read()?;
        let arr = match read_path(&cd, &collection) {
            Some(Value::Array(a)) => a,
            _ => return Ok(value_to_zval(&Value::Array(vec![]))),
        };
        let matched: Vec<Value> = arr
            .iter()
            .filter(|item| search_in_value(item, &kw))
            .cloned()
            .collect();
        Ok(value_to_zval(&Value::Array(matched)))
    }

    pub fn get(&self, path: String) -> PhpResult<Zval> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        let data = i.read().map_err(|e| PhpException::from(e))?;

        // Use Advanced JSONPath if path contains special selectors
        if path.contains("..") || path.contains("*") || path.contains('[') {
             use crate::query::path::PathSegment;
             use crate::query::executor::QueryExecutor;

             let segments = PathSegment::parse_json_path(&path).map_err(|e| {
                 PhpException::from(format!("{}", e))
             })?;
             
             let executor = QueryExecutor::new();
             let results = executor.execute_path(&data, &segments);
             
             if segments.iter().any(|s| matches!(s, PathSegment::Wildcard | PathSegment::RecursiveDescent(_) | PathSegment::Slice{..})) {
                 // Return array of results for multi-value selectors
                 Ok(value_to_zval(&Value::Array(results)))
             } else {
                 // Return single value for simple paths
                 Ok(results.first().map(|v| value_to_zval(v)).unwrap_or_else(|| Zval::new()))
             }
        } else {
            // Fast path for simple dot-notation
            Ok(read_path(&data, &path)
                .map(|v| value_to_zval(v))
                .unwrap_or_else(|| Zval::new()))
        }
    }

    pub fn has(&self, path: String) -> PhpResult<bool> {
        validate_path_depth(&path)?;
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let cd = i.read()?;
        Ok(read_path(&cd, &path).is_some())
    }

    pub fn count(&self, path: String) -> PhpResult<i64> {
        validate_path_depth(&path)?;
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let cd = i.read()?;
        match read_path(&cd, &path) {
            Some(Value::Array(a)) => Ok(a.len() as i64),
            Some(Value::Object(o)) => Ok(o.len() as i64),
            _ => Ok(0),
        }
    }

    pub fn keys(&self, path: String) -> PhpResult<Vec<String>> {
        validate_path_depth(&path)?;
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let cd = i.read()?;
        match read_path(&cd, &path) {
            Some(Value::Object(o)) => Ok(o.keys().cloned().collect()),
            _ => Ok(vec![]),
        }
    }

    pub fn set(&self, path: String, value: &Zval) -> PhpResult<bool> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        let v = zval_to_value(value);
        i.mutate(|d: &mut Value| write_path(d, &path, v))
            .map_err(|e| PhpException::from(e))?;
        Ok(true)
    }

    pub fn remove(&self, path: String) -> PhpResult<bool> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        i.mutate(|d: &mut Value| {
            remove_path(d, &path);
        })
        .map_err(|e| PhpException::from(e))?;
        Ok(true)
    }

    pub fn push(&self, path: String, value: &Zval) -> PhpResult<bool> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        let v = zval_to_value(value);
        i.mutate(|d: &mut Value| match read_path_mut(d, &path) {
            Some(Value::Array(a)) => {
                a.push(v);
            }
            _ => {}
        })
        .map_err(|e| PhpException::from(e))?;
        Ok(true)
    }


    pub fn merge(&self, path: String, value: &Zval) -> PhpResult<bool> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        let nv = zval_to_value(value);
        i.mutate(|d: &mut Value| {
            if let Some(e) = read_path_mut(d, &path) {
                merge_values(e, &nv);
            } else {
                write_path(d, &path, nv);
            }
        })
        .map_err(|e| PhpException::from(e))?;
        Ok(true)
    }

    pub fn increment(&self, path: String, amount: Option<f64>) -> PhpResult<bool> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        let amt: f64 = amount.unwrap_or(1.0);
        let i = self
            .inner
            .as_ref()
            .ok_or_else(|| PhpException::from("Not init"))?;
        i.mutate(|d: &mut Value| {
            if let Some(v) = read_path_mut(d, &path) {
                if let Some(n) = v.as_f64() {
                    *v = json!(n + (amt as f64));
                }
            }
        })
        .map_err(|e| PhpException::from(e))?;
        Ok(true)
    }

    pub fn decrement(&self, path: String, amount: Option<f64>) -> PhpResult<bool> {
        self.increment(path, Some(-(amount.unwrap_or(1.0))))
    }

    pub fn find(&self, collection: String, conditions: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let cond = zval_to_value(conditions);

        i.read()
            .map(|cd: Arc<Value>| {
                let arr = match read_path(&cd, &collection) {
                    Some(Value::Array(a)) => a,
                    _ => return value_to_zval(&Value::Array(vec![])),
                };

                use crate::query::optimizer::{optimize_query, ExecutionPlan};
                let plan = optimize_query(i, &collection, &cond);

                let matched: Vec<Value> = match plan {
                    ExecutionPlan::FullScan => arr
                        .iter()
                        .filter(|item| matches(item, &cond))
                        .cloned()
                        .collect(),
                    ExecutionPlan::IndexedScan {
                        field,
                        value,
                        remaining_conditions,
                        ..
                    } => {
                        if let Some(pos) = i.idx_lookup(&collection, &field, &value) {
                            let remaining = Value::Object(remaining_conditions);
                            pos.iter()
                                .filter_map(|&idx| arr.get(idx))
                                .filter(|item| matches(item, &remaining))
                                .cloned()
                                .collect()
                        } else {
                            vec![]
                        }
                    }
                };
                value_to_zval(&Value::Array(matched))
            })
            .unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "findOne")]
    pub fn find_one(&self, collection: String, conditions: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let c = zval_to_value(conditions);
        i.read()
            .map(|cd: Arc<Value>| match read_path(&cd, &collection) {
                Some(Value::Array(a)) => a
                    .iter()
                    .find(|item| matches(item, &c))
                    .map(|f| value_to_zval(f))
                    .unwrap_or_else(Zval::new),
                _ => Zval::new(),
            })
            .unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "executeQuery")]
    pub fn execute_query(&self, collection: String, query_spec: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let q = zval_to_value(query_spec);
        i.read()
            .map(|cd: Arc<Value>| match read_path(&cd, &collection) {
                Some(Value::Array(a)) => value_to_zval(&Value::Array(execute_query(a, &q))),
                _ => value_to_zval(&Value::Array(vec![])),
            })
            .unwrap_or_else(|_| Zval::new())
    }

    pub fn aggregate(&self, collection: String, field: String, operation: String) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        i.read()
            .map(|cd: Arc<Value>| match read_path(&cd, &collection) {
                Some(Value::Array(a)) => value_to_zval(&agg(a, &field, &operation)),
                _ => Zval::new(),
            })
            .unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "groupBy")]
    pub fn group_by(&self, collection: String, field: String) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        i.read()
            .map(|cd: Arc<Value>| match read_path(&cd, &collection) {
                Some(Value::Array(a)) => value_to_zval(&grp(a, &field)),
                _ => Zval::new(),
            })
            .unwrap_or_else(|_| Zval::new())
    }

    pub fn pluck(&self, collection: String, fields: Vec<String>) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let fr: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
        i.read()
            .map(|cd: Arc<Value>| match read_path(&cd, &collection) {
                Some(Value::Array(a)) => value_to_zval(&Value::Array(plk(a, &fr))),
                _ => Zval::new(),
            })
            .unwrap_or_else(|_| Zval::new())
    }

    pub fn validate(&self, path: String, schema: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let sv = zval_to_value(schema);
        i.read()
            .map(|cd: Arc<Value>| {
                let t = match read_path(&cd, &path) {
                    Some(v) => v,
                    None => return Zval::new(),
                };
                let e = validate(t, &sv, &path);
                value_to_zval(&json!({"valid":e.is_empty(),"error_count":e.len(),"errors":e}))
            })
            .unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "validateCollection")]
    pub fn validate_collection(&self, path: String, item_schema: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let sv = zval_to_value(item_schema);
        i.read().map(|cd: Arc<Value>| {
            let arr = match read_path(&cd, &path) { Some(Value::Array(a)) => a, _ => return Zval::new() };
            let mut ae = Vec::new(); let mut inv = 0usize;
            for (j, item) in arr.iter().enumerate() { let e = validate(item, &sv, &format!("{}.{}",path,j)); if !e.is_empty() { inv += 1; ae.push(json!({"index":j,"errors":e})); } }
            value_to_zval(&json!({"valid":ae.is_empty(),"total_items":arr.len(),"valid_items":arr.len()-inv,"invalid_items":inv,"details":ae}))
        }).unwrap_or_else(|_| Zval::new())
    }

    #[php(name = "createIndex")]
    pub fn create_index(&self, collection: String, field: String) -> bool {
        self.inner
            .as_ref()
            .map(|i: &StoreInner| i.build_index(&collection, &field).is_ok())
            .unwrap_or(false)
    }

    #[php(name = "createCompoundIndex")]
    pub fn create_compound_index(&self, collection: String, fields: Vec<String>) -> bool {
        self.inner
            .as_ref()
            .map(|i: &StoreInner| i.build_compound(&collection, &fields).is_ok())
            .unwrap_or(false)
    }

    #[php(name = "indexLookup")]
    pub fn index_lookup(&self, collection: String, field: String, value: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let v = zval_to_value(value);
        if let Some(pos) = i.idx_lookup(&collection, &field, &v) {
            i.read()
                .map(|cd: Arc<Value>| {
                    if let Some(Value::Array(a)) = read_path(&cd, &collection) {
                        value_to_zval(&Value::Array(
                            pos.iter()
                                .filter_map(|&j| a.get(j).cloned())
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        Zval::new()
                    }
                })
                .unwrap_or_else(|_| Zval::new())
        } else {
            Zval::new()
        }
    }

    #[php(name = "listIndexes")]
    pub fn list_indexes(&self) -> Zval {
        let i: &StoreInner = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let idx = match i.indexes.read() {
            Ok(lock) => lock,
            Err(e) => e.into_inner(), // Recovery
        };
        let mut r = Vec::new();
        for (c, s) in idx.iter() {
            for (f, im) in &s.single {
                let uv = im.len();
                let te: usize = im.values().map(|v: &Vec<usize>| v.len()).sum();
                r.push(json!({"collection":c,"type":"single","field":f,"unique_values":uv,"total_entries":te}));
            }
            for (f, im) in &s.compound {
                let uv = im.len();
                let te: usize = im.values().map(|v: &Vec<usize>| v.len()).sum();
                r.push(json!({"collection":c,"type":"compound","fields":f,"unique_values":uv,"total_entries":te}));
            }
        }
        value_to_zval(&Value::Array(r))
    }

    #[php(name = "dropIndex")]
    pub fn drop_index(&self, collection: String) -> bool {
        self.inner
            .as_ref()
            .map(|i: &StoreInner| {
                if let Ok(mut idx) = i.indexes.write() {
                    idx.remove(&collection).is_some()
                } else if let Err(e) = i.indexes.write() {
                    e.into_inner().remove(&collection).is_some()
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    #[php(name = "dropAllIndexes")]
    pub fn drop_all_indexes(&self) -> i64 {
        self.inner
            .as_ref()
            .map(|i: &StoreInner| {
                let mut idx = match i.indexes.write() {
                    Ok(lock) => lock,
                    Err(e) => e.into_inner(),
                };
                let c = idx.len() as i64;
                idx.clear();
                c
            })
            .unwrap_or(0)
    }

    pub fn stats(&self) -> Zval {
        let i = match &self.inner {
            Some(i) => i,
            None => return Zval::new(),
        };
        let meta = match fs::metadata(&i.path) {
            Ok(m) => m,
            Err(_) => return Zval::new(),
        };
        i.read()
            .map(|cd: Arc<Value>| {
                let fs = meta.len();
                let fsh = if fs < 1024 {
                    format!("{} B", fs)
                } else if fs < 1048576 {
                    format!("{:.2} KB", fs as f64 / 1024.0)
                } else {
                    format!("{:.2} MB", fs as f64 / 1048576.0)
                };
                let keys: Vec<Value> = if let Value::Object(o) = cd.as_ref() {
                    o.keys().map(|k| Value::String(k.clone())).collect()
                } else {
                    vec![]
                };
                let kc = if let Value::Object(o) = cd.as_ref() {
                    o.len()
                } else {
                    0
                };
                let mut m = Map::new();
                m.insert("file_path".to_string(), json!(i.path.to_string_lossy()));
                m.insert("file_size".to_string(), json!(fs));
                m.insert("file_size_h".to_string(), json!(fsh));
                m.insert("top_level_keys".to_string(), json!(keys));
                m.insert("key_count".to_string(), json!(kc));
                if let Some(i) = &self.inner {
                    let idx_lock = match i.indexes.read() {
                        Ok(lock) => lock,
                        Err(e) => e.into_inner(),
                    };
                    let ic: usize = idx_lock
                        .values()
                        .map(|s| s.single.len() + s.compound.len())
                        .sum();
                    m.insert("active_indexes".to_string(), json!(ic));
                }
                value_to_zval(&Value::Object(m))
            })
            .unwrap_or_else(|_| Zval::new())
    }

    pub fn backup(&self, backup_path: Option<String>) -> PhpResult<String> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let t = match backup_path {
            Some(p) if !p.is_empty() => p,
            _ => format!(
                "{}.backup.{}",
                i.path.to_string_lossy(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        };
        fs::copy(&i.path, &t).map_err(|e| PhpException::from(e.to_string()))?;
        Ok(t)
    }

    pub fn restore(&self, backup_path: String) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        fs::copy(&backup_path, &i.path).map_err(|e| format!("Restore failed: {}", e))?;

        // Clear caches
        if let Ok(mut cache) = i.cache.write() {
            *cache = None;
        } else if let Err(e) = i.cache.write() {
            *e.into_inner() = None;
        }

        if let Ok(mut idx) = i.indexes.write() {
            idx.clear();
        } else if let Err(e) = i.indexes.write() {
            e.into_inner().clear();
        }

        Ok(true)
    }

    #[php(name = "getMetrics")]
    pub fn get_metrics(&self) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let snapshot = i.metrics.snapshot();
        Ok(value_to_zval(
            &serde_json::to_value(snapshot).map_err(|e| e.to_string())?,
        ))
    }

    #[php(name = "stream")]
    pub fn stream(&self, pointer: String, conditions: Option<&Zval>, options: Option<&Zval>) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        
        let mut filter = StreamFilter::new();
        if let Some(cond) = conditions {
            filter = filter.with_conditions(zval_to_value(cond));
        }
        
        if let Some(opts) = options {
             let opt_val = zval_to_value(opts);
             if let Some(obj) = opt_val.as_object() {
                 if let Some(l) = obj.get("limit").and_then(|v| v.as_u64()) {
                     filter = filter.with_limit(l as usize);
                 }
                 if let Some(s) = obj.get("skip").and_then(|v| v.as_u64()) {
                     filter = filter.with_skip(s as usize);
                 }
                 if let Some(sel) = obj.get("select").and_then(|v| v.as_array()) {
                     let fields: Vec<String> = sel.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                     filter = filter.with_select(fields);
                 }
             }
        }

        let reader = StreamReader::new(&i.path.to_string_lossy(), &pointer)
            .map_err(|e| PhpException::from(e.to_string()))?;
            
        let stream = FilteredStream::new(reader, filter);
        
        let mut results = Vec::new();
        for item in stream {
            match item {
                Ok(val) => results.push(val),
                Err(e) => return Err(PhpException::from(e.to_string()).into()),
            }
        }
        
        Ok(value_to_zval(&Value::Array(results)))
    }

    #[php(name = "streamCount")]
    pub fn stream_count(&self, pointer: String, conditions: Option<&Zval>) -> PhpResult<i64> {
         let i = self.inner.as_ref().ok_or("Not init")?;
         
        let mut filter = StreamFilter::new();
        if let Some(cond) = conditions {
            filter = filter.with_conditions(zval_to_value(cond));
        }
        
        let reader = StreamReader::new(&i.path.to_string_lossy(), &pointer)
            .map_err(|e| PhpException::from(e.to_string()))?;
            
        let stream = FilteredStream::new(reader, filter);
        
        let mut count = 0;
        for item in stream {
             match item {
                Ok(_) => count += 1,
                Err(e) => return Err(PhpException::from(e.to_string()).into()),
            }
        }
        Ok(count)
    }
    
    #[php(name = "streamToFile")]
    pub fn stream_to_file(&self, pointer: String, output_path: String, conditions: Option<&Zval>, options: Option<&Zval>) -> PhpResult<i64> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        
        let mut filter = StreamFilter::new();
        if let Some(cond) = conditions {
            filter = filter.with_conditions(zval_to_value(cond));
        }
        
        let mut pretty = false;
        if let Some(opts) = options {
             let opt_val = zval_to_value(opts);
             if let Some(obj) = opt_val.as_object() {
                 if let Some(l) = obj.get("limit").and_then(|v| v.as_u64()) {
                     filter = filter.with_limit(l as usize);
                 }
                 if let Some(s) = obj.get("skip").and_then(|v| v.as_u64()) {
                     filter = filter.with_skip(s as usize);
                 }
                 if let Some(sel) = obj.get("select").and_then(|v| v.as_array()) {
                     let fields: Vec<String> = sel.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                     filter = filter.with_select(fields);
                 }
                 if let Some(p) = obj.get("pretty").and_then(|v| v.as_bool()) {
                     pretty = p;
                 }
             }
        }
        
        let reader = StreamReader::new(&i.path.to_string_lossy(), &pointer)
            .map_err(|e| PhpException::from(e.to_string()))?;
        let stream = FilteredStream::new(reader, filter);
        
        let file = fs::File::create(&output_path)
            .map_err(|e| PhpException::from(format!("Cannot create output file: {}", e)))?;
        let mut writer = std::io::BufWriter::new(file);
        
        // Write JSON array manually
        writer.write_all(b"[").map_err(|e| PhpException::from(e.to_string()))?;
        
        let mut first = true;
        let mut count = 0;
        
        for item in stream {
            match item {
                Ok(val) => {
                    if !first {
                        writer.write_all(b",").map_err(|e| PhpException::from(e.to_string()))?;
                    }
                    if pretty {
                        // Standard pretty print might not align perfectly with manual array, but it's acceptable
                        // serde_json::to_writer_pretty writes the value.
                        if !first { writer.write_all(b"\n").unwrap_or(()); }
                        serde_json::to_writer_pretty(&mut writer, &val)
                    } else {
                        serde_json::to_writer(&mut writer, &val)
                    }
                    .map_err(|e| PhpException::from(e.to_string()))?;
                    
                    first = false;
                    count += 1;
                },
                Err(e) => return Err(PhpException::from(e.to_string()).into()),
            }
        }
        
        if pretty && !first {
             writer.write_all(b"\n").unwrap_or(());
        }
        writer.write_all(b"]").map_err(|e| PhpException::from(e.to_string()))?;
        writer.flush().map_err(|e| PhpException::from(e.to_string()))?;
        
        Ok(count)
    }

    #[php(name = "streamAggregate")]
    pub fn stream_aggregate(&self, pointer: String, operation: String, field: String, conditions: Option<&Zval>) -> PhpResult<Zval> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        
        let mut filter = StreamFilter::new();
        if let Some(cond) = conditions {
            filter = filter.with_conditions(zval_to_value(cond));
        }
        
        let reader = StreamReader::new(&i.path.to_string_lossy(), &pointer)
            .map_err(|e| PhpException::from(e.to_string()))?;
        let stream = FilteredStream::new(reader, filter);
        
        let mut count = 0;
        let mut sum = 0.0;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        
        for item_res in stream {
            match item_res {
                Ok(item) => {
                     match operation.as_str() {
                         "count" => count += 1,
                         _ => {
                             if let Some(val) = read_nested(&item, &field) {
                                 if let Some(n) = val.as_f64() {
                                     sum += n;
                                     if n < min { min = n; }
                                     if n > max { max = n; }
                                     if operation.as_str() == "avg" { count += 1; }
                                 }
                             }
                         }
                     }
                },
                Err(e) => return Err(PhpException::from(e.to_string()).into()),
            }
        }
        
        let result = match operation.as_str() {
            "sum" => json!(sum),
            "avg" => if count > 0 { json!(sum / count as f64) } else { json!(0) },
            "min" => if min == f64::INFINITY { Value::Null } else { json!(min) },
            "max" => if max == f64::NEG_INFINITY { Value::Null } else { json!(max) },
            "count" => json!(count),
            _ => return Err(PhpException::from(format!("Invalid operation: {}", operation)).into()),
        };
        
        Ok(value_to_zval(&result))
    }

    #[php(name = "appendJsonl")]
    pub fn append_jsonl(&self, record: &Zval) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let value = zval_to_value(record);
        i.append_jsonl(&value).map_err(|e| PhpException::from(e))?;
        Ok(true)
    }

    #[php(name = "readJsonl")]
    pub fn read_jsonl(&self) -> PhpResult<Vec<String>> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let records: Vec<String> = i
            .read_jsonl_iter()
            .map_err(|e| PhpException::from(e))?
            .map(|v| serde_json::to_string(&v).unwrap_or_default())
            .collect();
        Ok(records)
    }

    #[php(name = "column")]
    pub fn column(&self, collection: String, field: String) -> PhpResult<Vec<Zval>> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let data = i.read().map_err(|e| PhpException::from(e))?;
        
        let arr = match read_path(&data, &collection) {
            Some(Value::Array(a)) => a,
            _ => return Ok(Vec::new()),
        };
        
        let values: Vec<Zval> = arr
            .iter()
            .filter_map(|item| {
                if let Value::Object(obj) = item {
                    obj.get(&field).map(value_to_zval)
                } else {
                    None
                }
            })
            .collect();
        
        Ok(values)
    }

    #[php(name = "chunk")]
    pub fn chunk(&self, collection: String, size: i64) -> PhpResult<Vec<Vec<String>>> {
        if size <= 0 {
            return Err(PhpException::from("Chunk size must be greater than 0"));
        }
        
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let data = i.read().map_err(|e| PhpException::from(e))?;
        
        let arr = match read_path(&data, &collection) {
            Some(Value::Array(a)) => a,
            _ => return Ok(Vec::new()),
        };
        
        let chunks: Vec<Vec<String>> = arr
            .chunks(size as usize)
            .map(|chunk| {
                chunk.iter()
                    .map(|v| serde_json::to_string(v).unwrap_or_default())
                    .collect()
            })
            .collect();
        
        Ok(chunks)
    }

    #[php(name = "implode")]
    pub fn implode(&self, collection: String, field: String, separator: String) -> PhpResult<String> {
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let data = i.read().map_err(|e| PhpException::from(e))?;
        
        let arr = match read_path(&data, &collection) {
            Some(Value::Array(a)) => a,
            _ => return Ok(String::new()),
        };

        let strings: Vec<String> = arr
            .iter()
            .filter_map(|item| {
                 if let Value::Object(obj) = item {
                    obj.get(&field).map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => serde_json::to_string(v).unwrap_or_default(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(strings.join(&separator))
    }

    #[php(name = "values")]
    pub fn values(&self, path: String) -> PhpResult<Vec<Zval>> {
        validate_path_depth(&path).map_err(|e| PhpException::from(e))?;
        
        let i = self.inner.as_ref().ok_or_else(|| "Not init".to_string())?;
        let data = i.read().map_err(|e| PhpException::from(e))?;
        
        let target = if path.is_empty() || path == "." {
            &*data
        } else {
            match read_path(&data, &path) {
                Some(v) => v,
                None => return Ok(Vec::new()),
            }
        };
        
        match target {
            Value::Object(obj) => Ok(obj.values().map(value_to_zval).collect()),
            _ => Ok(Vec::new()),
        }
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
    let parsed_size =
        crate::config::Config::parse_size(&size).map_err(|e| PhpException::from(e))?;

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

    let canonical = path_buf
        .canonicalize()
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

// ══════════ STREAM I/O & JSONL ══════════

#[php_function]
pub fn jsonq_write_to_file(
    path: String,
    output_path: String,
    pretty: Option<bool>,
) -> PhpResult<bool> {
    let store = StoreInner::new(path).map_err(|e| PhpException::from(e))?;

    let file = fs::File::create(&output_path)
        .map_err(|e| PhpException::from(format!("Cannot create file: {}", e)))?;
    let mut writer = std::io::BufWriter::new(file);

    if pretty.unwrap_or(false) {
        store.write_to_stream_pretty(&mut writer)
    } else {
        store.write_to_stream(&mut writer)
    }
    .map_err(|e| PhpException::from(e))?;

    writer
        .flush()
        .map_err(|e| PhpException::from(format!("Flush failed: {}", e)))?;

    Ok(true)
}

#[php_function]
pub fn jsonq_append_jsonl(path: String, record: String) -> PhpResult<bool> {
    let store = StoreInner::new(path).map_err(|e| PhpException::from(e))?;

    let value: Value = serde_json::from_str(&record)
        .map_err(|e| PhpException::from(format!("Invalid JSON: {}", e)))?;

    store
        .append_jsonl(&value)
        .map_err(|e| PhpException::from(e))?;

    Ok(true)
}

#[php_function]
pub fn jsonq_read_jsonl(path: String) -> PhpResult<Vec<String>> {
    let store = StoreInner::new(path).map_err(|e| PhpException::from(e))?;

    let records: Vec<String> = store
        .read_jsonl_iter()
        .map_err(|e| PhpException::from(e))?
        .map(|v| serde_json::to_string(&v).unwrap_or_default())
        .collect();

    Ok(records)
}

#[php_function]
pub fn jsonq_memory_stats(path: String) -> PhpResult<HashMap<String, i64>> {
    let store = StoreInner::new(path).map_err(|e| PhpException::from(e))?;

    // Force read to populate interner
    let _ = store.read().map_err(|e| PhpException::from(e))?;

    let (unique, total) = store.memory_stats();

    let mut stats = HashMap::new();
    stats.insert("unique_keys".to_string(), unique as i64);
    stats.insert("total_references".to_string(), total as i64);
    stats.insert(
        "memory_saved_percent".to_string(),
        if total > 0 {
            ((total - unique) as f64 / total as f64 * 100.0) as i64
        } else {
            0
        },
    );

    Ok(stats)
}

#[php_function]
pub fn jsonq_query_node(path: String, query_path: String) -> PhpResult<Vec<String>> {
    let store = StoreInner::new(path).map_err(|e| PhpException::from(e))?;

    // Read data (cached)
    let data = store.read().map_err(|e| PhpException::from(e))?;

    // Parse path
    let segments = PathSegment::parse_json_path(&query_path)
        .map_err(|e| PhpException::from(format!("Invalid path: {}", e)))?;

    let executor = QueryExecutor::new();
    // Start with root node
    let mut current_nodes = vec![(*data).clone()];

    // Apply segments sequentially
    for segment in segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            next_nodes.extend(executor.apply_segment(&node, &segment));
        }
        current_nodes = next_nodes;
    }

    // Serialize results
    let results: Vec<String> = current_nodes
        .into_iter()
        .map(|v| serde_json::to_string(&v).unwrap_or_default())
        .collect();

    Ok(results)
}

#[php_function]
pub fn jsonq_query(path: String, query: String) -> PhpResult<Vec<String>> {
    jsonq_query_node(path, query)
}

#[php_function]
pub fn jsonq_stream(path: String, pointer: String, conditions: Option<&Zval>, options: Option<&Zval>) -> PhpResult<Zval> {
    let store = JsonStore::__construct(path);
    store.stream(pointer, conditions, options)
}

#[php_function]
pub fn jsonq_stream_count(path: String, pointer: String, conditions: Option<&Zval>) -> PhpResult<i64> {
     let store = JsonStore::__construct(path);
     store.stream_count(pointer, conditions)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    // ✅ Initialize global configuration
    crate::config::Config::init();
    crate::config::php_ini::load_from_ini();

    // ✅ Initialize tracing (logging)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr) // Send logs to stderr
        .try_init();

    tracing::info!("JsonQ module initialized");

    module
        .function(wrap_function!(jsonq_version))
        .function(wrap_function!(jsonq_get_config))
        .function(wrap_function!(jsonq_set_max_file_size))
        .function(wrap_function!(jsonq_set_allowed_extensions))
        .function(wrap_function!(jsonq_set_base_path))
        .function(wrap_function!(jsonq_clear_base_path))
        .function(wrap_function!(jsonq_write_to_file))
        .function(wrap_function!(jsonq_append_jsonl))
        .function(wrap_function!(jsonq_read_jsonl))
        .function(wrap_function!(jsonq_memory_stats))
        .function(wrap_function!(jsonq_query_node))
        .function(wrap_function!(jsonq_query))
        .function(wrap_function!(jsonq_stream))
        .function(wrap_function!(jsonq_stream_count))
        .class::<JsonStore>()
}
