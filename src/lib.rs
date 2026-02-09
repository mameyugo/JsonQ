//! JsonQ - High-performance JSON file storage engine for PHP
//! Native PHP extension written in Rust via ext-php-rs.
#![allow(non_snake_case)]

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use memmap2::Mmap;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// ══════════ ZVAL CONVERSION ══════════

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
    if zval.is_true() { return Value::Bool(true); }
    if zval.is_false() { return Value::Bool(false); }
    if zval.is_bool() { return Value::Bool(zval.bool().unwrap_or(false)); }
    if zval.is_long() { return Value::Number(serde_json::Number::from(zval.long().unwrap_or(0))); }
    if zval.is_double() {
        return serde_json::Number::from_f64(zval.double().unwrap_or(0.0)).map(Value::Number).unwrap_or(Value::Null);
    }
    if zval.is_string() { return Value::String(zval.str().unwrap_or("").to_string()); }
    if zval.is_array() {
        if let Some(ht) = zval.array() { return ht_to_value(ht); }
    }
    Value::Null
}

fn ht_to_value(ht: &ext_php_rs::types::ZendHashTable) -> Value {
    let mut is_seq = true;
    let mut exp: u64 = 0;
    for (idx, key, _) in ht.iter() {
        if key.is_some() { is_seq = false; break; }
        if idx != exp { is_seq = false; break; }
        exp += 1;
    }
    if is_seq && ht.len() > 0 {
        let mut arr = Vec::with_capacity(ht.len());
        for (_, _, val) in ht.iter() {
            arr.push(zval_to_value(val));
        }
        Value::Array(arr)
    } else {
        let mut map = Map::new();
        for (idx, key, val) in ht.iter() {
            let k = key.map(|s| s.to_string()).unwrap_or_else(|| idx.to_string());
            map.insert(k, zval_to_value(val));
        }
        Value::Object(map)
    }
}

// ══════════ STORE ENGINE ══════════

struct StoreOpts { pretty: bool, fsync: bool }
impl Default for StoreOpts { fn default() -> Self { Self { pretty: false, fsync: false } } }

struct StoreInner {
    path: PathBuf,
    cache: RwLock<Option<Arc<CachedData>>>,
    indexes: RwLock<HashMap<String, IndexStore>>,
    opts: RwLock<StoreOpts>,
    in_transaction: RwLock<bool>,
    tx_data: RwLock<Option<Value>>,
}
struct CachedData { data: Value, mtime: u64 }
struct IndexStore {
    single: HashMap<String, HashMap<String, Vec<usize>>>,
    compound: HashMap<String, HashMap<String, Vec<usize>>>,
    built_at: u64,
}
impl IndexStore { fn new() -> Self { Self { single: HashMap::new(), compound: HashMap::new(), built_at: 0 } } }

impl StoreInner {
    fn new(path: String) -> Self {
        let p = PathBuf::from(&path);
        if !p.exists() { let _ = fs::write(&p, "{}"); }
        Self { path: p, cache: RwLock::new(None), indexes: RwLock::new(HashMap::new()), opts: RwLock::new(StoreOpts::default()), in_transaction: RwLock::new(false), tx_data: RwLock::new(None) }
    }

