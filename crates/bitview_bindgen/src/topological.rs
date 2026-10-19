use std::collections::BTreeSet;

/// Orders `count` nodes so each comes after the nodes it depends on (`edges` are
/// `(dependency, dependent)` pairs), and returns the nodes left out: those on a cycle and those
/// depending on one.
pub(crate) fn topological(
    count: usize,
    edges: impl IntoIterator<Item = (usize, usize)>,
) -> (Vec<usize>, Vec<usize>) {
    let mut dependents: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    let mut pending: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    for (dependency, dependent) in edges {
        dependents[dependency].insert(dependent);
        pending[dependent].insert(dependency);
    }
    let mut ready: Vec<usize> = (0..count).filter(|&n| pending[n].is_empty()).collect();
    let mut order = Vec::with_capacity(count);
    while let Some(node) = ready.pop() {
        order.push(node);
        for &dependent in &dependents[node] {
            pending[dependent].remove(&node);
            if pending[dependent].is_empty() {
                ready.push(dependent);
            }
        }
    }
    let stuck = (0..count).filter(|&n| !pending[n].is_empty()).collect();
    (order, stuck)
}
