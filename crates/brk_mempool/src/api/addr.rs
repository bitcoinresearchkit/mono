use std::sync::Arc;

use brk_error::{Error, Result};
use brk_types::{AddressBytes, AddressMempoolStats, Sats, TxOut, Txid, TxidPrefix, Vout};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    PendingView, PublishedTx, ReadOnlyState,
    key::{OutpointKey, TxKey},
    state::Pool,
    stores::addr_tracker::{AddrEntry, AddrTx},
};

/// An address's pending UTXO changes in one completed publication.
#[derive(Default)]
pub struct PendingUtxos {
    /// Outputs mempool transactions pay the address that no pending transaction spends, ordered
    /// by txid prefix and vout.
    pub created: Vec<(Txid, Vout, Sats)>,
    /// The most of the address's confirmed outputs pending transactions can spend: its own
    /// pending spends, and every input of a transaction still resolving.
    pub spends_at_most: usize,
    /// Identifies the address's mempool transactions, those still resolving (which can spend its
    /// outputs) and the reader's view of them: zero without any.
    pub token: u64,
    /// The publication read, as the reader sees it: what spends an outpoint.
    pool: Option<(Arc<Pool>, Arc<PendingView>)>,
}

impl PendingUtxos {
    /// Whether a pending transaction spends an outpoint, one still resolving included (the
    /// outpoint is known before what it pays). Each spender's inputs are read once, however many
    /// of the outpoints asked about it spends.
    pub fn spent_filter(&self) -> impl FnMut(&Txid, Vout) -> bool + '_ {
        let mut spends = SpendFilter::default();
        move |txid, vout| {
            self.pool
                .as_ref()
                .is_some_and(|(pool, view)| spends.spent(pool, view, txid, vout))
        }
    }
}

/// The pending spenders met so far, each with the outpoints its inputs spend (none for one the
/// reader's view settled).
#[derive(Default)]
struct SpendFilter(FxHashMap<TxKey, Option<FxHashSet<(Txid, Vout)>>>);

impl SpendFilter {
    fn spent(&mut self, pool: &Pool, view: &PendingView, txid: &Txid, vout: Vout) -> bool {
        let Some(key) = pool
            .outpoint_spends
            .get(&OutpointKey::new(TxKey::from(txid), vout))
        else {
            return false;
        };
        // The stored key can collide: the spender's own inputs decide.
        self.0
            .entry(key)
            .or_insert_with(|| {
                pool.txs
                    .record_by_key(&key)
                    .filter(|spender| view.pending(&spender.entry.txid))
                    .map(|spender| {
                        spender
                            .tx
                            .input
                            .iter()
                            .map(|input| (input.txid, input.vout))
                            .collect()
                    })
            })
            .as_ref()
            .is_some_and(|inputs| inputs.contains(&(*txid, vout)))
    }
}

