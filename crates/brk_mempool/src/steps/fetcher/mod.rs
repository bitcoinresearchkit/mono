mod fetched;

pub use fetched::Fetched;

use brk_rpc::Client;
use brk_types::{MempoolEntryInfo, Timestamp, Txid, VSize};
use parking_lot::RwLock;
use rustc_hash::FxHashSet;
use tracing::warn;

use crate::State;

/// Cap before the batch RPC so we never hand bitcoind an unbounded batch.
/// GBT-synthesized entries are not subject to this cap: they're bounded
/// by the block weight limit Core enforces on its own template.
const MAX_TX_FETCHES_PER_CYCLE: usize = 10_000;

/// Two batched round-trips per cycle, scaling with churn rather than
/// mempool size: `getblocktemplate` + `getrawmempool false` +
/// `getmempoolinfo` in one mixed batch, then `getmempoolentry` +
/// `getrawtransaction` for *new* non-GBT txids in a second mixed batch.
///
/// GBT entries already carry the full tx body and stats, so any GBT tx
/// not yet in the local pool is materialized inline from the GBT
/// payload instead of being refetched. That removes the GBT/listing
/// race that used to skip cycles when a tx vanished from the mempool
/// between the GBT and `getrawmempool` calls: block 0 always reflects
/// Core's exact selection because we never ask for that data twice.
///
/// Confirmed prevouts are resolved post-apply by the caller-supplied
/// resolver passed to `Mempool::tick_with`, so the in-crate path no
/// longer issues a third batch for parents.
pub struct Fetcher;

impl Fetcher {
    pub fn fetch(client: &Client, lock: &RwLock<State>) -> brk_error::Result<Fetched> {
        let (mut state, block_template) = client.fetch_mempool_state()?;

        // One read snapshot decides both the RPC fetch list and the
        // GBT-synthesis set, so they agree on what's "already known".
        let (new_txids, gbt_synth_set) = {
            let mempool = lock.read();
            let mut gbt_txids: FxHashSet<Txid> =
                FxHashSet::with_capacity_and_hasher(block_template.len(), Default::default());
            let mut gbt_synth_set: FxHashSet<Txid> = FxHashSet::default();
            for g in &block_template {
                gbt_txids.insert(g.txid);
                if !mempool.txs.contains(&g.txid) {
                    gbt_synth_set.insert(g.txid);
                }
            }
            let new_txids: Vec<Txid> = state
                .live_txids
                .iter()
                .filter(|t| !mempool.txs.contains(t) && !gbt_txids.contains(t))
                .take(MAX_TX_FETCHES_PER_CYCLE)
                .copied()
                .collect();
            if new_txids.len() == MAX_TX_FETCHES_PER_CYCLE {
                warn!(
                    cap = MAX_TX_FETCHES_PER_CYCLE,
                    "Fetcher: new-tx batch hit the per-cycle cap; remainder defers to the next cycle"
                );
            }
            (new_txids, gbt_synth_set)
        };

        let (mut new_entries, mut new_txs) = client.fetch_new_pool_data(&new_txids)?;
        new_entries.reserve(gbt_synth_set.len());
        new_txs.reserve(gbt_synth_set.len());

        // Consume `block_template` by value: GBT-only txs move their
        // body and depends into the synthesis path (no clones), and
        // the GBT ordering is captured as a `Vec<Txid>` for the
        // Rebuilder, which is the only downstream consumer and only
        // reads txids.
        //
        // GBT carries no per-tx arrival timestamp. `now` is correct to
        // within ~1 cycle for a tx that just entered Core's mempool
        // (the only kind that triggers synthesis: not in our pool yet
        // means it just appeared this cycle).
        let now = Timestamp::now();
        let block_template_txids: Vec<Txid> = block_template
            .into_iter()
            .map(|g| {
                let txid = g.txid;
                if gbt_synth_set.contains(&txid) {
                    new_entries.push(MempoolEntryInfo {
                        txid,
                        vsize: VSize::from(g.weight),
                        weight: g.weight,
                        fee: g.fee,
                        first_seen: now,
                        depends: g.depends,
                    });
                    new_txs.insert(txid, g.tx);
                }
                txid
            })
            .collect();

        // Promote `live_txids` to the union of `getrawmempool` and GBT:
        // the two RPC views can disagree by a cycle, so a tx visible to
        // GBT but missing from `getrawmempool` (or vice versa) is still
        // alive. Without the union, GBT-only txs would oscillate enter ↔
        // leave every cycle as `Preparer::classify_removals` buried what
        // GBT had just resurrected.
        state
            .live_txids
            .extend(block_template_txids.iter().copied());

        Ok(Fetched {
            state,
            new_entries,
            new_txs,
            block_template_txids,
        })
    }
}
