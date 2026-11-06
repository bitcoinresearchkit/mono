//! Linearization and chunking for topologically ordered dependency components.
//!
//! Up to [`EXACT_MAX`] members (Bitcoin Core's cluster limit), optimal: repeatedly take a
//! remaining subset that pays the most per weight unit among those that carry their own remaining
//! ancestors (a closure). That sequence is an optimal linearization (Core 31's cluster mempool
//! targets the same: the best feerate diagram), and its chunks follow: each group is a chunk,
//! never paying more than the one before. A closure's best feerate is found exactly: from the best
//! ancestor set's, Dinkelbach steps, each a maximum-weight closure (`fee * q - weight * p` at
//! feerate `p / q`) by a minimum cut, the cut's smallest source side. Among closures paying as
//! much, the smallest found inside that one is taken (equally good closures elsewhere can come
//! first: the diagram is the same). All of it in integers.
//!
//! Larger components (a block's, or a mempool's under an older node's or a raised limit) take the
//! best remaining ancestor set instead (ties: the larger set, then the earlier transaction), then
//! chunk that order: a group paying more than the chunk before it merges into it. Ancestor sets
//! are immutable bitsets, one row per transaction in a single flat buffer; taking a set subtracts
//! its fee and size from every remaining candidate that contains its members, O(n²) work and n²/8
//! bytes. Beyond [`ANCESTOR_MAX`] members (2 MiB of rows) that would let one component stall a
//! rebuild or exhaust memory, so the component's own topological order is chunked: O(n), optimal
//! for a chain (a block-filling one's shape), a fair order for the wide shallow ones a crafted
//! mempool cluster can take.
//!
//! Sizes are the caller's weights (sigop-adjusted for the mempool, as Core 31 linearizes): feerates
//! compare as exact fractions of them, and a chunk's published feerate is its fee over its weight's
//! exact quarter, as Core's chunks and mempool.space's effective feerates are.

use std::cmp::Ordering;

use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, Weight};

use crate::cluster::Node;

/// Members a component may have for the exact linearization: Bitcoin Core 31's cluster limit,
/// and one `u64` of members.
const EXACT_MAX: usize = 64;

/// Members a component may have for the ancestor-set heuristic.
const ANCESTOR_MAX: usize = 4_096;

/// Buffers reused across linearizations.
#[derive(Default)]
pub(crate) struct Scratch {
    /// The exact path's minimum-cut graph.
    flow: Flow,
    /// Row `i` (`ceil(n / 64)` words) is transaction `i`'s ancestor set, itself included.
    ancestors: Vec<u64>,
    /// Each candidate's remaining ancestor set totals.
    sets: Vec<Totals>,
    remaining: Vec<bool>,
    /// Transactions in selection order: every group, and so every chunk, is a contiguous run.
    order: Vec<CpfpClusterTxIndex>,
    chunks: Vec<Chunk>,
}

struct Totals {
    fee: Sats,
    weight: Weight,
    count: usize,
}

/// A chunk: `order[previous end..end]`, with its summed fee and size.
struct Chunk {
    end: usize,
    fee: Sats,
    weight: Weight,
}

/// `a`'s feerate against `b`'s, exactly: fee_a / weight_a vs fee_b / weight_b.
fn compare_rates(a: (Sats, Weight), b: (Sats, Weight)) -> Ordering {
    let (fee_a, weight_a) = (u128::from(u64::from(a.0)), u128::from(u64::from(a.1)));
    let (fee_b, weight_b) = (u128::from(u64::from(b.0)), u128::from(u64::from(b.1)));
    (fee_a * weight_b).cmp(&(fee_b * weight_a))
}

fn contains(row: &[u64], index: usize) -> bool {
    row[index / 64] & (1u64 << (index % 64)) != 0
}

/// The set bits of `row`, ascending.
fn members(row: &[u64]) -> impl Iterator<Item = usize> + '_ {
    row.iter().enumerate().flat_map(|(word_index, &word)| {
        let mut bits = word;
        std::iter::from_fn(move || {
            (bits != 0).then(|| {
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                word_index * 64 + bit
            })
        })
    })
}