    fn mtime(&self) -> u64 {
        fs::metadata(&self.path).and_then(|m| m.modified())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0)
    }

    fn read(&self) -> Result<Arc<CachedData>, String> {
        // During transaction, return tx_data
        if *self.in_transaction.read().unwrap() {
            if let Some(ref d) = *self.tx_data.read().unwrap() {
                return Ok(Arc::new(CachedData { data: d.clone(), mtime: self.mtime() }));
            }
        }
        let mt = self.mtime();
        { let c = self.cache.read().unwrap(); if let Some(ref cd) = *c { if cd.mtime >= mt { return Ok(Arc::clone(cd)); } } }
        let meta = fs::metadata(&self.path).map_err(|e| e.to_string())?;
        let flen = meta.len() as usize;
        let file = File::open(&self.path).map_err(|e| e.to_string())?;
        let data: Value = if flen == 0 { Value::Object(Map::new()) }
            else if flen < 64 { serde_json::from_str(&fs::read_to_string(&self.path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())? }
            else { serde_json::from_slice(&unsafe { Mmap::map(&file) }.map_err(|e| e.to_string())?).map_err(|e| e.to_string())? };
        let arc = Arc::new(CachedData { data, mtime: mt });
        *self.cache.write().unwrap() = Some(Arc::clone(&arc));
        Ok(arc)
    }

    fn write(&self, data: &Value) -> Result<(), String> {
        // During transaction, buffer changes in memory
        if *self.in_transaction.read().unwrap() {
            *self.tx_data.write().unwrap() = Some(data.clone());
            return Ok(());
        }
        self.flush(data)
    }

    fn flush(&self, data: &Value) -> Result<(), String> {
        let opts = self.opts.read().unwrap();
        let bytes = if opts.pretty { serde_json::to_vec_pretty(data) } else { serde_json::to_vec(data) }.map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("tmp");
        let mut f = File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        if opts.fsync { f.sync_all().map_err(|e| e.to_string())?; }
        fs::rename(&tmp, &self.path).map_err(|e| e.to_string())?;
        *self.cache.write().unwrap() = Some(Arc::new(CachedData { data: data.clone(), mtime: self.mtime() }));
        self.indexes.write().unwrap().clear();
        Ok(())
    }

    fn with_data<F, R>(&self, f: F) -> Result<R, String> where F: FnOnce(&Value) -> R {
        Ok(f(&self.read()?.data))
    }

    fn mutate<F>(&self, f: F) -> Result<(), String> where F: FnOnce(&mut Value) {
        let mut data = self.read()?.data.clone(); f(&mut data); self.write(&data)
    }

    fn build_index(&self, coll: &str, field: &str) -> Result<(), String> {
        let cd = self.read()?;
        let arr = match rp(&cd.data, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() { idx.entry(vkey(rn(item, field))).or_default().push(i); }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.into()).or_insert_with(IndexStore::new);
        store.single.insert(field.into(), idx); store.built_at = self.mtime();
        Ok(())
    }

    fn build_compound(&self, coll: &str, fields: &[String]) -> Result<(), String> {
        let cd = self.read()?;
        let arr = match rp(&cd.data, coll) { Some(Value::Array(a)) => a, _ => return Err(format!("'{}' not array", coll)) };
        let ck = fields.join("+");
        let mut idx: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, item) in arr.iter().enumerate() {
            let composite: String = fields.iter().map(|f| vkey(rn(item, f))).collect::<Vec<_>>().join("|");
            idx.entry(composite).or_default().push(i);
        }
        let mut indexes = self.indexes.write().unwrap();
        let store = indexes.entry(coll.into()).or_insert_with(IndexStore::new);
        store.compound.insert(ck, idx); store.built_at = self.mtime();
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

// ══════════ PATH HELPERS ══════════

fn rp<'a>(root: &'a Value, dp: &str) -> Option<&'a Value> {
    if dp.is_empty() { return Some(root); }
    let mut c = root;
    for k in dp.split('.') { c = match c { Value::Object(m) => m.get(k)?, Value::Array(a) => a.get(k.parse::<usize>().ok()?)?, _ => return None }; }
    Some(c)
}

fn rpm<'a>(root: &'a mut Value, dp: &str) -> Option<&'a mut Value> {
    if dp.is_empty() { return Some(root); }
    let keys: Vec<&str> = dp.split('.').collect();
    let mut c = root;
    for k in keys { c = match c { Value::Object(m) => m.get_mut(k)?, Value::Array(a) => a.get_mut(k.parse::<usize>().ok()?)?, _ => return None }; }
    Some(c)
}

fn sap(root: &mut Value, dp: &str, value: Value) {
    let keys: Vec<&str> = dp.split('.').collect();
    let mut c = root;
    for (i, k) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            match c { Value::Object(m) => { m.insert(k.to_string(), value); } Value::Array(a) => { if let Ok(idx) = k.parse::<usize>() { if idx < a.len() { a[idx] = value; } else { a.push(value); } } } _ => {} }
            return;
        }
        let nn = keys.get(i+1).map(|x| x.parse::<usize>().is_ok()).unwrap_or(false);
        match c {
            Value::Object(m) => { if !m.contains_key(*k) { m.insert(k.to_string(), if nn { Value::Array(vec![]) } else { Value::Object(Map::new()) }); } c = m.get_mut(*k).unwrap(); }
            Value::Array(a) => { if let Ok(idx) = k.parse::<usize>() { while a.len() <= idx { a.push(Value::Object(Map::new())); } c = &mut a[idx]; } else { return; } }
            _ => return,
        }
    }
}

fn rap(root: &mut Value, dp: &str) -> bool {
    let keys: Vec<&str> = dp.split('.').collect();
    if keys.is_empty() { return false; }
    if keys.len() == 1 { return match root { Value::Object(m) => m.remove(keys[0]).is_some(), Value::Array(a) => { if let Ok(i) = keys[0].parse::<usize>() { if i < a.len() { a.remove(i); return true; } } false } _ => false }; }
    let pp = keys[..keys.len()-1].join("."); let last = keys[keys.len()-1];
    if let Some(p) = rpm(root, &pp) { match p { Value::Object(m) => m.remove(last).is_some(), Value::Array(a) => { if let Ok(i) = last.parse::<usize>() { if i < a.len() { a.remove(i); return true; } } false } _ => false } } else { false }
}

