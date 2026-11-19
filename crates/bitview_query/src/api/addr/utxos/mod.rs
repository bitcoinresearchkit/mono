use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Lengths, TxOutIndex};
use bitview_types::Utxo;
use brk_mempool::PendingUtxos;
use brk_types::{Address, BlockHash, Height, Sats, TxIndex, TxStatus, Txid, Vout};

use crate::{Error, OptionData, Query, Result, api::addr::parse_addr};

// Keep a few independent reads in flight without filling the kernel's advice queue.
const READ_BATCH: usize = 32;

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
                // until one past the cap.
                let txids = self.indexer().vecs().transactions.txid.reader();
                let mut spends = pending.spends();
                let mut outpoints = Vec::new();
                let mut unspent = pin
                    .addr_unspent_outpoints(output_type, type_index)
                    .ok_or_else(|| Error::updating())?;
                let mut batch = Vec::with_capacity(READ_BATCH);
                while outpoints.len() <= max_utxos {
                    let count = (max_utxos - outpoints.len()).saturating_add(1).min(READ_BATCH);
                    batch.clear();
                    for outpoint in unspent.by_ref().take(count) {
                        batch.push(outpoint?);
                    }
                    if batch.is_empty() {
                        break;
                    }
                    if batch.len() > 1 {
                        txids.prefetch(batch.iter().map(|&(tx, _)| tx));
                    }
                    for &(tx_index, vout) in &batch {
                        let txid = txids.try_get(tx_index).data()?;
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
        let outputs = self.unspent_outputs(&outpoints, lengths)?;

        // Statuses read other vectors: the raw readers above are released by now.
        let mut cached_status: Option<(Height, TxStatus)> = None;
        let mut utxos = Vec::with_capacity(outputs.len() + pending.created.len());
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        let hashes = &self.indexer().vecs().blocks.blockhash;
        let mut heights = [Height::ZERO; READ_BATCH];
        for batch in outputs.chunks(READ_BATCH) {
            let heights = &mut heights[..batch.len()];
            for (&(tx, _, _, _), height) in batch.iter().zip(&mut *heights) {
                *height = tx_heights.get(tx).data()?;
            }
            if batch.len() > 1 {
                hashes.reader().prefetch(heights.iter().copied());
            }
            for (&(_, txid, vout, value), &height) in batch.iter().zip(heights.iter()) {
                let status = if let Some((h, ref s)) = cached_status
                    && h == height
                {
                    s.clone()
                } else {
                    let s = self.confirmed_status_at_bounded(height, lengths)?;
                    cached_status = Some((height, s.clone()));
                    s
                };
                utxos.push(Utxo { txid, vout, status, value });
            }
        }
        utxos.extend(pending.created.into_iter().map(|(txid, vout, value)| Utxo {
            txid,
            vout,
            status: TxStatus::UNCONFIRMED,
            value,
        }));
        Ok(utxos)
    }

    /// The confirmed outpoints' values.
    fn unspent_outputs(
        &self,
        outpoints: &[(TxIndex, Txid, Vout)],
        lengths: Lengths,
    ) -> Result<Vec<(TxIndex, Txid, Vout, Sats)>> {
        let vecs = self.indexer().vecs();
        let ranges = self.indexer().output_ranges(lengths);
        let value_reader = vecs.outputs.value.reader();
        let mut outputs = Vec::with_capacity(outpoints.len());
        let mut indexes = [TxOutIndex::ZERO; READ_BATCH];
        for batch in outpoints.chunks(READ_BATCH) {
            if batch.len() > 1 {
                ranges.prefetch(batch.iter().map(|&(tx, _, _)| tx));
            }
            let indexes = &mut indexes[..batch.len()];
            for (&(tx, _, vout), output) in batch.iter().zip(&mut *indexes) {
                *output = ranges
                    .get(tx)
                    .and_then(|outputs| outputs.get(vout))
                    .ok_or(Error::Internal("UTXO outside transaction output boundaries"))?;
            }
            if batch.len() > 1 {
                value_reader.prefetch(indexes.iter().copied());
            }
            for (&(tx, txid, vout), &output) in batch.iter().zip(indexes.iter()) {
                let value = value_reader.try_get(output).data()?;
                outputs.push((tx, txid, vout, value));
            }
        }
        Ok(outputs)
    }
}