fn ancestor_sets(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let n = nodes.len();
    let words = n.div_ceil(64);
    let Scratch {
        ancestors,
        sets,
        remaining,
        order,
        chunks,
        ..
    } = scratch;
    ancestors.clear();
    ancestors.resize(n * words, 0);
    sets.clear();
    remaining.clear();
    remaining.resize(n, true);
    order.clear();
    chunks.clear();

    for (index, node) in nodes.iter().enumerate() {
        let (earlier, row) = ancestors.split_at_mut(index * words);
        let row = &mut row[..words];
        row[index / 64] |= 1u64 << (index % 64);
        for &parent in &parents[node.parents.clone()] {
            let parent = u32::from(parent) as usize;
            for (word, parent_word) in row.iter_mut().zip(&earlier[parent * words..][..words]) {
                *word |= parent_word;
            }
        }
        let mut totals = Totals {
            fee: Sats::ZERO,
            weight: Weight::default(),
            count: 0,
        };
        for ancestor in members(row) {
            totals.fee += nodes[ancestor].fee;
            totals.weight += nodes[ancestor].weight;
            totals.count += 1;
        }
        sets.push(totals);
    }

    loop {
        let mut best: Option<usize> = None;
        for (index, set) in sets.iter().enumerate() {
            if !remaining[index] {
                continue;
            }
            if best.is_none_or(|best| {
                let best = &sets[best];
                compare_rates((set.fee, set.weight), (best.fee, best.weight))
                    .then(set.count.cmp(&best.count))
                    .is_gt()
            }) {
                best = Some(index);
            }
        }
        let Some(selected) = best else {
            break;
        };

        let group_start = order.len();
        for index in members(&ancestors[selected * words..][..words]) {
            if remaining[index] {
                remaining[index] = false;
                order.push(CpfpClusterTxIndex::from(index as u32));
            }
        }
        let group = &order[group_start..];

        // Each transaction is subtracted at most once from each candidate.
        for (index, set) in sets.iter_mut().enumerate() {
            if !remaining[index] {
                continue;
            }
            let row = &ancestors[index * words..][..words];
            for &removed in group {
                let removed = u32::from(removed) as usize;
                if contains(row, removed) {
                    set.fee -= nodes[removed].fee;
                    set.weight -= nodes[removed].weight;
                    set.count -= 1;
                }
            }
        }

        push_group(chunks, order.len(), sets[selected].fee, sets[selected].weight);
    }
    emit_chunks(order, chunks, emit);
}

/// The given topological order, each transaction a group, chunked.
fn topological(
    nodes: &[Node],
    scratch: &mut Scratch,
    emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let Scratch { order, chunks, .. } = scratch;
    order.clear();
    chunks.clear();
    for (index, node) in nodes.iter().enumerate() {
        order.push(CpfpClusterTxIndex::from(index as u32));
        push_group(chunks, index + 1, node.fee, node.weight);
    }
    emit_chunks(order, chunks, emit);
}

/// Appends the group ending at `end` of the order: one paying more than the chunk before it
/// joins that chunk, so chunk feerates never increase.
fn push_group(chunks: &mut Vec<Chunk>, end: usize, fee: Sats, weight: Weight) {
    let mut chunk = Chunk { end, fee, weight };
    while let Some(last) = chunks.last()
        && compare_rates((chunk.fee, chunk.weight), (last.fee, last.weight)).is_gt()
    {
        chunk.fee += last.fee;
        chunk.weight += last.weight;
        chunks.pop();
    }
    chunks.push(chunk);
}

/// Each chunk's members (ascending) and feerate, in chunk order.
fn emit_chunks(
    order: &mut [CpfpClusterTxIndex],
    chunks: &[Chunk],
    mut emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let mut start = 0;
    for chunk in chunks {
        let txs = &mut order[start..chunk.end];
        txs.sort_unstable_by_key(|index| u32::from(*index));
        emit(txs, FeeRate::from((chunk.fee, chunk.weight)));
        start = chunk.end;
    }
}

pub(crate) fn linearize(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    if nodes.len() <= EXACT_MAX {
        exact(nodes, parents, scratch, emit);
    } else if nodes.len() <= ANCESTOR_MAX {
        ancestor_sets(nodes, parents, scratch, emit);
    } else {
        topological(nodes, scratch, emit);
    }
}

/// A set of a component's members, bit `i` member `i`.
type Set = u64;

fn bit(index: usize) -> Set {
    1 << index
}

fn set_members(set: Set) -> impl Iterator<Item = usize> {
    let mut bits = set;
    std::iter::from_fn(move || {
        (bits != 0).then(|| {
            let index = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            index
        })
    })
}

