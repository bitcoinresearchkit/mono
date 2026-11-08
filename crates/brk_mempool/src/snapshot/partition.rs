//! Projected blocks 1..N, packed as Core's block assembler packs, by modified fees (the operator's
//! `prioritisetransaction` deltas included) within Core's default budgets (mining options shape
//! only block 0): block 0 is Core's `getblocktemplate` (the caller's); the rest take whole chunks by
//! descending feerate, in dependency order, each block within Core's weight and sigop budgets.
//! Equal feerates go as Core's chunk index orders them: the shorter run of equal-feerate chunks
//! within its cluster first, then the chunk whose greatest txid is lower. What block 0 left of a
//! component is split into the clusters it now forms, each linearized again. A chunk that
//! doesn't fit waits for the next block, and so does the rest of its component (Core's block
//! builder skips the cluster); a block closes once 1,000 chunks in a row failed with it within
//! 4,000 WU of full (Core's `BlockAssembler`). As in Core 32, a candidate is tested at its actual
//! weight, while feerates use its sigop-adjusted weight. A chunk no block could hold waits for the
//! last, which takes whatever is left (mempool.space's catch-all, then more than a block);
//! trailing empty blocks are not made. The budgets are Core's defaults (`-blockreservedweight`,
//! `-bytespersigop`).

use std::mem;

use brk_cpfp::Cluster;
use brk_types::{FeeRate, SigOps, Txid, Weight};
use rustc_hash::FxHashSet;

use super::{SnapTx, TxIndex, builder::ChunkCache, cluster};

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

    /// Each member's chunk, by `TxIndex` (`len` members).
    pub fn chunk_of(&self, len: usize) -> Vec<u32> {
        let mut chunk_of = vec![u32::MAX; len];
        for c in 0..self.chunks.len() {
            for m in self.members(c) {
                chunk_of[m.as_usize()] = c as u32;
            }
        }
        chunk_of
    }

    /// The chunks of the component holding chunk `c`, in linearization order.
    pub fn component_chunks(&self, c: usize) -> std::ops::Range<usize> {
        let i = self.components.partition_point(|&end| end as usize <= c);
        let start = i.checked_sub(1).map_or(0, |p| self.components[p] as usize);
        start..self.components[i] as usize
    }

    /// Chunk `c`'s feerate.
    pub fn rate(&self, c: usize) -> FeeRate {
        self.chunks[c].1
    }

    /// Chunk `c`'s members.
    pub fn members(&self, c: usize) -> &[TxIndex] {
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
    /// Whether the last block is a catch-all: what was left past the others, more than a block
    /// could hold, or what Core's block builder would never take (below its floor).
    pub overflow: bool,
}

pub fn partition(
    txs: &[SnapTx],
    chunks: &Chunks,
    excluded: &[bool],
    num_blocks: usize,
    cache: &mut ChunkCache,
) -> Projection {
    debug_assert!(num_blocks > 0);
    debug_assert_eq!(txs.len(), excluded.len());
    let mut left = Chunks::default();
    let mut graph = Cluster::default();
    let mut rest: Vec<TxIndex> = Vec::new();
    let mut ids = Vec::new();
    let mut kept = ChunkCache::default();
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
            for piece in pieces(txs, &rest, excluded) {
                let linearized = cache.linearized(txs, &piece, &mut ids, &mut graph, &mut kept);
                for positions in linearized.chunks() {
                    let chunk = positions.iter().map(|&p| ids[p as usize].1);
                    left.push_chunk(chunk.clone(), cluster::paid_rate(txs, chunk));
                }
                left.end_component();
            }
            continue;
        } else {
            continue;
        }
        left.end_component();
    }
    *cache = kept;
    let projection = pack(txs, &left, num_blocks);
    #[cfg(debug_assertions)]
    check_order(txs, excluded, &projection.blocks);
    projection
}

/// The clusters `rest` (what block 0 left of one component) forms without the excluded members,
/// each in txid order, as [`cluster::walk`] gives a component.
fn pieces(txs: &[SnapTx], rest: &[TxIndex], excluded: &[bool]) -> Vec<Vec<TxIndex>> {
    let mut placed: FxHashSet<TxIndex> = FxHashSet::default();
    let mut pieces = Vec::new();
    for &seed in rest {
        if placed.contains(&seed) {
            continue;
        }
        let piece = cluster::walk_within(txs, seed, |n| !excluded[n.as_usize()]);
        placed.extend(piece.iter().copied());
        pieces.push(piece);
    }
    pieces
}

