use std::sync::Arc;
use jsonq::store::StoreInner;
use serde_json::json;
use tempfile::TempDir;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <test_file>", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let content = std::fs::read_to_string(file_path).expect("Failed to read file");
    
    // Replace .write(&json!(...)) with .write(Arc::new(json!(...)))
    let mut result = content.clone();
    
    // Pattern 1: .write(&json!({...}))
    result = regex::Regex::new(r#"\.write\(&(json!\(\{[^}]+\}\))\)"#)
        .unwrap()
        .replace_all(&result, ".write(Arc::new($1))")
        .to_string();
    
    // Pattern 2: .write(&variable)
    result = regex::Regex::new(r#"\.write\(&([a-z_][a-z0-9_]*)\)"#)
        .unwrap()
        .replace_all(&result, ".write(Arc::new($1))")
        .to_string();
    
    // Add Arc import if not present
    if !result.contains("use std::sync::Arc") {
        result = format!("use std::sync::Arc;\n{}", result);
    }
    
    std::fs::write(file_path, result).expect("Failed to write file");
    println!("Fixed {}", file_path);
}
