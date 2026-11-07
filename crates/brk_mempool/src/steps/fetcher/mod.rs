pub mod fetched;

use std::sync::Arc;

use brk_error::Result;
use brk_rpc::{BlockTemplateTx, Client};
use brk_types::{MempoolEntryInfo, Timestamp, Txid, VSize};
pub use fetched::Fetched;
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::warn;

use crate::{State, steps::preparer::TxAddition, stores::tx_store::TxRecord};

/// Bounds the non-template transactions fetched per cycle (the RPC client
/// separately chunks its batches). New template transactions are not subject
/// to this cap: they're bounded by the block weight limit Core enforces on its
/// own template.
pub(crate) const MAX_TX_FETCHES_PER_CYCLE: usize = 10_000;

/// Scales with churn rather than mempool size: `getblocktemplate` +
/// `getrawmempool false` + `getmempoolinfo` in one mixed batch, then
/// `getmempoolentry` (+ `getrawtransaction` outside the template) for *new*
/// txids, in batches of at most 2,000 txids.
///
/// New listed template transactions lead that batch, uncapped (Core bounds the
/// template), entries only: their bodies, weights, policy sizes (from the
/// template's weight and sigops) and fees are the template's, their arrival
/// times Core's entries'. One that left between the two batches is still
/// materialized from the template, so block 0 always reflects Core's exact
/// selection; only then is its arrival time (`now`) estimated.
///
/// The template's transactions the listing lacks (Core rebuilds its template at most every few
/// seconds while its pool changes: they left since) are kept for block 0 alone, as records: the
/// pool's own while it still holds them (it buries them this cycle), else block 0's last ones,
/// the graveyard's, or the template's body. Nothing is fetched for them.
///
/// Confirmed prevouts are resolved after apply, by the resolver passed to
/// `Mempool::tick_with`.
/// `failing`: listed txids that failed to load before, with how often; the untried go first,
/// then the retries fewest failures first, so the cap is never filled by the same stuck ones
/// every cycle and every retry comes round.
pub fn fetch(
    client: &Client,
    mempool: &State,
    failing: &RapidHashMap<Txid, u32>,
) -> Result<Fetched> {
    let (state, block_template) = client.fetch_mempool_state()?;
    let mut template = block_template.is_some();
    let mut block_template = block_template.unwrap_or_default();

    // The same private pool decides the fetch list and template synthesis,
    // so they agree on what's already known. The template's txids the listing lacks: the
    // block-sized set trimmed by one pass over the listing, no second full-pool set beside the
    // one the preparer builds.
    let mut template_only: RapidHashSet<Txid> = block_template.iter().map(|g| g.txid).collect();
    for txid in &state.live_txids {
        template_only.remove(txid);
    }
    // Listed and new to the pool, or a witness variant of a body it holds (Core keeps one per
    // txid, so a variant follows only the original's leaving, which a poll can miss): fetched
    // afresh.
    let mut template_new: RapidHashSet<Txid> = block_template
        .iter()
        .filter(|g| {
            !template_only.contains(&g.txid)
                && mempool
                    .txs
                    .record(&g.txid)
                    .is_none_or(|record| record.entry.wtxid != g.wtxid)
        })
        .map(|g| g.txid)
        .collect();
    // What block 0 takes from the template alone (the unlisted held nowhere, new bodies), read
    // before anything is fetched: a body that doesn't decode sets the template aside for the
    // cycle (block 0 packs from the pool, the listing fetches what it held), not the cycle.
    let now = Timestamp::now();
    let previous: RapidHashMap<Txid, &TxRecord> = mempool
        .template_only
        .iter()
        .map(|record| (record.entry.txid, record))
        .collect();
    let read = || -> Result<_> {
        let mut unlisted = Vec::with_capacity(template_only.len());
        let mut bodies = RapidHashMap::default();
        for g in &block_template {
            if template_only.contains(&g.txid) {
                unlisted.push(template_record(g, mempool, &previous, now)?);
            } else if template_new.contains(&g.txid) {
                bodies.insert(g.txid, g.decode()?);
            }
        }
        Ok((unlisted, bodies))
    };
    let (unlisted, mut bodies) = read().unwrap_or_else(|error| {
        warn!(%error, "getblocktemplate: a body doesn't decode, set aside");
        template = false;
        block_template.clear();
        template_only.clear();
        template_new.clear();
        Default::default()
    });
    let template_txids: Vec<Txid> = block_template
        .iter()
        .map(|g| g.txid)
        .filter(|txid| template_new.contains(txid))
        .collect();
    let unknown = || {
        state
            .live_txids
            .iter()
            .filter(|t| !mempool.txs.contains(t) && !template_new.contains(t))
    };
    let mut new_txids: Vec<Txid> = unknown()
        .filter(|t| !failing.contains_key(t))
        .take(MAX_TX_FETCHES_PER_CYCLE)
        .copied()
        .collect();
    if !failing.is_empty() {
        let room = MAX_TX_FETCHES_PER_CYCLE - new_txids.len();
        let mut retries: Vec<(u32, Txid)> = unknown()
            .filter_map(|t| failing.get(t).map(|&failures| (failures, *t)))
            .collect();
        retries.sort_unstable_by_key(|&(failures, _)| failures);
        new_txids.extend(retries.into_iter().take(room).map(|(_, txid)| txid));
    }
    let capped = new_txids.len() == MAX_TX_FETCHES_PER_CYCLE;

    let (mut new_entries, mut new_txs) = client.fetch_new_pool_data(&template_txids, &new_txids)?;
    let requested = new_txids;
    let fetched: RapidHashSet<Txid> = new_entries
        .iter()
        .map(|entry| entry.txid)
        .filter(|txid| template_new.contains(txid))
        .collect();

    // Template entries lead (fetched in request order, synthesized ones after
    // them), listing entries follow.
    let listed_entries = new_entries.split_off(fetched.len());
    debug_assert!(
        new_entries
            .iter()
            .all(|entry| fetched.contains(&entry.txid))
    );

    // Consume `block_template` by value: the template's order is captured as a
    // `Vec<Txid>` for the Rebuilder, which only reads txids, and new bodies are
    // decoded from the template (known bodies stay undecoded).
    //
    // The body is the template's, and so are its weight, policy size and fee: the node may hold
    // another witness variant by the time its entry was fetched, so only the entry's arrival
    // time is kept.
    let fetched_at: RapidHashMap<Txid, usize> = new_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| (entry.txid, i))
        .collect();
    let mut block_template_txids: Vec<Txid> = Vec::with_capacity(block_template.len());
    for g in block_template {
        let txid = g.txid;
        if let Some(body) = bodies.remove(&txid) {
            new_txs.insert(txid, body);
            let entry = MempoolEntryInfo {
                txid,
                wtxid: g.wtxid,
                vsize: g.sigops.adjust_vsize(VSize::from(g.weight)),
                weight: g.weight,
                fee: g.fee,
                first_seen: now,
            };
            match fetched_at.get(&txid) {
                Some(&i) => {
                    new_entries[i] = MempoolEntryInfo {
                        first_seen: new_entries[i].first_seen,
                        ..entry
                    }
                }
                None => new_entries.push(entry),
            }
        }
        block_template_txids.push(txid);
    }
    new_entries.extend(listed_entries);

    Ok(Fetched {
        state,
        new_entries,
        new_txs,
        block_template_txids: template.then_some(block_template_txids),
        template_only: unlisted,
        capped,
        requested,
    })
}

