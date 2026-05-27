//! Basic SQL SELECT query translator and execution engine
use serde_json::{json, Map, Value};
use crate::store::StoreInner;
use crate::path::read_path;
use crate::query::execute_query;

#[derive(Debug)]
pub struct ParsedSqlQuery {
    pub fields: Vec<String>,
    pub collection: String,
    pub conditions: Value,
    pub order_by: Option<(String, bool)>, // (field, is_desc)
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

fn find_keyword(sql: &str, keyword: &str) -> Option<usize> {
    let sql_upper = sql.to_uppercase();
    let kw_upper = keyword.to_uppercase();
    
    let mut start = 0;
    while let Some(idx) = sql_upper[start..].find(&kw_upper) {
        let absolute_idx = start + idx;
        
        let prev_char_ok = absolute_idx == 0 || sql_upper.chars().nth(absolute_idx - 1).unwrap().is_whitespace();
        let next_idx = absolute_idx + kw_upper.len();
        let next_char_ok = next_idx >= sql_upper.len() || sql_upper.chars().nth(next_idx).unwrap().is_whitespace();
        
        if prev_char_ok && next_char_ok {
            return Some(absolute_idx);
        }
        start = absolute_idx + 1;
    }
    None
}

fn split_case_insensitive<'a>(s: &'a str, pattern: &str) -> Vec<&'a str> {
    let s_upper = s.to_uppercase();
    let pat_upper = pattern.to_uppercase();
    let mut parts = Vec::new();
    let mut last_idx = 0;
    
    let mut start = 0;
    while let Some(idx) = s_upper[start..].find(&pat_upper) {
        let absolute_idx = start + idx;
        parts.push(&s[last_idx..absolute_idx]);
        last_idx = absolute_idx + pattern.len();
        start = last_idx;
    }
    parts.push(&s[last_idx..]);
    parts
}

fn find_substring_case_insensitive(s: &str, pattern: &str) -> Option<usize> {
    let s_upper = s.to_uppercase();
    let pat_upper = pattern.to_uppercase();
    s_upper.find(&pat_upper)
}

fn parse_sql_value(s: &str) -> Result<Value, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty value in SQL condition".to_string());
    }

    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        let inner = &s[1..s.len() - 1];
        return Ok(Value::String(inner.to_string()));
    }

    if s.eq_ignore_ascii_case("true") {
        return Ok(Value::Bool(true));
    }
    if s.eq_ignore_ascii_case("false") {
        return Ok(Value::Bool(false));
    }
    if s.eq_ignore_ascii_case("null") {
        return Ok(Value::Null);
    }

    if let Ok(i) = s.parse::<i64>() {
        return Ok(json!(i));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(json!(f));
    }

    Ok(Value::String(s.to_string()))
}