fn merge_v(base: &mut Value, over: &Value) {
    match (base, over) { (Value::Object(b), Value::Object(o)) => { for (k, v) in o { if let Some(e) = b.get_mut(k) { merge_v(e, v); } else { b.insert(k.clone(), v.clone()); } } } (b, o) => { *b = o.clone(); } }
}

// ══════════ QUERY ENGINE ══════════

fn rn<'a>(item: &'a Value, key: &str) -> Option<&'a Value> {
    if let Some(o) = item.as_object() { if let Some(v) = o.get(key) { return Some(v); } }
    rp(item, key)
}

fn mat(item: &Value, cond: &Value) -> bool {
    let co = match cond.as_object() { Some(o) => o, None => return false };
    for (k, c) in co {
        match k.as_str() {
            "$and" => { if let Some(a) = c.as_array() { for s in a { if !mat(item, s) { return false; } } } }
            "$or"  => { if let Some(a) = c.as_array() { if !a.iter().any(|s| mat(item, s)) { return false; } } }
            "$not" => { if mat(item, c) { return false; } }
            f => { let fv = rn(item, f); if c.is_object() { for (op, opd) in c.as_object().unwrap() { if !eop(&fv, op, opd) { return false; } } } else if fv.unwrap_or(&Value::Null) != c { return false; } }
        }
    }
    true
}

fn eop(fv: &Option<&Value>, op: &str, opd: &Value) -> bool {
    let v = match fv { Some(v) => *v, None => &Value::Null };
    match op {
        "$eq" => v == opd, "$ne" => v != opd,
        "$gt" => cv(v, opd) == Some(std::cmp::Ordering::Greater),
        "$gte" => matches!(cv(v, opd), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
        "$lt" => cv(v, opd) == Some(std::cmp::Ordering::Less),
        "$lte" => matches!(cv(v, opd), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
        "$in" => opd.as_array().map_or(false, |a| a.iter().any(|x| x == v)),
        "$nin" => opd.as_array().map_or(true, |a| !a.iter().any(|x| x == v)),
        "$contains" => if let (Some(s), Some(n)) = (v.as_str(), opd.as_str()) { s.contains(n) } else { false },
        "$startsWith" => if let (Some(s), Some(p)) = (v.as_str(), opd.as_str()) { s.starts_with(p) } else { false },
        "$endsWith" => if let (Some(s), Some(x)) = (v.as_str(), opd.as_str()) { s.ends_with(x) } else { false },
        "$exists" => { let ex = !v.is_null(); opd.as_bool().map_or(false, |e| ex == e) }
        "$size" => v.as_array().map_or(false, |a| opd.as_u64().map_or(false, |s| a.len() as u64 == s)),
        "$type" => opd.as_str().map_or(false, |e| tn(v) == e),
        _ => false,
    }
}

fn cv(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) { (Value::Number(na), Value::Number(nb)) => na.as_f64()?.partial_cmp(&nb.as_f64()?), (Value::String(sa), Value::String(sb)) => Some(sa.cmp(sb)), _ => None }
}

fn tn(v: &Value) -> &'static str {
    match v { Value::Null => "null", Value::Bool(_) => "boolean", Value::Number(n) => if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) { "number" } else { "integer" }, Value::String(_) => "string", Value::Array(_) => "array", Value::Object(_) => "object" }
}

fn vkey(val: Option<&Value>) -> String {
    match val { None | Some(Value::Null) => "__null__".into(), Some(Value::Bool(b)) => b.to_string(), Some(Value::Number(n)) => n.to_string(), Some(Value::String(s)) => s.clone(), Some(o) => o.to_string() }
}

fn search_in_value(val: &Value, keyword: &str) -> bool {
    match val {
        Value::String(s) => s.to_lowercase().contains(keyword),
        Value::Object(m) => m.values().any(|v| search_in_value(v, keyword)),
        Value::Array(a) => a.iter().any(|v| search_in_value(v, keyword)),
        Value::Number(n) => n.to_string().contains(keyword),
        _ => false,
    }
}

// ══════════ FLUENT QUERY ══════════

