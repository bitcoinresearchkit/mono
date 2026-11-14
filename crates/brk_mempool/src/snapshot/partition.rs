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
#[derive(Clone, Default)]
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
    let projection = pack(txs, chunks.clone(), excluded.to_vec(), num_blocks, cache, &mut kept);
    *cache = kept;
    #[cfg(debug_assertions)]
    check_order(txs, excluded, &projection.blocks);
    projection
}

/// The clusters `rest` (what a virtual block left of one component) forms without the consumed
/// members, each in txid order, as [`cluster::walk`] gives a component.
fn pieces(txs: &[SnapTx], rest: &[TxIndex], consumed: &[bool]) -> Vec<Vec<TxIndex>> {
    let mut placed: FxHashSet<TxIndex> = FxHashSet::default();
    let mut pieces = Vec::new();
    for &seed in rest {
        if placed.contains(&seed) {
            continue;
        }
        let piece = cluster::walk_within(txs, seed, |n| !consumed[n.as_usize()]);
        placed.extend(piece.iter().copied());
        pieces.push(piece);
    }
    pieces
}

/// A chunk as the packer ranks and tests it: candidate `i` is chunk `i`.
struct Candidate {
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
    /// At or above the default mining floor: 1 sat/kvB over rounded-up policy vsize.
    mineable: bool,
}

impl Candidate {
    /// Whether it fits a block holding `weight` and `sigops` of transactions.
    fn fits(&self, weight: Weight, sigops: SigOps) -> bool {
        weight + self.weight < MAX_WEIGHT && sigops + self.sigops < MAX_SIGOPS
    }

    /// Core's mining order: feerate, then the equal-feerate run, then the greatest txid: a strict
    /// order, as a snapshot's txids are unique (the node's listing, then the template's txids it
    /// lacks), so chunks never tie. A component's chunks keep their order (an equal-feerate later
    /// one has the longer run).
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        compare_feerates((other.fee, other.policy), (self.fee, self.policy))
            .then(self.prefix.cmp(&other.prefix))
            .then_with(|| self.max_txid.cmp(&other.max_txid))
    }
}

/// Appends the candidates of the chunks pushed since the last, component by component.
fn push_candidates(txs: &[SnapTx], chunks: &Chunks, cands: &mut Vec<Candidate>) {
    let first = chunks.components.partition_point(|&end| end as usize <= cands.len());
    for component in first..chunks.components.len() {
        let (from, to) = chunks.component(component);
        debug_assert_eq!(from, cands.len());
        let mut run: Option<(i128, Weight)> = None;
        let mut prefix = Weight::default();
        for c in from..to {
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
                component: component as u32,
                rate: chunks.chunks[c].1,
                fee,
                policy,
                prefix,
                max_txid: members().map(|t| t.txid).max_by(|a, b| a.cmp(b)).unwrap(),
                weight: members().map(|t| t.weight).sum(),
                sigops: members().map(|t| t.sigops).sum(),
                mineable: compare_feerates(
                    (fee, Weight::from(u64::from(policy).div_ceil(4))),
                    (1, Weight::from(1_000u64)),
                )
                .is_ge(),
            });
        }
    }
}

/// Splits each `touched` component (some members consumed) into the components its other members
/// form, each linearized again and appended to `chunks` with its candidates. `pending` drops the
/// touched components' candidates, takes the new ones and comes back in mining order; untouched
/// components keep their chunks, candidates and places.
#[allow(clippy::too_many_arguments)]
fn split(
    txs: &[SnapTx],
    chunks: &mut Chunks,
    cands: &mut Vec<Candidate>,
    pending: &mut Vec<usize>,
    touched: &[usize],
    consumed: &[bool],
    cache: &mut ChunkCache,
    kept: &mut ChunkCache,
) {
    let mut is_touched = vec![false; chunks.components.len()];
    let mut graph = Cluster::default();
    let mut ids = Vec::new();
    let mut rest = Vec::new();
    for &component in touched {
        is_touched[component] = true;
        rest.clear();
        rest.extend(chunks.span(chunks.component(component)).iter().filter(|m| !consumed[m.as_usize()]));
        for piece in pieces(txs, &rest, consumed) {
            let linearized = cache.linearized(txs, &piece, &mut ids, &mut graph, kept);
            for positions in linearized.chunks() {
                let chunk = positions.iter().map(|&p| ids[p as usize].1);
                chunks.push_chunk(chunk.clone(), cluster::paid_rate(txs, chunk));
            }
            chunks.end_component();
        }
    }
    let new = cands.len();
    push_candidates(txs, chunks, cands);
    pending.retain(|&i| !is_touched[cands[i].component as usize]);
    pending.extend(new..cands.len());
    // Stable: a run already in order merges with the new candidates in linear time.
    pending.sort_by(|&a, &b| cands[a].order(&cands[b]));
}

/// Blocks after block 0 from `chunks`, `consumed` holding block 0's members and the withheld.
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
    let mut cands = Vec::with_capacity(chunks.chunks.len());
    push_candidates(txs, &chunks, &mut cands);
    // Block 0 and the withheld: components they took whole leave, ones they took part of split.
    let mut pending = Vec::new();
    let mut touched = Vec::new();
    for component in 0..chunks.components.len() {
        let range = chunks.component(component);
        let members = chunks.span(range);
        match members.iter().filter(|m| consumed[m.as_usize()]).count() {
            0 => pending.extend(range.0..range.1),
            taken if taken < members.len() => touched.push(component),
            _ => {}
        }
    }
    split(txs, &mut chunks, &mut cands, &mut pending, &touched, &consumed, cache, kept);
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
            let cand = &cands[index];
            // Below Core's floor: it and all after it wait for the last block.
            if !cand.mineable {
                deferred.extend_from_slice(&pending[position..]);
                break;
            }
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
            for &member in chunks.members(index) {
                consumed[member.as_usize()] = true;
                block.push((member, cand.rate));
            }
        }
        if last || block.is_empty() {
            // The last slot, or a pool no finite block can take: preserve the backlog contract.
            // Withheld members were already excluded before partition and remain omitted.
            for &index in &pending {
                for &member in chunks.members(index) {
                    if !consumed[member.as_usize()] {
                        consumed[member.as_usize()] = true;
                        block.push((member, cands[index].rate));
                        overflow = true;
                    }
                }
            }
            blocks.push(block);
            break;
        }
        blocks.push(block);
        std::mem::swap(&mut pending, &mut deferred);
        deferred.clear();
        // What the block took part of splits; the equal-rate prefixes of its pieces start anew.
        selected_components.retain(|&component| {
            chunks.span(chunks.component(component)).iter().any(|m| !consumed[m.as_usize()])
        });
        split(txs, &mut chunks, &mut cands, &mut pending, &selected_components, &consumed, cache, kept);
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
