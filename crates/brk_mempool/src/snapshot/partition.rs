//! Projected blocks 1..N, packed as Core's block assembler packs, from base fees within Core's
//! default budgets (an operator's `prioritisetransaction` deltas and mining options shape only
//! block 0): block 0 is Core's `getblocktemplate` (the caller's); the rest take whole chunks by
//! descending feerate, in dependency order, each block within Core's weight and sigop budgets.
//! Components that block 0 took part of are linearized again without those members. A chunk that
//! doesn't fit waits for the next block, and so does the rest of its component (Core's block
//! builder skips the cluster); a block closes once 1,000 chunks in a row failed with it within
//! 4,000 WU of full (Core's `BlockAssembler`). As Core's, a candidate is tested at its
//! sigop-adjusted weight and the block counts its actual weight. A chunk no block could hold waits
//! for the last, which takes whatever is left (mempool.space's catch-all, then more than a block);
//! trailing empty blocks are not made. The budgets are Core's defaults (`-blockreservedweight`,
//! `-bytespersigop`).

use std::mem;

use brk_cpfp::Cluster;
use brk_types::{FeeRate, Sats, SigOps, Txid, Weight};

use super::{SnapTx, TxIndex, cluster};

/// A block's transactions stay under 3,992,000 WU (Core's 4,000,000 less its 8,000 reserved for
/// the header and coinbase) and 79,600 sigops (80,000 less 400).
const MAX_WEIGHT: Weight = Weight::new(3_992_000);
const MAX_SIGOPS: SigOps = SigOps::new(79_600);
const FULL_ENOUGH: Weight = Weight::new(3_988_000);
const MAX_CONSECUTIVE_FAILURES: u32 = 1_000;

/// Every component's chunks in linearization order, components one after another.
#[derive(Default)]
pub struct Chunks {
    /// Members, chunk after chunk, each chunk's in topological order.
    txs: Vec<TxIndex>,
    /// Each chunk's end in `txs` and its feerate.
    chunks: Vec<(u32, FeeRate)>,
    /// Each component's end in `chunks`.
    components: Vec<u32>,
}

impl Chunks {
    pub fn push_chunk(&mut self, members: impl IntoIterator<Item = TxIndex>, rate: FeeRate) {
        self.txs.extend(members);
        self.chunks.push((self.txs.len() as u32, rate));
    }

    /// Closes the component whose chunks were pushed since the last one closed.
    pub fn end_component(&mut self) {
        debug_assert!(self.components.last().copied().unwrap_or(0) < self.chunks.len() as u32);
        self.components.push(self.chunks.len() as u32);
    }

    /// Chunk `c`'s members.
    fn members(&self, c: usize) -> &[TxIndex] {
        let start = c.checked_sub(1).map_or(0, |p| self.chunks[p].0 as usize);
        &self.txs[start..self.chunks[c].0 as usize]
    }

    /// Each component's chunk range.
    fn components(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let mut start = 0;
        self.components.iter().map(move |&end| {
            let range = (start, end as usize);
            start = end as usize;
            range
        })
    }

    /// The members of chunks `from..to`.
    fn span(&self, (from, to): (usize, usize)) -> &[TxIndex] {
        let start = from.checked_sub(1).map_or(0, |p| self.chunks[p].0 as usize);
        &self.txs[start..self.chunks[to - 1].0 as usize]
    }
}

/// Blocks after block 0 (the `excluded` members), each member with the feerate it was packed at.
pub struct Projection {
    pub blocks: Vec<Vec<(TxIndex, FeeRate)>>,
    /// Whether the last block holds more than a block could: what was left past the others.
    pub overflow: bool,
}

pub fn partition(
    txs: &[SnapTx],
    chunks: &Chunks,
    excluded: &[bool],
    num_blocks: usize,
) -> Projection {
    debug_assert!(num_blocks > 0);
    debug_assert_eq!(txs.len(), excluded.len());
    let mut left = Chunks::default();
    let mut graph = Cluster::default();
    let mut rest: Vec<TxIndex> = Vec::new();
    for component in chunks.components() {
        let members = chunks.span(component);
        let taken = members.iter().filter(|m| excluded[m.as_usize()]).count();
        if taken == 0 {
            for c in component.0..component.1 {
                left.push_chunk(chunks.members(c).iter().copied(), chunks.chunks[c].1);
            }
        } else if taken < members.len() {
            rest.clear();
            rest.extend(members.iter().filter(|m| !excluded[m.as_usize()]));
            rest.sort_unstable_by(|&a, &b| txs[a.as_usize()].txid.cmp(&txs[b.as_usize()].txid));
            let order = cluster::load(txs, &rest, &mut graph);
            graph.linearize(|local, rate| {
                let members = local.iter().map(|&l| order[u32::from(l) as usize]);
                left.push_chunk(members, rate);
            });
        } else {
            continue;
        }
        left.end_component();
    }
    let projection = pack(txs, &left, num_blocks);
    #[cfg(debug_assertions)]
    check_order(txs, excluded, &projection.blocks);
    projection
}

