use bitcoin::{
    hashes::{Hash, sha256d},
    hex::DisplayHex,
};
use brk_error::{Error, OptionData, Result};
use brk_plugin::Plugin;
use brk_types::{
    BlockHash, Height, MerkleProof, Timestamp, Transaction, TxInIndex, TxIndex, TxOutIndex,
    TxOutspend, TxStatus, Txid, TxidPrefix, Vin, Vout,
};
use vecdb::{ReadableVec, VecIndex};

use crate::Query;

impl Query {
    // ── Txid → TxIndex resolution (single source of truth) ─────────

    /// Resolve a txid to its internal TxIndex via prefix lookup.
    /// Raw store hit — caller should prefer [`Self::resolve_tx_index_bounded`]
    /// when subsequent reads dereference indexer/computer vecs by `tx_index`.
    /// Use this raw form only for "is this mined?" probes that don't deref
    /// derived data (mempool merge, cpfp fee-rate fall-through).
    #[inline]
    pub(crate) fn resolve_tx_index(&self, txid: &Txid) -> Result<TxIndex> {
        self.indexer()
            .stores()
            .tx_index(&TxidPrefix::from(txid))?
            .ok_or(Error::UnknownTxid)
    }

    /// `resolve_tx_index` clamped against the safe-lengths snapshot.
    /// Returns `UnknownTxid` for tx_indices the store knows but the snapshot
    /// has not yet covered. Use this from any path that will subsequently
    /// dereference indexer/computer vecs by `tx_index`.
    #[inline]
    pub(crate) fn resolve_tx_index_bounded(&self, txid: &Txid) -> Result<TxIndex> {
        let tx_index = self.resolve_tx_index(txid)?;
        if tx_index >= self.safe_lengths().tx_index {
            return Err(Error::UnknownTxid);
        }
        Ok(tx_index)
    }

