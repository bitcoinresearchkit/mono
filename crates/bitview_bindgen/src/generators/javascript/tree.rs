//! JavaScript tree structure generation.

use std::collections::BTreeSet;
use std::fmt::Write;

use brk_types::TreeNode;

use crate::{
    ClientMetadata, Endpoint, GenericSyntax, JavaScriptSyntax, PatternField, build_child_path,
    generate_leaf_field, generate_tree_node_field, prepare_tree_node, to_camel_case,
};

use super::api::generate_api_methods;
use super::client::generate_static_constants;

/// Generate JSDoc typedefs for the series tree.
pub fn generate_tree_typedefs(output: &mut String, catalog: &TreeNode, metadata: &ClientMetadata) {
    writeln!(output, "// Catalog tree typedefs\n").unwrap();

    let pattern_lookup = metadata.pattern_lookup();
    let mut generated = BTreeSet::new();
    generate_tree_typedef(
        output,
        "SeriesTree",
        "",
        catalog,
        pattern_lookup,
        metadata,
        &mut generated,
    );
}

fn generate_tree_typedef(
    output: &mut String,
    name: &str,
    path: &str,
    node: &TreeNode,
    pattern_lookup: &std::collections::BTreeMap<Vec<PatternField>, String>,
    metadata: &ClientMetadata,
    generated: &mut BTreeSet<String>,
) {
    let Some(ctx) = prepare_tree_node(node, name, path, pattern_lookup, metadata, generated) else {
        return;
    };

    writeln!(output, "/**").unwrap();
    writeln!(output, " * @typedef {{Object}} {}", name).unwrap();

    for child in &ctx.children {
        let js_type = if child.should_inline {
            child.inline_type_name.clone()
        } else {
            metadata.field_type_annotation(&child.field, false, None, GenericSyntax::JAVASCRIPT)
        };

        writeln!(
            output,
            " * @property {{{}}} {}",
            js_type,
            to_camel_case(&child.field.name)
        )
        .unwrap();
    }

    writeln!(output, " */\n").unwrap();

    // Generate child typedefs
    for child in &ctx.children {
        if child.should_inline {
            let child_path = build_child_path(path, child.name);
            generate_tree_typedef(
                output,
                &child.inline_type_name,
                &child_path,
                child.node,
                pattern_lookup,
                metadata,
                generated,
            );
        }
    }
}

