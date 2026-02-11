//! JsonQ - High-performance JSON file storage engine for PHP
#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(non_snake_case)]

use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ArrayKey};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use memmap2::Mmap;

// ══════════ HELPERS ══════════

fn as_u64(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)).or_else(|| v.as_f64().map(|f| f as u64))
}

// ══════════ ZVAL CONVERSION ══════════

fn value_to_zval(val: &Value) -> Zval {
    let mut z = Zval::new();
    match val {
        Value::Null => { z.set_null(); }
        Value::Bool(b) => { z.set_bool(*b); }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { z.set_long(i); }
            else { z.set_double(n.as_f64().unwrap_or(0.0)); }
        }
        Value::String(s) => { let _ = z.set_string(s, false); }
        Value::Array(arr) => {
            let mut ht = ext_php_rs::types::ZendHashTable::new();
            for item in arr { let _ = ht.push(value_to_zval(item)); }
            z.set_hashtable(ht);
        }
        Value::Object(map) => {
            let mut ht = ext_php_rs::types::ZendHashTable::new();
            for (k, v) in map { let _ = ht.insert(k.as_str(), value_to_zval(v)); }
            z.set_hashtable(ht);
        }
    }
    z
}

fn zval_to_value(zval: &Zval) -> Value {
    if zval.is_null() { return Value::Null; }
    if zval.is_bool() { return Value::Bool(zval.bool().unwrap_or(false)); }
    if zval.is_long() { return Value::Number(serde_json::Number::from(zval.long().unwrap_or(0))); }
    if zval.is_double() { return serde_json::Number::from_f64(zval.double().unwrap_or(0.0)).map(Value::Number).unwrap_or(Value::Null); }
    if zval.is_string() { return Value::String(zval.str().unwrap_or("").to_string()); }
    if let Some(ht) = zval.array() { return ht_to_value(ht); }
    Value::Null
}

fn ht_to_value(ht: &ext_php_rs::types::ZendHashTable) -> Value {
    let mut is_seq = true;
    let mut exp: u64 = 0;
    for (key, _) in ht.iter() {
        match key {
            ArrayKey::Long(idx) if idx as u64 == exp => { exp += 1; }
            ArrayKey::String(_) | ArrayKey::Str(_) => { is_seq = false; break; }
            _ => { is_seq = false; break; }
        }
    }
    if is_seq && ht.len() > 0 {
        let mut arr = Vec::with_capacity(ht.len());
        for (_, val) in ht.iter() { arr.push(zval_to_value(val)); }
        Value::Array(arr)
    } else {
        let mut map = Map::new();
        for (key, val) in ht.iter() {
            let k = match key {
                ArrayKey::String(s) => s.to_string(),
                ArrayKey::Str(s) => s.to_string(),
                ArrayKey::Long(idx) => idx.to_string(),
            };
            map.insert(k, zval_to_value(val));
        }
        Value::Object(map)
    }
}

// ══════════ STORE ENGINE ══════════

struct StoreOpts { pretty: bool, fsync: bool }
impl Default for StoreOpts { fn default() -> Self { Self { pretty: false, fsync: false } } }

struct IndexStore {
    single: HashMap<String, HashMap<String, Vec<usize>>>,
    compound: HashMap<Vec<String>, HashMap<String, Vec<usize>>>,
    built_at: u64,
}
impl IndexStore { fn new() -> Self { Self { single: HashMap::new(), compound: HashMap::new(), built_at: 0 } } }

struct CachedData { data: Arc<Value>, mtime: u64 }

struct StoreInner {
    path: PathBuf,
    cache: RwLock<Option<CachedData>>,
    indexes: RwLock<HashMap<String, IndexStore>>,
    opts: RwLock<StoreOpts>,
    in_transaction: RwLock<bool>,
    tx_data: RwLock<Option<Arc<Value>>>,
}

impl StoreInner {
    fn new(path: String) -> Self {
        let p = PathBuf::from(&path);
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        if !p.exists() { let _ = fs::write(&p, "{}"); }
        Self {
            path: p,
            cache: RwLock::new(None),
            indexes: RwLock::new(HashMap::new()),
            opts: RwLock::new(StoreOpts::default()),
            in_transaction: RwLock::new(false),
            tx_data: RwLock::new(None),
        }
    }