    pub fn txid_by_index(&self, index: TxIndex) -> Result<Txid> {
        if index >= self.safe_lengths().tx_index {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        self.indexer()
            .vecs()
            .transactions
            .txid
            .collect_one(index)
            .ok_or_else(|| Error::OutOfRange("Transaction index out of range".into()))
    }

    /// Resolve a txid to (TxIndex, Height).
    pub fn resolve_tx(&self, txid: &Txid) -> Result<(TxIndex, Height)> {
        let tx_index = self.resolve_tx_index_bounded(txid)?;
        let height = self.confirmed_status_height(tx_index)?;
        Ok((tx_index, height))
    }

    // ── TxStatus construction (single source of truth) ─────────────

    /// Height for a confirmed tx_index via in-memory TxHeights lookup.
    /// Bounded against the safe-lengths snapshot so rejected tx_indices
    /// never dereference slots a concurrent writer might be populating.
    #[inline]
    pub(crate) fn confirmed_status_height(&self, tx_index: TxIndex) -> Result<Height> {
        let bound = self.safe_lengths();
        if tx_index >= bound.tx_index {
            return Err(Error::UnknownTxid);
        }
        self.computer()
            .indexes
            .tx_heights
            .get_shared(tx_index)
            .data()
    }

    /// Full confirmed TxStatus from a known height.
    #[inline]
    pub(crate) fn confirmed_status_at(&self, height: Height) -> Result<TxStatus> {
        let (block_hash, block_time) = self.block_hash_and_time(height)?;
        Ok(TxStatus::confirmed(height, block_hash, block_time))
    }

    /// Block hash + timestamp for a height (cached vecs, fast).
    #[inline]
    pub(crate) fn block_hash_and_time(&self, height: Height) -> Result<(BlockHash, Timestamp)> {
        let indexer = self.indexer();
        let hash = indexer.vecs().blocks.blockhash.collect_one(height).data()?;
        let time = indexer.vecs().blocks.timestamp.collect_one(height).data()?;
        Ok((hash, time))
    }

    // ── Transaction queries ────────────────────────────────────────

    /// Resolve a tx body: live mempool → indexer → `Vanished` tombstone.
    /// The tombstone fallback covers the race where a mined tx has been
    /// buried but `safe_lengths.tx_index` hasn't caught up. `Replaced`
    /// tombstones are excluded since they will never confirm.
    fn lookup_tx<R>(
        &self,
        txid: &Txid,
        f: impl Fn(&Transaction) -> R,
        indexed: impl FnOnce(TxIndex) -> Result<R>,
    ) -> Result<R> {
        if let Some(mempool) = self.mempool()
            && let Some(r) = mempool.with_tx(txid, &f)
        {
            return Ok(r);
        }
        match self.resolve_tx_index_bounded(txid) {
            Ok(idx) => indexed(idx),
            Err(Error::UnknownTxid) => self
                .mempool()
                .and_then(|m| m.with_vanished_tx(txid, &f))
                .ok_or(Error::UnknownTxid),
            Err(e) => Err(e),
        }
    }

    pub fn transaction(&self, txid: &Txid) -> Result<Transaction> {
        self.lookup_tx(txid, Transaction::clone, |idx| {
            self.transaction_by_index(idx)
        })
    }

    pub fn transaction_status(&self, txid: &Txid) -> Result<TxStatus> {
        if self.mempool().is_some_and(|m| m.contains_txid(txid)) {
            return Ok(TxStatus::UNCONFIRMED);
        }
        let (_, height) = self.resolve_tx(txid)?;
        self.confirmed_status_at(height)
    }

    pub fn transaction_raw(&self, txid: &Txid) -> Result<Vec<u8>> {
        self.lookup_tx(txid, Transaction::encode_bytes, |idx| {
            self.transaction_raw_by_index(idx)
        })
    }

    pub fn transaction_hex(&self, txid: &Txid) -> Result<String> {
        self.lookup_tx(
            txid,
            |tx| tx.encode_bytes().to_lower_hex_string(),
            |idx| self.transaction_hex_by_index(idx),
        )
    }

    // ── Outspend queries ───────────────────────────────────────────

    pub fn outspend(&self, txid: &Txid, vout: Vout) -> Result<TxOutspend> {
        if self.mempool().is_some_and(|m| m.contains_txid(txid)) {
            return Ok(self.mempool_outspend(txid, vout));
        }
        let _guard = self.computer().outputs.gate().read();
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

    pub fn outspends(&self, txid: &Txid) -> Result<Vec<TxOutspend>> {
        if let Some(mempool) = self.mempool()
            && let Some(output_count) = mempool.with_tx(txid, |tx| tx.output.len())
        {
            return Ok((0..output_count)
                .map(|i| self.mempool_outspend(txid, Vout::from(i)))
                .collect());
        }
        let _guard = self.computer().outputs.gate().read();
        let (_, first_txout, output_count) = self.resolve_tx_outputs(txid)?;
        let mut spends = self.resolve_outspends(first_txout, output_count)?;
        for (i, spend) in spends.iter_mut().enumerate() {
            if !spend.spent {
                *spend = self.mempool_outspend(txid, Vout::from(i));
            }
        }
        Ok(spends)
    }

    fn mempool_outspend(&self, txid: &Txid, vout: Vout) -> TxOutspend {
        let Some((spender_txid, vin)) = self.mempool().and_then(|m| m.lookup_spender(txid, vout))
        else {
            return TxOutspend::UNSPENT;
        };
        TxOutspend {
            spent: true,
            txid: Some(spender_txid),
            vin: Some(vin),
            status: Some(TxStatus::UNCONFIRMED),
        }
    }

    /// Resolve spend status for a single output. Minimal reads.
    fn resolve_outspend(&self, txout_index: TxOutIndex) -> Result<TxOutspend> {
        let txin_index = self
            .computer()
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

    /// Resolve spend status for a contiguous range of outputs.
    /// Readers/cursors created once, reused for all outputs.
    fn resolve_outspends(
        &self,
        first_txout: TxOutIndex,
        output_count: usize,
    ) -> Result<Vec<TxOutspend>> {
        let indexer = self.indexer();
        let txin_index_reader = self.computer().outputs.spent.txin_index.reader();
        let txid_reader = indexer.vecs().transactions.txid.reader();

        let tx_heights = &self.computer().indexes.tx_heights;
        let mut input_tx_cursor = indexer.vecs().inputs.tx_index.cursor();
        let mut first_txin_cursor = indexer.vecs().transactions.first_txin_index.cursor();

        let bound = self.safe_lengths();

        let mut cached_status: Option<(Height, BlockHash, Timestamp)> = None;
        let mut outspends = Vec::with_capacity(output_count);
        for i in 0..output_count {
            let txin_index = txin_index_reader.get(first_txout + Vout::from(i));

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

            let (block_hash, block_time) = if let Some((h, ref bh, bt)) = cached_status
                && h == spending_height
            {
                (*bh, bt)
            } else {
                let (bh, bt) = self.block_hash_and_time(spending_height)?;
                cached_status = Some((spending_height, bh, bt));
                (bh, bt)
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

    /// Resolve txid to (tx_index, first_txout_index, output_count).
    /// Snapshots `safe_lengths` once and uses `safe.txout_index` as the
    /// upper bound for the tip-of-safe tx, so the fallback never reads past
    /// the writer's stamped boundary (`vecs.outputs.value.len()` can be
    /// ahead of `safe.txout_index` when the writer is mid-block).
    fn resolve_tx_outputs(&self, txid: &Txid) -> Result<(TxIndex, TxOutIndex, usize)> {
        let safe = self.safe_lengths();
        let tx_index = self.resolve_tx_index(txid)?;
        if tx_index >= safe.tx_index {
            return Err(Error::UnknownTxid);
        }
        let first_txout_vec = &self.indexer().vecs().transactions.first_txout_index;
        let first_txout_reader = first_txout_vec.reader();
        let first = first_txout_reader.try_get(tx_index).ok_or(Error::Internal(
            "resolve_tx_outputs: first txout index past data",
        ))?;
        let next_tx = tx_index.incremented();
        let next = if next_tx < safe.tx_index {
            first_txout_reader.try_get(next_tx).ok_or(Error::Internal(
                "resolve_tx_outputs: next first txout index past data",
            ))?
        } else {
            safe.txout_index
        };
        Ok((tx_index, first, usize::from(next) - usize::from(first)))
    }

    // === Helper methods ===

    fn transaction_by_index(&self, tx_index: TxIndex) -> Result<Transaction> {
        Ok(self
            .transactions_by_indices(&[tx_index])?
            .into_iter()
            .next()
            .expect("transactions_by_indices returns one tx per input index"))
    }

    fn transaction_raw_by_index(&self, tx_index: TxIndex) -> Result<Vec<u8>> {
        let indexer = self.indexer();
        let total_size = indexer
            .vecs()
            .transactions
            .total_size
            .collect_one(tx_index)
            .data()?;
        let position = indexer
            .vecs()
            .transactions
            .position
            .collect_one(tx_index)
            .data()?;
        self.reader().read_raw_bytes(position, *total_size as usize)
    }

    fn transaction_hex_by_index(&self, tx_index: TxIndex) -> Result<String> {
        Ok(self
            .transaction_raw_by_index(tx_index)?
            .to_lower_hex_string())
    }

    pub fn broadcast_transaction(&self, hex: &str) -> Result<Txid> {
        self.client().send_raw_transaction(hex)
    }

    pub fn merkleblock_proof(&self, txid: &Txid) -> Result<String> {
        let (_, height) = self.resolve_tx(txid)?;
        let header = self.read_block_header(height)?;
        let txids = self.block_txids_by_height(height)?;

        let target: bitcoin::Txid = txid.into();
        let mb = bitcoin::MerkleBlock::from_header_txids_with_predicate(
            &header,
            Txid::as_bitcoin_slice(&txids),
            |t| *t == target,
        );
        Ok(bitcoin::consensus::encode::serialize_hex(&mb))
    }

    pub fn merkle_proof(&self, txid: &Txid) -> Result<MerkleProof> {
        let (tx_index, height) = self.resolve_tx(txid)?;
        let first_tx = self
            .indexer()
            .vecs()
            .transactions
            .first_tx_index
            .collect_one(height)
            .data()?;
        let pos = tx_index.to_usize() - first_tx.to_usize();
        let txids = self.block_txids_by_height(height)?;

        Ok(MerkleProof {
            block_height: height,
            merkle: merkle_path(&txids, pos),
            pos,
        })
    }
}

fn merkle_path(txids: &[Txid], pos: usize) -> Vec<String> {
    // Txid bytes are in internal order (same layout as bitcoin::Txid)
    let mut hashes: Vec<[u8; 32]> = txids
        .iter()
        .map(|t| <&bitcoin::Txid>::from(t).to_byte_array())
        .collect();

    let mut proof = Vec::new();
    let mut idx = pos;

    while hashes.len() > 1 {
        let sibling = if idx ^ 1 < hashes.len() { idx ^ 1 } else { idx };
        // Display order: reverse bytes for hex output
        let mut display = hashes[sibling];
        display.reverse();
        proof.push(display.to_lower_hex_string());

        hashes = hashes
            .chunks(2)
            .map(|pair| {
                let right = pair.last().unwrap();
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(&pair[0]);
                combined[32..].copy_from_slice(right);
                sha256d::Hash::hash(&combined).to_byte_array()
            })
            .collect();
        idx /= 2;
    }

    proof
}
