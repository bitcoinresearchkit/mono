//! Linearization and chunking of a dependency component, its transactions given in topological
//! order.
//!
//! A linearization orders a component's transactions so that each comes after its parents. Its
//! chunks are the runs a miner takes whole, each paying no more per weight unit than the one
//! before. The algorithm depends on the component's size:
//!
//! - Up to [`EXACT_MAX`] members (Bitcoin Core 31's cluster limit): the optimal linearization,
//!   the one Core 31 aims for ([`exact`]).
//! - Up to [`ANCESTOR_MAX`] members (a block's components, or a mempool's under a raised limit):
//!   the best remaining ancestor set, repeatedly, then Core's PostLinearize ([`ancestor`]).
//! - Larger: the component's own topological order, chunked. That is O(n) and optimal for a
//!   chain, the usual shape of a block-filling component; the ancestor-set rows (8 MiB at their
//!   limit) would let one crafted component stall a rebuild.
//!
//! Sizes are the caller's weights (sigop-adjusted for the mempool, as Core 31 linearizes) and fees
//! the ranked ones: Core's modified fees, signed, since an operator's delta can make one negative.
//! Feerates compare as exact fractions. A chunk's published feerate is its fee over a quarter of
//! its weight, as Core's chunks and mempool.space's effective feerates are, and never below zero.

mod ancestor;
mod exact;
mod flow;
mod post;

use std::cmp::Ordering;

use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, Weight};

pub(crate) use ancestor::ancestor_sets;
use exact::exact;
use flow::Flow;
use post::Post;

use crate::cluster::Node;

/// A ranked fee's published rate over `weight`: zero for one at or below zero.
fn published_rate(fee: impl Into<i128>, weight: Weight) -> FeeRate {
    FeeRate::from((Sats::from(u64::try_from(fee.into()).unwrap_or(0)), weight))
}

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
    /// Row `i`: transaction `i`'s descendants, itself included.
    descendants: Vec<u64>,
    post: Post,
    /// Each candidate's remaining ancestor set totals.
    sets: Vec<Totals>,
    remaining: Vec<bool>,
    /// Transactions in selection order: every group, and so every chunk, is a contiguous run.
    order: Vec<CpfpClusterTxIndex>,
    chunks: Vec<Chunk>,
}

struct Totals {
    fee: i64,
    weight: Weight,
    count: usize,
}

/// A chunk: `order[previous end..end]`, with its summed fee (wide: past 4,096 members the
/// fees can sum past 64 bits) and size.
struct Chunk {
    end: usize,
    fee: i128,
    weight: Weight,
}

/// `a`'s feerate against `b`'s, exactly: fee_a / weight_a vs fee_b / weight_b (weights positive).
fn compare_rates(a: (impl Into<i128>, Weight), b: (impl Into<i128>, Weight)) -> Ordering {
    let (fee_a, weight_a) = (a.0.into(), i128::from(u64::from(a.1)));
    let (fee_b, weight_b) = (b.0.into(), i128::from(u64::from(b.1)));
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
        push_group(chunks, index + 1, node.modified, node.weight);
    }
    emit_chunks(order, chunks, emit);
}

/// Appends the group ending at `end` of the order: one paying more than the chunk before it
/// joins that chunk, so chunk feerates never increase.
fn push_group(chunks: &mut Vec<Chunk>, end: usize, fee: i64, weight: Weight) {
    let mut chunk = Chunk {
        end,
        fee: i128::from(fee),
        weight,
    };
    while let Some(last) = chunks.last()
        && compare_rates((chunk.fee, chunk.weight), (last.fee, last.weight)).is_gt()
    {
        chunk.fee += last.fee;
        chunk.weight += last.weight;
        chunks.pop();
    }
    chunks.push(chunk);
}

/// Each chunk's members (in the linearization's order) and feerate, in chunk order.
fn emit_chunks(
    order: &[CpfpClusterTxIndex],
    chunks: &[Chunk],
    mut emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let mut start = 0;
    for chunk in chunks {
        emit(&order[start..chunk.end], published_rate(chunk.fee, chunk.weight));
        start = chunk.end;
    }
}

/// `nodes`' chunks, through the tier their count takes (see the module).
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
