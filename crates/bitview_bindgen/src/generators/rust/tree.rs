//! Rust series tree: one generic struct per model shape, declared through `bitview_client`'s `shape!` macro.

use std::fmt::Write;

use crate::{
    IndexSetPattern, ValueTypes, accessor_of,
    model::{ChildKind, Model, TyExpr, param_name},
    rust_field_name,
};

pub(crate) fn generate_tree(
    output: &mut String,
    model: &Model,
    names: &[String],
    accessors: &[IndexSetPattern],
    value_types: &ValueTypes,
) {
    // Values that can be undefined are already optional; the others become optional where missing.
    for kind in &value_types.defined {
        writeln!(
            output,
            "impl SeriesValue for {kind} {{ type Nullable = Option<{kind}>; }}"
        )
        .unwrap();
    }
    // Shapes live in their own module: their names must not hide the types the crate re-exports.
    writeln!(
        output,
        "/// The series tree's node types, one generic struct per shape."
    )
    .unwrap();
    writeln!(
        output,
        "pub mod tree {{
use super::*;
"
    )
    .unwrap();
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
        let at: Vec<String> = paths[shape]
            .iter()
            .map(|key| rust_field_name(key))
            .collect();
        let at = ["series()".to_owned()]
            .into_iter()
            .chain(at)
            .collect::<Vec<_>>()
            .join(".");
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
                    format!(
                        "{}<{}>",
                        accessors[accessor_of(accessors, access)].name,
                        ty(expr)
                    )
                }
                ChildKind::Branch => ty(expr),
            };
            writeln!(
                output,
                "    {}: {child} = {:?},",
                rust_field_name(key),
                rule.template()
            )
            .unwrap();
        }
        writeln!(output, "}} }}").unwrap();
    }
    writeln!(output, "}}\npub use tree::SeriesTree;").unwrap();
}