fn totals(nodes: &[Node], set: Set) -> (Sats, Weight) {
    set_members(set).fold((Sats::ZERO, Weight::default()), |(fee, weight), index| {
        (fee + nodes[index].fee, weight + nodes[index].weight)
    })
}

fn exact(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    mut emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let n = nodes.len();
    let mut direct = [0 as Set; EXACT_MAX];
    let mut ancestors = [0 as Set; EXACT_MAX];
    for (index, node) in nodes.iter().enumerate() {
        ancestors[index] = bit(index);
        for &parent in &parents[node.parents.clone()] {
            let parent = u32::from(parent) as usize;
            direct[index] |= bit(parent);
            ancestors[index] |= ancestors[parent];
        }
    }
    let mut remaining: Set = if n == EXACT_MAX { Set::MAX } else { bit(n) - 1 };
    let mut txs = Vec::with_capacity(n);
    while remaining != 0 {
        // From the best remaining ancestor set (more members, then the earliest, on ties).
        let mut best = set_members(remaining)
            .map(|index| ancestors[index] & remaining)
            .max_by(|&a, &b| {
                compare_rates(totals(nodes, a), totals(nodes, b))
                    .then(a.count_ones().cmp(&b.count_ones()))
                    .then(b.trailing_zeros().cmp(&a.trailing_zeros()))
            })
            .expect("a remaining member");
        // Dinkelbach: a closure paying more than `best` raises the bar, until none does. Then
        // the smallest closure paying exactly as much is the group.
        loop {
            let (fee, weight) = totals(nodes, best);
            let (p, q) = (i128::from(u64::from(fee)), i128::from(u64::from(weight)));
            let weight = |index: usize| {
                i128::from(u64::from(nodes[index].fee)) * q
                    - i128::from(u64::from(nodes[index].weight)) * p
            };
            let (closure, value) = scratch.flow.max_closure(remaining, &direct, weight);
            if value > 0 {
                best = closure;
                continue;
            }
            // At the best feerate every best closure weighs zero: the smallest nonempty one (a
            // lone member is one).
            if best.count_ones() > 1
                && let Some(smaller) = scratch
                .flow
                .smallest_tight_closure(remaining, &direct, &ancestors, best, weight)
            {
                best = smaller;
            }
            break;
        }
        txs.clear();
        txs.extend(set_members(best).map(|index| CpfpClusterTxIndex::from(index as u32)));
        emit(&txs, FeeRate::from(totals(nodes, best)));
        remaining &= !best;
    }
}

/// A minimum-cut graph over a component's remaining members (source and sink last), reused.
#[derive(Default)]
struct Flow {
    /// Edge lists by node: each edge's index into `to`/`cap`, its reverse at `index ^ 1`.
    edges: Vec<Vec<usize>>,
    to: Vec<usize>,
    cap: Vec<i128>,
    level: Vec<u32>,
    next: Vec<usize>,
    queue: Vec<usize>,
    /// Remaining member `i`'s node, and back.
    node_of: Vec<usize>,
    member_of: Vec<usize>,
}

/// More than any finite cut: closure edges never cut.
const INFINITE: i128 = i128::MAX / 4;

impl Flow {
    /// The maximum-weight closure of `remaining` (a member's remaining parents with it) and its
    /// weight, its smallest one on ties: the source side of a minimum cut, as small as it goes.
    fn max_closure(
        &mut self,
        remaining: Set,
        direct: &[Set],
        weight: impl Fn(usize) -> i128,
    ) -> (Set, i128) {
        self.build(remaining, direct, &weight, None);
        let positive: i128 = set_members(remaining).map(&weight).filter(|&w| w > 0).sum();
        let cut = self.max_flow();
        (self.source_side(), positive - cut)
    }

    /// The smallest nonempty closure of `remaining` weighing zero (the best feerate's), if one is
    /// smaller than `best` (each member's smallest closure holding it, the smallest of them).
    fn smallest_tight_closure(
        &mut self,
        remaining: Set,
        direct: &[Set],
        ancestors: &[Set],
        best: Set,
        weight: impl Fn(usize) -> i128,
    ) -> Option<Set> {
        let mut smallest = best;
        for member in set_members(best) {
            // A closure holding `member` holds its remaining ancestors: none smaller than those.
            if (ancestors[member] & remaining).count_ones() >= smallest.count_ones() {
                continue;
            }
            self.build(remaining, direct, &weight, Some(member));
            let positive: i128 = set_members(remaining)
                .filter(|&index| index != member)
                .map(&weight)
                .filter(|&w| w > 0)
                .sum();
            let cut = self.max_flow();
            let closure = self.source_side();
            // The forced member's own weight is outside the cut: add it back.
            if positive - cut + weight(member) == 0
                && (closure.count_ones(), closure.trailing_zeros())
                    < (smallest.count_ones(), smallest.trailing_zeros())
            {
                smallest = closure;
            }
        }
        (smallest != best).then_some(smallest)
    }

