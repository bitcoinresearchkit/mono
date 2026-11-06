pub mod fetched;

use crate::State;
use brk_error::Result;
use brk_rpc::Client;
use brk_types::{MempoolEntryInfo, Timestamp, Txid, VSize};
pub use fetched::Fetched;
use rapidhash::{RapidHashMap, RapidHashSet};

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
/// New template transactions lead that batch, uncapped (Core bounds the
/// template), entries only: their bodies, weights, policy sizes (from the
/// template's weight and sigops) and fees are the template's, their arrival
/// times Core's entries'. One that left between the two batches is still
/// materialized from the template, so block 0 always reflects Core's exact
/// selection; only then is its arrival time (`now`) estimated.
///
/// Confirmed prevouts are resolved after apply, by the resolver passed to
/// `Mempool::tick_with`.
pub fn fetch(client: &Client, mempool: &State) -> Result<Fetched> {
    let (mut state, block_template) = client.fetch_mempool_state()?;

    // The same private pool decides the fetch list and template synthesis,
    // so they agree on what's already known. The template's txids, trimmed
    // below to those the raw listing lacks.
    let mut template_only: RapidHashSet<Txid> = block_template.iter().map(|g| g.txid).collect();
    // New to the pool, or a witness variant of a body it holds (Core keeps one per txid, so a
    // variant follows only the original's leaving, which a poll can miss): fetched afresh.
    let template_new: RapidHashSet<Txid> = block_template
        .iter()
        .filter(|g| {
            mempool
                .txs
                .record(&g.txid)
                .is_none_or(|record| record.entry.wtxid != g.wtxid)
        })
        .map(|g| g.txid)
        .collect();
    let template_txids: Vec<Txid> = block_template
        .iter()
        .map(|g| g.txid)
        .filter(|txid| template_new.contains(txid))
        .collect();
    let new_txids: Vec<Txid> = state
        .live_txids
        .iter()
        .filter(|t| !mempool.txs.contains(t) && !template_only.contains(t))
        .take(MAX_TX_FETCHES_PER_CYCLE)
        .copied()
        .collect();
    let capped = new_txids.len() == MAX_TX_FETCHES_PER_CYCLE;

    let (mut new_entries, mut new_txs) = client.fetch_new_pool_data(&template_txids, &new_txids)?;
    let fetched: RapidHashSet<Txid> = new_entries
        .iter()
        .map(|entry| entry.txid)
        .filter(|txid| template_new.contains(txid))
        .collect();

    // Template entries lead (fetched in request order, synthesized ones after
    // them), listing entries follow: a removal spent by both a stale template
    // tx and its listed replacer names the replacer.
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
    let now = Timestamp::now();
    let fetched_at: RapidHashMap<Txid, usize> = new_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| (entry.txid, i))
        .collect();
    let mut block_template_txids: Vec<Txid> = Vec::with_capacity(block_template.len());
    for g in block_template {
        let txid = g.txid;
        if template_new.contains(&txid) {
            new_txs.insert(txid, g.decode()?);
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

    // Keep GBT-only bodies for the exact template projection, but do not
    // call their union a coherent address view: a removed GBT transaction
    // can conflict with a replacement in the newer raw listing.
    // Reuse the block-sized GBT set instead of allocating a second full
    // mempool set beside the one Preparer needs.
    for txid in &state.live_txids {
        template_only.remove(txid);
    }
    state.live_txids.extend(
        block_template_txids
            .iter()
            .filter(|txid| template_only.contains(*txid))
            .copied(),
    );

    Ok(Fetched {
        state,
        new_entries,
        new_txs,
        block_template_txids,
        template_only,
        capped,
    })
}