fn parse_where(where_str: &str) -> Result<Value, String> {
    let where_str = where_str.trim();
    if where_str.is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    let mut conditions = Map::new();
    let parts = split_case_insensitive(where_str, " AND ");

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let operators = [
            (">=", "$gte"),
            ("<=", "$lte"),
            ("<>", "$ne"),
            ("!=", "$ne"),
            ("=", "$eq"),
            (">", "$gt"),
            ("<", "$lt"),
            (" LIKE ", "$like"),
            (" like ", "$like"),
            (" IN ", "$in"),
            (" in ", "$in"),
        ];

        let mut parsed_op = None;
        for &(op_sql, op_mongo) in &operators {
            if let Some(idx) = find_substring_case_insensitive(part, op_sql) {
                let field = part[..idx].trim().to_string();
                let val_str = part[idx + op_sql.len()..].trim();
                parsed_op = Some((field, op_mongo, val_str));
                break;
            }
        }

        let (field, op_mongo, val_str) = parsed_op.ok_or_else(|| {
            format!("Unsupported or malformed WHERE condition: '{}'", part)
        })?;

        if op_mongo == "$like" {
            let value = parse_sql_value(val_str)?;
            let pat = value.as_str().ok_or_else(|| "LIKE value must be a string".to_string())?;
            let has_leading = pat.starts_with('%');
            let has_trailing = pat.ends_with('%');
            
            let mut field_cond = Map::new();
            if has_leading && has_trailing {
                let inner_pat = &pat[1..pat.len() - 1];
                field_cond.insert("contains".to_string(), Value::String(inner_pat.to_string()));
            } else if has_leading {
                let inner_pat = &pat[1..];
                field_cond.insert("endsWith".to_string(), Value::String(inner_pat.to_string()));
            } else if has_trailing {
                let inner_pat = &pat[..pat.len() - 1];
                field_cond.insert("startsWith".to_string(), Value::String(inner_pat.to_string()));
            } else {
                field_cond.insert("eq".to_string(), Value::String(pat.to_string()));
            }
            // For checking condition operator, fluent uses 'op' and 'value' directly.
            // But wait, the MongoDB query uses `$startsWith` etc.
            // Let's translate it directly to the fluent query {"field": field, "op": op, "value": value}!
            // Wait, we will format this in `translate_to_fluent_query` below.
            // So here, let's just store the parsed operator and value in a temporary structure, or simply return conditions as Value!
            // To make it easy, let's store conditions as:
            // {"field_name": {"op": "operator", "value": val}}
            // So set conditions:
            let mut cond_obj = Map::new();
            if has_leading && has_trailing {
                cond_obj.insert("op".to_string(), json!("contains"));
                cond_obj.insert("value".to_string(), json!(&pat[1..pat.len() - 1]));
            } else if has_leading {
                cond_obj.insert("op".to_string(), json!("endsWith"));
                cond_obj.insert("value".to_string(), json!(&pat[1..]));
            } else if has_trailing {
                cond_obj.insert("op".to_string(), json!("startsWith"));
                cond_obj.insert("value".to_string(), json!(&pat[..pat.len() - 1]));
            } else {
                cond_obj.insert("op".to_string(), json!("="));
                cond_obj.insert("value".to_string(), json!(pat));
            }
            conditions.insert(field, Value::Object(cond_obj));
        } else if op_mongo == "$in" {
            let stripped = val_str.trim();
            if !stripped.starts_with('(') || !stripped.ends_with(')') {
                return Err(format!("Malformed IN clause value: '{}'", val_str));
            }
            let inner = &stripped[1..stripped.len() - 1];
            let mut list = Vec::new();
            for item in inner.split(',') {
                list.push(parse_sql_value(item.trim())?);
            }
            let mut cond_obj = Map::new();
            cond_obj.insert("op".to_string(), json!("in"));
            cond_obj.insert("value".to_string(), Value::Array(list));
            conditions.insert(field, Value::Object(cond_obj));
        } else {
            let value = parse_sql_value(val_str)?;
            let op_str = match op_mongo {
                "$gte" => ">=",
                "$lte" => "<=",
                "$ne" => "!=",
                "$eq" => "=",
                "$gt" => ">",
                "$lt" => "<",
                _ => "=",
            };
            let mut cond_obj = Map::new();
            cond_obj.insert("op".to_string(), json!(op_str));
            cond_obj.insert("value".to_string(), value);
            conditions.insert(field, Value::Object(cond_obj));
        }
    }

    Ok(Value::Object(conditions))
}

fn parse_order_by(order_str: &str) -> Result<(String, bool), String> {
    let order_str = order_str.trim();
    let parts: Vec<&str> = order_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty ORDER BY clause".to_string());
    }

    let field = parts[0].to_string();
    let is_desc = if parts.len() > 1 {
        parts[1].eq_ignore_ascii_case("desc")
    } else {
        false
    };
    Ok((field, is_desc))
}

fn parse_limit(limit_str: &str) -> Result<usize, String> {
    limit_str.trim().parse::<usize>().map_err(|e| e.to_string())
}

fn parse_offset(offset_str: &str) -> Result<usize, String> {
    offset_str.trim().parse::<usize>().map_err(|e| e.to_string())
}

