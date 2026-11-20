use std::cmp::Ordering;
use brk_types::{Sats, Txid, Weight};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{snapshot::{cluster::paid_totals, partition::MAX_WEIGHT}, state::Pool};

use super::{Audit, Capture, Difference, Expected, Mined, MinedBlock, Summary};

/// Seconds after a transaction is first seen past which every miner is taken to have it
/// (mempool.space's `PROPAGATION_MARGIN`): a younger omission is fresh, not missing.
const PROPAGATION_MARGIN: u32 = 180;

/// Sats per policy weight unit; the common factor of four cancels in comparisons.
#[derive(Clone, Copy)]
struct Rate { fee: u64, weight: u64 }

impl Rate {
    fn from_totals((fee, weight): (Sats, Weight)) -> Self { Self { fee: fee.into(), weight: weight.into() } }

    fn compare(self, other: Self) -> Ordering {
        (u128::from(self.fee) * u128::from(other.weight))
            .cmp(&(u128::from(other.fee) * u128::from(self.weight)))
    }
}

struct Chunk { newest: u32, rate: Rate }
struct Omission { txid: Txid, weight: u64, rate: Rate }

fn chunk<'a>(chunks: &'a mut FxHashMap<u32, Chunk>, pool: &Pool, txid: &Txid) -> &'a mut Chunk {
    let index = pool.graph.audit_chunk(txid);
    chunks.entry(index).or_insert_with(|| {
        let members = pool.graph.audit_chunk_members(index);
        let rate = Rate::from_totals(paid_totals(members.clone()));
        let newest = members.map(|tx| *pool.txs.entry(&tx.txid)
            .expect("a captured chunk's members are in its pool").first_seen).max()
            .expect("a captured chunk has members");
        Chunk { newest, rate }
    })
}

pub(super) fn compare(capture: &Capture, block: &MinedBlock) -> Audit {
    let pool = capture.state.pool.as_ref().expect("a capture has a pool");
    let graph = &pool.graph;
    let expected: FxHashSet<_> = graph.block0.iter().map(|r| r.entry.txid).collect();
    let mined: FxHashSet<_> = block.transactions.iter().skip(1).map(|t| t.txid).collect();
    let mut chunks = FxHashMap::default();
    let mut diff = Difference::default();
    let mut conflicts = FxHashSet::default();
    let mut replacers = FxHashSet::default();
    let mut omissions = Vec::new();
    let mut matched = 0u64;
    let mut matched_weight = 0u64;
    for record in &graph.block0 {
        let entry = &record.entry;
        let txid = entry.txid;
        if mined.contains(&txid) {
            matched += 1;
            matched_weight += u64::from(entry.weight);
            continue;
        }
        let mut conflict = false;
        for input in &record.tx.input {
            if let Some(&position) = block.spends.get(&(input.txid, input.vout)) {
                conflict = true;
                replacers.insert(position);
            }
            conflict |= conflicts.contains(&input.txid);
        }
        if conflict {
            conflicts.insert(txid);
            diff.expected.push((txid, Expected::Conflict));
        } else {
            let chunk = chunk(&mut chunks, pool, &txid);
            // Mempool's censor policy excludes sub-1 sat/vB omissions, even on a sparse block.
            if (*capture.receipt).saturating_sub(chunk.newest) <= PROPAGATION_MARGIN {
                diff.expected.push((txid, Expected::Fresh));
            } else if chunk.rate.compare(Rate { fee: 1, weight: 4 }).is_ge() {
                omissions.push(Omission { txid, weight: entry.weight.into(), rate: chunk.rate });
            }
        }
    }
    let mut extras = Vec::new();
    let mut mined_weight = 0u64;
    for (position, tx) in block.transactions.iter().enumerate().skip(1) {
        mined_weight += u64::from(tx.weight);
        if expected.contains(&tx.txid) { continue; }
        let position = position as u16;
        let in_pool = pool.txs.record(&tx.txid).is_some();
        let known = in_pool || pool.graveyard.get(&tx.txid).is_some();
        let rate = if in_pool {
            chunk(&mut chunks, pool, &tx.txid).rate
        } else {
            Rate { fee: tx.fee.into(), weight: u64::from(tx.weight).div_ceil(4) * 4 }
        };
        let flags = if known { Mined::ADDED } else { Mined::UNSEEN }
            | if replacers.contains(&position) { Mined::CONFLICT } else { 0 };
        diff.mined.push(Mined { position, flags });
        extras.push(Omission { txid: tx.txid, weight: tx.weight.into(), rate });
    }
    let order = |a: &Omission, b: &Omission| a.rate.compare(b.rate).then(a.txid.cmp(&b.txid));
    omissions.sort_unstable_by(order);
    extras.sort_unstable_by(order);
    // Both sides use the packer's non-coinbase transaction budget, not consensus headroom.
    let mut forced = (mined_weight + omissions.iter().map(|t| t.weight).sum::<u64>())
        .saturating_sub(u64::from(MAX_WEIGHT));
    let mut credit = extras.iter().map(|t| t.weight).sum::<u64>();
    let mut extras = extras.into_iter().peekable();
    let mut missing = 0u64;
    for omission in omissions {
        while extras.peek().is_some_and(|tx| tx.rate.compare(omission.rate).is_lt()) {
            credit -= extras.next().unwrap().weight;
        }
        let need = omission.weight.min(forced);
        if need > 0 && need <= credit {
            forced -= need;
            credit -= need;
            // Spend the cheapest eligible weight, reserving higher-rate credit for later rows.
            let mut remaining = need;
            while remaining > 0 {
                let extra = extras.peek_mut().unwrap();
                let used = remaining.min(extra.weight);
                remaining -= used;
                extra.weight -= used;
                if extra.weight == 0 { extras.next(); }
            }
        } else {
            missing += 1;
            diff.expected.push((omission.txid, Expected::Missing));
        }
    }
    diff.expected.sort_unstable_by_key(|(txid, _)| **txid);
    let stats = &graph.block_stats[0];
    let denominator = matched + missing;
    let coinbase = u64::from(block.transactions[0].weight);
    let total_weight = u64::from(stats.total_weight) + coinbase;
    Audit {
        summary: Summary {
            hash: block.hash, height: block.height, timestamp: block.timestamp,
            match_rate: (10_000 * matched + denominator / 2).checked_div(denominator).unwrap_or(10_000) as u16,
            expected_fees: stats.total_fee,
            expected_weight: stats.total_weight,
            similarity: if total_weight == 0 { 1.0 } else {
                (matched_weight + coinbase) as f64 / total_weight as f64
            },
        },
        difference: diff,
    }
}