fn exec_fluent(data: &[Value], q: &Value) -> Vec<Value> {
    let mut r: Vec<&Value> = data.iter().collect();
    if let Some(ws) = q.get("where").and_then(|w| w.as_array()) {
        r.retain(|item| { for c in ws { let f = c.get("field").and_then(|x| x.as_str()).unwrap_or(""); let op = c.get("op").and_then(|x| x.as_str()).unwrap_or("="); let v = c.get("value").unwrap_or(&Value::Null); if !efop(&rn(item, f), op, v) { return false; } } true });
    }
    if let Some(ob) = q.get("order_by").and_then(|o| o.as_object()) {
        let f = ob.get("field").and_then(|x| x.as_str()).unwrap_or(""); let desc = ob.get("direction").and_then(|d| d.as_str()) == Some("desc");
        r.sort_by(|a, b| { let c = match (rn(a, f), rn(b, f)) { (Some(va), Some(vb)) => cv(va, vb).unwrap_or(std::cmp::Ordering::Equal), (Some(_), None) => std::cmp::Ordering::Less, (None, Some(_)) => std::cmp::Ordering::Greater, _ => std::cmp::Ordering::Equal }; if desc { c.reverse() } else { c } });
    }
    let off = q.get("offset").and_then(|o| o.as_u64()).unwrap_or(0) as usize;
    let lim = q.get("limit").and_then(|l| l.as_u64());
    let r: Vec<&Value> = if let Some(l) = lim { r.into_iter().skip(off).take(l as usize).collect() } else { r.into_iter().skip(off).collect() };
    if let Some(fs) = q.get("select").and_then(|s| s.as_array()) {
        let fns: Vec<&str> = fs.iter().filter_map(|f| f.as_str()).collect();
        return r.iter().map(|item| { let mut o = Map::new(); for f in &fns { if let Some(v) = rn(item, f) { o.insert(f.to_string(), v.clone()); } } Value::Object(o) }).collect();
    }
    r.into_iter().cloned().collect()
}

