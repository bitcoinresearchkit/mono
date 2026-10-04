//! The client model: every series-tree branch is an instance of a shape, and every child's series
//! name (a leaf) or base (a branch) derives from its parent's base by one [`Rule`]. Emitters render
//! each shape once and compose names at runtime instead of listing them.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use bitview_catalog::TreeNode;

use crate::{Access, to_pascal_case};

/// How a child's name derives from its parent's base. Names are `_`-separated token lists.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Rule {
    /// `prefix_base_suffix`, empty parts skipped (an empty base gives `prefix_suffix`).
    Affix(Vec<String>, Vec<String>),
    /// A fixed name, whatever the base.
    Literal(String),
}

impl Rule {
    /// The child's name with `*` standing for the parent's base, as clients compose it at runtime:
    /// `*` becomes the base, and with an empty base the `_` joining it goes too.
    pub(crate) fn template(&self) -> String {
        match self {
            Self::Affix(prefix, suffix) => [prefix.join("_"), "*".to_owned(), suffix.join("_")]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("_"),
            Self::Literal(name) => name.clone(),
        }
    }

    fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    fn affix_len(&self) -> usize {
        match self {
            Self::Affix(prefix, suffix) => prefix.len() + suffix.len(),
            Self::Literal(_) => 0,
        }
    }
}

/// A branch's structure without names: its ordered children, each a leaf with its index set or a
/// branch (whose shape is a type parameter, not part of this one).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Signature(pub(crate) Vec<(String, ChildKind)>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ChildKind {
    Leaf(Access),
    Branch,
}

/// A child's name from its template, exactly as the clients compose it: `*` becomes the base, and
/// with an empty base the `_` joining it goes too. (Names hold no `*`, so a template has at most one.)
fn compose(template: &str, base: &str) -> String {
    if !base.is_empty() {
        template.replacen('*', base, 1)
    } else if let Some(rest) = template.strip_prefix("*_") {
        rest.to_owned()
    } else {
        template.replacen("_*", "", 1).replacen('*', "", 1)
    }
}

/// A reusable branch structure: its signature and the naming rule of each child.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Shape {
    pub(crate) signature: Signature,
    pub(crate) rules: Vec<Rule>,
}

enum Body {
    Leaf,
    Branch { shape: usize, children: Vec<usize> },
}

struct Node {
    /// A leaf's series name, or a branch's base.
    name: String,
    body: Body,
    /// A leaf's value type, or a branch's shape with its type arguments.
    ty: Ty,
}

/// A concrete node type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Ty {
    Value(String),
    Shape(usize, Vec<Ty>),
}

/// A child's type inside a shape, in terms of the shape's parameters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum TyExpr {
    Param(usize),
    Value(String),
    Shape(usize, Vec<TyExpr>),
}

impl TyExpr {
    /// The type in a client's syntax: shapes by `names` with `brackets` around their arguments,
    /// leaf values through `value`, parameters as letters.
    pub(crate) fn render(
        &self,
        names: &[String],
        brackets: [&str; 2],
        value: &dyn Fn(&str) -> String,
    ) -> String {
        match self {
            Self::Param(p) => param_name(*p),
            Self::Value(kind) => value(kind),
            Self::Shape(shape, args) if args.is_empty() => names[*shape].clone(),
            Self::Shape(shape, args) => {
                let args: Vec<String> = args
                    .iter()
                    .map(|arg| arg.render(names, brackets, value))
                    .collect();
                format!(
                    "{}{}{}{}",
                    names[*shape],
                    brackets[0],
                    args.join(", "),
                    brackets[1]
                )
            }
        }
    }
}

/// The name of a shape's `p`-th type parameter.
pub(crate) fn param_name(p: usize) -> String {
    assert!(p < 26, "a shape has more type parameters than letters");
    char::from(b'A' + p as u8).to_string()
}