impl ReadOnlyState {
    /// `addr`'s entry, none without pending transactions: `StateUpdating` while one paying it is
    /// still resolving (its view isn't complete yet), unless a newer block settled it.
    fn addr_entry<'a>(
        state: &'a Pool,
        addr: &AddressBytes,
        view: &PendingView,
    ) -> Result<Option<&'a AddrEntry>> {
        if state.addrs.awaited(addr).any(|key| view.pending_key(key)) {
            return Err(Error::StateUpdating);
        }
        Ok(state.addrs.get(addr))
    }

    /// The address's pending statistics.
    pub fn addr_stats(&self, addr: &AddressBytes, view: &PendingView) -> Result<AddressMempoolStats> {
        let state = self.pool()?.at(view.tip())?;
        let Some(entry) = Self::addr_entry(state, addr, view)? else {
            return Ok(AddressMempoolStats::default());
        };
        if view.settles_nothing() || entry.txs.iter().all(|tx| view.pending_key(&tx.key)) {
            return Ok(entry.stats.clone());
        }
        // Folded again without what the newer blocks settled.
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        let mut stats = AddressMempoolStats::default();
        for tx in entry.txs.iter().filter(|tx| view.pending_key(&tx.key)) {
            let Some(record) = state.txs.record_by_key(&tx.key) else {
                continue;
            };
            for prevout in record.tx.input.iter().filter_map(|txin| txin.prevout.as_ref()) {
                if pays(prevout) {
                    stats.add_spend(prevout);
                }
            }
            for txout in record.tx.output.iter().filter(|txout| pays(txout)) {
                stats.add_funding(txout);
            }
            stats.tx_count += 1;
        }
        Ok(stats)
    }

    /// The address's unspent pending outputs, and what spends its outputs.
    pub fn addr_pending_utxos(
        &self,
        addr: &AddressBytes,
        view: &Arc<PendingView>,
    ) -> Result<PendingUtxos> {
        let pool = self.pool_arc()?;
        let state = pool.at(view.tip())?;
        let mut pending = PendingUtxos {
            spends_at_most: state.addrs.waiting_inputs(),
            token: state.addrs.waiting_token() ^ view.identity(),
            pool: Some((pool.clone(), view.clone())),
            ..PendingUtxos::default()
        };
        let Some(entry) = Self::addr_entry(state, addr, view)? else {
            return Ok(pending);
        };
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        for at in entry.txs.iter().filter(|tx| view.pending_key(&tx.key)) {
            let Some(record) = state.txs.record_by_key(&at.key) else {
                continue;
            };
            pending.token ^= *at.key;
            let tx = &record.tx;
            pending.spends_at_most += tx
                .input
                .iter()
                .filter(|txin| txin.prevout.as_ref().is_some_and(pays))
                .count();
            for (vout, txout) in tx.output.iter().enumerate() {
                if pays(txout) {
                    pending
                        .created
                        .push((tx.txid, Vout::from(vout), txout.value));
                }
            }
        }
        let mut spends = SpendFilter::default();
        pending
            .created
            .retain(|(txid, vout, _)| !spends.spent(state, view, txid, *vout));
        pending
            .created
            .sort_unstable_by_key(|(txid, vout, _)| (TxidPrefix::from(txid), *vout));
        Ok(pending)
    }

    /// Whether `txid` is pending in `view`.
    pub fn is_pending(&self, txid: &Txid, view: &PendingView) -> Result<bool> {
        let state = self.pool()?.at(view.tip())?;
        Ok(view.pending(txid) && state.txs.contains(txid))
    }

    /// Whether the pending tx `txid` touches `addr`: `StateUpdating` while it waits to fold in
    /// (what it touches isn't known yet).
    pub fn addr_has_tx(&self, addr: &AddressBytes, txid: &Txid, view: &PendingView) -> Result<bool> {
        let state = self.pool()?.at(view.tip())?;
        if view.pending(txid) && state.addrs.is_waiting(txid) {
            return Err(Error::StateUpdating);
        }
        let Some(record) = state.txs.record(txid).filter(|_| view.pending(txid)) else {
            return Ok(false);
        };
        Ok(Self::addr_entry(state, addr, view)?
            .is_some_and(|entry| entry.txs.contains(&AddrTx::from(record))))
    }

    /// Pending txs touching `addr`, newest first by `first_seen` (ties by txid prefix, then key),
    /// after the `after` cursor (older than it) or from the newest, capped at `limit`. A cursor
    /// that isn't one of the address's transactions has none after it. O(log n + the page) in
    /// the address's pending transactions (more past the chain's newer blocks: those settled are
    /// skipped). Shares immutable bodies from one completed publication; later prevout fills use
    /// copy-on-write and cannot mutate this selection.
    pub fn addr_txs(
        &self,
        addr: &AddressBytes,
        after: Option<&Txid>,
        limit: usize,
        view: &PendingView,
    ) -> Result<Vec<PublishedTx>> {
        let state = self.pool()?.at(view.tip())?;
        let Some(entry) = Self::addr_entry(state, addr, view)? else {
            return Ok(Vec::new());
        };
        let cursor = match after {
            None => None,
            Some(txid) => match state.txs.record(txid).filter(|_| view.pending(txid)) {
                Some(record) => Some(AddrTx::from(record)),
                None => return Ok(Vec::new()),
            },
        };
        let Some(newest) = entry.txs.newest_before(cursor.as_ref()) else {
            return Ok(Vec::new());
        };
        Ok(newest
            .filter(|tx| view.pending_key(&tx.key))
            .filter_map(|tx| state.txs.record_by_key(&tx.key))
            .take(limit)
            .map(PublishedTx::from_record)
            .collect())
    }
}
