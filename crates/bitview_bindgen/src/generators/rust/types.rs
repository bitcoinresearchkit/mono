use crate::Ty;

pub(crate) fn rust_type(ty: &Ty) -> String {
    match ty {
        Ty::List(item) => format!("Vec<{}>", rust_type(item)),
        // API consumers read the version value; they do not need vecdb's
        // persisted-version arithmetic or file operations.
        Ty::Ref(name) if name == "Version" => "u32".to_owned(),
        Ty::Ref(name) => name.clone(),
        Ty::String | Ty::Literals(_) => "String".to_owned(),
        Ty::Integer => "i64".to_owned(),
        Ty::Number => "f64".to_owned(),
        Ty::Boolean => "bool".to_owned(),
        _ => "serde_json::Value".to_owned(),
    }
}

/// A parameter's Rust type, borrowed where it can be.
pub(crate) fn rust_param_type(ty: &Ty) -> String {
    match ty {
        Ty::List(item) => format!("&[{}]", rust_param_type(item)),
        Ty::Ref(name) => name.clone(),
        Ty::Integer => "i64".to_owned(),
        Ty::Number => "f64".to_owned(),
        Ty::Boolean => "bool".to_owned(),
        _ => "&str".to_owned(),
    }
}
