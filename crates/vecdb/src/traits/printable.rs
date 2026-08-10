use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Extracts the short type name from a full type path and caches it.
pub fn short_type_name<T: 'static>() -> &'static str {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, &'static str>>> = OnceLock::new();

    let full: &'static str = std::any::type_name::<T>();

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();

    if let Some(&short) = guard.get(full) {
        return short;
    }

    let short_owned = shorten_type_name(full);

    let short: &'static str = Box::leak(short_owned.into_boxed_str());
    guard.insert(full, short);
    short
}

/// Shortens a type name by removing path prefixes, including inside generics.
/// `some::module::Close<another::path::Dollars>` -> `Close<Dollars>`
///
/// Also unwraps `Option<T>` to just `T` since Option is a serialization
/// concern (null in JSON) not a type identity.
fn shorten_type_name(full: &str) -> String {
    if let Some(inner) = full
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        && let Some((element, len)) = inner.rsplit_once(';')
    {
        return format!("[{}; {}]", shorten_type_name(element.trim()), len.trim());
    }

    let generic_start = full.find('<');

    let (base, generics) = match generic_start {
        Some(pos) => (&full[..pos], Some(&full[pos + 1..full.len() - 1])),
        None => (full, None),
    };

    let short_base = base.rsplit("::").next().unwrap_or(base);

    // Option is a serialization concern (null in JSON), not a type identity.
    if short_base == "Option"
        && let Some(inner) = generics
    {
        return shorten_type_name(inner.trim());
    }

    match generics {
        Some(params) => {
            let shortened_params = split_generic_params(params)
                .into_iter()
                .map(|p| shorten_type_name(p.trim()))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", short_base, shortened_params)
        }
        None => short_base.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::shorten_type_name;

    #[test]
    fn shortens_array_element_paths_without_dropping_brackets() {
        assert_eq!(
            shorten_type_name("[some::module::Value; 23]"),
            "[Value; 23]"
        );
    }
}

/// Split generic parameters respecting nested angle brackets.
/// `A, B<C, D>, E` -> ["A", "B<C, D>", "E"]
fn split_generic_params(params: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in params.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&params[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&params[start..]);
    result
}

/// Provides string representations of index types for display and region naming.
pub trait PrintableIndex {
    /// Returns the canonical string name for this index type.
    fn to_string() -> &'static str;

    /// Returns all accepted string representations for this index type.
    /// Used for parsing and type identification.
    fn to_possible_strings() -> &'static [&'static str];
}

impl PrintableIndex for usize {
    fn to_string() -> &'static str {
        "usize"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["usize"]
    }
}
