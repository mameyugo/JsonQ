//! Basic SQL SELECT, INSERT, UPDATE, and DELETE query translator and execution engine
use serde_json::{json, Map, Value};
use crate::store::StoreInner;
use crate::path::{read_path, write_path};
use crate::query::execute_query;

#[derive(Debug)]
pub struct SqlQueryResult {
    pub value: Value,
    pub mutation: Option<SqlMutationInfo>,
}

#[derive(Debug)]
pub struct SqlMutationInfo {
    pub op: String,
    pub collection: String,
    pub old_value: Value,
    pub new_value: Value,
    pub existed: bool,
}

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

// Quote-aware splitting functions for INSERT and UPDATE
fn split_comma_separated_values(s: &str) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            current.push(c);
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            current.push(c);
        } else if c == ',' && !in_single_quote && !in_double_quote {
            values.push(parse_sql_value(current.trim())?);
            current.clear();
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        values.push(parse_sql_value(current.trim())?);
    }
    Ok(values)
}

fn parse_assignment(s: &str) -> Result<(String, Value), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid assignment format: '{}'", s));
    }
    let field = parts[0].trim().to_string();
    let val = parse_sql_value(parts[1].trim())?;
    Ok((field, val))
}

fn split_comma_separated_assignments(s: &str) -> Result<Vec<(String, Value)>, String> {
    let mut assignments = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            current.push(c);
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            current.push(c);
        } else if c == ',' && !in_single_quote && !in_double_quote {
            assignments.push(parse_assignment(current.trim())?);
            current.clear();
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        assignments.push(parse_assignment(current.trim())?);
    }
    Ok(assignments)
}

// Parsers for INSERT, UPDATE, DELETE
fn parse_insert(sql: &str) -> Result<(String, Vec<String>, Vec<Value>), String> {
    let sql = sql.trim().trim_end_matches(';');
    
    let insert_idx = find_keyword(sql, "INSERT INTO").ok_or_else(|| "Missing INSERT INTO".to_string())?;
    let values_idx = find_keyword(sql, "VALUES").ok_or_else(|| "Missing VALUES clause".to_string())?;
    
    if insert_idx != 0 {
        return Err("Query must start with INSERT INTO".to_string());
    }
    
    let table_fields_part = sql[insert_idx + 11..values_idx].trim();
    
    let (collection, fields) = if let Some(paren_idx) = table_fields_part.find('(') {
        let collection = table_fields_part[..paren_idx].trim().to_string();
        let fields_str = table_fields_part[paren_idx + 1..].trim();
        if !fields_str.ends_with(')') {
            return Err("Malformed fields list in INSERT statement".to_string());
        }
        let fields_inner = &fields_str[..fields_str.len() - 1];
        let fields: Vec<String> = fields_inner.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        (collection, fields)
    } else {
        (table_fields_part.to_string(), Vec::new())
    };
    
    if collection.is_empty() {
        return Err("Collection name cannot be empty".to_string());
    }
    
    if fields.is_empty() {
        return Err("Columns list is required for JSON INSERT statements".to_string());
    }
    
    let values_part = sql[values_idx + 6..].trim();
    if !values_part.starts_with('(') || !values_part.ends_with(')') {
        return Err("Malformed VALUES clause in INSERT statement".to_string());
    }
    let values_inner = &values_part[1..values_part.len() - 1];
    let values = split_comma_separated_values(values_inner)?;
    
    if fields.len() != values.len() {
        return Err(format!("Column count ({}) does not match value count ({})", fields.len(), values.len()));
    }
    
    Ok((collection, fields, values))
}

fn parse_update(sql: &str) -> Result<(String, Vec<(String, Value)>, Value), String> {
    let sql = sql.trim().trim_end_matches(';');
    
    let update_idx = find_keyword(sql, "UPDATE").ok_or_else(|| "Missing UPDATE".to_string())?;
    let set_idx = find_keyword(sql, "SET").ok_or_else(|| "Missing SET clause".to_string())?;
    
    if update_idx != 0 {
        return Err("Query must start with UPDATE".to_string());
    }
    
    let collection = sql[update_idx + 6..set_idx].trim().to_string();
    if collection.is_empty() {
        return Err("Collection name cannot be empty".to_string());
    }
    
    let where_idx = find_keyword(sql, "WHERE");
    
    let assignments_str = match where_idx {
        Some(w_idx) => &sql[set_idx + 3..w_idx],
        None => &sql[set_idx + 3..],
    }.trim();
    
    let assignments = split_comma_separated_assignments(assignments_str)?;
    if assignments.is_empty() {
        return Err("Missing assignments in SET clause".to_string());
    }
    
    let conditions = match where_idx {
        Some(w_idx) => {
            let where_str = &sql[w_idx + 5..];
            parse_where(where_str)?
        }
        None => Value::Object(Map::new()),
    };
    
    Ok((collection, assignments, conditions))
}