struct Candidate {
    chunk: u32,
    component: u32,
    /// The published feerate; the order compares `fee / policy` exactly, as the linearization
    /// did, so a component's chunks never reorder.
    rate: FeeRate,
    fee: Sats,
    /// Sigop-adjusted weight.
    policy: Weight,
    /// The component's first member: ties between components break by it, the same on every node.
    key: Txid,
    /// Actual weight, what a block counts (a candidate is tested at `policy`, as Core's).
    weight: Weight,
    sigops: SigOps,
}

impl Candidate {
    /// Whether it fits a block holding `weight` and `sigops` of transactions.
    fn fits(&self, weight: Weight, sigops: SigOps) -> bool {
        weight + self.policy < MAX_WEIGHT && sigops + self.sigops < MAX_SIGOPS
    }
}

fn pack(txs: &[SnapTx], chunks: &Chunks, num_blocks: usize) -> Projection {
    let mut cands: Vec<Candidate> = Vec::with_capacity(chunks.chunks.len());
    for (component, range) in chunks.components().enumerate() {
        let key = txs[chunks.span(range)[0].as_usize()].txid;
        for c in range.0..range.1 {
            let members = || chunks.members(c).iter().map(|m| &txs[m.as_usize()]);
            cands.push(Candidate {
                chunk: c as u32,
                component: component as u32,
                rate: chunks.chunks[c].1,
                fee: members().map(|t| t.fee).sum(),
                policy: members().map(|t| t.sigops.adjust_weight(t.weight)).sum(),
                key,
                weight: members().map(|t| t.weight).sum(),
                sigops: members().map(|t| t.sigops).sum(),
            });
        }
    }
    // A component's chunks keep their order: their feerates never increase, ties go by chunk.
    let per_weight = |c: &Candidate| {
        (
            u128::from(u64::from(c.fee)),
            u128::from(u64::from(c.policy)),
        )
    };
    cands.sort_unstable_by(|a, b| {
        let ((fee_a, weight_a), (fee_b, weight_b)) = (per_weight(a), per_weight(b));
        (fee_b * weight_a)
            .cmp(&(fee_a * weight_b))
            .then_with(|| a.key.cmp(&b.key))
            .then(a.chunk.cmp(&b.chunk))
    });

    let mut blocks: Vec<Vec<(TxIndex, FeeRate)>> = Vec::new();
    // The pass a component last missed a chunk in, plus one: its later chunks wait for the next.
    let mut missed: Vec<u32> = vec![0; chunks.components.len()];
    let mut pass = 0;
    let mut pending: Vec<usize> = (0..cands.len()).collect();
    let mut deferred: Vec<usize> = Vec::new();
    // Set once nothing left fits a block: what's left goes to the last.
    let mut stuck = false;
    let mut overflow = false;
    while !pending.is_empty() {
        let last = stuck || blocks.len() + 1 == num_blocks;
        pass += 1;
        let mut block: Vec<(TxIndex, FeeRate)> = Vec::new();
        let (mut weight, mut sigops) = (Weight::default(), SigOps::ZERO);
        let mut failures = 0;
        for (i, &c) in pending.iter().enumerate() {
            let cand = &cands[c];
            // The rest of a component that missed waits without counting as a failure (Core's
            // block builder skips the cluster; only attempted chunks count).
            if missed[cand.component as usize] == pass {
                deferred.push(c);
                continue;
            }
            let fits = cand.fits(weight, sigops);
            overflow |= last && !fits;
            if !(last || fits) {
                missed[cand.component as usize] = pass;
                deferred.push(c);
                failures += 1;
                if failures > MAX_CONSECUTIVE_FAILURES && weight > FULL_ENOUGH {
                    deferred.extend_from_slice(&pending[i + 1..]);
                    break;
                }
                continue;
            }
            failures = 0;
            weight += cand.weight;
            sigops += cand.sigops;
            let members = chunks.members(cand.chunk as usize);
            block.extend(members.iter().map(|&m| (m, cand.rate)));
        }
        if block.is_empty() {
            stuck = true;
        } else {
            blocks.push(block);
        }
        pending.clear();
        mem::swap(&mut pending, &mut deferred);
    }
    Projection { blocks, overflow }
}

/// Every member comes after its parents: block 0's, earlier blocks' or its own block's earlier.
#[cfg(debug_assertions)]
fn check_order(txs: &[SnapTx], excluded: &[bool], blocks: &[Vec<(TxIndex, FeeRate)>]) {
    let mut placed = excluded.to_vec();
    for &(m, _) in blocks.iter().flatten() {
        let t = &txs[m.as_usize()];
        debug_assert!(
            t.parents.iter().all(|p| placed[p.as_usize()]),
            "{} before a parent",
            t.txid
        );
        placed[m.as_usize()] = true;
    }
    debug_assert!(
        placed.iter().all(|&p| p),
        "a transaction left out of every block"
    );
}
