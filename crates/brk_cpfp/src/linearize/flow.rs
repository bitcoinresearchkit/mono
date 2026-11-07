//! Maximum-weight closures by minimum cut, for the exact tier.
//!
//! A closure's weight at feerate `p / q` sums `fee * q - weight * p` over its members. The
//! heaviest closure is the source side of a minimum cut (Dinic's maximum flow). Among closures
//! weighing the same, the smallest is read off the last flow's residual graph (Picard and
//! Queyranne).

use super::{
    EXACT_MAX,
    exact::{Set, bit, set_members},
};

/// A minimum-cut graph over a component's remaining members (source and sink last), reused.
#[derive(Default)]
pub(super) struct Flow {
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
    pub(super) fn max_closure(
        &mut self,
        remaining: Set,
        direct: &[Set],
        weight: impl Fn(usize) -> i128,
    ) -> (Set, i128) {
        self.build(remaining, direct, &weight);
        let positive: i128 = set_members(remaining).map(&weight).filter(|&w| w > 0).sum();
        let cut = self.max_flow();
        (self.source_side(), positive - cut)
    }

    /// The smallest nonempty closure of `remaining` weighing zero (the best feerate's), from the
    /// residual graph of the last maximum flow, taken at that feerate (`best`, a closure weighing
    /// zero there, is a minimum cut's source side). Every minimum cut's source side is a set the
    /// residual graph never leaves (Picard and Queyranne): the smallest holding a member is what
    /// the source and that member reach. Each member's, the smallest of them (ties: the lowest
    /// member), one breadth-first search each instead of a flow.
    pub(super) fn smallest_tight_closure(&mut self, remaining: Set, ancestors: &[Set], best: Set) -> Set {
        let mut smallest = best;
        for member in set_members(best) {
            // A closure holding `member` holds its remaining ancestors: none smaller than those.
            if (ancestors[member] & remaining).count_ones() >= smallest.count_ones() {
                continue;
            }
            let closure = self.reach(self.node_of[member]);
            if (closure.count_ones(), closure.trailing_zeros())
                < (smallest.count_ones(), smallest.trailing_zeros())
            {
                smallest = closure;
            }
        }
        smallest
    }

    /// The members the source and `start` reach over edges with capacity left.
    fn reach(&mut self, start: usize) -> Set {
        let nodes = self.edges.len();
        let (source, sink) = (nodes - 2, nodes - 1);
        self.level.clear();
        self.level.resize(nodes, u32::MAX);
        self.queue.clear();
        for node in [source, start] {
            self.level[node] = 0;
            self.queue.push(node);
        }
        let mut head = 0;
        while let Some(&node) = self.queue.get(head) {
            head += 1;
            for &edge in &self.edges[node] {
                let to = self.to[edge];
                if self.cap[edge] > 0 && self.level[to] == u32::MAX {
                    self.level[to] = 0;
                    self.queue.push(to);
                }
            }
        }
        debug_assert_eq!(self.level[sink], u32::MAX, "a member of a minimum cut's source side");
        (0..nodes - 2)
            .filter(|&node| self.level[node] != u32::MAX)
            .fold(0, |set, node| set | bit(self.member_of[node]))
    }

    /// The cut graph of `remaining` under `weight`.
    fn build(&mut self, remaining: Set, direct: &[Set], weight: &impl Fn(usize) -> i128) {
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
            match weight(member) {
                w if w > 0 => self.add(source, node, w),
                w if w < 0 => self.add(node, sink, -w),
                _ => {}
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