fn parse_delete(sql: &str) -> Result<(String, Value), String> {
    let sql = sql.trim().trim_end_matches(';');
    
    let delete_idx = find_keyword(sql, "DELETE FROM").ok_or_else(|| "Missing DELETE FROM".to_string())?;
    if delete_idx != 0 {
        return Err("Query must start with DELETE FROM".to_string());
    }
    
    let where_idx = find_keyword(sql, "WHERE");
    
    let collection = match where_idx {
        Some(w_idx) => sql[delete_idx + 11..w_idx].trim().to_string(),
        None => sql[delete_idx + 11..].trim().to_string(),
    };
    
    if collection.is_empty() {
        return Err("Collection name cannot be empty".to_string());
    }
    
    let conditions = match where_idx {
        Some(w_idx) => {
            let where_str = &sql[w_idx + 5..];
            parse_where(where_str)?
        }
        None => Value::Object(Map::new()),
    };
    
    Ok((collection, conditions))
}

// Executors for mutations
fn execute_insert(i: &StoreInner, sql: &str) -> Result<SqlQueryResult, String> {
    let (collection, fields, values) = parse_insert(sql)?;
    
    let mut map = Map::new();
    for (f, v) in fields.iter().zip(values.iter()) {
        map.insert(f.clone(), v.clone());
    }
    let new_record = Value::Object(map);
    
    let mut old_collection_val = Value::Array(vec![]);
    let mut new_collection_val = Value::Array(vec![]);
    let mut collection_existed = false;
    
    i.mutate(|data: &mut Value| {
        let current_arr = match read_path(data, &collection) {
            Some(Value::Array(a)) => {
                collection_existed = true;
                old_collection_val = Value::Array(a.clone());
                a.clone()
            }
            _ => vec![],
        };
        
        let mut new_arr = current_arr;
        new_arr.push(new_record.clone());
        new_collection_val = Value::Array(new_arr.clone());
        
        write_path(data, &collection, Value::Array(new_arr));
    })?;
    
    Ok(SqlQueryResult {
        value: json!(1), // 1 row affected
        mutation: Some(SqlMutationInfo {
            op: "set".to_string(),
            collection: collection,
            old_value: old_collection_val,
            new_value: new_collection_val,
            existed: collection_existed,
        }),
    })
}

fn execute_update(i: &StoreInner, sql: &str) -> Result<SqlQueryResult, String> {
    let (collection, assignments, conditions) = parse_update(sql)?;
    
    let mut where_list = Vec::new();
    if let Some(cond_map) = conditions.as_object() {
        for (field, cond_val) in cond_map {
            if let Some(cond_inner) = cond_val.as_object() {
                let op = cond_inner.get("op").cloned().unwrap_or_else(|| json!("="));
                let value = cond_inner.get("value").cloned().unwrap_or(Value::Null);
                where_list.push(json!({
                    "field": field,
                    "op": op,
                    "value": value
                }));
            }
        }
    }
    
    let mut old_collection_val = Value::Array(vec![]);
    let mut new_collection_val = Value::Array(vec![]);
    let mut collection_existed = false;
    let mut affected_rows = 0;
    
    i.mutate(|data: &mut Value| {
        let current_arr = match read_path(data, &collection) {
            Some(Value::Array(a)) => {
                collection_existed = true;
                old_collection_val = Value::Array(a.clone());
                a.clone()
            }
            _ => vec![],
        };
        
        let mut new_arr = Vec::new();
        for mut item in current_arr {
            let is_match = where_list.iter().all(|cond| {
                crate::query::fluent::check_condition(&item, cond)
            });
            if is_match {
                for (field, val) in &assignments {
                    crate::path::write_path(&mut item, field, val.clone());
                }
                affected_rows += 1;
            }
            new_arr.push(item);
        }
        
        new_collection_val = Value::Array(new_arr.clone());
        write_path(data, &collection, Value::Array(new_arr));
    })?;
    
    Ok(SqlQueryResult {
        value: json!(affected_rows),
        mutation: Some(SqlMutationInfo {
            op: "set".to_string(),
            collection: collection,
            old_value: old_collection_val,
            new_value: new_collection_val,
            existed: collection_existed,
        }),
    })
}

