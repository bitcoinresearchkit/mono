//! Rust type conversion utilities.

/// Convert JS-style type to Rust type.
pub fn js_type_to_rust(js_type: &str) -> String {
    if let Some(inner) = js_type.strip_suffix("[]") {
        format!("Vec<{}>", js_type_to_rust(inner))
    } else {
        match js_type {
            // API consumers read the version value; they do not need vecdb's
            // persisted-version arithmetic or file operations.
            "Version" => "u32".to_string(),
            "string" => "String".to_string(),
            "integer" => "i64".to_string(),
            "number" => "f64".to_string(),
            "boolean" => "bool".to_string(),
            "*" | "Object" => "serde_json::Value".to_string(),
            other => other.to_string(),
        }
    }
}
