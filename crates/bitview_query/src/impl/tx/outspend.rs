use brk_error::{OptionData, Result};
use brk_types::{
    BlockHash, Height, Timestamp, TxInIndex, TxIndex, TxOutIndex, TxOutspend, TxStatus, Txid, Vin,
    Vout,
};
use vecdb::{ReadableVec, VecIndex};

use crate::{Query, RepresentationId};

#[cfg(test)]
use crate::representation_id::content_hash;

impl Query {
    pub fn outspend(&self, txid: &Txid, vout: Vout) -> Result<TxOutspend> {
        if let Some(outspend) = self
            .mempool()
            .and_then(|mempool| mempool.outspend_if_present(txid, vout))
        {
            return Ok(outspend);
        }

        let _guard = self.read_plugin(self.plugins().outputs)?;
        let (_, first_txout, output_count) = self.resolve_tx_outputs(txid)?;
        if usize::from(vout) >= output_count {
            return Ok(TxOutspend::UNSPENT);
        }
        let confirmed = self.resolve_outspend(first_txout + vout)?;
        if confirmed.spent {
            return Ok(confirmed);
        }
        Ok(self.mempool_outspend(txid, vout))
    }

    /// Resolve and serialize one outspend exactly once, identifying confirmed
    /// results by their spending block and live results by their content.
    pub fn outspend_json(&self, txid: &Txid, vout: Vout) -> Result<(Vec<u8>, RepresentationId)> {
        let outspend = self.outspend(txid, vout)?;
        let bytes = serde_json::to_vec(&outspend).unwrap();
        let identity = outspend_identity(&outspend, &bytes);
        Ok((bytes, identity))
    }

    pub fn outspends(&self, txid: &Txid) -> Result<Vec<TxOutspend>> {
        if let Some(outspends) = self
            .mempool()
            .and_then(|mempool| mempool.outspends_if_present(txid))
        {
            return Ok(outspends);
        }

        let _guard = self.read_plugin(self.plugins().outputs)?;
        let (_, first_txout, output_count) = self.resolve_tx_outputs(txid)?;
        let mut outspends = self.resolve_outspends(first_txout, output_count)?;
        if let Some(mempool) = self.mempool() {
            mempool.merge_outspends(txid, &mut outspends);
        }
        Ok(outspends)
    }

    /// Resolve and serialize all outspends exactly once. Fully confirmed
    /// arrays are identified by their newest spending block; arrays with any
    /// live element are identified by their exact content.
    pub fn outspends_json(&self, txid: &Txid) -> Result<(Vec<u8>, RepresentationId)> {
        let outspends = self.outspends(txid)?;
        let bytes = serde_json::to_vec(&outspends).unwrap();
        let identity = outspends_identity(&outspends, &bytes);
        Ok((bytes, identity))
    }

    pub(super) fn mempool_outspend(&self, txid: &Txid, vout: Vout) -> TxOutspend {
        self.mempool()
            .map_or(TxOutspend::UNSPENT, |mempool| mempool.outspend(txid, vout))
    }

    /// Resolve spend status for a single output. Minimal reads.
    fn resolve_outspend(&self, txout_index: TxOutIndex) -> Result<TxOutspend> {
        let txin_index = self
            .plugins()
            .outputs
            .spent
            .txin_index
            .collect_one(txout_index)
            .data()?;

        if txin_index == TxInIndex::UNSPENT {
            return Ok(TxOutspend::UNSPENT);
        }

        self.build_outspend(txin_index)
    }

    /// Build a single TxOutspend from a known-spent TxInIndex.
    fn build_outspend(&self, txin_index: TxInIndex) -> Result<TxOutspend> {
        let indexer = self.indexer();
        let spending_tx_index: TxIndex = indexer
            .vecs()
            .inputs
            .tx_index
            .collect_one(txin_index)
            .data()?;
        let spending_first_txin: TxInIndex = indexer
            .vecs()
            .transactions
            .first_txin_index
            .collect_one(spending_tx_index)
            .data()?;
        let vin = Vin::from(usize::from(txin_index) - usize::from(spending_first_txin));
        let spending_txid = indexer
            .vecs()
            .transactions
            .txid
            .collect_one(spending_tx_index)
            .data()?;
        let spending_height = self.confirmed_status_height(spending_tx_index)?;
        let (block_hash, block_time) = self.block_hash_and_time(spending_height)?;

        Ok(TxOutspend {
            spent: true,
            txid: Some(spending_txid),
            vin: Some(vin),
            status: Some(TxStatus::confirmed(spending_height, block_hash, block_time)),
        })
    }

