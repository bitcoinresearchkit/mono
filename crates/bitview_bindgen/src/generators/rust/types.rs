use crate::Ty;

pub(crate) fn rust_type(ty: &Ty) -> String {
    match ty {
        Ty::List(item) => format!("Vec<{}>", rust_type(item)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(rust_type).collect();
            format!("({})", items.join(", "))
        }
        // API consumers read the version value; they do not need vecdb's
        // persisted-version arithmetic or file operations.
        Ty::Ref(name) if name == "Version" => "u32".to_owned(),
        Ty::Ref(name) => name.clone(),
        Ty::String | Ty::Literals(_) => "String".to_owned(),
        Ty::Integer(format) => rust_integer(format.as_deref()),
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
        Ty::Integer(format) => rust_integer(format.as_deref()),
        Ty::Number => "f64".to_owned(),
        Ty::Boolean => "bool".to_owned(),
        _ => "&str".to_owned(),
    }
}

/// The Rust type of an integer `format`: `uint` is `usize`, `uintN` `uN` and `intN` `iN`;
/// without a format, `i64`. Panics on a format with no Rust type.
fn rust_integer(format: Option<&str>) -> String {
    let Some(format) = format else {
        return "i64".to_owned();
    };
    let (prefix, bits) = if let Some(bits) = format.strip_prefix("uint") {
        ("u", bits)
    } else if let Some(bits) = format.strip_prefix("int") {
        ("i", bits)
    } else {
        panic!("integer format `{format}` has no Rust type")
    };
    match bits {
        "" => format!("{prefix}size"),
        "8" | "16" | "32" | "64" | "128" => format!("{prefix}{bits}"),
        _ => panic!("integer format `{format}` has no Rust type"),
    }
}