/// Generate the main BitviewClient class.
pub fn generate_main_client(
    output: &mut String,
    catalog: &TreeNode,
    metadata: &ClientMetadata,
    endpoints: &[Endpoint],
) {
    let pattern_lookup = metadata.pattern_lookup();

    writeln!(output, "/**").unwrap();
    writeln!(
        output,
        " * Main Bitview client with series tree and API methods"
    )
    .unwrap();
    writeln!(output, " * @extends BitviewClientBase").unwrap();
    writeln!(output, " */").unwrap();
    writeln!(output, "class BitviewClient extends BitviewClientBase {{").unwrap();

    generate_static_constants(output);

    writeln!(output, "  /**").unwrap();
    writeln!(
        output,
        "   * @param {{BitviewClientOptions|string}} options"
    )
    .unwrap();
    writeln!(output, "   */").unwrap();
    writeln!(output, "  constructor(options) {{").unwrap();
    writeln!(output, "    super(options);").unwrap();
    writeln!(output, "    /** @type {{SeriesTree}} */").unwrap();
    writeln!(output, "    this.series = this._buildTree();").unwrap();
    writeln!(output, "  }}\n").unwrap();

    output.push_str(r##"  /**
   * Compute the RapidHash v3 hash-prefix for raw address payload bytes.
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
   * @param {number} nibbles
   * @returns {string}
   */
  static addressPayloadHashPrefix(payload, nibbles) {
    return addressPayloadHashPrefix(payload, nibbles);
  }

  /**
   * Fetch address hash-prefix matches from raw address payload bytes.
   * @param {OutputType} addrType
   * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw payload bytes matching addrType length
   * @param {number} nibbles
   * @param {{ signal?: AbortSignal, onValue?: (value: AddrHashPrefixMatches) => void, cache?: boolean, memCache?: boolean }} [options]
   * @returns {Promise<AddrHashPrefixMatches>}
   */
  getAddressPayloadHashPrefixMatches(addrType, payload, nibbles, options = {}) {
    _validateAddressPayloadForType(addrType, payload);
    const prefix = addressPayloadHashPrefix(payload, nibbles);
    return this.getAddressHashPrefixMatches(addrType, prefix, options);
  }

"##);

    writeln!(output, "  /**").unwrap();
    writeln!(output, "   * @private").unwrap();
    writeln!(output, "   * @returns {{SeriesTree}}").unwrap();
    writeln!(output, "   */").unwrap();
    writeln!(output, "  _buildTree() {{").unwrap();
    writeln!(output, "    return {{").unwrap();
    let mut generated = BTreeSet::new();
    generate_tree_initializer(
        output,
        catalog,
        "SeriesTree",
        "",
        3,
        pattern_lookup,
        metadata,
        &mut generated,
    );
    writeln!(output, "    }};").unwrap();
    writeln!(output, "  }}\n").unwrap();

    writeln!(output, "  /**").unwrap();
    writeln!(
        output,
        "   * Create a dynamic series endpoint builder for any series/index combination."
    )
    .unwrap();
    writeln!(output, "   *").unwrap();
    writeln!(
        output,
        "   * Use this for programmatic access when the series name is determined at runtime."
    )
    .unwrap();
    writeln!(
        output,
        "   * For type-safe access, use the `series` tree instead."
    )
    .unwrap();
    writeln!(output, "   *").unwrap();
    writeln!(output, "   * @param {{string}} series - The series name").unwrap();
    writeln!(output, "   * @param {{Index}} index - The index name").unwrap();
    writeln!(output, "   * @returns {{SeriesEndpoint<unknown>}}").unwrap();
    writeln!(output, "   */").unwrap();
    writeln!(output, "  seriesEndpoint(series, index) {{").unwrap();
    writeln!(output, "    return _endpoint(this, series, index);").unwrap();
    writeln!(output, "  }}\n").unwrap();

    generate_api_methods(output, endpoints);

    writeln!(output, "}}\n").unwrap();

    writeln!(
        output,
        "export {{ BitviewClient, BitviewError, addressPayloadHashPrefix }};"
    )
    .unwrap();
}

#[allow(clippy::too_many_arguments)]
fn generate_tree_initializer(
    output: &mut String,
    node: &TreeNode,
    name: &str,
    path: &str,
    indent: usize,
    pattern_lookup: &std::collections::BTreeMap<Vec<PatternField>, String>,
    metadata: &ClientMetadata,
    generated: &mut BTreeSet<String>,
) {
    let indent_str = "  ".repeat(indent);

    let Some(ctx) = prepare_tree_node(node, name, path, pattern_lookup, metadata, generated) else {
        return;
    };

    let syntax = JavaScriptSyntax;
    for child in &ctx.children {
        let field_name = to_camel_case(child.name);

        if child.is_leaf {
            if let TreeNode::Leaf(leaf) = child.node {
                generate_leaf_field(
                    output,
                    &syntax,
                    "this",
                    child.name,
                    leaf,
                    metadata,
                    &indent_str,
                );
            }
        } else if child.should_inline {
            // Inline object
            let child_path = build_child_path(path, child.name);
            writeln!(output, "{}{}: {{", indent_str, field_name).unwrap();
            generate_tree_initializer(
                output,
                child.node,
                &child.inline_type_name,
                &child_path,
                indent + 1,
                pattern_lookup,
                metadata,
                generated,
            );
            writeln!(output, "{}}},", indent_str).unwrap();
        } else {
            generate_tree_node_field(
                output,
                &syntax,
                &child.field,
                metadata,
                &indent_str,
                "this",
                &child.base_result,
            );
        }
    }
}