fn efop(fv: &Option<&Value>, op: &str, opd: &Value) -> bool {
    let v = match fv { Some(v) => *v, None => &Value::Null };
    match op {
        "="|"==" => v == opd, "!="|"<>" => v != opd,
        ">" => cv(v, opd) == Some(std::cmp::Ordering::Greater), ">=" => matches!(cv(v, opd), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
        "<" => cv(v, opd) == Some(std::cmp::Ordering::Less), "<=" => matches!(cv(v, opd), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
        "in" => opd.as_array().map_or(false, |a| a.iter().any(|x| x == v)), "not in" => opd.as_array().map_or(true, |a| !a.iter().any(|x| x == v)),
        "contains" => if let (Some(s), Some(n)) = (v.as_str(), opd.as_str()) { s.contains(n) } else { false },
        "starts_with" => if let (Some(s), Some(p)) = (v.as_str(), opd.as_str()) { s.starts_with(p) } else { false },
        "ends_with" => if let (Some(s), Some(x)) = (v.as_str(), opd.as_str()) { s.ends_with(x) } else { false },
        "between" => if let Some(a) = opd.as_array() { a.len() == 2 && matches!(cv(v, &a[0]), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) && matches!(cv(v, &a[1]), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)) } else { false },
        _ => false,
    }
}

// ══════════ AGGREGATION ══════════

fn agg(data: &[Value], field: &str, op: &str) -> Value {
    let vs: Vec<f64> = data.iter().filter_map(|i| rn(i, field)).filter_map(|v| v.as_f64()).collect();
    if vs.is_empty() { return Value::Null; }
    match op { "sum" => json!(vs.iter().sum::<f64>()), "avg" => json!(vs.iter().sum::<f64>() / vs.len() as f64), "min" => json!(vs.iter().cloned().fold(f64::INFINITY, f64::min)), "max" => json!(vs.iter().cloned().fold(f64::NEG_INFINITY, f64::max)), "count" => json!(vs.len()), _ => Value::Null }
}

fn grp(data: &[Value], field: &str) -> Value {
    let mut g: Map<String, Value> = Map::new();
    for item in data { let k = rn(item, field).map(|v| match v { Value::String(s) => s.clone(), o => o.to_string() }).unwrap_or_else(|| "__null__".into()); g.entry(k).or_insert_with(|| Value::Array(vec![])).as_array_mut().unwrap().push(item.clone()); }
    Value::Object(g)
}

fn plk(data: &[Value], fields: &[&str]) -> Vec<Value> {
    data.iter().map(|item| { if fields.len() == 1 { rn(item, fields[0]).cloned().unwrap_or(Value::Null) } else { let mut o = Map::new(); for f in fields { o.insert(f.to_string(), rn(item, f).cloned().unwrap_or(Value::Null)); } Value::Object(o) } }).collect()
}

// ══════════ VALIDATION ══════════

fn vld(data: &Value, schema: &Value, path: &str) -> Vec<Value> {
    let mut errs = Vec::new();
    let so = match schema.as_object() { Some(o) => o, None => return errs };
    if let Some(t) = so.get("type").and_then(|t| t.as_str()) {
        let ok = match t { "string" => data.is_string(), "integer" => data.is_i64() || data.is_u64() || (data.is_f64() && data.as_f64().map(|f| f.fract() == 0.0).unwrap_or(false)), "number"|"float"|"double" => data.is_number(), "boolean"|"bool" => data.is_boolean(), "array" => data.is_array(), "object" => data.is_object(), "null" => data.is_null(), "any" => true, _ => true };
        if !ok { errs.push(json!({"path":if path.is_empty(){"(root)"}else{path},"error":format!("Expected '{}', got '{}'",t,tn(data)),"code":"TYPE_MISMATCH"})); return errs; }
    }
    if data.is_null() && so.get("nullable").and_then(|v| v.as_bool()).unwrap_or(false) { return errs; }
    if let Some(s) = data.as_str() {
        if let Some(mn) = so.get("minLength").and_then(|v| v.as_u64()) { if (s.len() as u64) < mn { errs.push(json!({"path":path,"error":format!("Min length {} (got {})",mn,s.len()),"code":"MIN_LENGTH"})); } }
        if let Some(mx) = so.get("maxLength").and_then(|v| v.as_u64()) { if (s.len() as u64) > mx { errs.push(json!({"path":path,"error":format!("Max length {} (got {})",mx,s.len()),"code":"MAX_LENGTH"})); } }
        if let Some(fmt) = so.get("format").and_then(|v| v.as_str()) {
            let ok = match fmt { "email" => s.contains('@') && s.contains('.') && s.len() > 5, "url"|"uri" => s.starts_with("http://") || s.starts_with("https://"), "ipv4" => { let p:Vec<&str>=s.split('.').collect(); p.len()==4 && p.iter().all(|x| x.parse::<u8>().is_ok()) }, "date" => s.len()==10 && s.split('-').count()==3, "uuid" => s.len()==36 && s.chars().filter(|c|*c=='-').count()==4, _ => true };
            if !ok { errs.push(json!({"path":path,"error":format!("Invalid format: '{}'",fmt),"code":"FORMAT_INVALID"})); }
        }
    }
    if let Some(n) = data.as_f64() {
        if let Some(mn) = so.get("min").and_then(|v| v.as_f64()) { if n < mn { errs.push(json!({"path":path,"error":format!("Min {} (got {})",mn,n),"code":"MIN_VALUE"})); } }
        if let Some(mx) = so.get("max").and_then(|v| v.as_f64()) { if n > mx { errs.push(json!({"path":path,"error":format!("Max {} (got {})",mx,n),"code":"MAX_VALUE"})); } }
    }
    if let Some(ev) = so.get("enum").and_then(|v| v.as_array()) { if !ev.iter().any(|v| v == data) { errs.push(json!({"path":path,"error":"Not in enum","code":"ENUM_MISMATCH"})); } }
    if let Some(arr) = data.as_array() {
        if let Some(mn) = so.get("minItems").and_then(|v| v.as_u64()) { if (arr.len() as u64) < mn { errs.push(json!({"path":path,"error":format!("Min {} items",mn),"code":"MIN_ITEMS"})); } }
        if let Some(mx) = so.get("maxItems").and_then(|v| v.as_u64()) { if (arr.len() as u64) > mx { errs.push(json!({"path":path,"error":format!("Max {} items",mx),"code":"MAX_ITEMS"})); } }
        if so.get("uniqueItems").and_then(|v| v.as_bool()).unwrap_or(false) { let mut seen = Vec::new(); for item in arr { if seen.contains(item) { errs.push(json!({"path":path,"error":"Duplicates","code":"UNIQUE_ITEMS"})); break; } seen.push(item.clone()); } }
        if let Some(is) = so.get("items") { for (i, item) in arr.iter().enumerate() { errs.extend(vld(item, is, &format!("{}.{}",path,i))); } }
    }
    if let Some(obj) = data.as_object() {
        if let Some(req) = so.get("required").and_then(|v| v.as_array()) { for r in req { if let Some(f) = r.as_str() { if !obj.contains_key(f) { errs.push(json!({"path":format!("{}.{}",path,f),"error":format!("Required: '{}'",f),"code":"REQUIRED","field":f})); } } } }
        if let Some(props) = so.get("properties").and_then(|v| v.as_object()) { for (pn, ps) in props { if let Some(pv) = obj.get(pn) { errs.extend(vld(pv, ps, &if path.is_empty(){pn.clone()}else{format!("{}.{}",path,pn)})); } } }
        if so.get("additionalProperties").and_then(|v| v.as_bool()) == Some(false) { if let Some(props) = so.get("properties").and_then(|v| v.as_object()) { for k in obj.keys() { if !props.contains_key(k) { errs.push(json!({"path":format!("{}.{}",path,k),"error":format!("Additional: '{}'",k),"code":"ADDITIONAL_PROPERTY"})); } } } }
    }
    if let Some(ifs) = so.get("if") { if vld(data, ifs, path).is_empty() { if let Some(t) = so.get("then") { errs.extend(vld(data, t, path)); } } else { if let Some(e) = so.get("else") { errs.extend(vld(data, e, path)); } } }
    if let Some(oo) = so.get("oneOf").and_then(|v| v.as_array()) { let m: usize = oo.iter().filter(|s| vld(data, s, path).is_empty()).count(); if m != 1 { errs.push(json!({"path":path,"error":format!("oneOf: {} matched",m),"code":"ONE_OF"})); } }
    if let Some(ao) = so.get("anyOf").and_then(|v| v.as_array()) { if !ao.iter().any(|s| vld(data, s, path).is_empty()) { errs.push(json!({"path":path,"error":"anyOf: none","code":"ANY_OF"})); } }
    errs
}

// ══════════ PHP CLASS ══════════

#[php_class(name = "JsonQ\\Store")]
pub struct JsonStore { inner: Option<StoreInner> }

#[php_impl]
impl JsonStore {
    #[php_method] pub fn __construct(path: String) -> JsonStore { JsonStore { inner: Some(StoreInner::new(path)) } }

    // ── Options ──
    #[php_method] pub fn setOption(&self, key: String, value: &Zval) -> bool {
        let i = match &self.inner { Some(i) => i, None => return false };
        let mut opts = i.opts.write().unwrap();
        match key.as_str() {
            "pretty" | "pretty_print" => { opts.pretty = value.bool().unwrap_or(false); true }
            "fsync" | "sync" => { opts.fsync = value.bool().unwrap_or(false); true }
            _ => false,
        }
    }
    #[php_method] pub fn getOption(&self, key: String) -> Zval {
        let i = match &self.inner { Some(i) => i, None => return Zval::new() };
        let opts = i.opts.read().unwrap();
        let mut z = Zval::new();
        match key.as_str() { "pretty"|"pretty_print" => { z.set_bool(opts.pretty); } "fsync"|"sync" => { z.set_bool(opts.fsync); } _ => {} }
        z
    }

    // ── Transactions ──
    #[php_method] pub fn beginTransaction(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let data = i.read().map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?.data.clone();
        *i.tx_data.write().unwrap() = Some(data);
        *i.in_transaction.write().unwrap() = true;
        Ok(true)
    }
    #[php_method] pub fn commit(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let data = i.tx_data.read().unwrap().clone().ok_or("No active transaction")?;
        *i.in_transaction.write().unwrap() = false;
        *i.tx_data.write().unwrap() = None;
        i.flush(&data).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        Ok(true)
    }
    #[php_method] pub fn rollback(&self) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        *i.in_transaction.write().unwrap() = false;
        *i.tx_data.write().unwrap() = None;
        Ok(true)
    }
    #[php_method] pub fn inTransaction(&self) -> bool {
        self.inner.as_ref().map(|i| *i.in_transaction.read().unwrap()).unwrap_or(false)
    }

    // ── Batch Operations ──
    #[php_method] pub fn setMany(&self, pairs: &Zval) -> PhpResult<i64> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let pv = zval_to_value(pairs);
        let po = match pv.as_object() { Some(o) => o, None => return Ok(0) };
        let cd = i.read().map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        let mut data = cd.data.clone();
        let mut count = 0i64;
        for (path, value) in po { sap(&mut data, path, value.clone()); count += 1; }
        i.write(&data).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        Ok(count)
    }
    #[php_method] pub fn removeMany(&self, paths: Vec<String>) -> PhpResult<i64> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        let mut data = cd.data.clone();
        let mut count = 0i64;
        for path in &paths { if rap(&mut data, path) { count += 1; } }
        i.write(&data).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        Ok(count)
    }

    // ── Import/Export ──
    #[php_method] pub fn toJson(&self, pretty: Option<bool>) -> PhpResult<String> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let cd = i.read().map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        if pretty.unwrap_or(false) { serde_json::to_string_pretty(&cd.data).map_err(|e| e.to_string().into()) } else { serde_json::to_string(&cd.data).map_err(|e| e.to_string().into()) }
    }
    #[php_method] pub fn fromJson(&self, json_str: String) -> PhpResult<bool> {
        let i = self.inner.as_ref().ok_or("Not init")?;
        let data: Value = serde_json::from_str(&json_str).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        i.write(&data).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?;
        Ok(true)
    }

    // ── Extra ──
    #[php_method] pub fn getAll(&self) -> Zval { self.inner.as_ref().and_then(|i| i.with_data(|d| value_to_zval(d)).ok()).unwrap_or_else(Zval::new) }
    #[php_method] pub fn clear(&self) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; i.write(&Value::Object(Map::new())).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?; Ok(true) }
    #[php_method] pub fn search(&self, collection: String, keyword: String) -> Zval {
        let i = match &self.inner { Some(i) => i, None => return Zval::new() };
        let kw = keyword.to_lowercase();
        i.with_data(|d| {
            let arr = match rp(d, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            let matched: Vec<Value> = arr.iter().filter(|item| search_in_value(item, &kw)).cloned().collect();
            value_to_zval(&Value::Array(matched))
        }).unwrap_or_else(|_| Zval::new())
    }

    #[php_method] pub fn get(&self, path: String) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; i.with_data(|d| rp(d, &path).map(|v| value_to_zval(v)).unwrap_or_else(Zval::new)).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn has(&self, path: String) -> bool { self.inner.as_ref().and_then(|i| i.with_data(|d| rp(d, &path).is_some()).ok()).unwrap_or(false) }
    #[php_method] pub fn count(&self, path: String) -> i64 { self.inner.as_ref().and_then(|i| i.with_data(|d| match rp(d, &path) { Some(Value::Array(a)) => a.len() as i64, Some(Value::Object(o)) => o.len() as i64, _ => -1 }).ok()).unwrap_or(-1) }
    #[php_method] pub fn keys(&self, path: String) -> Vec<String> { self.inner.as_ref().and_then(|i| i.with_data(|d| match if path.is_empty(){Some(d)}else{rp(d,&path)} { Some(Value::Object(o)) => o.keys().cloned().collect(), _ => vec![] }).ok()).unwrap_or_default() }

    #[php_method] pub fn set(&self, path: String, value: &Zval) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; let v = zval_to_value(value); i.mutate(|d| sap(d, &path, v)).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn remove(&self, path: String) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; let p = path.clone(); i.mutate(|d| { rap(d, &p); }).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn push(&self, path: String, value: &Zval) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; let v = zval_to_value(value); let cd = i.read().map_err(|e| ext_php_rs::exception::PhpException::default(e))?; let mut data = cd.data.clone(); match if path.is_empty(){Some(&mut data)}else{rpm(&mut data, &path)} { Some(Value::Array(a)) => { a.push(v); i.write(&data).map_err(|e| e.into()).map(|_| true) } _ => Ok(false) } }
    #[php_method] pub fn merge(&self, path: String, value: &Zval) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; let nv = zval_to_value(value); let p = path.clone(); i.mutate(|d| { if let Some(e) = if p.is_empty(){Some(&mut *d)}else{rpm(d,&p)} { merge_v(e, &nv); } else { sap(d, &p, nv); } }).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn increment(&self, path: String, amount: Option<f64>) -> PhpResult<bool> { let amt = amount.unwrap_or(1.0); let i = self.inner.as_ref().ok_or("Not init")?; i.mutate(|d| { if let Some(v) = rpm(d, &path) { if let Some(n) = v.as_f64() { *v = json!(n + amt); } } }).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn decrement(&self, path: String, amount: Option<f64>) -> PhpResult<bool> { self.increment(path, Some(-(amount.unwrap_or(1.0)))) }

    #[php_method] pub fn find(&self, collection: String, conditions: &Zval) -> Zval {
        let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let cond = zval_to_value(conditions);
        i.with_data(|d| {
            let arr = match rp(d, &collection) { Some(Value::Array(a)) => a, _ => return value_to_zval(&Value::Array(vec![])) };
            if let Some(co) = cond.as_object() { if co.len() == 1 { if let Some((f, v)) = co.iter().next() { if !f.starts_with('$') && !v.is_object() { if let Some(pos) = i.idx_lookup(&collection, f, v) { return value_to_zval(&Value::Array(pos.iter().filter_map(|&j| arr.get(j).cloned()).collect())); } } } } }
            value_to_zval(&Value::Array(arr.iter().filter(|item| mat(item, &cond)).cloned().collect()))
        }).unwrap_or_else(|_| Zval::new())
    }
    #[php_method] pub fn findOne(&self, collection: String, conditions: &Zval) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let c = zval_to_value(conditions); i.with_data(|d| match rp(d, &collection) { Some(Value::Array(a)) => a.iter().find(|item| mat(item, &c)).map(|f| value_to_zval(f)).unwrap_or_else(Zval::new), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn executeQuery(&self, collection: String, query_spec: &Zval) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let q = zval_to_value(query_spec); i.with_data(|d| match rp(d, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(exec_fluent(a, &q))), _ => value_to_zval(&Value::Array(vec![])) }).unwrap_or_else(|_| Zval::new()) }

    #[php_method] pub fn aggregate(&self, collection: String, field: String, operation: String) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; i.with_data(|d| match rp(d, &collection) { Some(Value::Array(a)) => value_to_zval(&agg(a, &field, &operation)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn groupBy(&self, collection: String, field: String) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; i.with_data(|d| match rp(d, &collection) { Some(Value::Array(a)) => value_to_zval(&grp(a, &field)), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn pluck(&self, collection: String, fields: Vec<String>) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let fr: Vec<&str> = fields.iter().map(|s| s.as_str()).collect(); i.with_data(|d| match rp(d, &collection) { Some(Value::Array(a)) => value_to_zval(&Value::Array(plk(a, &fr))), _ => Zval::new() }).unwrap_or_else(|_| Zval::new()) }

    #[php_method] pub fn validate(&self, path: String, schema: &Zval) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(schema); i.with_data(|d| { let t = if path.is_empty(){d}else{match rp(d,&path){Some(v)=>v,None=>return Zval::new()}}; let e = vld(t, &sv, &path); value_to_zval(&json!({"valid":e.is_empty(),"error_count":e.len(),"errors":e})) }).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn validateCollection(&self, path: String, item_schema: &Zval) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let sv = zval_to_value(item_schema); i.with_data(|d| { let arr = match rp(d, &path) { Some(Value::Array(a)) => a, _ => return Zval::new() }; let mut ae = Vec::new(); let mut inv = 0usize; for (j, item) in arr.iter().enumerate() { let e = vld(item, &sv, &format!("{}.{}",path,j)); if !e.is_empty() { inv += 1; ae.push(json!({"index":j,"errors":e})); } } value_to_zval(&json!({"valid":ae.is_empty(),"total_items":arr.len(),"valid_items":arr.len()-inv,"invalid_items":inv,"details":ae})) }).unwrap_or_else(|_| Zval::new()) }

    #[php_method] pub fn createIndex(&self, collection: String, field: String) -> PhpResult<bool> { self.inner.as_ref().ok_or("Not init")?.build_index(&collection, &field).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn createCompoundIndex(&self, collection: String, fields: Vec<String>) -> PhpResult<bool> { self.inner.as_ref().ok_or("Not init")?.build_compound(&collection, &fields).map_err(|e| e.into()).map(|_| true) }
    #[php_method] pub fn indexLookup(&self, collection: String, field: String, value: &Zval) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let v = zval_to_value(value); if let Some(pos) = i.idx_lookup(&collection, &field, &v) { i.with_data(|d| if let Some(Value::Array(a)) = rp(d, &collection) { value_to_zval(&Value::Array(pos.iter().filter_map(|&j| a.get(j).cloned()).collect())) } else { Zval::new() }).unwrap_or_else(|_| Zval::new()) } else { Zval::new() } }
    #[php_method] pub fn listIndexes(&self) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let idx = i.indexes.read().unwrap(); let mut r = Vec::new(); for (c, s) in idx.iter() { for (f, im) in &s.single { r.push(json!({"collection":c,"type":"single","field":f,"unique_values":im.len(),"total_entries":im.values().map(|v|v.len()).sum::<usize>()})); } for (f, im) in &s.compound { r.push(json!({"collection":c,"type":"compound","fields":f,"unique_values":im.len(),"total_entries":im.values().map(|v|v.len()).sum::<usize>()})); } } value_to_zval(&Value::Array(r)) }
    #[php_method] pub fn dropIndex(&self, collection: String) -> bool { self.inner.as_ref().map(|i| i.indexes.write().unwrap().remove(&collection).is_some()).unwrap_or(false) }
    #[php_method] pub fn dropAllIndexes(&self) -> i64 { self.inner.as_ref().map(|i| { let mut idx = i.indexes.write().unwrap(); let c = idx.len() as i64; idx.clear(); c }).unwrap_or(0) }

    #[php_method] pub fn stats(&self) -> Zval { let i = match &self.inner { Some(i) => i, None => return Zval::new() }; let meta = match fs::metadata(&i.path) { Ok(m) => m, Err(_) => return Zval::new() }; i.with_data(|d| { let fs = meta.len(); let fsh = if fs<1024{format!("{} B",fs)}else if fs<1048576{format!("{:.2} KB",fs as f64/1024.0)}else{format!("{:.2} MB",fs as f64/1048576.0)}; let keys: Vec<Value> = if let Value::Object(o) = d { o.keys().map(|k| Value::String(k.clone())).collect() } else { vec![] }; let kc = if let Value::Object(o) = d { o.len() } else { 0 }; let ic: usize = i.indexes.read().unwrap().values().map(|s| s.single.len()+s.compound.len()).sum(); value_to_zval(&json!({"file_path":i.path.to_string_lossy(),"file_size":fs,"file_size_h":fsh,"top_level_keys":keys,"key_count":kc,"active_indexes":ic})) }).unwrap_or_else(|_| Zval::new()) }
    #[php_method] pub fn backup(&self, backup_path: Option<String>) -> PhpResult<String> { let i = self.inner.as_ref().ok_or("Not init")?; let t = match backup_path { Some(p) if !p.is_empty() => p, _ => format!("{}.backup.{}",i.path.to_string_lossy(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()) }; fs::copy(&i.path, &t).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?; Ok(t) }
    #[php_method] pub fn restore(&self, backup_path: String) -> PhpResult<bool> { let i = self.inner.as_ref().ok_or("Not init")?; fs::copy(&backup_path, &i.path).map_err(|e| ext_php_rs::exception::PhpException::default(e.to_string()))?; *i.cache.write().unwrap() = None; i.indexes.write().unwrap().clear(); Ok(true) }
}

#[php_function] pub fn jsonq_version() -> String { env!("CARGO_PKG_VERSION").to_string() }

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder { module }
