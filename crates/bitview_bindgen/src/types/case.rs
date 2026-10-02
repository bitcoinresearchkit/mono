use brk_types::Index;

/// Convert a string to PascalCase (e.g., "fee_rate" -> "FeeRate").
pub(crate) fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for word in s.split(['-', '_']) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
        }
    }
    result
}

/// Convert a string to snake_case (no keyword escaping — backends handle that).
pub(crate) fn to_snake_case(s: &str) -> String {
    let mut sanitized = s.to_lowercase();
    if sanitized.contains('-') {
        sanitized = sanitized.replace('-', "_");
    }

    if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        sanitized.insert(0, '_');
    }
    sanitized
}

/// Escape Rust reserved keywords with `_` suffix (consistent with Python).
pub(crate) fn escape_rust_keyword(name: &str) -> String {
    match name {
        "type" | "const" | "static" | "match" | "if" | "else" | "loop" | "while" | "for"
        | "break" | "continue" | "return" | "fn" | "let" | "mut" | "ref" | "self" | "super"
        | "mod" | "use" | "pub" | "crate" | "extern" | "impl" | "trait" | "struct" | "enum"
        | "where" | "async" | "await" | "dyn" | "move" => format!("{}_", name),
        _ => name.to_string(),
    }
}

/// Convert a string to camelCase (e.g., "fee_rate" -> "feeRate").
pub(crate) fn to_camel_case(s: &str) -> String {
    let mut result = to_pascal_case(s);
    if let Some(first) = result.chars().next() {
        if first.is_ascii() {
            result[..1].make_ascii_lowercase();
        } else {
            result.replace_range(
                ..first.len_utf8(),
                &first.to_lowercase().collect::<String>(),
            );
        }
    }

    // Prefix with _ if starts with digit
    if result.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        result.insert(0, '_');
    }
    result
}

/// Convert an Index to a snake_case field name (e.g., Day1 -> day1).
pub(crate) fn index_to_field_name(index: &Index) -> String {
    to_snake_case(index.name())
}

/// Generate a child type/struct/class name (e.g., ParentName + child_name -> ParentName_ChildName).
pub(crate) fn child_type_name(parent: &str, child: &str) -> String {
    format!("{}_{}", parent, to_pascal_case(child))
}

/// Escape Python reserved keywords by appending an underscore.
/// Also prefixes names starting with digits with an underscore.
pub(crate) fn escape_python_keyword(name: &str) -> String {
    const PYTHON_KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ];

    // Strip characters invalid in identifiers (e.g. `[]` from `txId[]`)
    let mut name = name.replace(['[', ']'], "");

    // Prefix with underscore if starts with digit
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert(0, '_');
    }

    // Append underscore if it's a keyword
    if PYTHON_KEYWORDS.contains(&name.as_str()) {
        name.push('_');
    }
    name
}
