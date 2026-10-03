use std::str::FromStr;

use bitview_plugin::PublicationReadGuard;
use bitview_plugin_indexer::SafeLengths;
use brk_error::{Error, OptionData, Result};
use brk_types::{Addr, AddrBytes, BlockHash, Height, TxIndex, TxOutIndex, TxStatus, Utxo, Vout};

use crate::Query;

/// An owned UTXO selection retaining only rollback protection until consumed.
pub struct ResolvedAddrUtxos {
    pin: SafeLengths,
    outpoints: Vec<(TxIndex, Vout)>,
    anchor: BlockHash,
}

impl ResolvedAddrUtxos {
    pub fn block_hash(&self) -> BlockHash {
        self.anchor
    }
}

impl Query {
    pub fn resolve_addr_utxos(&self, addr: &Addr, max_utxos: usize) -> Result<ResolvedAddrUtxos> {
        let addr = AddrBytes::from_str(addr)?;
        let guard = self.read_publication()?;
        self.resolve_addr_utxos_guarded(&addr, guard, max_utxos)
    }

    fn resolve_addr_utxos_guarded(
        &self,
        addr: &AddrBytes,
        guard: PublicationReadGuard,
        max_utxos: usize,
    ) -> Result<ResolvedAddrUtxos> {
        let (output_type, type_index) = self.resolve_addr_bytes(addr)?;
        let pin = self.pin_safe_lengths()?;
        let lengths = pin.lengths();
        let height =
            self.addr_last_activity_height_bounded(output_type, type_index, None, lengths)?;
        let anchor = self.block_hash_by_height(height, &pin)?;
        let outpoints: Vec<(TxIndex, Vout)> = self
            .indexer()
            .stores()
            .addr_unspent_outpoints(output_type, type_index)?
            // Store keys order outpoints by transaction index, then vout.
            .take_while(|(tx_index, _)| *tx_index < lengths.tx_index)
            .take(max_utxos.saturating_add(1))
            .collect();
        if outpoints.len() > max_utxos {
            return Err(Error::TooManyUtxos);
        }
        drop(guard);
        Ok(ResolvedAddrUtxos {
            pin,
            outpoints,
            anchor,
        })
    }

    /// Load the captured selection from its pinned immutable prefix.
    pub fn addr_utxos_resolved(
        &self,
        resolved: ResolvedAddrUtxos,
        max_utxos: usize,
    ) -> Result<(Vec<Utxo>, BlockHash)> {
        let ResolvedAddrUtxos {
            pin,
            outpoints,
            anchor: block_hash,
        } = resolved;
        let lengths = pin.lengths();
        let indexer = self.indexer();
        let vecs = indexer.vecs();
        if outpoints.len() > max_utxos {
            return Err(Error::TooManyUtxos);
        }

        let txid_reader = vecs.transactions.txid.reader();
        let first_txout_index_reader = vecs.transactions.first_txout_index.reader();
        let value_reader = vecs.outputs.value.reader();

        let mut cached_status: Option<(Height, TxStatus)> = None;
        let mut utxos = Vec::with_capacity(outpoints.len());

        for (tx_index, vout) in outpoints {
            let txid = txid_reader.try_get(tx_index).data()?;
            let first = first_txout_index_reader.try_get(tx_index).data()?;
            let next_tx = tx_index.incremented();
            let next = if next_tx < lengths.tx_index {
                first_txout_index_reader.try_get(next_tx).data()?
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

            let height = self.confirmed_status_height_bounded(tx_index, lengths)?;
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

        Ok((utxos, block_hash))
    }
}