impl From<&Ty> for TyExpr {
    fn from(ty: &Ty) -> Self {
        match ty {
            Ty::Value(kind) => Self::Value(kind.clone()),
            Ty::Shape(shape, args) => Self::Shape(*shape, args.iter().map(Self::from).collect()),
        }
    }
}

pub(crate) struct Model {
    /// Children precede their parents.
    nodes: Vec<Node>,
    root: usize,
    pub(crate) shapes: Vec<Shape>,
    /// Each shape's parameter count.
    pub(crate) params: Vec<usize>,
    /// Each shape's child types, in its parameters.
    pub(crate) child_types: Vec<Vec<TyExpr>>,
    /// Each shape's parameters that stand for whole branch types (the others are leaf values),
    /// in order: emitters pass a builder for each of them.
    pub(crate) shape_params: Vec<Vec<usize>>,
}

/// A branch flattened before naming.
struct Flat {
    signature: Signature,
    children: Vec<usize>,
    height: usize,
}

impl Model {
    /// Builds the model and checks that composing every leaf's name from the root reproduces the
    /// catalog exactly.
    pub(crate) fn build(tree: &TreeNode) -> Self {
        let mut names: Vec<String> = Vec::new();
        let mut kinds: Vec<Option<String>> = Vec::new();
        let mut flats: Vec<Option<Flat>> = Vec::new();
        let root = flatten(tree, &mut names, &mut kinds, &mut flats);

        let max_height = flats.iter().flatten().map(|f| f.height).max().unwrap_or(0);
        let mut rules: Vec<Vec<Rule>> = vec![Vec::new(); flats.len()];
        for height in 1..=max_height {
            let mut groups: BTreeMap<&Signature, Vec<usize>> = BTreeMap::new();
            for (id, flat) in flats.iter().enumerate() {
                if let Some(flat) = flat
                    && flat.height == height
                    && id != root
                {
                    groups.entry(&flat.signature).or_default().push(id);
                }
            }
            for instances in groups.into_values() {
                let child_names: Vec<Vec<Vec<String>>> = instances
                    .iter()
                    .map(|&id| {
                        flats[id]
                            .as_ref()
                            .unwrap()
                            .children
                            .iter()
                            .map(|&c| tokens(&names[c]))
                            .collect()
                    })
                    .collect();
                for (&id, (base, chosen)) in instances.iter().zip(solve(&child_names)) {
                    names[id] = base;
                    rules[id] = chosen;
                }
            }
        }
        // The root's base is empty: its children are named outright.
        let root_flat = flats[root].as_ref().unwrap();
        rules[root] = root_flat
            .children
            .iter()
            .map(|&c| Rule::Literal(names[c].clone()))
            .collect();
        names[root] = String::new();

        // Shapes keyed by structure alone can contain themselves (an identical wrapper nested in
        // another): the shapes on such a cycle are keyed by height too, so every shape's children
        // come before it.
        let keys: Vec<Option<(Shape, usize)>> = flats
            .iter_mut()
            .zip(&mut rules)
            .map(|(flat, rules)| {
                flat.as_mut().map(|flat| {
                    let signature = std::mem::take(&mut flat.signature);
                    let rules = std::mem::take(rules);
                    (Shape { signature, rules }, flat.height)
                })
            })
            .collect();
        let mut split: HashSet<&Shape> = HashSet::new();
        let (shape_of, shapes) = loop {
            let mut ids: HashMap<(&Shape, usize), usize> = HashMap::new();
            let mut shapes: Vec<&Shape> = Vec::new();
            let shape_of: Vec<Option<usize>> = keys
                .iter()
                .map(|key| {
                    key.as_ref().map(|(shape, height)| {
                        let height = if split.contains(shape) { *height } else { 0 };
                        *ids.entry((shape, height)).or_insert_with(|| {
                            shapes.push(shape);
                            shapes.len() - 1
                        })
                    })
                })
                .collect();
            let edges: Vec<(usize, usize)> = flats
                .iter()
                .zip(&shape_of)
                .flat_map(|(flat, parent)| {
                    let children = flat.as_ref().map_or(&[][..], |flat| &flat.children[..]);
                    children
                        .iter()
                        .filter_map(|&child| Some((shape_of[child]?, (*parent)?)))
                })
                .collect();
            let (_, above) = topological(shapes.len(), edges.iter().copied());
            if above.is_empty() {
                break (shape_of, shapes.into_iter().cloned().collect::<Vec<_>>());
            }
            // Left out both ways: on a cycle (or between two), not merely above or below one.
            let (_, below) = topological(shapes.len(), edges.iter().map(|&(c, p)| (p, c)));
            let below: HashSet<usize> = below.into_iter().collect();
            split.extend(
                above
                    .into_iter()
                    .filter(|s| below.contains(s))
                    .map(|s| shapes[s]),
            );
        };

        let nodes = (0..flats.len())
            .map(|id| match (flats[id].take(), shape_of[id]) {
                (Some(flat), Some(shape)) => Node {
                    name: names[id].clone(),
                    body: Body::Branch {
                        shape,
                        children: flat.children,
                    },
                    ty: Ty::Shape(shape, Vec::new()),
                },
                _ => Node {
                    name: names[id].clone(),
                    ty: Ty::Value(kinds[id].take().unwrap()),
                    body: Body::Leaf,
                },
            })
            .collect();

        let mut model = Self {
            params: vec![0; shapes.len()],
            child_types: vec![Vec::new(); shapes.len()],
            shape_params: vec![Vec::new(); shapes.len()],
            nodes,
            root,
            shapes,
        };
        model.check(model.root, "");
        model.infer_types();
        model
    }