/// A template transaction the listing lacks, as a record for block 0: one already held of the
/// same witness (the pool's, leaving this cycle, block 0's last, or the graveyard's), else built
/// from the template's body, first seen `now` unless the graveyard saw it before.
fn template_record(
    g: &BlockTemplateTx,
    mempool: &State,
    previous: &RapidHashMap<Txid, &TxRecord>,
    now: Timestamp,
) -> Result<TxRecord> {
    let held = |record: &TxRecord| (record.entry.wtxid == g.wtxid).then(|| record.clone());
    if let Some(record) = mempool
        .txs
        .record(&g.txid)
        .and_then(held)
        .or_else(|| previous.get(&g.txid).and_then(|record| held(record)))
        .or_else(|| {
            mempool
                .graveyard
                .get(&g.txid)
                .and_then(|tomb| held(&tomb.record))
        })
    {
        return Ok(record);
    }
    let info = MempoolEntryInfo {
        txid: g.txid,
        wtxid: g.wtxid,
        vsize: g.sigops.adjust_vsize(VSize::from(g.weight)),
        weight: g.weight,
        fee: g.fee,
        first_seen: now,
    };
    let TxAddition { tx, entry } =
        TxAddition::new(&info, g.decode()?, &mempool.txs, &mempool.graveyard);
    Ok(TxRecord::new(Arc::new(tx), entry))
}
