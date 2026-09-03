use bitcoin::{
    hashes::{Hash, sha256d},
    hex::DisplayHex,
};
use brk_error::{Error, OptionData, Result};
use brk_types::{
    BlockHash, Height, MerkleProof, Timestamp, Transaction, TxIndex, TxOutIndex, TxStatus, Txid,
    TxidPrefix,
};
use vecdb::{ReadableVec, VecIndex};

mod confirmed;
mod info;
mod outspend;
mod raw;
mod resolved;

pub use confirmed::ResolvedConfirmedTx;
pub use info::ResolvedTransaction;
pub use raw::ResolvedRawTransaction;

use crate::Query;

impl Query {
    // ── Txid → TxIndex resolution (single source of truth) ─────────

    /// Resolve a txid to its internal TxIndex via prefix lookup.
    /// Raw store hit — caller should prefer [`Self::resolve_tx_index_bounded`]
    /// when subsequent reads dereference indexer/plugins vecs by `tx_index`.
    /// Use this raw form only for "is this mined?" probes that don't deref
    /// derived data (mempool merge, cpfp fee-rate fall-through).
    #[inline]
    fn resolve_tx_index(&self, txid: &Txid) -> Result<TxIndex> {
        self.indexer()
            .stores()
            .tx_index(&TxidPrefix::from(txid))?
            .ok_or(Error::UnknownTxid)
    }

    /// `resolve_tx_index` clamped against the safe-lengths snapshot.
    /// Returns `UnknownTxid` for tx_indices the store knows but the snapshot
    /// has not yet covered. Use this from any path that will subsequently
    /// dereference indexer/plugins vecs by `tx_index`.
    #[inline]
    fn resolve_tx_index_bounded(&self, txid: &Txid) -> Result<TxIndex> {
        let tx_index = self.resolve_tx_index(txid)?;
        if tx_index >= self.safe_lengths().tx_index {
            return Err(Error::UnknownTxid);
        }
        Ok(tx_index)
    }

    pub fn txid_by_index(&self, index: TxIndex) -> Result<Txid> {
        self.txid_and_height_by_index(index).map(|(txid, _)| txid)
    }

    /// Resolve a transaction index to its txid and containing block height from
    /// one guarded indexer/mappings snapshot.
    pub fn txid_and_height_by_index(&self, index: TxIndex) -> Result<(Txid, Height)> {
        let plugins = self.plugins();
        let _guard = self.read_plugins(&[plugins.indexer, plugins.mappings])?;
        if index >= self.safe_lengths().tx_index {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        let txid = self
            .indexer()
            .vecs()
            .transactions
            .txid
            .collect_one(index)
            .ok_or_else(|| Error::OutOfRange("Transaction index out of range".into()))?;
        let height = plugins.mappings.tx_heights.get_shared(index).data()?;
        Ok((txid, height))
    }

    /// Resolve a txid to (TxIndex, Height).
    pub fn resolve_tx(&self, txid: &Txid) -> Result<(TxIndex, Height)> {
        self.resolve_confirmed_position(txid)
    }

    // ── TxStatus construction (single source of truth) ─────────────

    /// Height for a confirmed tx_index via in-memory TxHeights lookup.
    /// Bounded against the safe-lengths snapshot so rejected tx_indices
    /// never dereference slots a concurrent writer might be populating.
    #[inline]
    fn confirmed_status_height(&self, tx_index: TxIndex) -> Result<Height> {
        let bound = self.safe_lengths();
        if tx_index >= bound.tx_index {
            return Err(Error::UnknownTxid);
        }
        self.plugins()
            .mappings
            .tx_heights
            .get_shared(tx_index)
            .data()
    }

    /// Full confirmed TxStatus from a known height.
    #[inline]
    fn confirmed_status_at(&self, height: Height) -> Result<TxStatus> {
        if height >= self.safe_lengths().height {
            return Err(Error::UnknownTxid);
        }
        let (block_hash, block_time) = self.block_hash_and_time(height)?;
        if height >= self.safe_lengths().height {
            return Err(Error::UnknownTxid);
        }
        Ok(TxStatus::confirmed(height, block_hash, block_time))
    }

    /// Block hash + timestamp for a height (cached vecs, fast).
    #[inline]
    fn block_hash_and_time(&self, height: Height) -> Result<(BlockHash, Timestamp)> {
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
        self.merkleblock_proof_at(*txid, height)
    }

    /// Build a merkleblock proof from a pre-resolved confirmed transaction.
    pub fn merkleblock_proof_resolved(&self, tx: ResolvedConfirmedTx) -> Result<String> {
        let (txid, _, height) = self.revalidate_confirmed_tx(tx)?;
        self.merkleblock_proof_at(txid, height)
    }

    fn merkleblock_proof_at(&self, txid: Txid, height: Height) -> Result<String> {
        let header = self.read_block_header(height)?;
        let txids = super::block::block_txids_by_height(self, height)?;

        let target: bitcoin::Txid = (&txid).into();
        let mb = bitcoin::MerkleBlock::from_header_txids_with_predicate(
            &header,
            Txid::as_bitcoin_slice(&txids),
            |t| *t == target,
        );
        Ok(bitcoin::consensus::encode::serialize_hex(&mb))
    }

    pub fn merkle_proof(&self, txid: &Txid) -> Result<MerkleProof> {
        let (tx_index, height) = self.resolve_tx(txid)?;
        self.merkle_proof_at(tx_index, height)
    }

    /// Build a merkle proof from a pre-resolved confirmed transaction.
    pub fn merkle_proof_resolved(&self, tx: ResolvedConfirmedTx) -> Result<MerkleProof> {
        let (_, tx_index, height) = self.revalidate_confirmed_tx(tx)?;
        self.merkle_proof_at(tx_index, height)
    }

    fn merkle_proof_at(&self, tx_index: TxIndex, height: Height) -> Result<MerkleProof> {
        let first_tx = self
            .indexer()
            .vecs()
            .transactions
            .first_tx_index
            .collect_one(height)
            .data()?;
        let pos = tx_index.to_usize() - first_tx.to_usize();
        let txids = super::block::block_txids_by_height(self, height)?;

        Ok(MerkleProof {
            block_height: height,
            merkle: merkle_path(&txids, pos),
            pos,
        })
    }
}

#[inline]
pub fn resolve_tx_index(query: &Query, txid: &Txid) -> Result<TxIndex> {
    query.resolve_tx_index(txid)
}

#[inline]
pub fn resolve_tx_index_bounded(query: &Query, txid: &Txid) -> Result<TxIndex> {
    query.resolve_tx_index_bounded(txid)
}

#[inline]
pub fn confirmed_status_height(query: &Query, tx_index: TxIndex) -> Result<Height> {
    query.confirmed_status_height(tx_index)
}

#[inline]
pub fn confirmed_status_at(query: &Query, height: Height) -> Result<TxStatus> {
    query.confirmed_status_at(height)
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