    /// Anti-unifies each shape's child types over all its instances, children's shapes first: a
    /// constant column is a concrete type, identical varying columns share one parameter, and a
    /// branch child with the same shape everywhere contributes one column per parameter of it.
    fn infer_types(&mut self) {
        let mut instances: Vec<Vec<usize>> = vec![Vec::new(); self.shapes.len()];
        for (id, node) in self.nodes.iter().enumerate() {
            if let Body::Branch { shape, .. } = node.body {
                instances[shape].push(id);
            }
        }
        for shape in self.shape_order() {
            let ids = &instances[shape];
            let mut columns: Vec<Vec<Ty>> = Vec::new();
            let column = |values: Vec<Ty>, columns: &mut Vec<Vec<Ty>>| -> TyExpr {
                if values.windows(2).all(|pair| pair[0] == pair[1]) {
                    return TyExpr::from(&values[0]);
                }
                let at = columns
                    .iter()
                    .position(|c| *c == values)
                    .unwrap_or_else(|| {
                        columns.push(values);
                        columns.len() - 1
                    });
                TyExpr::Param(at)
            };
            let width = self.shapes[shape].rules.len();
            let mut exprs = Vec::with_capacity(width);
            for position in 0..width {
                let types: Vec<&Ty> = ids
                    .iter()
                    .map(|&id| match &self.nodes[id].body {
                        Body::Branch { children, .. } => &self.nodes[children[position]].ty,
                        Body::Leaf => unreachable!(),
                    })
                    .collect();
                let expr = match types[0] {
                    Ty::Shape(child, args)
                        if types
                            .iter()
                            .all(|t| matches!(t, Ty::Shape(s, _) if s == child)) =>
                    {
                        let child = *child;
                        let args = (0..args.len())
                            .map(|p| {
                                let values = types
                                    .iter()
                                    .map(|t| match t {
                                        Ty::Shape(_, args) => args[p].clone(),
                                        Ty::Value(_) => unreachable!(),
                                    })
                                    .collect();
                                column(values, &mut columns)
                            })
                            .collect();
                        TyExpr::Shape(child, args)
                    }
                    _ => column(types.into_iter().cloned().collect(), &mut columns),
                };
                exprs.push(expr);
            }
            self.params[shape] = columns.len();
            self.child_types[shape] = exprs;
            self.shape_params[shape] = (0..columns.len())
                .filter(|&p| matches!(columns[p][0], Ty::Shape(..)))
                .collect();
            for (i, &id) in ids.iter().enumerate() {
                let args = columns.iter().map(|c| c[i].clone()).collect();
                self.nodes[id].ty = Ty::Shape(shape, args);
            }
        }
    }

