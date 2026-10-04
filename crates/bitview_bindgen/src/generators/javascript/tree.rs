//! JavaScript series tree: a JSDoc typedef and a builder per model shape, and the main client.

use std::{collections::BTreeSet, fmt::Write};

use super::{api::generate_api_methods, client::generate_static_constants};
use crate::{
    Endpoint, IndexSetPattern, accessor_of, client_value_type, js_field_name,
    model::{ChildKind, Model, TyExpr, param_name},
};

const RUNTIME: &str = r#"/**
 * Builds a series-tree node, or a leaf, from its base or series name.
 * @typedef {(c: BitviewClient, b: string, ...f: _Make[]) => any} _Make
 */

/**
 * A series-tree node over base `b`. Each child is `[make, template]`: its series name (a leaf, made
 * from its index list) or base (a node, made by its builder) is the template with `*` replaced by
 * `b`, dropping the joining `_` when `b` is empty. Children materialize on first access.
 * @param {BitviewClient} c
 * @param {string} b
 * @param {Record<string, [_Make | readonly Index[], string]>} children
 * @returns {any}
 */
function _n(c, b, children) {
  const node = {};
  for (const [key, [make, t]] of Object.entries(children)) {
    Object.defineProperty(node, key, {
      get() {
        const name = t.replace(b ? '*' : /^\*_|_?\*/, b);
        return _lazy(this, key, () => typeof make === 'function' ? make(c, name) : _mp(c, name, make));
      },
      enumerable: true,
      configurable: true,
    });
  }
  return node;
}

/**
 * The builder of a node whose children are fixed.
 * @param {Record<string, [_Make | readonly Index[], string]>} children
 * @returns {_Make}
 */
const _s = (children) => (c, b) => _n(c, b, children);

"#;

/// One typedef and one builder per shape, children's shapes first: a builder refers to the
/// builders of its fixed children when it is defined.
pub(crate) fn generate_tree(
    output: &mut String,
    model: &Model,
    names: &[String],
    accessors: &[IndexSetPattern],
    undefined_types: &BTreeSet<String>,
) {
    writeln!(output, "// Series tree\n").unwrap();
    output.push_str(RUNTIME);
    let tree = Tree {
        model,
        names,
        accessors,
        undefined_types,
    };
    for shape in model.shape_order() {
        tree.typedef(output, shape);
        tree.builder(output, shape);
    }
}

struct Tree<'a> {
    model: &'a Model,
    names: &'a [String],
    accessors: &'a [IndexSetPattern],
    undefined_types: &'a BTreeSet<String>,
}

impl Tree<'_> {
    fn typedef(&self, output: &mut String, shape: usize) {
        writeln!(output, "/**").unwrap();
        let params = self.model.params[shape];
        if params > 0 {
            let letters: Vec<String> = (0..params).map(param_name).collect();
            writeln!(output, " * @template {}", letters.join(", ")).unwrap();
        }
        writeln!(output, " * @typedef {{{{").unwrap();
        let signature = &self.model.shapes[shape].signature.0;
        for ((key, kind), expr) in signature.iter().zip(&self.model.child_types[shape]) {
            let ty = match kind {
                ChildKind::Leaf(access) => {
                    format!(
                        "{}<{}>",
                        self.accessors[accessor_of(self.accessors, access)].name,
                        self.ty(expr)
                    )
                }
                ChildKind::Branch => self.ty(expr),
            };
            writeln!(output, " *   {}: {ty},", js_field_name(key)).unwrap();
        }
        writeln!(output, " * }}}} {}\n */", self.names[shape]).unwrap();
    }

    fn builder(&self, output: &mut String, shape: usize) {
        let name = &self.names[shape];
        let shape_params = self.model.shape_params[shape].len();
        if shape_params == 0 {
            writeln!(output, "const _{name} = _s({{").unwrap();
        } else {
            let makers: Vec<String> = (0..shape_params).map(|i| format!("f{i}")).collect();
            writeln!(output, "/** @type {{_Make}} */").unwrap();
            writeln!(
                output,
                "const _{name} = (c, b, {}) => _n(c, b, {{",
                makers.join(", ")
            )
            .unwrap();
        }
        let shape_def = &self.model.shapes[shape];
        for (((key, kind), rule), expr) in shape_def
            .signature
            .0
            .iter()
            .zip(&shape_def.rules)
            .zip(&self.model.child_types[shape])
        {
            let make = match kind {
                ChildKind::Leaf(access) => format!("_i{}", accessor_of(self.accessors, access) + 1),
                ChildKind::Branch => self.make(shape, expr),
            };
            writeln!(
                output,
                "  {}: [{make}, '{}'],",
                js_field_name(key),
                rule.template()
            )
            .unwrap();
        }
        writeln!(output, "}});\n").unwrap();
    }

    /// A value type with an undefined value of its own (NaN, a sentinel) is nullable: `?Kind`.
    fn ty(&self, expr: &TyExpr) -> String {
        expr.render(self.names, ["<", ">"], &|kind| {
            let value = client_value_type(kind, |element| format!("{element}[]"));
            if self.undefined_types.contains(kind) {
                format!("?{value}")
            } else {
                value
            }
        })
    }

    /// The builder of a branch child typed `expr` inside `shape`, whose own builder receives one
    /// builder per shape parameter as `f0`, `f1`, ...
    fn make(&self, shape: usize, expr: &TyExpr) -> String {
        match expr {
            TyExpr::Param(p) => {
                let at = self.model.shape_params[shape].iter().position(|q| q == p);
                format!(
                    "f{}",
                    at.expect("a branch child's parameter stands for a shape")
                )
            }
            TyExpr::Shape(child, args) => {
                let name = &self.names[*child];
                let child_params = &self.model.shape_params[*child];
                if child_params.is_empty() {
                    return format!("_{name}");
                }
                let makers: Vec<String> = child_params
                    .iter()
                    .map(|&p| self.make(shape, &args[p]))
                    .collect();
                format!("(c, b) => _{name}(c, b, {})", makers.join(", "))
            }
            TyExpr::Value(_) => unreachable!("a branch child is typed by a shape"),
        }
    }
}

/// Generate the main BitviewClient class.
pub(crate) fn generate_main_client(output: &mut String, endpoints: &[Endpoint]) {
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
    writeln!(output, "  }}\n").unwrap();

    writeln!(output, "  /** @returns {{SeriesTree}} */").unwrap();
    writeln!(output, "  get series() {{").unwrap();
    writeln!(
        output,
        "    return _lazy(this, 'series', () => _SeriesTree(this, ''));"
    )
    .unwrap();
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
    writeln!(output, "   * @template {{Index}} I").unwrap();
    writeln!(output, "   * @param {{string}} series - The series name").unwrap();
    writeln!(
        output,
        "   * @param {{I}} index - The index name; date indexes also slice by Date"
    )
    .unwrap();
    writeln!(
        output,
        "   * @returns {{I extends DateIndex ? DateSeriesEndpoint<unknown> : SeriesEndpoint<unknown>}}"
    )
    .unwrap();
    writeln!(output, "   */").unwrap();
    writeln!(output, "  seriesEndpoint(series, index) {{").unwrap();
    writeln!(
        output,
        "    return /** @type {{any}} */ (_endpoint(this, series, index));"
    )
    .unwrap();
    writeln!(output, "  }}\n").unwrap();

    generate_api_methods(output, endpoints);

    writeln!(output, "}}\n").unwrap();

    writeln!(
        output,
        "export {{ BitviewClient, BitviewError, addressPayloadHashPrefix }};"
    )
    .unwrap();
}
