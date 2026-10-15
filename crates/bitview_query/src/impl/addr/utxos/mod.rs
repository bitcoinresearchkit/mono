use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Lengths, TxOutIndex};
use bitview_types::Utxo;
use brk_mempool::PendingUtxos;
use brk_types::{Address, BlockHash, Height, Sats, TxIndex, TxStatus, Txid, Vout};

use crate::{
    Error, OptionData, Query, Result,
    r#impl::{addr::parse_addr, mempool::anchored},
};

/// An owned UTXO selection retaining only rollback protection until consumed.
pub struct ResolvedAddrUtxos {
    pin: SafeLengths,
    outpoints: Vec<(TxIndex, Vout)>,
    pending: PendingUtxos,
    anchor: BlockHash,
}

impl ResolvedAddrUtxos {
    /// Identifies the selection: the last confirmed activity and the pending transactions.
    pub fn representation(&self) -> String {
        format!("{}-{:x}", self.anchor, self.pending.token)
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
        let pending = mempool
            .map(|mempool| anchored(mempool.addr_pending_utxos(addr, &tip)))
            .transpose()?
            .unwrap_or_default();
        let (outpoints, anchor) = match found {
            Some((output_type, type_index)) => {
                let height =
                    self.addr_last_activity_height_bounded(output_type, type_index, None, lengths)?;
                // The unspent set as of the pin: unavailable while a lowered publication sits
                // below what the stores hold.
                let outpoints: Vec<(TxIndex, Vout)> = pin
                    .addr_unspent_outpoints(output_type, type_index)
                    .ok_or(Error::StateUpdating)?
                    // Pending spends may remove some: the cap applies after them.
                    .take(
                        max_utxos
                            .saturating_add(pending.spent.len())
                            .saturating_add(1),
                    )
                    .collect::<brk_error::Result<_>>()?;
                (outpoints, self.block_hash_by_height(height, &pin)?)
            }
            // Never confirmed: its pending outputs only, as Esplora answers.
            None => (Vec::new(), tip),
        };
        if outpoints.len() + pending.created.len() > max_utxos + pending.spent.len() {
            return Err(Error::TooManyUtxos { max: max_utxos });
        }
        Ok(ResolvedAddrUtxos {
            pin: pin.into_inner(),
            outpoints,
            pending,
            anchor,
        })
    }

    /// Load the captured selection from its pinned immutable prefix.
    pub fn addr_utxos_resolved(
        &self,
        resolved: ResolvedAddrUtxos,
        max_utxos: usize,
    ) -> Result<Vec<Utxo>> {
        let ResolvedAddrUtxos {
            pin,
            outpoints,
            pending,
            ..
        } = resolved;
        let lengths = pin.lengths();
        let outputs = self.unspent_outputs(&outpoints, &pending, lengths)?;

        // Statuses read other vectors: the raw readers above are released by now.
        let mut cached_status: Option<(Height, TxStatus)> = None;
        let mut utxos = Vec::with_capacity(outputs.len() + pending.created.len());
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        for (tx_index, txid, vout, value) in outputs {
            if tx_index >= lengths.tx_index {
                return Err(Error::UnknownTxid);
            }
            let height: Height = tx_heights.get(tx_index).data()?;
            let status = if let Some((h, ref s)) = cached_status
                && h == height
            {
                s.clone()
            } else {
                let s = self.confirmed_status_at_bounded(height, lengths)?;
                cached_status = Some((height, s.clone()));
                s
            };
            utxos.push(Utxo {
                txid,
                vout,
                status,
                value,
            });
        }
        utxos.extend(pending.created.into_iter().map(|(txid, vout, value)| Utxo {
            txid,
            vout,
            status: TxStatus::UNCONFIRMED,
            value,
        }));
        if utxos.len() > max_utxos {
            return Err(Error::TooManyUtxos { max: max_utxos });
        }
        Ok(utxos)
    }

    /// The confirmed outpoints' txids and values, without those pending transactions spend.
    fn unspent_outputs(
        &self,
        outpoints: &[(TxIndex, Vout)],
        pending: &PendingUtxos,
        lengths: Lengths,
    ) -> Result<Vec<(TxIndex, Txid, Vout, Sats)>> {
        let vecs = self.indexer().vecs();
        let txid_reader = vecs.transactions.txid.reader();
        let first_txout_index_reader = vecs.transactions.first_txout_index.reader();
        let value_reader = vecs.outputs.value.reader();
        let mut outputs = Vec::with_capacity(outpoints.len());
        for &(tx_index, vout) in outpoints {
            let txid = txid_reader.try_get(tx_index).data()?;
            if pending.spent.contains(&(txid, vout)) {
                continue;
            }
            let first = first_txout_index_reader.try_get(tx_index).data()?.get();
            let next_tx = tx_index.incremented();
            let next = if next_tx < lengths.tx_index {
                first_txout_index_reader.try_get(next_tx).data()?.get()
            } else {
                lengths.txout_index
            };
            let output = usize::from(first)
                .checked_add(usize::from(vout))
                .ok_or(Error::Internal("UTXO output index overflow"))?;
            if first > next || next > lengths.txout_index || output >= usize::from(next) {
                return Err(Error::Internal(
                    "UTXO outside transaction output boundaries",
                ));
            }
            let value = value_reader.try_get(TxOutIndex::from(output)).data()?;
            outputs.push((tx_index, txid, vout, value));
        }
        Ok(outputs)
    }
}
