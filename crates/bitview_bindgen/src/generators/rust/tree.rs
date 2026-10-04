//! Rust series tree: one generic struct per model shape, declared through a local `shape!` macro.

use std::fmt::Write;

use crate::{
    IndexSetPattern, ValueTypes, accessor_of, rust_field_name,
    model::{ChildKind, Model, TyExpr, param_name},
};

const RUNTIME: &str = r#"
/// A series value type, as a type parameter of the leaf accessors.
pub trait SeriesValue {
    /// The value where it can be missing (e.g. a period without blocks): `Option<Self>`, or the
    /// value itself when it is already optional.
    type Nullable;
}

impl<T> SeriesValue for Option<T> {
    type Nullable = Option<T>;
}

/// A series-tree node or leaf, built from its series name or base.
pub(crate) trait Node: Sized + Send + Sync + 'static {
    fn build(client: Arc<BitviewClientBase>, name: String) -> Self;
    #[cfg(test)]
    fn visit(&self, path: &str, f: &mut dyn FnMut(&str, &dyn AnySeriesPattern));
}

/// A child named by `template`: `*` stands for the parent's base, and with an empty base the `_`
/// joining it goes too.
fn child<T: Node>(client: &Arc<BitviewClientBase>, base: &Arc<str>, template: &'static str) -> LazyNode<T> {
    let (client, base) = (client.clone(), base.clone());
    LazyLock::new(Box::new(move || {
        let name = if !base.is_empty() {
            template.replacen('*', &base, 1)
        } else if let Some(rest) = template.strip_prefix("*_") {
            rest.to_owned()
        } else {
            template.replacen("_*", "", 1).replacen('*', "", 1)
        };
        Box::new(T::build(client, name))
    }))
}

/// A series-tree shape: a struct with one lazy field per child, documented with its canonical path.
macro_rules! shape {
    ($name:ident $(<$($p:ident),+>)? at $at:literal { $($field:ident: $ty:ty = $template:literal,)* }) => {
        #[doc = concat!("Series-tree node, e.g. at `", $at, "`.")]
        pub struct $name $(<$($p),+>)? { $(pub $field: LazyNode<$ty>,)* }
        impl $(<$($p: Send + Sync + 'static),+>)? Node for $name $(<$($p),+>)? where $($ty: Node,)* {
            fn build(client: Arc<BitviewClientBase>, base: String) -> Self {
                let base: Arc<str> = base.into();
                Self { $($field: child(&client, &base, $template),)* }
            }
            #[cfg(test)]
            fn visit(&self, path: &str, f: &mut dyn FnMut(&str, &dyn AnySeriesPattern)) {
                $(self.$field.visit(&format!("{path}{}{}", if path.is_empty() { "" } else { "." }, stringify!($field)), f);)*
            }
        }
    };
}
"#;

pub(crate) fn generate_tree(
    output: &mut String,
    model: &Model,
    names: &[String],
    accessors: &[IndexSetPattern],
    value_types: &ValueTypes,
) {
    output.push_str(RUNTIME);
    // Values that can be undefined are already optional; the others become optional where missing.
    for kind in &value_types.defined {
        writeln!(output, "impl SeriesValue for {kind} {{ type Nullable = Option<{kind}>; }}").unwrap();
    }
    // Shapes live in their own module: their names must not hide the types the crate re-exports.
    writeln!(output, "/// The series tree's node types, one generic struct per shape.").unwrap();
    writeln!(output, "pub mod tree {{
use super::*;
").unwrap();
    let ty = |expr: &TyExpr| {
        expr.render(names, ["<", ">"], &|kind| {
            if value_types.undefined.contains(kind) {
                format!("Option<{kind}>")
            } else {
                kind.to_owned()
            }
        })
    };
    let paths = model.shape_paths();
    for shape in model.shape_order() {
        let params = model.params[shape];
        let generic = if params == 0 {
            String::new()
        } else {
            let letters: Vec<String> = (0..params).map(param_name).collect();
            format!("<{}>", letters.join(", "))
        };
        let at: Vec<String> = paths[shape].iter().map(|key| rust_field_name(key)).collect();
        let at = ["series()".to_owned()].into_iter().chain(at).collect::<Vec<_>>().join(".");
        writeln!(output, "shape! {{ {}{generic} at {at:?} {{", names[shape]).unwrap();
        let shape_def = &model.shapes[shape];
        for (((key, kind), rule), expr) in shape_def
            .signature
            .0
            .iter()
            .zip(&shape_def.rules)
            .zip(&model.child_types[shape])
        {
            let child = match kind {
                ChildKind::Leaf(access) => {
                    format!("{}<{}>", accessors[accessor_of(accessors, access)].name, ty(expr))
                }
                ChildKind::Branch => ty(expr),
            };
            writeln!(output, "    {}: {child} = {:?},", rust_field_name(key), rule.template()).unwrap();
        }
        writeln!(output, "}} }}").unwrap();
    }
    writeln!(output, "}}\npub use tree::SeriesTree;").unwrap();
    writeln!(
        output,
        "fn create_series_tree(client: Arc<BitviewClientBase>) -> SeriesTree {{ SeriesTree::build(client, String::new()) }}"
    )
    .unwrap();
}
