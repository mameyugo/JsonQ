//! PHP INI configuration support (Temporarily disabled while fixing FFI/imports)

// use ext_php_rs::ini::Ini;
// use crate::config::Config;

/// Initialize configuration from php.ini
pub fn load_from_ini() {
    /*
    // Load max_file_size
    if let Some(size_str) = Ini::get::<String>("jsonq.max_file_size") {
        if let Ok(size) = Config::parse_size(&size_str) {
            Config::update(|cfg| {
                cfg.max_file_size = size;
            });
        }
    }
    
    // Load max_validation_depth
    if let Some(depth) = Ini::get::<i64>("jsonq.max_validation_depth") {
        if depth > 0 {
            Config::update(|cfg| {
                cfg.max_validation_depth = depth as usize;
            });
        }
    }
    
    // Load max_path_depth
    if let Some(depth) = Ini::get::<i64>("jsonq.max_path_depth") {
        if depth > 0 {
            Config::update(|cfg| {
                cfg.max_path_depth = depth as usize;
            });
        }
    }
    
    // Load allowed_extensions
    if let Some(exts_str) = Ini::get::<String>("jsonq.allowed_extensions") {
        let exts = Config::parse_extensions(&exts_str);
        if !exts.is_empty() {
            Config::update(|cfg| {
                cfg.allowed_extensions = exts;
            });
        }
    }
    */
}