    fn mtime(&self) -> u64 {
        fs::metadata(&self.path).and_then(|m| m.modified())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0)
    }

    fn read(&self) -> Result<Arc<Value>, String> {
        if *self.in_transaction.read().unwrap() {
            if let Some(ref d) = *self.tx_data.read().unwrap() {
                return Ok(d.clone());
            }
        }
        let mt = self.mtime();
        {
            let cache = self.cache.read().unwrap();
            if let Some(ref c) = *cache { if c.mtime == mt { return Ok(c.data.clone()); } }
        }

        let file = File::open(&self.path).map_err(|e| e.to_string())?;
        let mmap = unsafe { Mmap::map(&file).map_err(|e| e.to_string())? };
        let data: Value = serde_json::from_slice(&mmap).map_err(|e| e.to_string())?;
        let arc_data = Arc::new(data);
        *self.cache.write().unwrap() = Some(CachedData { data: arc_data.clone(), mtime: mt });
        Ok(arc_data)
    }

    fn write(&self, data: &Value) -> Result<(), String> {
        if *self.in_transaction.read().unwrap() {
            *self.tx_data.write().unwrap() = Some(Arc::new(data.clone()));
            return Ok(());
        }
        self.flush(data)
    }

    fn flush(&self, data: &Value) -> Result<(), String> {
        let opts = self.opts.read().unwrap();
        let s = if opts.pretty { serde_json::to_string_pretty(data) } else { serde_json::to_string(data) }.map_err(|e| e.to_string())?;

        let tmp_path = self.path.with_extension("tmp");
        {
            let mut f = File::create(&tmp_path).map_err(|e| e.to_string())?;
            f.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
            if opts.fsync { f.sync_all().map_err(|e| e.to_string())?; }
        }
        fs::rename(&tmp_path, &self.path).map_err(|e| e.to_string())?;

        *self.cache.write().unwrap() = Some(CachedData { data: Arc::new(data.clone()), mtime: self.mtime() });
        Ok(())
    }

    fn mutate<F>(&self, f: F) -> Result<(), String> where F: FnOnce(&mut Value) {
        let arc_data = self.read()?;
        let mut data = (*arc_data).clone();
        f(&mut data);
        self.write(&data)
    }

    fn build_index(&self, coll: &str, field: &str) -> Result<(), String> {
        let mt = self.mtime();
        let cd = self.read()?;
        let arr = match rp(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() { idx.entry(vkey(rn(item, field))).or_default().push(i); }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.single.insert(field.to_string(), idx); store.built_at = mt;
        Ok(())
    }

    fn build_compound(&self, coll: &str, fields: &[String]) -> Result<(), String> {
        let cd = self.read()?;
        let arr = match rp(&cd, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            let k: String = fields.iter().map(|f| vkey(rn(item, f))).collect::<Vec<_>>().join("|");
            idx.entry(k).or_default().push(i);
        }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.to_string()).or_insert_with(IndexStore::new);
        store.compound.insert(fields.to_vec(), idx); store.built_at = self.mtime();
        Ok(())
    }

    fn idx_lookup(&self, coll: &str, field: &str, value: &Value) -> Option<Vec<usize>> {
        let mt = self.mtime();
        let indexes = self.indexes.read().unwrap();
        let store = indexes.get(coll)?;
        if store.built_at < mt { return None; }
        store.single.get(field)?.get(&vkey(Some(value))).cloned()
    }
}

// ══════════ HELPERS ══════════

fn sap(root: &mut Value, dp: &str, value: Value) {
    let keys: Vec<&str> = dp.split('.').collect();
    let mut c = root;
    for (i, &k) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            match c {
                Value::Object(m) => { m.insert(k.to_string(), value); }
                Value::Array(a) => { if let Ok(idx) = k.parse::<usize>() { if idx < a.len() { a[idx] = value; } else { a.push(value); } } }
                _ => {}
            }
            return;
        }
        let nn = keys.get(i+1).map(|x| x.parse::<usize>().is_ok()).unwrap_or(false);
        match c {
            Value::Object(m) => { if !m.contains_key(k) { m.insert(k.to_string(), if nn { Value::Array(vec![]) } else { Value::Object(Map::new()) }); } c = m.get_mut(k).unwrap(); }
            Value::Array(a) => { if let Ok(idx) = k.parse::<usize>() { while a.len() <= idx { a.push(Value::Object(Map::new())); } c = &mut a[idx]; } else { return; } }
            _ => return,
        }
    }
}