    /// Each shape's canonical path: the keys to its shallowest (then smallest) instance.
    pub(crate) fn shape_paths(&self) -> Vec<Vec<String>> {
        let mut paths: Vec<Option<Vec<String>>> = vec![None; self.shapes.len()];
        let mut stack = vec![(self.root, Vec::new())];
        while let Some((id, path)) = stack.pop() {
            let Body::Branch { shape, children } = &self.nodes[id].body else {
                continue;
            };
            let known = &mut paths[*shape];
            if known
                .as_ref()
                .is_none_or(|known| (path.len(), &path) < (known.len(), known))
            {
                *known = Some(path.clone());
            }
            for ((key, _), &child) in self.shapes[*shape].signature.0.iter().zip(children) {
                let mut path = path.clone();
                path.push(key.clone());
                stack.push((child, path));
            }
        }
        paths.into_iter().map(Option::unwrap).collect()
    }

    /// Each shape's type name: `SeriesTree` for the root, otherwise the PascalCase tail of the
    /// shape's shallowest (then smallest) path, with as many trailing keys as it takes to be unique,
    /// stay clear of `reserved` and the type-parameter letters, and start with a letter. Names are
    /// unique and never reserved.
    pub(crate) fn shape_names(&self, reserved: &BTreeSet<String>) -> Vec<String> {
        let max_params = self.params.iter().copied().max().unwrap_or(0);
        let reserved: BTreeSet<String> = reserved
            .iter()
            .cloned()
            .chain((0..max_params).map(param_name))
            .collect();
        let paths = self.shape_paths();
        let root = match &self.nodes[self.root].body {
            Body::Branch { shape, .. } => *shape,
            Body::Leaf => unreachable!("the catalog root is a branch"),
        };
        let tail = |shape: usize, depth: usize| {
            let path = &paths[shape];
            to_pascal_case(&path[path.len() - depth.min(path.len())..].join("_"))
        };

        let mut depths = vec![1; self.shapes.len()];
        let owners = loop {
            let mut owners: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for shape in (0..self.shapes.len()).filter(|&s| s != root) {
                owners
                    .entry(tail(shape, depths[shape]))
                    .or_default()
                    .push(shape);
            }
            let mut deepened = false;
            for (name, shapes) in &owners {
                let clashes = shapes.len() > 1
                    || reserved.contains(name)
                    || name == "SeriesTree"
                    || !name.starts_with(|c: char| c.is_ascii_alphabetic());
                for &shape in shapes.iter().filter(|_| clashes) {
                    if depths[shape] < paths[shape].len() {
                        depths[shape] += 1;
                        deepened = true;
                    }
                }
            }
            if !deepened {
                break owners;
            }
        };

        // A name still shared after all keys (`a_b.c` and `a.b_c`) goes to the first path; the
        // others, and any name still unusable, take the first free numeric suffix.
        let mut names = vec![String::new(); self.shapes.len()];
        names[root] = "SeriesTree".to_owned();
        assert!(!reserved.contains(&names[root]), "`SeriesTree` is reserved");
        let mut taken: HashSet<String> = reserved.iter().cloned().collect();
        taken.insert(names[root].clone());
        let mut suffixed = Vec::new();
        for (name, mut shapes) in owners {
            shapes.sort_by(|&a, &b| paths[a].cmp(&paths[b]));
            let name = if name.starts_with(|c: char| c.is_ascii_alphabetic()) {
                name
            } else {
                format!("Node{name}")
            };
            for (rank, shape) in shapes.into_iter().enumerate() {
                if rank == 0 && taken.insert(name.clone()) {
                    names[shape] = name.clone();
                } else {
                    suffixed.push((name.clone(), shape));
                }
            }
        }
        for (name, shape) in suffixed {
            let name = (2..)
                .map(|n| format!("{name}{n}"))
                .find(|candidate| !taken.contains(candidate))
                .unwrap();
            taken.insert(name.clone());
            names[shape] = name;
        }
        names
    }

