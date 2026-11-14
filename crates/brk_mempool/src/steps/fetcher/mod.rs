pub mod fetched;

use brk_error::Result;
use brk_rpc::{Client, MempoolObservation};
use brk_types::{MempoolEntryInfo, Txid, VSize};
pub use fetched::Fetched;
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::warn;

use crate::State;

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
/// Core's template is a source of bodies, not of block 0 (the packer's): new listed template
/// transactions lead that batch, uncapped (Core bounds the template), entries only: their bodies,
/// weights, policy sizes (from the template's weight and sigops) and fees are the template's,
/// their arrival times Core's entries'. One that left between the two batches is set aside: the
/// listing no longer holds it.
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
    let MempoolObservation { state, block_template, template_json, modified_fees } =
        client.fetch_mempool_state(previous_modified)?;
    let block_template = block_template.unwrap_or_default();

    // Listed and new to the pool, or a witness variant of a body it holds (Core keeps one per
    // txid, so a variant follows only the original's leaving, which a poll can miss): its body is
    // the template's. The listing is checked in one pass over it.
    let in_template: RapidHashSet<Txid> = block_template.iter().map(|g| g.txid).collect();
    let listed: RapidHashSet<Txid> =
        state.live_txids.iter().filter(|txid| in_template.contains(*txid)).copied().collect();
    let new_from_template = block_template.iter().filter(|g| {
        listed.contains(&g.txid)
            && mempool.txs.record(&g.txid).is_none_or(|record| record.entry.wtxid != g.wtxid)
    });
    // A body that doesn't decode is fetched with the listing instead.
    let mut bodies = RapidHashMap::default();
    for g in new_from_template {
        match g.decode() {
            Ok(body) => {
                bodies.insert(g.txid, (g, body));
            }
            Err(error) => warn!(%error, txid = %g.txid, "getblocktemplate: a body doesn't decode"),
        }
    }
    let template_txids: Vec<Txid> = bodies.keys().copied().collect();
    let unknown = || {
        state
            .live_txids
            .iter()
            .filter(|t| !mempool.txs.contains(t) && !bodies.contains_key(*t))
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

    // Records restored from a shutdown save, a cap's worth per cycle, against Core's entries: one
    // Core now holds under another witness is fetched afresh. Only while it is listed (the pool
    // holds what one listing held), and not a template's (its body is the template's).
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
            .filter(|txid| !bodies.contains_key(txid))
            .collect();
        if !variants.is_empty() {
            new_txids.extend(state.live_txids.iter().filter(|txid| variants.contains(*txid)));
        }
    }

    let (mut new_entries, mut new_txs) = client.fetch_new_pool_data(&template_txids, &new_txids)?;
    let requested = new_txids;
    // Template entries lead (in request order), listing entries follow. The body is the
    // template's, and so are its weight, policy size and fee: the node may hold another witness
    // variant by the time its entry was fetched, so only the entry's arrival time is kept. One
    // whose entry didn't come back left the pool between the two batches.
    let fetched = new_entries.iter().take_while(|entry| bodies.contains_key(&entry.txid)).count();
    let listed_entries = new_entries.split_off(fetched);
    for entry in &mut new_entries {
        let (g, body) = bodies.remove(&entry.txid).expect("a template entry");
        new_txs.insert(g.txid, body);
        *entry = MempoolEntryInfo {
            txid: g.txid,
            wtxid: g.wtxid,
            vsize: g.sigops.adjust_vsize(VSize::from(g.weight)),
            weight: g.weight,
            fee: g.fee,
            first_seen: entry.first_seen,
        };
    }
    new_entries.extend(listed_entries);

    Ok(Fetched {
        state,
        modified_fees,
        checked,
        recheck,
        new_entries,
        new_txs,
        template_json: template_json.map(Into::into),
        capped,
        requested,
    })
}