fn rp<'a>(root: &'a Value, dp: &str) -> Option<&'a Value> {
    if dp.is_empty() { return Some(root); }
    let mut c = root;
    for k in dp.split('.') { c = match c { Value::Object(m) => m.get(k)?, Value::Array(a) => a.get(k.parse::<usize>().ok()?)?, _ => return None }; }
    Some(c)
}

fn rpm<'a>(root: &'a mut Value, dp: &str) -> Option<&'a mut Value> {
    if dp.is_empty() { return Some(root); }
    let mut c = root;
    for k in dp.split('.') { c = match c { Value::Object(m) => m.get_mut(k)?, Value::Array(a) => a.get_mut(k.parse::<usize>().ok()?)?, _ => return None }; }
    Some(c)
}

fn rap(root: &mut Value, dp: &str) -> bool {
    let mut keys: Vec<&str> = dp.split('.').collect();
    if keys.is_empty() { return false; }
    let k = keys.pop().unwrap();
    let p = keys.join(".");
    if let Some(target) = if p.is_empty() { Some(root) } else { rpm(root, &p) } {
        match target {
            Value::Object(m) => m.remove(k).is_some(),
            Value::Array(a) => { if let Ok(idx) = k.parse::<usize>() { if idx < a.len() { a.remove(idx); return true; } } false }
            _ => false,
        }
    } else { false }
}

fn rn<'a>(v: &'a Value, path: &str) -> Option<&'a Value> { path.split('.').fold(Some(v), |acc, k| match acc { Some(Value::Object(m)) => m.get(k), Some(Value::Array(a)) => a.get(k.parse::<usize>().ok()?), _ => None }) }
fn vkey(v: Option<&Value>) -> String { v.map(|x| match x { Value::String(s) => s.clone(), _ => x.to_string() }).unwrap_or_else(|| "null".into()) }

fn search_in_value(v: &Value, kw: &str) -> bool {
    match v {
        Value::String(s) => s.to_lowercase().contains(kw),
        Value::Object(m) => m.values().any(|x| search_in_value(x, kw)),
        Value::Array(a) => a.iter().any(|x| search_in_value(x, kw)),
        _ => v.to_string().to_lowercase().contains(kw),
    }
}

fn mat(item: &Value, cond: &Value) -> bool {
    let co = match cond.as_object() { Some(o) => o, None => return false };
    co.iter().all(|(k, v)| {
        if k.starts_with('$') {
            match k.as_str() {
                "$or" => v.as_array().map(|a| a.iter().any(|c| mat(item, c))).unwrap_or(false),
                "$and" => v.as_array().map(|a| a.iter().all(|c| mat(item, c))).unwrap_or(false),
                "$nor" => !v.as_array().map(|a| a.iter().any(|c| mat(item, c))).unwrap_or(true),
                "$not" => !mat(item, v),
                _ => false
            }
        } else {
            let iv = rn(item, k);
            if let Some(vo) = v.as_object() {
                vo.iter().all(|(op, ov)| match op.as_str() {
                    "$eq" => iv == Some(ov), "$ne" => iv != Some(ov),
                    "$gt" => iv.and_then(|x| x.as_f64()) > ov.as_f64(),
                    "$gte" => iv.and_then(|x| x.as_f64()) >= ov.as_f64(),
                    "$lt" => iv.and_then(|x| x.as_f64()) < ov.as_f64(),
                    "$lte" => iv.and_then(|x| x.as_f64()) <= ov.as_f64(),
                    "$in" => ov.as_array().map(|a| a.contains(iv.unwrap_or(&Value::Null))).unwrap_or(false),
                    "$nin" => !ov.as_array().map(|a| a.contains(iv.unwrap_or(&Value::Null))).unwrap_or(true),
                    "$exists" => ov.as_bool() == Some(iv.is_some()),
                    "$regex" => { let re = ov.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.contains(re)).unwrap_or(false) }
                    "$contains" => { let sub = ov.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.contains(sub)).unwrap_or(false) }
                    "$startsWith" => { let sub = ov.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.starts_with(sub)).unwrap_or(false) }
                    "$endsWith" => { let sub = ov.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.ends_with(sub)).unwrap_or(false) }
                    _ => false
                })
            } else { iv == Some(v) }
        }
    })
}

