use std::{hash::Hasher, sync::Arc};

use brk_error::Result;
use brk_types::{AddressBytes, AddressMempoolStats, Sats, TxOut, Txid, TxidPrefix, Vout};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    PendingView, PublishedTx, ReadOnlyState, json,
    key::{OutpointKey, TxKey},
    state::Pool,
    stores::addr_tracker::AddrTx,
};

/// An address's pending UTXO changes in one completed publication.
#[derive(Default)]
pub struct PendingUtxos {
    /// Outputs mempool transactions pay the address that no pending transaction spends, ordered
    /// by txid prefix and vout.
    pub created: Vec<(Txid, Vout, Sats)>,
    /// Identifies the address's mempool transactions and the reader's view of them: zero
    /// without any. With the spenders of its confirmed outputs ([`Spends::token`]), it identifies
    /// its unspent outputs.
    pub token: u64,
    /// The publication read, as the reader sees it: what spends an outpoint.
    pool: Option<(Arc<Pool>, Arc<PendingView>)>,
}

impl PendingUtxos {
    /// What pending transactions spend of the address's confirmed outputs, asked one outpoint at
    /// a time.
    pub fn spends(&self) -> Spends<'_> {
        Spends {
            pending: self,
            filter: SpendFilter::default(),
            met: FxHashSet::default(),
        }
    }
}

/// The pending spenders of the outpoints asked about, met so far.
pub struct Spends<'a> {
    pending: &'a PendingUtxos,
    filter: SpendFilter,
    met: FxHashSet<TxKey>,
}

impl Spends<'_> {
    /// Whether a pending transaction spends `txid:vout`. A spender whose prevouts are still
    /// resolving counts, because its outpoints are known first. Each spender's inputs are read
    /// once, however many of the outpoints it spends.
    pub fn spent(&mut self, txid: &Txid, vout: Vout) -> bool {
        let Some((pool, view)) = &self.pending.pool else {
            return false;
        };
        let Some(spender) = self.filter.spender(pool, view, txid, vout) else {
            return false;
        };
        self.met.insert(spender);
        true
    }

    /// Identifies the unspent outputs read so far: the address's pending transactions, plus the
    /// spenders found. Once every outpoint was asked, that covers all of them.
    pub fn token(&self) -> u64 {
        self.met
            .iter()
            .fold(self.pending.token, |token, key| token ^ token_part(SPENDER, key))
    }
}

/// A transaction's role in an address's token: one of its own, or a spender of its outputs.
const OWN: u8 = 0;
const SPENDER: u8 = 1;

/// `key`'s contribution to a token, hashed per role so a transaction in both roles doesn't cancel
/// itself out.
fn token_part(role: u8, key: &TxKey) -> u64 {
    let mut hasher = json::keyed_hasher();
    hasher.write_u8(role);
    hasher.write_u64(**key);
    hasher.finish()
}

/// The pending spenders found so far, each with the outpoints its inputs spend. `None` for one the
/// reader's view settled.
#[derive(Default)]
struct SpendFilter(FxHashMap<TxKey, Option<FxHashSet<(Txid, Vout)>>>);

impl SpendFilter {
    /// The pending transaction spending `txid:vout`, if any.
    fn spender(
        &mut self,
        pool: &Pool,
        view: &PendingView,
        txid: &Txid,
        vout: Vout,
    ) -> Option<TxKey> {
        let key = pool
            .outpoint_spends
            .get(&OutpointKey::new(TxKey::from(txid), vout))?;
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
            .then_some(key)
    }
}

impl ReadOnlyState {
    /// The address's pending statistics, as far as known. A transaction's outputs count from its
    /// arrival, and each input once its prevout resolves.
    pub fn addr_stats(&self, addr: &AddressBytes, view: &PendingView) -> Result<AddressMempoolStats> {
        let state = self.pool()?.at(view.tip())?;
        let Some(entry) = state.addrs.get(addr) else {
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
            token: view.identity(),
            pool: Some((pool.clone(), view.clone())),
            ..PendingUtxos::default()
        };
        let Some(entry) = state.addrs.get(addr) else {
            return Ok(pending);
        };
        let pays = |txout: &TxOut| txout.addr_bytes().as_ref() == Some(addr);
        for at in entry.txs.iter().filter(|tx| view.pending_key(&tx.key)) {
            let Some(record) = state.txs.record_by_key(&at.key) else {
                continue;
            };
            pending.token ^= token_part(OWN, &at.key);
            let tx = &record.tx;
            for (vout, txout) in tx.output.iter().enumerate() {
                if pays(txout) {
                    pending
                        .created
                        .push((tx.txid, Vout::from(vout), txout.value));
                }
            }
        }
        // Drop outputs a pending transaction spends. Such a spender pays from the address's own
        // output, so it is one of the address's transactions and already in the token.
        let mut spends = SpendFilter::default();
        pending
            .created
            .retain(|(txid, vout, _)| spends.spender(state, view, txid, *vout).is_none());
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

    /// Whether the pending tx `txid` touches `addr`, as far as that is known (an input still
    /// unresolved touches none yet).
    pub fn addr_has_tx(&self, addr: &AddressBytes, txid: &Txid, view: &PendingView) -> Result<bool> {
        let state = self.pool()?.at(view.tip())?;
        let Some(record) = state.txs.record(txid).filter(|_| view.pending(txid)) else {
            return Ok(false);
        };
        Ok(state
            .addrs
            .get(addr)
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
        let Some(entry) = state.addrs.get(addr) else {
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