fn execute_delete(i: &StoreInner, sql: &str) -> Result<SqlQueryResult, String> {
    let (collection, conditions) = parse_delete(sql)?;
    
    let mut where_list = Vec::new();
    if let Some(cond_map) = conditions.as_object() {
        for (field, cond_val) in cond_map {
            if let Some(cond_inner) = cond_val.as_object() {
                let op = cond_inner.get("op").cloned().unwrap_or_else(|| json!("="));
                let value = cond_inner.get("value").cloned().unwrap_or(Value::Null);
                where_list.push(json!({
                    "field": field,
                    "op": op,
                    "value": value
                }));
            }
        }
    }
    
    let mut old_collection_val = Value::Array(vec![]);
    let mut new_collection_val = Value::Array(vec![]);
    let mut collection_existed = false;
    let mut affected_rows = 0;
    
    i.mutate(|data: &mut Value| {
        let current_arr = match read_path(data, &collection) {
            Some(Value::Array(a)) => {
                collection_existed = true;
                old_collection_val = Value::Array(a.clone());
                a.clone()
            }
            _ => vec![],
        };
        
        let mut new_arr = Vec::new();
        for item in current_arr {
            let is_match = where_list.iter().all(|cond| {
                crate::query::fluent::check_condition(&item, cond)
            });
            if is_match {
                affected_rows += 1;
            } else {
                new_arr.push(item);
            }
        }
        
        new_collection_val = Value::Array(new_arr.clone());
        write_path(data, &collection, Value::Array(new_arr));
    })?;
    
    Ok(SqlQueryResult {
        value: json!(affected_rows),
        mutation: Some(SqlMutationInfo {
            op: "set".to_string(),
            collection: collection,
            old_value: old_collection_val,
            new_value: new_collection_val,
            existed: collection_existed,
        }),
    })
}

pub fn execute_sql(i: &StoreInner, sql: &str) -> Result<SqlQueryResult, String> {
    let sql_trimmed = sql.trim();
    let sql_upper = sql_trimmed.to_uppercase();
    
    if sql_upper.starts_with("SELECT") {
        let parsed = parse_sql_select(sql_trimmed)?;
        let db_data = i.read().map_err(|e| e.to_string())?;
        
        let arr = match read_path(&db_data, &parsed.collection) {
            Some(Value::Array(a)) => a,
            _ => return Ok(SqlQueryResult {
                value: Value::Array(vec![]),
                mutation: None,
            }),
        };

        // Construct Fluent Query JSON
        let mut query_obj = Map::new();

        // 1. Where filters
        if let Some(cond_map) = parsed.conditions.as_object() {
            let mut where_list = Vec::new();
            for (field, cond_val) in cond_map {
                if let Some(cond_inner) = cond_val.as_object() {
                    let op = cond_inner.get("op").cloned().unwrap_or_else(|| json!("="));
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
        Ok(SqlQueryResult {
            value: Value::Array(results),
            mutation: None,
        })
    } else if sql_upper.starts_with("INSERT") {
        execute_insert(i, sql_trimmed)
    } else if sql_upper.starts_with("UPDATE") {
        execute_update(i, sql_trimmed)
    } else if sql_upper.starts_with("DELETE") {
        execute_delete(i, sql_trimmed)
    } else {
        Err("Unsupported SQL statement. Only SELECT, INSERT, UPDATE, and DELETE are supported.".to_string())
    }
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

    #[test]
    fn test_parse_insert() {
        let (collection, fields, values) = parse_insert("INSERT INTO users (name, age, active) VALUES ('Alice', 30, true)").unwrap();
        assert_eq!(collection, "users");
        assert_eq!(fields, vec!["name", "age", "active"]);
        assert_eq!(values, vec![json!("Alice"), json!(30), json!(true)]);
    }

    #[test]
    fn test_parse_update() {
        let (collection, assignments, conditions) = parse_update("UPDATE users SET age = 31, active = false WHERE id = 42").unwrap();
        assert_eq!(collection, "users");
        assert_eq!(assignments, vec![("age".to_string(), json!(31)), ("active".to_string(), json!(false))]);
        assert_eq!(conditions.get("id").unwrap().get("value").unwrap(), &json!(42));
    }

    #[test]
    fn test_parse_delete() {
        let (collection, conditions) = parse_delete("DELETE FROM users WHERE active = false").unwrap();
        assert_eq!(collection, "users");
        assert_eq!(conditions.get("active").unwrap().get("value").unwrap(), &json!(false));
    }
}