    /// Shapes ordered so every shape comes after the shapes of its branch children.
    pub(crate) fn shape_order(&self) -> Vec<usize> {
        let edges = self.nodes.iter().flat_map(|node| {
            let (shape, children) = match &node.body {
                Body::Branch { shape, children } => (Some(*shape), &children[..]),
                Body::Leaf => (None, &[][..]),
            };
            children
                .iter()
                .filter_map(move |&child| match self.nodes[child].body {
                    Body::Branch { shape: inner, .. } => Some((inner, shape?)),
                    Body::Leaf => None,
                })
        });
        let (order, stuck) = topological(self.shapes.len(), edges);
        assert!(stuck.is_empty(), "shape dependencies form a cycle");
        order
    }

    /// Composing names from `base` down reproduces every catalog name.
    fn check(&self, id: usize, base: &str) {
        let node = &self.nodes[id];
        assert_eq!(
            node.name, base,
            "bindgen naming model disagrees with the catalog"
        );
        if let Body::Branch { shape, children } = &node.body {
            for (rule, &child) in self.shapes[*shape].rules.iter().zip(children) {
                self.check(child, &compose(&rule.template(), base));
            }
        }
    }
}

/// Orders `count` shapes so each comes after the shapes it contains (`edges` are `(child, parent)`
/// pairs), and returns the shapes left out: those in a cycle and those above one.
fn topological(
    count: usize,
    edges: impl IntoIterator<Item = (usize, usize)>,
) -> (Vec<usize>, Vec<usize>) {
    let mut dependents: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    let mut pending: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    for (child, parent) in edges {
        dependents[child].insert(parent);
        pending[parent].insert(child);
    }
    let mut ready: Vec<usize> = (0..count).filter(|&s| pending[s].is_empty()).collect();
    let mut order = Vec::with_capacity(count);
    while let Some(shape) = ready.pop() {
        order.push(shape);
        for &dependent in &dependents[shape] {
            pending[dependent].remove(&shape);
            if pending[dependent].is_empty() {
                ready.push(dependent);
            }
        }
    }
    let stuck = (0..count).filter(|&s| !pending[s].is_empty()).collect();
    (order, stuck)
}

fn flatten(
    node: &TreeNode,
    names: &mut Vec<String>,
    kinds: &mut Vec<Option<String>>,
    flats: &mut Vec<Option<Flat>>,
) -> usize {
    match node {
        TreeNode::Leaf(leaf) => {
            let name = leaf.name();
            assert!(
                !name.starts_with('_')
                    && !name.ends_with('_')
                    && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "series names are ASCII letters, digits and inner `_`: {name:?}"
            );
            names.push(leaf.name().to_string());
            kinds.push(Some(leaf.kind().to_string()));
            flats.push(None);
        }
        TreeNode::Branch(branch) => {
            let mut signature = Vec::new();
            let mut children = Vec::new();
            let mut height = 0;
            for (key, child) in &branch.children {
                assert!(
                    !key.is_empty()
                        && key
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
                    "catalog keys are ASCII letters, digits, `_` and `-`: {key:?}"
                );
                let id = flatten(child, names, kinds, flats);
                let kind = match child {
                    TreeNode::Leaf(leaf) => ChildKind::Leaf(Access::of(leaf)),
                    TreeNode::Branch(_) => {
                        height = height.max(flats[id].as_ref().unwrap().height);
                        ChildKind::Branch
                    }
                };
                signature.push((key.clone(), kind));
                children.push(id);
            }
            names.push(String::new());
            kinds.push(None);
            flats.push(Some(Flat {
                signature: Signature(signature),
                children,
                height: height + 1,
            }));
        }
    }
    names.len() - 1
}