pub fn parse_sql_select(sql: &str) -> Result<ParsedSqlQuery, String> {
    let sql = sql.trim().trim_end_matches(';');
    
    let select_idx = find_keyword(sql, "SELECT").ok_or_else(|| "Query must start with SELECT".to_string())?;
    let from_idx = find_keyword(sql, "FROM").ok_or_else(|| "Missing FROM clause".to_string())?;
    
    if select_idx != 0 {
        return Err("Query must start with SELECT".to_string());
    }

    let select_slice = &sql[select_idx + 6..from_idx];
    let fields_str = select_slice.trim();
    let fields = if fields_str == "*" {
        Vec::new()
    } else {
        fields_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    };

    let clauses = [
        ("WHERE", 5),
        ("ORDER BY", 8),
        ("LIMIT", 5),
        ("OFFSET", 6),
    ];

    let mut found_clauses = Vec::new();
    for &(kw, len) in &clauses {
        if let Some(idx) = find_keyword(sql, kw) {
            found_clauses.push((kw, idx, len));
        }
    }

    found_clauses.sort_by_key(|c| c.1);

    let from_end_idx = found_clauses.first().map(|c| c.1).unwrap_or(sql.len());
    let collection = sql[from_idx + 4..from_end_idx].trim().to_string();

    let mut conditions = Value::Object(Map::new());
    let mut order_by = None;
    let mut limit = None;
    let mut offset = None;

    for i in 0..found_clauses.len() {
        let (kw, start_idx, len) = found_clauses[i];
        let end_idx = if i + 1 < found_clauses.len() {
            found_clauses[i + 1].1
        } else {
            sql.len()
        };

        let slice = &sql[start_idx + len..end_idx];

        match kw {
            "WHERE" => {
                conditions = parse_where(slice)?;
            }
            "ORDER BY" => {
                order_by = Some(parse_order_by(slice)?);
            }
            "LIMIT" => {
                limit = Some(parse_limit(slice)?);
            }
            "OFFSET" => {
                offset = Some(parse_offset(slice)?);
            }
            _ => {}
        }
    }

    Ok(ParsedSqlQuery {
        fields,
        collection,
        conditions,
        order_by,
        limit,
        offset,
    })
}

pub fn execute_sql(i: &StoreInner, sql: &str) -> Result<Value, String> {
    let parsed = parse_sql_select(sql)?;
    let db_data = i.read().map_err(|e| e.to_string())?;
    
    let arr = match read_path(&db_data, &parsed.collection) {
        Some(Value::Array(a)) => a,
        _ => return Ok(Value::Array(vec![])),
    };

    // Construct Fluent Query JSON
    let mut query_obj = Map::new();

    // 1. Where filters
    if let Some(cond_map) = parsed.conditions.as_object() {
        let mut where_list = Vec::new();
        for (field, cond_val) in cond_map {
            if let Some(cond_inner) = cond_val.as_object() {
                let op = cond_inner.get("op").cloned().unwrap_or(json!("="));
                let value = cond_inner.get("value").cloned().unwrap_or(Value::Null);
                where_list.push(json!({
                    "field": field,
                    "op": op,
                    "value": value
                }));
            }
        }
        if !where_list.is_empty() {
            query_obj.insert("where".to_string(), Value::Array(where_list));
        }
    }

    // 2. Order by
    if let Some((field, is_desc)) = parsed.order_by {
        let mut order_obj = Map::new();
        order_obj.insert("field".to_string(), json!(field));
        order_obj.insert("direction".to_string(), json!(if is_desc { "desc" } else { "asc" }));
        query_obj.insert("order_by".to_string(), Value::Object(order_obj));
    }

    // 3. Limit
    if let Some(l) = parsed.limit {
        query_obj.insert("limit".to_string(), json!(l as u64));
    }

    // 4. Skip/Offset
    if let Some(o) = parsed.offset {
        query_obj.insert("skip".to_string(), json!(o as u64));
    }

    // 5. Select projection
    if !parsed.fields.is_empty() {
        let field_vals: Vec<Value> = parsed.fields.iter().map(|f| json!(f)).collect();
        query_obj.insert("select".to_string(), Value::Array(field_vals));
    }

    let query_val = Value::Object(query_obj);
    let results = execute_query(arr, &query_val);
    Ok(Value::Array(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let q = parse_sql_select("SELECT * FROM users").unwrap();
        assert_eq!(q.collection, "users");
        assert!(q.fields.is_empty());
        assert!(q.limit.is_none());
    }

    #[test]
    fn test_parse_complex() {
        let q = parse_sql_select("SELECT name, age FROM users WHERE age >= 25 AND role = 'admin' ORDER BY name DESC LIMIT 10 OFFSET 5").unwrap();
        assert_eq!(q.collection, "users");
        assert_eq!(q.fields, vec!["name", "age"]);
        assert_eq!(q.limit, Some(10));
        assert_eq!(q.offset, Some(5));
        assert_eq!(q.order_by, Some(("name".to_string(), true)));
    }
}
