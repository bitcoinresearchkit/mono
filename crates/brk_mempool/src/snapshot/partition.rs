//! Projected blocks 1..N, packed as Core's block assembler packs, by modified fees (the operator's
//! `prioritisetransaction` deltas included) within Core's default budgets (mining options shape
//! only block 0): block 0 is Core's `getblocktemplate` (the caller's); the rest take whole chunks by
//! descending feerate, in dependency order, each block within Core's weight and sigop budgets.
//! Equal feerates go as Core's chunk index orders them: the shorter run of equal-feerate chunks
//! within its cluster first, then the chunk whose greatest txid is lower. At every virtual block
//! boundary, partially consumed components split and are linearized again. A chunk that
//! doesn't fit waits for the next block, and so does the rest of its component (Core's block
//! builder skips the cluster); a block closes once 1,000 chunks in a row failed with it within
//! 4,000 WU of full (Core's `BlockAssembler`). As in Core 32, a candidate is tested at its actual
//! weight, while feerates use its sigop-adjusted weight. A chunk no block could hold waits for the
//! last, which takes whatever is left (mempool.space's catch-all, then more than a block);
//! trailing empty blocks are not made. The budgets are Core's defaults (`-blockreservedweight`,
//! `-bytespersigop`).

use brk_cpfp::{Cluster, compare_feerates};
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
        let (start, end) = self.component(i);
        start..end
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
        (0..self.components.len()).map(|index| self.component(index))
    }

    /// One component's chunk range, by component index.
    fn component(&self, index: usize) -> (usize, usize) {
        let start = index.checked_sub(1).map_or(0, |previous| self.components[previous] as usize);
        (start, self.components[index] as usize)
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
    let mut kept = ChunkCache::default();
    let left = residual(txs, chunks, excluded, cache, &mut kept);
    let projection = pack(txs, left, excluded.to_vec(), num_blocks, cache, &mut kept);
    *cache = kept;
    #[cfg(debug_assertions)]
    check_order(txs, excluded, &projection.blocks);
    projection
}

/// One operation for GBT and all local virtual boundaries. Untouched components reuse their
/// chunks. Partially consumed ones split and use the common solver, including its heuristic
/// tiers; no assumption about a heuristic suffix being optimal or connected is needed.
fn residual(
    txs: &[SnapTx],
    chunks: &Chunks,
    excluded: &[bool],
    cache: &mut ChunkCache,
    kept: &mut ChunkCache,
) -> Chunks {
    let mut left = Chunks::default();
    let mut graph = Cluster::default();
    let mut rest: Vec<TxIndex> = Vec::new();
    let mut ids = Vec::new();
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
                let linearized = cache.linearized(txs, &piece, &mut ids, &mut graph, kept);
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
    left
}

/// The clusters `rest` (what a virtual block left of one component) forms without the excluded members,
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

fn candidates(txs: &[SnapTx], chunks: &Chunks) -> Vec<Candidate> {
    let mut cands: Vec<Candidate> = Vec::with_capacity(chunks.chunks.len());
    for (component, range) in chunks.components().enumerate() {
        let mut run: Option<(i128, Weight)> = None;
        let mut prefix = Weight::default();
        for c in range.0..range.1 {
            let members = || chunks.members(c).iter().map(|m| &txs[m.as_usize()]);
            let fee: i128 = members().map(|t| i128::from(t.modified)).sum();
            let policy: Weight = members().map(|t| t.sigops.adjust_weight(t.weight)).sum();
            let rate = (fee, policy);
            // A component's chunk feerates never increase: an equal one extends the run.
            prefix = match run {
                Some(previous) if compare_feerates(previous, rate).is_eq() => prefix + policy,
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
        compare_feerates((b.fee, b.policy), (a.fee, a.policy))
            .then(a.prefix.cmp(&b.prefix))
            .then_with(|| a.max_txid.cmp(&b.max_txid))
            .then(a.chunk.cmp(&b.chunk))
    });

    cands
}

/// The default mining floor: 1 sat/kvB over rounded-up policy vsize. Whole-component
/// removal changes neither candidate rates nor this cutoff in their retained order.
fn mineable_count(cands: &[Candidate]) -> usize {
    cands.iter().position(|cand| {
        compare_feerates(
            (cand.fee, Weight::from(u64::from(cand.policy).div_ceil(4))),
            (1, Weight::from(1_000u64)),
        ).is_lt()
    }).unwrap_or(cands.len())
}

fn pack(
    txs: &[SnapTx],
    mut chunks: Chunks,
    mut consumed: Vec<bool>,
    num_blocks: usize,
    cache: &mut ChunkCache,
    kept: &mut ChunkCache,
) -> Projection {
    let mut blocks = Vec::new();
    let mut overflow = false;
    let mut cands = candidates(txs, &chunks);
    let mut mineable = mineable_count(&cands);
    let mut pending: Vec<usize> = (0..cands.len()).collect();
    let mut deferred = Vec::new();
    while !pending.is_empty() {
        let last = blocks.len() + 1 == num_blocks;
        let mut block = Vec::new();
        let mut missed = vec![false; chunks.components.len()];
        let mut selected = vec![false; chunks.components.len()];
        let mut selected_components = Vec::new();
        let (mut weight, mut sigops) = (Weight::ZERO, SigOps::ZERO);
        let mut failures = 0;
        for (position, &index) in pending.iter().enumerate() {
            if index >= mineable {
                deferred.extend_from_slice(&pending[position..]);
                break;
            }
            let cand = &cands[index];
            if missed[cand.component as usize] {
                deferred.push(index);
                continue;
            }
            let fits = cand.fits(weight, sigops);
            if !(last || fits) {
                missed[cand.component as usize] = true;
                deferred.push(index);
                failures += 1;
                if failures > MAX_CONSECUTIVE_FAILURES && weight > FULL_ENOUGH {
                    deferred.extend_from_slice(&pending[position + 1..]);
                    break;
                }
                continue;
            }
            overflow |= last && !fits;
            let component = cand.component as usize;
            if !selected[component] {
                selected[component] = true;
                selected_components.push(component);
            }
            failures = 0;
            weight += cand.weight;
            sigops += cand.sigops;
            for &member in chunks.members(cand.chunk as usize) {
                consumed[member.as_usize()] = true;
                block.push((member, cand.rate));
            }
        }
        if last || block.is_empty() {
            // The last slot, or a pool no finite block can take: preserve the backlog contract.
            // Withheld members were already excluded before partition and remain omitted.
            for &index in &pending {
                let cand = &cands[index];
                for &member in chunks.members(cand.chunk as usize) {
                    if !consumed[member.as_usize()] {
                        consumed[member.as_usize()] = true;
                        block.push((member, cand.rate));
                        overflow = true;
                    }
                }
            }
            blocks.push(block);
            break;
        }
        blocks.push(block);
        let partial = selected_components.into_iter().any(|component| {
            chunks.span(chunks.component(component)).iter().any(|m| !consumed[m.as_usize()])
        });
        if partial {
            // Split touched components and recompute equal-rate prefixes for the new graph.
            chunks = residual(txs, &chunks, &consumed, cache, kept);
            cands = candidates(txs, &chunks);
            mineable = mineable_count(&cands);
            pending = (0..cands.len()).collect();
        } else {
            // Only complete components were consumed. Surviving identities and tie prefixes are
            // unchanged: keep their existing sorted order rather than solving/sorting again.
            std::mem::swap(&mut pending, &mut deferred);
        }
        deferred.clear();
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