struct Candidate {
    chunk: u32,
    component: u32,
    /// The published feerate; the order compares `fee / policy` exactly, as the linearization
    /// did, so a component's chunks never reorder.
    rate: FeeRate,
    /// Modified fee, what the order ranks by (negative under a negative delta); wide, a chunk past
    /// 4,096 members summing past 64 bits.
    fee: i128,
    /// Sigop-adjusted weight.
    policy: Weight,
    /// The policy weight of its component's run of equal-feerate chunks through this one: Core's
    /// first tie-break (`m_main_equal_feerate_chunk_prefix_size`).
    prefix: Weight,
    /// Its members' greatest txid (Core's fallback order compares the bytes): the second.
    max_txid: Txid,
    /// Actual weight: what a block counts, and what a candidate is tested at (Core 32).
    weight: Weight,
    sigops: SigOps,
}

impl Candidate {
    /// Whether it fits a block holding `weight` and `sigops` of transactions.
    fn fits(&self, weight: Weight, sigops: SigOps) -> bool {
        weight + self.weight < MAX_WEIGHT && sigops + self.sigops < MAX_SIGOPS
    }
}

fn pack(txs: &[SnapTx], chunks: &Chunks, num_blocks: usize) -> Projection {
    let per_weight = |fee: i128, policy: Weight| (fee, i128::from(u64::from(policy)));
    // `b`'s feerate against `a`'s, exactly (weights positive): the higher first.
    let by_rate = |(fee_a, weight_a): (i128, i128), (fee_b, weight_b): (i128, i128)| {
        (fee_b * weight_a).cmp(&(fee_a * weight_b))
    };
    let mut cands: Vec<Candidate> = Vec::with_capacity(chunks.chunks.len());
    for (component, range) in chunks.components().enumerate() {
        let mut run: Option<(i128, i128)> = None;
        let mut prefix = Weight::default();
        for c in range.0..range.1 {
            let members = || chunks.members(c).iter().map(|m| &txs[m.as_usize()]);
            let fee: i128 = members().map(|t| i128::from(t.modified)).sum();
            let policy: Weight = members().map(|t| t.sigops.adjust_weight(t.weight)).sum();
            let rate = per_weight(fee, policy);
            // A component's chunk feerates never increase: an equal one extends the run.
            prefix = match run {
                Some(previous) if by_rate(previous, rate).is_eq() => prefix + policy,
                _ => policy,
            };
            run = Some(rate);
            cands.push(Candidate {
                chunk: c as u32,
                component: component as u32,
                rate: chunks.chunks[c].1,
                fee,
                policy,
                prefix,
                max_txid: members().map(|t| t.txid).max_by(|a, b| a.cmp(b)).unwrap(),
                weight: members().map(|t| t.weight).sum(),
                sigops: members().map(|t| t.sigops).sum(),
            });
        }
    }
    // Core's mining order: feerate, then the equal-feerate run, then the greatest txid. A
    // component's chunks keep their order (an equal-feerate later one has the longer run).
    cands.sort_unstable_by(|a, b| {
        by_rate(per_weight(a.fee, a.policy), per_weight(b.fee, b.policy))
            .then(a.prefix.cmp(&b.prefix))
            .then_with(|| a.max_txid.cmp(&b.max_txid))
            .then(a.chunk.cmp(&b.chunk))
    });

    // Core's block builder stops at the first chunk below `-blockmintxfee` (its default, 1 sat/kvB,
    // over the chunk's vsize rounded up): only an operator's delta takes one there, and from it on
    // nothing is mined; those go to the catch-all.
    let mineable = cands
        .iter()
        .position(|cand| cand.fee * 1_000 < i128::from(u64::from(cand.policy).div_ceil(4)))
        .unwrap_or(cands.len());

    let mut blocks: Vec<Vec<(TxIndex, FeeRate)>> = Vec::new();
    // The pass a component last missed a chunk in, plus one: its later chunks wait for the next.
    let mut missed: Vec<u32> = vec![0; chunks.components.len()];
    let mut pass = 0;
    let mut pending: Vec<usize> = (0..mineable).collect();
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
    if mineable < cands.len() {
        let unmined = cands[mineable..].iter().flat_map(|cand| {
            chunks
                .members(cand.chunk as usize)
                .iter()
                .map(|&m| (m, cand.rate))
        });
        let full = blocks.len() == num_blocks;
        match blocks.last_mut() {
            Some(last) if overflow || full => last.extend(unmined),
            _ => blocks.push(unmined.collect()),
        }
        overflow = true;
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