fn merge_v(v: &mut Value, n: &Value) {
    match (v, n) {
        (Value::Object(a), Value::Object(b)) => { for (k, v) in b { merge_v(a.entry(k.clone()).or_insert(Value::Null), v); } }
        (Value::Array(a), Value::Array(b)) => { a.extend(b.clone()); }
        (a, b) => { *a = b.clone(); }
    }
}

fn exec_fluent(arr: &Vec<Value>, q: &Value) -> Vec<Value> {
    let mut res = arr.clone();
    if let Some(w) = q.get("where").and_then(|x| x.as_array()) {
        res = res.into_iter().filter(|item| {
            w.iter().all(|cond| {
                let f = cond.get("field").and_then(|x| x.as_str()).unwrap_or("");
                let op = cond.get("op").and_then(|x| x.as_str()).unwrap_or("=");
                let v = cond.get("value").unwrap_or(&Value::Null);
                let iv = rn(item, f);
                match op {
                    "=" | "eq" => iv == Some(v),
                    "!=" | "ne" => iv != Some(v),
                    ">" | "gt" => iv.and_then(|x| x.as_f64()) > v.as_f64(),
                    ">=" | "gte" => iv.and_then(|x| x.as_f64()) >= v.as_f64(),
                    "<" | "lt" => iv.and_then(|x| x.as_f64()) < v.as_f64(),
                    "<=" | "lte" => iv.and_then(|x| x.as_f64()) <= v.as_f64(),
                    "in" => v.as_array().map(|a| a.contains(iv.unwrap_or(&Value::Null))).unwrap_or(false),
                    "contains" => { let sub = v.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.contains(sub)).unwrap_or(false) }
                    "startsWith" => { let sub = v.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.starts_with(sub)).unwrap_or(false) }
                    "endsWith" => { let sub = v.as_str().unwrap_or(""); iv.and_then(|x| x.as_str()).map(|s| s.ends_with(sub)).unwrap_or(false) }
                    "between" => {
                        if let Some(a) = v.as_array() {
                            if a.len() == 2 {
                                let val = iv.and_then(|x| x.as_f64()).unwrap_or(0.0);
                                let min = a[0].as_f64().unwrap_or(0.0);
                                let max = a[1].as_f64().unwrap_or(0.0);
                                return val >= min && val <= max;
                            }
                        }
                        false
                    }
                    _ => false
                }
            })
        }).collect();
    }
    if let Some(o) = q.get("order_by").or(q.get("sort")) {
        let f = o.get("field").and_then(|x| x.as_str()).unwrap_or("");
        let dir = o.get("direction").and_then(|x| x.as_str()).unwrap_or("asc");
        let desc = dir == "desc" || o.get("desc").and_then(|x| x.as_bool()).unwrap_or(false);
        res.sort_by(|a, b| {
            let va = rn(a, f); let vb = rn(b, f);
            let cmp = match (va, vb) {
                (Some(Value::Number(x)), Some(Value::Number(y))) => x.as_f64().partial_cmp(&y.as_f64()).unwrap_or(std::cmp::Ordering::Equal),
                (Some(Value::String(x)), Some(Value::String(y))) => x.cmp(y),
                _ => std::cmp::Ordering::Equal
            };
            if desc { cmp.reverse() } else { cmp }
        });
    }
    if let Some(sk) = q.get("skip").or(q.get("offset")).and_then(as_u64) { if sk < res.len() as u64 { res = res.split_off(sk as usize); } else { res.clear(); } }
    if let Some(l) = q.get("limit").and_then(as_u64) { res.truncate(l as usize); }
    if let Some(s) = q.get("select").and_then(|x| x.as_array()) {
        let fields: Vec<&str> = s.iter().filter_map(|x| x.as_str()).collect();
        res = plk(&res, &fields);
    }
    res
}

