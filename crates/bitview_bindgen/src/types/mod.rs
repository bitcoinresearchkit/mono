//! Core types for client generation.

mod case;
mod index_set_pattern;
mod metadata;
mod pattern_field;
mod positions;
mod schema;
mod structural_pattern;

pub(crate) use case::*;
pub use index_set_pattern::*;
pub use metadata::*;
pub use pattern_field::*;
pub use positions::*;
pub(crate) use schema::*;
pub use structural_pattern::*;

/// Language-specific syntax for generic type annotations.
#[derive(Clone, Copy)]
pub(crate) struct GenericSyntax {
    open: char,
    close: char,
    default_type: &'static str,
}

impl GenericSyntax {
    pub(crate) const PYTHON: Self = Self {
        open: '[',
        close: ']',
        default_type: "Any",
    };
    pub(crate) const JAVASCRIPT: Self = Self {
        open: '<',
        close: '>',
        default_type: "unknown",
    };

    fn wrap(&self, name: &str, type_param: &str) -> String {
        // Convert the type_param from Rust syntax to target syntax
        let converted = self.convert(type_param);
        format!("{}{}{}{}", name, self.open, converted, self.close)
    }

    /// Convert a type string from Rust generic syntax to target language syntax.
    ///
    /// For Python, wrapper newtypes like `Close<Cents>` are flattened to just `Cents`
    /// because Python type aliases can't be parameterized. This matches JS behavior.
    fn convert(&self, type_str: &str) -> String {
        // Flatten nested generics to innermost type (e.g., Close<Cents> -> Cents)
        // This is needed because wrapper types like Close, Open, High, Low are
        // just type aliases in generated code, not actual generic classes.
        let converted = extract_inner_type_recursive(type_str);

        let Some(element) = rust_array_element_type(converted) else {
            return converted.to_owned();
        };

        match self.default_type {
            "Any" => format!("List[{}]", self.convert(element)),
            "unknown" => format!("{}[]", self.convert(element)),
            _ => converted.to_owned(),
        }
    }
}

/// Extract the innermost type from nested generics.
/// E.g., `Close<Cents>` -> `Cents`, `Foo<Bar<Baz>>` -> `Baz`
fn extract_inner_type_recursive(type_str: &str) -> &str {
    if let Some(start) = type_str.find('<')
        && let Some(end) = type_str.rfind('>')
        && start < end
    {
        let inner = &type_str[start + 1..end];
        return extract_inner_type_recursive(inner);
    }
    type_str
}
