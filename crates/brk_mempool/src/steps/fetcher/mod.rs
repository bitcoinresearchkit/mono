pub mod fetched;

use brk_error::Result;
use brk_rpc::{Client, MempoolObservation};
use brk_types::Txid;
pub use fetched::Fetched;
use rapidhash::{RapidHashMap, RapidHashSet};

use crate::State;

/// Bounds the new transactions fetched per cycle (the RPC client separately chunks its batches).
pub(crate) const MAX_TX_FETCHES_PER_CYCLE: usize = 10_000;

/// Scales with churn rather than mempool size: `getbestblockhash` + `getmempoolinfo` +
/// `getrawmempool false` + `getprioritisedtransactions` in one mixed batch, then
/// `getmempoolentry` + `getrawtransaction` for *new* txids, in batches of at most 2,000 txids.
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
    previous_modified: &[(Txid, i64)],
) -> Result<Fetched> {
    let MempoolObservation { state, modified_fees } = client.fetch_mempool_state(previous_modified)?;

    let unknown = || state.live_txids.iter().filter(|t| !mempool.txs.contains(t));
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

    // Records restored from a shutdown save, a cap's worth per cycle, against Core's entries: one
    // Core now holds under another witness is fetched afresh. Only while it is listed (the pool
    // holds what one listing held).
    let checked = mempool.unverified.len().min(MAX_TX_FETCHES_PER_CYCLE);
    let checking = &mempool.unverified[mempool.unverified.len() - checked..];
    let mut recheck: RapidHashSet<Txid> = checking.iter().copied().collect();
    if !checking.is_empty() {
        let (entries, _) = client.fetch_new_pool_data(checking, &[])?;
        for entry in &entries {
            if mempool.txs.record(&entry.txid).is_some_and(|record| record.entry.wtxid == entry.wtxid) {
                recheck.remove(&entry.txid);
            }
        }
        let variants: RapidHashSet<Txid> = entries
            .iter()
            .filter(|entry| {
                mempool
                    .txs
                    .record(&entry.txid)
                    .is_some_and(|record| record.entry.wtxid != entry.wtxid)
            })
            .map(|entry| entry.txid)
            .collect();
        if !variants.is_empty() {
            new_txids.extend(state.live_txids.iter().filter(|txid| variants.contains(*txid)));
        }
    }

    let (new_entries, new_txs) = client.fetch_new_pool_data(&[], &new_txids)?;
    let requested = new_txids;

    Ok(Fetched {
        state,
        modified_fees,
        checked,
        recheck,
        new_entries,
        new_txs,
        capped,
        requested,
    })
}
