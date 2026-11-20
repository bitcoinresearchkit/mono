use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Lengths, TxOutIndex};
use bitview_types::Utxo;
use brk_mempool::PendingUtxos;
use brk_types::{Address, BlockHash, Height, Sats, TxIndex, TxStatus, Txid, Vout};
use vecdb::VecIndex;

use crate::{Error, OptionData, Query, Result, api::addr::parse_addr, read_sorted};

/// An owned UTXO selection retaining only rollback protection until consumed.
pub struct ResolvedAddrUtxos {
    pin: SafeLengths,
    /// The confirmed outpoints no pending transaction spends.
    outpoints: Vec<(TxIndex, Txid, Vout)>,
    pending: PendingUtxos,
    anchor: BlockHash,
    /// The pending transactions' part ([`brk_mempool::Spends::token`]).
    token: u64,
}

impl ResolvedAddrUtxos {
    /// Identifies the selection: the last confirmed activity, the address's pending transactions
    /// and what spends its confirmed outputs.
    pub fn representation(&self) -> String {
        format!("{}-{:x}", self.anchor, self.token)
    }
}

impl Query {
    pub fn resolve_addr_utxos(
        &self,
        addr: &Address,
        max_utxos: usize,
    ) -> Result<ResolvedAddrUtxos> {
        let addr = &parse_addr(addr)?;
        let (mempool, pin) = self.mempool_then_pin()?;
        let lengths = pin.lengths();
        let found = self.find_addr_bytes(addr, &pin)?;
        let tip = pin.tip();
        let pending = self.with_pending(mempool.as_deref(), &pin, |mempool, view| {
            mempool.addr_pending_utxos(addr, max_utxos, view)
        })?;
        let (outpoints, anchor, token) = match found {
            Some((output_type, type_index)) => {
                let last_tx = self
                    .addr_tx_history(output_type, type_index, None, &pin)?
                    .next_back()
                    .transpose()?
                    .ok_or(Error::UnknownAddress)?;
                let height = self.confirmed_status_height_bounded(last_tx, lengths)?;
                // The unspent set as of the pin (unavailable while a lowered publication sits
                // below what the stores hold), without what pending transactions spend, read
                // until one past the cap: each round reads as many as the cap still allows, their
                // txids in one batch.
                let txids = &self.indexer().vecs().transactions.txid;
                let mut spends = pending.spends();
                let mut outpoints = Vec::new();
                let mut unspent = pin
                    .addr_unspent_outpoints(output_type, type_index)
                    .ok_or_else(|| Error::updating())?;
                while outpoints.len() <= max_utxos {
                    let count = (max_utxos - outpoints.len()).saturating_add(1);
                    let batch = unspent
                        .by_ref()
                        .take(count)
                        .collect::<brk_error::Result<Vec<_>>>()?;
                    if batch.is_empty() {
                        break;
                    }
                    let rows: Vec<usize> = batch.iter().map(|&(tx, _)| tx.to_usize()).collect();
                    let read = read_sorted(txids, &rows)?;
                    for ((tx_index, vout), txid) in batch.into_iter().zip(read) {
                        if !spends.spent(&txid, vout) {
                            outpoints.push((tx_index, txid, vout));
                        }
                    }
                }
                let token = spends.token();
                (outpoints, self.block_hash_by_height(height, &pin)?, token)
            }
            // Never confirmed: its pending outputs only, as Esplora answers.
            None => (Vec::new(), tip, pending.token),
        };
        if outpoints.len() + pending.created.len() > max_utxos {
            return Err(Error::TooManyUtxos { max: max_utxos });
        }
        Ok(ResolvedAddrUtxos {
            pin: pin.into_inner(),
            outpoints,
            pending,
            anchor,
            token,
        })
    }

    /// Load the captured selection from its pinned immutable prefix.
    pub fn addr_utxos_resolved(&self, resolved: ResolvedAddrUtxos) -> Result<Vec<Utxo>> {
        let ResolvedAddrUtxos {
            pin,
            outpoints,
            pending,
            ..
        } = resolved;
        let lengths = pin.lengths();
        let values = self.unspent_values(&outpoints, lengths)?;
        // Outpoints in transaction order: their heights are non-decreasing.
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        let heights = outpoints
            .iter()
            .map(|&(tx, ..)| tx_heights.get(tx).data())
            .collect::<Result<Vec<Height>>>()?;
        let statuses = self.confirmed_statuses_bounded(&heights, lengths)?;

        let confirmed = outpoints.into_iter().zip(values).zip(statuses).map(
            |(((_, txid, vout), value), status)| Utxo {
                txid,
                vout,
                status,
                value,
            },
        );
        let pending = pending.created.into_iter().map(|(txid, vout, value)| Utxo {
            txid,
            vout,
            status: TxStatus::UNCONFIRMED,
            value,
        });
        Ok(confirmed.chain(pending).collect())
    }

    /// The confirmed outpoints' values, in their (transaction) order: their output indexes are
    /// sorted too, so the values are one batch read.
    fn unspent_values(
        &self,
        outpoints: &[(TxIndex, Txid, Vout)],
        lengths: Lengths,
    ) -> Result<Vec<Sats>> {
        let ranges = self.indexer().output_ranges(lengths);
        ranges.prefetch(outpoints.iter().map(|&(tx, ..)| tx));
        let rows = outpoints
            .iter()
            .map(|&(tx, _, vout)| {
                ranges
                    .get(tx)
                    .and_then(|outputs| outputs.get(vout))
                    .map(TxOutIndex::to_usize)
                    .ok_or(Error::Internal(
                        "UTXO outside transaction output boundaries",
                    ))
            })
            .collect::<Result<Vec<usize>>>()?;
        read_sorted(&self.indexer().vecs().outputs.value, &rows)
    }
}