fn tokens(name: &str) -> Vec<String> {
    if name.is_empty() {
        Vec::new()
    } else {
        name.split('_').map(str::to_owned).collect()
    }
}

/// The rules of `base` for each child name: an affix around its first occurrence, else a literal.
fn rules_for(base: &[String], names: &[Vec<String>]) -> Vec<Rule> {
    names
        .iter()
        .map(|name| match find(name, base) {
            Some(at) => Rule::Affix(name[..at].to_vec(), name[at + base.len()..].to_vec()),
            None => Rule::Literal(name.join("_")),
        })
        .collect()
}

fn find(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// The base under which `names` follow `rules`, if any (empty when every rule is literal).
fn realize(names: &[Vec<String>], rules: &[Rule]) -> Option<String> {
    let mut base: Option<&[String]> = None;
    for (name, rule) in names.iter().zip(rules) {
        match rule {
            Rule::Literal(literal) => {
                if name.join("_") != *literal {
                    return None;
                }
            }
            Rule::Affix(prefix, suffix) => {
                if name.len() < prefix.len() + suffix.len()
                    || name[..prefix.len()] != prefix[..]
                    || name[name.len() - suffix.len()..] != suffix[..]
                {
                    return None;
                }
                let middle = &name[prefix.len()..name.len() - suffix.len()];
                match base {
                    None => base = Some(middle),
                    Some(base) if base == middle => {}
                    Some(_) => return None,
                }
            }
        }
    }
    Some(base.map(|base| base.join("_")).unwrap_or_default())
}

/// Picks, for every instance of one signature, a base and its rules so that as many instances as
/// possible share one rule vector: the vector proposed by the most instances first, preferring
/// fewer literals, then shorter affixes (a longer base).
fn solve(instances: &[Vec<Vec<String>>]) -> Vec<(String, Vec<Rule>)> {
    let candidates: Vec<BTreeSet<Vec<Rule>>> = instances
        .iter()
        .map(|names| {
            let mut bases: BTreeSet<&[String]> = BTreeSet::new();
            for name in names {
                for start in 0..name.len() {
                    for end in start + 1..=name.len() {
                        bases.insert(&name[start..end]);
                    }
                }
            }
            let mut rules: BTreeSet<Vec<Rule>> = bases
                .into_iter()
                .map(|base| rules_for(base, names))
                .collect();
            rules.insert(
                names
                    .iter()
                    .map(|name| Rule::Literal(name.join("_")))
                    .collect(),
            );
            rules
        })
        .collect();

    let mut result: Vec<Option<(String, Vec<Rule>)>> = vec![None; instances.len()];
    let mut remaining: Vec<usize> = (0..instances.len()).collect();
    while !remaining.is_empty() {
        let mut counts: HashMap<&Vec<Rule>, usize> = HashMap::new();
        for &i in &remaining {
            for rules in &candidates[i] {
                *counts.entry(rules).or_default() += 1;
            }
        }
        let best = counts
            .into_iter()
            .max_by(|(a, count_a), (b, count_b)| {
                let literals = |rules: &[Rule]| rules.iter().filter(|r| r.is_literal()).count();
                let affix = |rules: &[Rule]| rules.iter().map(Rule::affix_len).sum::<usize>();
                count_a
                    .cmp(count_b)
                    .then_with(|| literals(b).cmp(&literals(a)))
                    .then_with(|| affix(b).cmp(&affix(a)))
                    .then_with(|| b.cmp(a))
            })
            .map(|(rules, _)| rules.clone())
            .unwrap();
        remaining.retain(|&i| match realize(&instances[i], &best) {
            Some(base) => {
                result[i] = Some((base, best.clone()));
                false
            }
            None => true,
        });
    }
    result.into_iter().map(Option::unwrap).collect()
}