    /// The cut graph of `remaining` under `weight`, `forced` tied to the source.
    fn build(
        &mut self,
        remaining: Set,
        direct: &[Set],
        weight: &impl Fn(usize) -> i128,
        forced: Option<usize>,
    ) {
        self.member_of.clear();
        self.member_of.extend(set_members(remaining));
        let members = self.member_of.len();
        let (source, sink) = (members, members + 1);
        self.node_of.clear();
        self.node_of.resize(EXACT_MAX, usize::MAX);
        for (node, &member) in self.member_of.iter().enumerate() {
            self.node_of[member] = node;
        }
        for list in &mut self.edges {
            list.clear();
        }
        self.edges.resize_with(members + 2, Vec::new);
        self.to.clear();
        self.cap.clear();
        for node in 0..members {
            let member = self.member_of[node];
            if forced == Some(member) {
                self.add(source, node, INFINITE);
            } else {
                match weight(member) {
                    w if w > 0 => self.add(source, node, w),
                    w if w < 0 => self.add(node, sink, -w),
                    _ => {}
                }
            }
            // Taking a member takes its parents.
            for parent in set_members(direct[member] & remaining) {
                let parent = self.node_of[parent];
                self.add(node, parent, INFINITE);
            }
        }
    }

    fn add(&mut self, from: usize, to: usize, cap: i128) {
        self.edges[from].push(self.to.len());
        self.to.push(to);
        self.cap.push(cap);
        self.edges[to].push(self.to.len());
        self.to.push(from);
        self.cap.push(0);
    }

    /// Dinic's maximum flow from the source to the sink.
    fn max_flow(&mut self) -> i128 {
        let nodes = self.edges.len();
        let (source, sink) = (nodes - 2, nodes - 1);
        let mut flow = 0;
        while self.levels(source, sink) {
            self.next.clear();
            self.next.resize(nodes, 0);
            loop {
                let pushed = self.push(source, sink, INFINITE);
                if pushed == 0 {
                    break;
                }
                flow += pushed;
            }
        }
        flow
    }

    /// Breadth-first levels over edges with capacity left; whether the sink is reached.
    fn levels(&mut self, source: usize, sink: usize) -> bool {
        self.level.clear();
        self.level.resize(self.edges.len(), u32::MAX);
        self.level[source] = 0;
        self.queue.clear();
        self.queue.push(source);
        let mut head = 0;
        while let Some(&node) = self.queue.get(head) {
            head += 1;
            for &edge in &self.edges[node] {
                let to = self.to[edge];
                if self.cap[edge] > 0 && self.level[to] == u32::MAX {
                    self.level[to] = self.level[node] + 1;
                    self.queue.push(to);
                }
            }
        }
        self.level[sink] != u32::MAX
    }

    /// One augmenting path's flow along increasing levels, at most `limit`.
    fn push(&mut self, node: usize, sink: usize, limit: i128) -> i128 {
        if node == sink {
            return limit;
        }
        while self.next[node] < self.edges[node].len() {
            let edge = self.edges[node][self.next[node]];
            let to = self.to[edge];
            if self.cap[edge] > 0 && self.level[to] == self.level[node] + 1 {
                let pushed = self.push(to, sink, limit.min(self.cap[edge]));
                if pushed > 0 {
                    self.cap[edge] -= pushed;
                    self.cap[edge ^ 1] += pushed;
                    return pushed;
                }
            }
            self.next[node] += 1;
        }
        0
    }

    /// The members the source still reaches: a minimum cut's smallest source side.
    fn source_side(&mut self) -> Set {
        let nodes = self.edges.len();
        let source = nodes - 2;
        self.levels(source, nodes - 1);
        (0..nodes - 2)
            .filter(|&node| self.level[node] != u32::MAX)
            .fold(0, |set, node| set | bit(self.member_of[node]))
    }
}
