//! Rust language syntax implementation.

use crate::{GenericSyntax, LanguageSyntax, escape_rust_keyword, to_snake_case};

/// Rust-specific code generation syntax.
pub struct RustSyntax;

/// Escape braces in a template string for use in `format!()`, preserving `{disc}`.
fn escape_rust_format(template: &str) -> String {
    template
        .replace('{', "{{")
        .replace('}', "}}")
        .replace("{{disc}}", "{disc}")
}

impl LanguageSyntax for RustSyntax {
    fn field_name(&self, name: &str) -> String {
        escape_rust_keyword(&to_snake_case(name))
    }

    fn path_expr(&self, base_var: &str, suffix: &str) -> String {
        format!("format!(\"{{{}}}{}\")", base_var, suffix)
    }

    fn suffix_expr(&self, acc_var: &str, relative: &str) -> String {
        if relative.is_empty() {
            self.owned_expr(acc_var)
        } else {
            format!("_m(&{}, \"{}\")", acc_var, relative)
        }
    }

    fn owned_expr(&self, var: &str) -> String {
        format!("{}.clone()", var)
    }

    fn prefix_expr(&self, prefix: &str, acc_var: &str) -> String {
        if prefix.is_empty() {
            self.owned_expr(acc_var)
        } else {
            let prefix_base = prefix.trim_end_matches('_');
            format!("_p(\"{}\", &{})", prefix_base, acc_var)
        }
    }

    fn constructor(&self, type_name: &str, path_expr: &str) -> String {
        format!("{}::new(client.clone(), {})", type_name, path_expr)
    }

    fn field_init(&self, indent: &str, name: &str, _type_ann: &str, value: &str) -> String {
        // Rust struct initialization; type is in struct definition, not in init
        format!("{}{}: {},", indent, name, value)
    }

    fn generic_syntax(&self) -> GenericSyntax {
        GenericSyntax::RUST
    }

    fn string_literal(&self, value: &str) -> String {
        format!("\"{}\".to_string()", value)
    }

    fn constructor_name(&self, type_name: &str) -> String {
        format!("{}::new", type_name)
    }

    fn disc_arg_expr(&self, template: &str) -> String {
        if template == "{disc}" {
            "disc.clone()".to_string()
        } else if template.is_empty() {
            "String::new()".to_string()
        } else if !template.contains("{disc}") {
            format!("\"{}\".to_string()", template)
        } else if template.ends_with("{disc}") {
            let static_part = template.trim_end_matches("{disc}").trim_end_matches('_');
            format!("_m(\"{}\", &disc)", static_part)
        } else {
            format!("format!(\"{}\")", escape_rust_format(template))
        }
    }

    fn template_expr(&self, acc_var: &str, template: &str) -> String {
        if template == "{disc}" {
            format!("_m(&{}, &disc)", acc_var)
        } else if template.is_empty() {
            acc_var.to_string()
        } else if !template.contains("{disc}") {
            format!("_m(&{}, \"{}\")", acc_var, template)
        } else {
            format!(
                "_m(&{}, &format!(\"{}\", disc=disc))",
                acc_var,
                escape_rust_format(template)
            )
        }
    }
}
