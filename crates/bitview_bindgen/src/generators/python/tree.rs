//! Python series tree: a generic class per model shape whose children are lazy attributes.

use std::{collections::BTreeSet, fmt::Write};

use crate::{
    IndexSetPattern, accessor_of, client_value_type, python_field_name,
    model::{ChildKind, Model, TyExpr, param_name},
};

const RUNTIME: &str = r#"class _Child:
    """A series-tree child, materialized on first access. Its series name (a leaf) or base (a node)
    is the template with `*` replaced by the parent's base, dropping the joining `_` when the base
    is empty. It is made from (client, name) by an accessor or node class, by the index of one of
    the parent's builders, or by a tuple applying a node class to builders."""

    def __init__(self, make: Any, template: str):
        self._make, self._template = make, template

    def __set_name__(self, owner: type, key: str) -> None:
        self._key = key

    def __get__(self, node: Any, owner: Any = None) -> Any:
        if node is None:
            return self
        base = node._b
        if base:
            name = self._template.replace('*', base)
        else:
            name = re.sub(r'^\*_|_?\*', '', self._template, count=1)
        value = _builder(node, self._make)(node._c, name)
        node.__dict__[self._key] = value
        return value


def _builder(node: _Node, make: Any) -> Callable[[BitviewClient, str], Any]:
    if isinstance(make, int):
        return node._f[make]
    if isinstance(make, tuple):
        cls, *args = make
        builders = [_builder(node, arg) for arg in args]
        return lambda c, b: cls(c, b, *builders)
    return make


def _at(make: Any, template: str) -> Any:
    return _Child(make, template)


class _Node:
    """A series-tree node over base `_b`, built with one builder per shape parameter."""

    def __init__(self, c: BitviewClient, b: str, *f: Callable[[BitviewClient, str], Any]):
        self._c, self._b, self._f = c, b, f

"#;

/// One class per shape, children's shapes first: a class refers to its children's classes when
/// it is defined.
pub(crate) fn generate_tree(
    output: &mut String,
    model: &Model,
    names: &[String],
    accessors: &[IndexSetPattern],
    undefined_types: &BTreeSet<String>,
) {
    writeln!(output, "# Series tree\n").unwrap();
    output.push_str(RUNTIME);
    let max_params = model.params.iter().copied().max().unwrap_or(0);
    for p in 0..max_params {
        let letter = param_name(p);
        writeln!(output, "{letter} = TypeVar('{letter}')").unwrap();
    }
    writeln!(output).unwrap();

    // A value type with an undefined value of its own (NaN, a sentinel) is `Optional`.
    let value = |kind: &str| {
        let value = client_value_type(kind, |element| format!("List[{element}]"));
        if undefined_types.contains(kind) { format!("Optional[{value}]") } else { value }
    };
    let ty = |expr: &TyExpr| expr.render(names, ["[", "]"], &value);
    for shape in model.shape_order() {
        let params = model.params[shape];
        if params == 0 {
            writeln!(output, "class {}(_Node):", names[shape]).unwrap();
        } else {
            let letters: Vec<String> = (0..params).map(param_name).collect();
            writeln!(output, "class {}(_Node, Generic[{}]):", names[shape], letters.join(", "))
                .unwrap();
        }
        let shape_def = &model.shapes[shape];
        for (((key, kind), rule), expr) in shape_def
            .signature
            .0
            .iter()
            .zip(&shape_def.rules)
            .zip(&model.child_types[shape])
        {
            let (annotation, make) = match kind {
                ChildKind::Leaf(access) => {
                    let accessor = &accessors[accessor_of(accessors, access)].name;
                    (format!("{accessor}[{}]", ty(expr)), accessor.clone())
                }
                ChildKind::Branch => (ty(expr), builder(model, names, shape, expr)),
            };
            writeln!(
                output,
                "    {}: {annotation} = _at({make}, '{}')",
                python_field_name(key),
                rule.template()
            )
            .unwrap();
        }
        if shape_def.rules.is_empty() {
            writeln!(output, "    pass").unwrap();
        }
        writeln!(output, "\n").unwrap();
    }
}

/// How a node builds its branch child typed `expr`: a class, the index of one of its own builders,
/// or a tuple applying a class to the builders of that class's shape parameters.
fn builder(model: &Model, names: &[String], shape: usize, expr: &TyExpr) -> String {
    match expr {
        TyExpr::Param(p) => {
            let at = model.shape_params[shape].iter().position(|q| q == p);
            at.expect("a branch child's parameter stands for a shape").to_string()
        }
        TyExpr::Shape(child, args) => {
            let child_params = &model.shape_params[*child];
            if child_params.is_empty() {
                return names[*child].clone();
            }
            let builders: Vec<String> =
                child_params.iter().map(|&p| builder(model, names, shape, &args[p])).collect();
            format!("({}, {})", names[*child], builders.join(", "))
        }
        TyExpr::Value(_) => unreachable!("a branch child is typed by a shape"),
    }
}
