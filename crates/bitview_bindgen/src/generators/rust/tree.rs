//! Typed Rust records and exact catalog bindings; no name-pattern inference.

use std::fmt::Write;

use crate::{CatalogTree, CatalogType, CatalogValue, IndexSetPattern, rust_field_name};

pub(crate) fn generate_tree(output: &mut String, tree: &CatalogTree, indexes: &[IndexSetPattern]) {
    output.push_str(
        r#"
enum CatalogBinding {
    Leaf(&'static str),
    Branch(&'static [usize]),
}

trait FromCatalog: Sized + Send + Sync + 'static {
    fn from_catalog(client: Arc<BitviewClientBase>, binding: usize) -> Self;
}
"#,
    );

    for family in &tree.families {
        let parameters = (0..family.parameters)
            .map(|i| format!("T{i}"))
            .collect::<Vec<_>>();
        let generic = if parameters.is_empty() {
            String::new()
        } else {
            format!("<{}>", parameters.join(", "))
        };
        writeln!(output, "/// Catalog projection of {}.", family.source).unwrap();
        writeln!(output, "pub struct {}{generic} {{", family.name).unwrap();
        for (field, slot) in &family.fields {
            let field = rust_field_name(field);
            writeln!(output, "    pub {field}: LazyNode<T{slot}>,").unwrap();
        }
        writeln!(output, "}}\n").unwrap();
        let bounds = if parameters.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                parameters
                    .iter()
                    .map(|p| format!("{p}: FromCatalog"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        writeln!(
            output,
            "impl{bounds} FromCatalog for {}{generic} {{",
            family.name
        )
        .unwrap();
        output.push_str("    fn from_catalog(client: Arc<BitviewClientBase>, binding: usize) -> Self {\n        let CatalogBinding::Branch(children) = &CATALOG_BINDINGS[binding] else { unreachable!(\"expected catalog branch\") };\n        Self {\n");
        for (i, (field, slot)) in family.fields.iter().enumerate() {
            let field = rust_field_name(field);
            writeln!(output, "            {field}: lazy_node((client.clone(), children[{i}]), |(client, binding)| T{slot}::from_catalog(client, binding)),").unwrap();
        }
        output.push_str("        }\n    }\n}\n\n");
    }

    for accessor in indexes {
        writeln!(
            output,
            "impl<T: DeserializeOwned + Send + Sync + 'static> FromCatalog for {}<T> {{",
            accessor.name
        )
        .unwrap();
        output.push_str("    fn from_catalog(client: Arc<BitviewClientBase>, binding: usize) -> Self {\n        let CatalogBinding::Leaf(name) = &CATALOG_BINDINGS[binding] else { unreachable!(\"expected catalog leaf\") };\n        Self::new(client, (*name).to_string())\n    }\n}\n\n");
    }

    // Child type IDs precede their parents. Aliases keep deeply nested
    // instantiations compact without guessing generic relationships.
    for (id, ty) in tree.types.iter().enumerate() {
        let ty = match ty {
            CatalogType::Leaf { accessor, value } => {
                format!("{}<{value}>", indexes[*accessor].name)
            }
            CatalogType::Branch { family, arguments } => {
                let name = &tree.families[*family].name;
                if arguments.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{name}<{}>",
                        arguments
                            .iter()
                            .map(|id| format!("_CatalogType{id}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        };
        writeln!(output, "type _CatalogType{id} = {ty};").unwrap();
    }
    writeln!(
        output,
        "pub type SeriesTree = _CatalogType{};",
        tree.nodes[tree.root].type_id
    )
    .unwrap();
    writeln!(output, "fn create_series_tree(client: Arc<BitviewClientBase>) -> SeriesTree {{ FromCatalog::from_catalog(client, {}) }}", tree.root).unwrap();

    output.push_str("static CATALOG_BINDINGS: &[CatalogBinding] = &[\n");
    for node in &tree.nodes {
        match &node.value {
            CatalogValue::Leaf(name) => {
                writeln!(output, "    CatalogBinding::Leaf({name:?}),").unwrap()
            }
            CatalogValue::Branch(children) => {
                writeln!(output, "    CatalogBinding::Branch(&{children:?}),").unwrap()
            }
        }
    }
    output.push_str("];\n");
}
