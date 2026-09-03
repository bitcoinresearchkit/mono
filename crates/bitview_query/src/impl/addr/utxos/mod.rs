use std::str::FromStr;

use brk_error::{Error, Result};
use brk_types::{
    Addr, AddrBytes, BlockHash, Height, OutputType, TxIndex, TxStatus, TypeIndex, Utxo, Vout,
};

use crate::{
    Query,
    r#impl::tx::{confirmed_status_at, confirmed_status_height},
};

mod resolved;

pub use resolved::ResolvedAddrUtxos;

impl Query {
    pub fn addr_utxos(&self, addr: Addr, max_utxos: usize) -> Result<Vec<Utxo>> {
        let _guard = self.read_plugin(self.indexer())?;
        let addr = AddrBytes::from_str(&addr)?;
        let (output_type, type_index) = super::resolve::resolve_addr_bytes(self, &addr)?;
        self.addr_utxos_for(output_type, type_index, max_utxos)
    }

    /// Resolve one address and its exact chain snapshot without waiting if the
    /// indexer is updating. The server falls back to the guarded query path
    /// when this returns `None`.
    pub fn addr_utxos_preflight(&self, addr: &Addr) -> Result<Option<ResolvedAddrUtxos>> {
        let addr = AddrBytes::from_str(addr)?;
        let Some(_guard) = self.try_read_plugin(self.indexer()) else {
            return Ok(None);
        };
        let (output_type, type_index) = super::resolve::resolve_addr_bytes(self, &addr)?;
        let anchor = self.addr_utxos_anchor_for(output_type, type_index)?;
        let tip = self.tip_blockhash();
        Ok(Some(ResolvedAddrUtxos::new(
            addr,
            output_type,
            type_index,
            anchor,
            tip,
        )))
    }

    /// Load UTXOs for a resolved address and return its exact block identity for
    /// the loaded representation. A tip change re-resolves only the potentially
    /// stale identity and activity anchor, never the original address text.
    pub fn addr_utxos_resolved(
        &self,
        resolved: ResolvedAddrUtxos,
        max_utxos: usize,
    ) -> Result<(Vec<Utxo>, BlockHash)> {
        let _guard = self.read_plugin(self.indexer())?;
        let (addr, output_type, type_index, (height, hash), tip) = resolved.into_parts();
        let (output_type, type_index, block_hash) = if self.tip_blockhash() == tip {
            self.validate_block_at_height(&hash, height)?;
            (output_type, type_index, hash)
        } else {
            let (output_type, type_index) = super::resolve::resolve_addr_bytes(self, &addr)?;
            let (_, hash) = self.addr_utxos_anchor_for(output_type, type_index)?;
            (output_type, type_index, hash)
        };
        let utxos = self.addr_utxos_for(output_type, type_index, max_utxos)?;
        Ok((utxos, block_hash))
    }

    fn addr_utxos_anchor_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> Result<(Height, BlockHash)> {
        let height = self.addr_last_activity_height_for(output_type, type_index, None)?;
        let hash = self.block_hash_by_height(height)?;
        Ok((height, hash))
    }

    fn addr_utxos_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        max_utxos: usize,
    ) -> Result<Vec<Utxo>> {
        let indexer = self.indexer();
        let stores = indexer.stores();
        let vecs = indexer.vecs();

        let tx_index_len = self.safe_lengths().tx_index;
        let outpoints: Vec<(TxIndex, Vout)> = stores
            .addr_unspent_outpoints(output_type, type_index)?
            .filter(|(tx_index, _)| *tx_index < tx_index_len)
            .take(max_utxos.saturating_add(1))
            .collect();
        if outpoints.len() > max_utxos {
            return Err(Error::TooManyUtxos);
        }

        let txid_reader = vecs.transactions.txid.reader();
        let first_txout_index_reader = vecs.transactions.first_txout_index.reader();
        let value_reader = vecs.outputs.value.reader();

        let mut cached_status: Option<(Height, TxStatus)> = None;
        let mut utxos = Vec::with_capacity(outpoints.len());

        for (tx_index, vout) in outpoints {
            let txid = txid_reader.get(tx_index);
            let first_txout_index = first_txout_index_reader.get(tx_index);
            let value = value_reader.get(first_txout_index + vout);

            let height = confirmed_status_height(self, tx_index)?;
            let status = if let Some((h, ref s)) = cached_status
                && h == height
            {
                s.clone()
            } else {
                let s = confirmed_status_at(self, height)?;
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

        Ok(utxos)
    }
}