    /// Resolve spend status for a contiguous range of outputs.
    /// Readers/cursors created once, reused for all outputs.
    fn resolve_outspends(
        &self,
        first_txout: TxOutIndex,
        output_count: usize,
    ) -> Result<Vec<TxOutspend>> {
        let indexer = self.indexer();
        let txin_index_reader = self.plugins().outputs.spent.txin_index.reader();
        let txid_reader = indexer.vecs().transactions.txid.reader();

        let tx_heights = &self.plugins().mappings.tx_heights;
        let mut input_tx_cursor = indexer.vecs().inputs.tx_index.cursor();
        let mut first_txin_cursor = indexer.vecs().transactions.first_txin_index.cursor();

        let bound = self.safe_lengths();

        let mut cached_status: Option<(Height, BlockHash, Timestamp)> = None;
        let mut outspends = Vec::with_capacity(output_count);
        for index in 0..output_count {
            let txin_index = txin_index_reader.get(first_txout + Vout::from(index));

            if txin_index == TxInIndex::UNSPENT {
                outspends.push(TxOutspend::UNSPENT);
                continue;
            }

            let spending_tx_index = input_tx_cursor.get(usize::from(txin_index)).data()?;
            if spending_tx_index >= bound.tx_index {
                outspends.push(TxOutspend::UNSPENT);
                continue;
            }
            let spending_first_txin = first_txin_cursor.get(spending_tx_index.to_usize()).data()?;
            let vin = Vin::from(usize::from(txin_index) - usize::from(spending_first_txin));
            let spending_txid = txid_reader.get(spending_tx_index);
            let spending_height: Height = tx_heights.get_shared(spending_tx_index).data()?;

            let (block_hash, block_time) = if let Some((height, hash, time)) = cached_status
                && height == spending_height
            {
                (hash, time)
            } else {
                let (hash, time) = self.block_hash_and_time(spending_height)?;
                cached_status = Some((spending_height, hash, time));
                (hash, time)
            };

            outspends.push(TxOutspend {
                spent: true,
                txid: Some(spending_txid),
                vin: Some(vin),
                status: Some(TxStatus::confirmed(spending_height, block_hash, block_time)),
            });
        }

        Ok(outspends)
    }
}

fn outspend_identity(outspend: &TxOutspend, bytes: &[u8]) -> RepresentationId {
    confirmed_spending_block(outspend).map_or_else(
        || RepresentationId::content(bytes),
        |(hash, height)| RepresentationId::Block { hash, height },
    )
}

fn confirmed_spending_block(outspend: &TxOutspend) -> Option<(BlockHash, Height)> {
    if !outspend.spent {
        return None;
    }
    let status = outspend.status.as_ref()?;
    if !status.confirmed {
        return None;
    }
    status.block_hash.zip(status.block_height)
}

fn outspends_identity(outspends: &[TxOutspend], bytes: &[u8]) -> RepresentationId {
    let content = || RepresentationId::content(bytes);
    let mut newest = None;

    for outspend in outspends {
        let Some((hash, height)) = confirmed_spending_block(outspend) else {
            return content();
        };

        match newest {
            Some((newest_hash, newest_height)) if newest_height == height => {
                if newest_hash != hash {
                    return content();
                }
            }
            Some((_, newest_height)) if newest_height > height => {}
            _ => newest = Some((hash, height)),
        }
    }

    newest.map_or_else(content, |(hash, height)| RepresentationId::Block {
        hash,
        height,
    })
}

#[cfg(test)]
mod tests {
    use brk_types::{BlockHash, Height, Timestamp};

    use super::*;

    #[test]
    fn identity_is_content_based_until_a_spending_block_is_known() {
        let bytes = br#"{"spent":false}"#;
        assert!(matches!(
            outspend_identity(&TxOutspend::UNSPENT, bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));

        let unconfirmed = TxOutspend {
            spent: true,
            txid: Some(Txid::COINBASE),
            vin: Some(Vin::from(0usize)),
            status: Some(TxStatus::UNCONFIRMED),
        };
        assert!(matches!(
            outspend_identity(&unconfirmed, bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));

        let hash = BlockHash::default();
        let height = Height::new(42);
        let confirmed = TxOutspend {
            status: Some(TxStatus::confirmed(height, hash, Timestamp::ZERO)),
            ..unconfirmed
        };
        assert!(matches!(
            outspend_identity(&confirmed, bytes),
            RepresentationId::Block {
                hash: bound_hash,
                height: bound_height,
            } if bound_hash == hash && bound_height == height
        ));
    }

    #[test]
    fn array_identity_uses_content_until_every_spend_is_confirmed() {
        let bytes = br#"[{"spent":false}]"#;
        assert!(matches!(
            outspends_identity(&[], bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));
        assert!(matches!(
            outspends_identity(&[TxOutspend::UNSPENT], bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));

        let unconfirmed = TxOutspend {
            spent: true,
            txid: Some(Txid::COINBASE),
            vin: Some(Vin::from(0usize)),
            status: Some(TxStatus::UNCONFIRMED),
        };
        assert!(matches!(
            outspends_identity(&[unconfirmed], bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));

        let confirmed = TxOutspend {
            spent: true,
            txid: Some(Txid::COINBASE),
            vin: Some(Vin::from(0usize)),
            status: Some(TxStatus::confirmed(
                Height::new(10),
                BlockHash::default(),
                Timestamp::ZERO,
            )),
        };
        assert!(matches!(
            outspends_identity(&[confirmed, TxOutspend::UNSPENT], bytes),
            RepresentationId::Content(hash) if hash == content_hash(bytes)
        ));
    }

    #[test]
    fn array_identity_uses_the_newest_confirmed_spending_block() {
        let older_hash = BlockHash::default();
        let newer_hash =
            BlockHash::try_from("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let confirmed = |hash, height| TxOutspend {
            spent: true,
            txid: Some(Txid::COINBASE),
            vin: Some(Vin::from(0usize)),
            status: Some(TxStatus::confirmed(height, hash, Timestamp::ZERO)),
        };
        let outspends = [
            confirmed(newer_hash, Height::new(20)),
            confirmed(older_hash, Height::new(10)),
        ];

        assert!(matches!(
            outspends_identity(&outspends, b"ignored"),
            RepresentationId::Block { hash, height }
                if hash == newer_hash && height == Height::new(20)
        ));
    }
}