fn agg(arr: &Vec<Value>, field: &str, op: &str) -> Value {
    let nums: Vec<f64> = arr.iter().filter_map(|i| rn(i, field).and_then(|v| v.as_f64())).collect();
    if nums.is_empty() { return Value::Null; }
    match op {
        "sum" => json!(nums.iter().sum::<f64>()), "avg" => json!(nums.iter().sum::<f64>() / nums.len() as f64),
        "min" => json!(nums.iter().fold(f64::INFINITY, |a, &b| a.min(b))), "max" => json!(nums.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
        "count" => json!(arr.len()), _ => Value::Null
    }
}

fn grp(arr: &Vec<Value>, field: &str) -> Value {
    let mut m = Map::new();
    for i in arr { let k = vkey(rn(i, field)); m.entry(k).or_insert(Value::Array(vec![])).as_array_mut().unwrap().push(i.clone()); }
    Value::Object(m)
}

fn plk(arr: &Vec<Value>, fields: &[&str]) -> Vec<Value> {
    if fields.len() == 1 {
        let f = fields[0];
        return arr.iter().map(|i| rn(i, f).cloned().unwrap_or(Value::Null)).collect();
    }
    arr.iter().map(|i| {
        let mut o = Map::new();
        for &f in fields { if let Some(v) = rn(i, f) { o.insert(f.to_string(), v.clone()); } }
        Value::Object(o)
    }).collect()
}

fn vld(v: &Value, s: &Value, p: &str) -> Vec<Value> {
    let mut e = Vec::new();
    if let Some(so) = s.as_object() {
        if let Some(t) = so.get("type").and_then(|x| x.as_str()) {
            let ok = match t {
                "string" => v.is_string(),
                "number" => v.is_number(),
                "integer" => v.is_number() && (v.as_f64().unwrap_or(0.1) % 1.0 == 0.0),
                "boolean" => v.is_boolean(),
                "array" => v.is_array(),
                "object" => v.is_object(),
                "null" => v.is_null(),
                _ => true
            };
            if !ok { e.push(json!({"path":p,"error":format!("Expected {}, found {:?}",t,v)})); }
        }
        if let Some(min) = so.get("min").or(so.get("minimum")).and_then(|x| x.as_f64()) { if v.as_f64().map_or(false, |val| val < min) { e.push(json!({"path":p,"error":format!("Value {} < minimum {}", v, min)})); } }
        if let Some(max) = so.get("max").or(so.get("maximum")).and_then(|x| x.as_f64()) { if v.as_f64().map_or(false, |val| val > max) { e.push(json!({"path":p,"error":format!("Value {} > maximum {}", v, max)})); } }
        if let Some(ml) = so.get("minLength").and_then(as_u64) { if v.as_str().map_or(false, |s| (s.len() as u64) < ml) { e.push(json!({"path":p,"error":format!("String length < {}", ml)})); } }
        if let Some(ml) = so.get("maxLength").and_then(as_u64) { if v.as_str().map_or(false, |s| (s.len() as u64) > ml) { e.push(json!({"path":p,"error":format!("String length > {}", ml)})); } }
        if let Some(pat) = so.get("pattern").and_then(|x| x.as_str()) { if v.as_str().map_or(false, |s| !s.contains(pat)) { e.push(json!({"path":p,"error":format!("Value '{}' does not match pattern '{}'", v, pat)})); } }
        if let Some(fmt) = so.get("format").and_then(|x| x.as_str()) {
            if fmt == "email" && v.as_str().map_or(false, |s| !s.contains('@') || !s.contains('.')) {
                e.push(json!({"path":p,"error":"Invalid email format"}));
            }
        }
        if let Some(Value::Array(choices)) = so.get("enum") { if !choices.contains(v) { e.push(json!({"path":p,"error":format!("Value {:?} not in enum {:?}", v, choices)})); } }
        if let Some(Value::Array(req)) = so.get("required") { if let Value::Object(vo) = v { for r in req { if let Some(rk) = r.as_str() { if !vo.contains_key(rk) { e.push(json!({"path":format!("{}.{}",p,rk),"error":"Required field missing"})); } } } } }
        if let Some(Value::Object(props)) = so.get("properties") { if let Value::Object(vo) = v { for (pk, pv) in props { e.extend(vld(vo.get(pk).unwrap_or(&Value::Null), pv, &format!("{}.{}",p,pk))); } } }
        if let Some(items_val) = so.get("items") { if let Value::Array(va) = v { for (j, vi) in va.iter().enumerate() { e.extend(vld(vi, items_val, &format!("{}.{}",p,j))); } } }
    }
    e
}

// ══════════ PHP CLASS ══════════

#[php_class]
#[php(name = "JsonQ\\Store")]
pub struct JsonStore { inner: Option<StoreInner> }

#[php_impl]
impl JsonStore {
    pub fn __construct(path: String) -> Self { Self { inner: Some(StoreInner::new(path)) } }

    #[php(name = "setOption")]
    pub fn set_option(&self, key: String, value: &Zval) -> bool {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return false };
        let mut opts = i.opts.write().unwrap();
        match key.as_str() {
            "pretty" | "pretty_print" => { opts.pretty = value.bool().unwrap_or(false); true }
            "fsync" | "sync" => { opts.fsync = value.bool().unwrap_or(false); true }
            _ => false,
        }
    }
    
    #[php(name = "getOption")]
    pub fn get_option(&self, key: String) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() };
        let opts = i.opts.read().unwrap();
        let mut z = Zval::new();
        match key.as_str() { "pretty"|"pretty_print" => { z.set_bool(opts.pretty); } "fsync"|"sync" => { z.set_bool(opts.fsync); } _ => {} }
        z
    }

    #[php(name = "beginTransaction")]
    pub fn begin_transaction(&self) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let data = i.read().map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        *i.tx_data.write().unwrap() = Some(data);
        *i.in_transaction.write().unwrap() = true;
        Ok(true)
    }
    
    pub fn commit(&self) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let data = i.tx_data.read().unwrap().clone().ok_or("No active transaction")?;
        *i.in_transaction.write().unwrap() = false;
        *i.tx_data.write().unwrap() = None;
        i.flush(&data).map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        Ok(true)
    }
    
    pub fn rollback(&self) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        *i.in_transaction.write().unwrap() = false;
        *i.tx_data.write().unwrap() = None;
        Ok(true)
    }
    
    #[php(name = "inTransaction")]
    pub fn in_transaction(&self) -> bool { self.inner.as_ref().map(|i: &StoreInner| *i.in_transaction.read().unwrap()).unwrap_or(false) }

    #[php(name = "setMany")]
    pub fn set_many(&self, pairs: &Zval) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let pv = zval_to_value(pairs);
        let po = match pv.as_object() { Some(o) => o, None => return Ok(0) };
        let cd = i.read().map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for (path, value) in po { sap(&mut data, path, value.clone()); count += 1; }
        i.write(&data).map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        Ok(count)
    }
    
    #[php(name = "removeMany")]
    pub fn remove_many(&self, paths: Vec<String>) -> PhpResult<i64> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        let mut data = (*cd).clone();
        let mut count = 0i64;
        for path in &paths { if rap(&mut data, path) { count += 1; } }
        i.write(&data).map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        Ok(count)
    }

    #[php(name = "toJson")]
    pub fn to_json(&self, pretty: Option<bool>) -> PhpResult<String> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        if pretty.unwrap_or(false) { serde_json::to_string_pretty(&*cd).map_err(|e| e.to_string().into()) } else { serde_json::to_string(&*cd).map_err(|e| e.to_string().into()) }
    }
    
    #[php(name = "fromJson")]
    pub fn from_json(&self, json_str: String) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let data: Value = serde_json::from_str(&json_str).map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        i.write(&data).map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        Ok(true)
    }

    #[php(name = "getAll")]
    pub fn get_all(&self) -> Zval { self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| value_to_zval(&cd)).ok()).unwrap_or_else(Zval::new) }
    
    pub fn clear(&self) -> PhpResult<bool> { let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?; i.write(&Value::Object(Map::new())).map_err(|e: String| ext_php_rs::exception::PhpException::from(e.to_string()))?; Ok(true) }
    
    pub fn search(&self, collection: String, keyword: String) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() };
        let kw = keyword.to_lowercase();
        i.read().map(|cd: Arc<Value>| {
            let arr = match rp(&cd, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            let matched: Vec<Value> = arr.iter().filter(|item| search_in_value(item, &kw)).cloned().collect();
            value_to_zval(&Value::Array(matched))
        }).unwrap_or_else(|_| Zval::new())
    }

    pub fn get(&self, path: String) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; i.read().map(|cd: Arc<Value>| rp(&cd, &path).map(|v| value_to_zval(v)).unwrap_or_else(Zval::new)).unwrap_or_else(|_| Zval::new()) }
    pub fn has(&self, path: String) -> bool { self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| rp(&cd, &path).is_some()).ok()).unwrap_or(false) }
    pub fn count(&self, path: String) -> i64 { self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| match rp(&cd, &path) { Some(Value::Array(a)) => a.len() as i64, Some(Value::Object(o)) => o.len() as i64, _ => -1 }).ok()).unwrap_or(-1) }
    pub fn keys(&self, path: String) -> Vec<String> { self.inner.as_ref().and_then(|i: &StoreInner| i.read().map(|cd: Arc<Value>| match if path.is_empty(){Some(cd.as_ref())}else{rp(&cd,&path)} { Some(Value::Object(o)) => o.keys().cloned().collect(), _ => vec![] }).ok()).unwrap_or_default() }

    pub fn set(&self, path: String, value: &Zval) -> bool { let i: &StoreInner = match &self.inner { Some(i) => i, None => return false }; let v = zval_to_value(value); i.mutate(|d: &mut Value| sap(d, &path, v)).is_ok() }
    pub fn remove(&self, path: String) -> bool { let i: &StoreInner = match &self.inner { Some(i) => i, None => return false }; i.mutate(|d: &mut Value| { rap(d, &path); }).is_ok() }
    pub fn push(&self, path: String, value: &Zval) -> bool { let i: &StoreInner = match &self.inner { Some(i) => i, None => return false }; let v = zval_to_value(value); i.mutate(|d: &mut Value| { match if path.is_empty(){Some(d)}else{rpm(d, &path)} { Some(Value::Array(a)) => { a.push(v); } _ => {} } }).is_ok() }
    pub fn merge(&self, path: String, value: &Zval) -> bool { let i: &StoreInner = match &self.inner { Some(i) => i, None => return false }; let nv = zval_to_value(value); i.mutate(|d: &mut Value| { if let Some(e) = if path.is_empty(){Some(&mut *d)}else{rpm(d,&path)} { merge_v(e, &nv); } else { sap(d, &path, nv); } }).is_ok() }
    pub fn increment(&self, path: String, amount: Option<f64>) -> bool { let amt: f64 = amount.unwrap_or(1.0); let i: &StoreInner = match &self.inner { Some(i) => i, None => return false }; i.mutate(|d: &mut Value| { if let Some(v) = rpm(d, &path) { if let Some(n) = v.as_f64() { *v = json!(n + amt); } } }).is_ok() }
    pub fn decrement(&self, path: String, amount: Option<f64>) -> bool { self.increment(path, Some(-(amount.unwrap_or(1.0)))) }

    pub fn find(&self, collection: String, conditions: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let cond = zval_to_value(conditions);
        i.read().map(|cd: Arc<Value>| {
            let arr = match rp(&cd, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            if let Some(co) = cond.as_object() { if co.len() == 1 { if let Some((f, v)) = co.iter().next() { if !f.starts_with('$') && !v.is_object() { if let Some(pos) = i.idx_lookup(&collection, f, v) { return value_to_zval(&Value::Array(pos.iter().filter_map(|&j| arr.get(j).cloned()).collect::<Vec<Value>>())); } } } } }
            value_to_zval(&Value::Array(arr.iter().filter(|item| mat(item, &cond)).cloned().collect()))
        }).unwrap_or_else(|_| Zval::new())
    }
    
    #[php(name = "findOne")]
    pub fn find_one(&self, collection: String, conditions: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let c = zval_to_value(conditions); i.read().map(|cd: Arc<Value>| match rp(&cd, &collection) { Some(Value::Array(a)) => a.iter().find(|item| mat(item, &c)).map(|f| value_to_zval(f)).unwrap_or_else(Zval::new), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "executeQuery")]
    pub fn execute_query(&self, collection: String, query_spec: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let q = zval_to_value(query_spec); i.read().map(|cd: Arc<Value>| match rp(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(exec_fluent(a, &q))), _ => value_to_zval(&Value::Array(vec![])) }).unwrap_or_else(|_| Zval::new()) }

    pub fn aggregate(&self, collection: String, field: String, operation: String) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; i.read().map(|cd: Arc<Value>| match rp(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&agg(a, &field, &operation)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "groupBy")]
    pub fn group_by(&self, collection: String, field: String) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; i.read().map(|cd: Arc<Value>| match rp(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&grp(a, &field)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    
    pub fn pluck(&self, collection: String, fields: Vec<String>) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let fr: Vec<&str> = fields.iter().map(|s| s.as_str()).collect(); i.read().map(|cd: Arc<Value>| match rp(&cd, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(plk(a, &fr))), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }

    pub fn validate(&self, path: String, schema: &Zval) -> Zval { let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(schema); i.read().map(|cd: Arc<Value>| { let t = if path.is_empty(){cd.as_ref()}else{match rp(&cd,&path){Some(v)=>v,None=>return Zval::new()}}; let e = vld(t, &sv, &path); value_to_zval(&json!({"valid":e.is_empty(),"error_count":e.len(),"errors":e})) }).unwrap_or_else(|_| Zval::new()) }
    
    #[php(name = "validateCollection")]
    pub fn validate_collection(&self, path: String, item_schema: &Zval) -> Zval {
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(item_schema);
        i.read().map(|cd: Arc<Value>| {
            let arr = match rp(&cd, &path) { Some(Value::Array(a)) => a, _ => return Zval::new() };
            let mut ae = Vec::new(); let mut inv = 0usize;
            for (j, item) in arr.iter().enumerate() { let e = vld(item, &sv, &format!("{}.{}",path,j)); if !e.is_empty() { inv += 1; ae.push(json!({"index":j,"errors":e})); } }
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
            i.read().map(|cd: Arc<Value>| if let Some(Value::Array(a)) = rp(&cd, &collection) { value_to_zval(&Value::Array(pos.iter().filter_map(|&j| a.get(j).cloned()).collect::<Vec<Value>>())) } else { Zval::new() }).unwrap_or_else(|_| Zval::new())
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
        let i: &StoreInner = match &self.inner { Some(i) => i, None => return Zval::new() }; let meta = match fs::metadata(&i.path) { Ok(m) => m, Err(_) => return Zval::new() };
        i.read().map(|cd: Arc<Value>| {
            let fs = meta.len(); let fsh = if fs<1024{format!("{} B",fs)}else if fs<1048576{format!("{:.2} KB",fs as f64/1024.0)}else{format!("{:.2} MB",fs as f64/1048576.0)};
            let keys: Vec<Value> = if let Value::Object(o) = cd.as_ref() { o.keys().map(|k| Value::String(k.clone())).collect() } else { vec![] };
            let kc = if let Value::Object(o) = cd.as_ref() { o.len() } else { 0 };
            let ic: usize = i.indexes.read().unwrap().values().map(|s| s.single.len()+s.compound.len()).sum();
            value_to_zval(&json!({"file_path":i.path.to_string_lossy(),"file_size":fs,"file_size_h":fsh,"top_level_keys":keys,"key_count":kc,"active_indexes":ic}))
        }).unwrap_or_else(|_| Zval::new())
    }
    
    pub fn backup(&self, backup_path: Option<String>) -> PhpResult<String> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        let t = match backup_path { Some(p) if !p.is_empty() => p, _ => format!("{}.backup.{}",i.path.to_string_lossy(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()) };
        fs::copy(&i.path, &t).map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        Ok(t)
    }
    
    pub fn restore(&self, backup_path: String) -> PhpResult<bool> {
        let i: &StoreInner = self.inner.as_ref().ok_or("Not init")?;
        fs::copy(&backup_path, &i.path).map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        *i.cache.write().unwrap() = None; i.indexes.write().unwrap().clear();
        Ok(true)
    }
}

#[php_function] 
pub fn jsonq_version() -> String { 
    env!("CARGO_PKG_VERSION").to_string() 
}

#[php_module] 
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { 
    module
        .function(wrap_function!(jsonq_version))
        .class::<JsonStore>()
}