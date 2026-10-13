use std::sync::Arc;

use bitcoin::{
    MerkleBlock, Txid as BitcoinTxid,
    consensus::encode::serialize_hex,
    hashes::{Hash, sha256d},
    hex::DisplayHex,
};
use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Lengths, TxOutIndex};
use bitview_types::MerkleProof;
use brk_types::{BlockHash, Height, Timestamp, Transaction, TxIndex, TxStatus, Txid, TxidPrefix};
use vecdb::{ReadableVec, VecIndex};

use super::indexed_transaction;
use crate::{Error, OptionData, Query, Result};

pub(crate) mod body;
pub mod confirmed;
pub mod info;
pub mod output_count;
pub mod outspend;
pub mod raw;

pub use confirmed::ResolvedConfirmedTx;
pub use info::ResolvedTransaction;
pub use raw::ResolvedRawTransaction;

enum TransactionSource {
    Memory(Arc<Transaction>),
    Chain(ResolvedConfirmedTx),
}

impl Query {
    /// Resolve one exact live, confirmed, or recently vanished transaction.
    fn resolve_transaction_source(&self, txid: &Txid) -> Result<TransactionSource> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let read = self.read_indexer_at(pin);
        match read.resolve_confirmed_tx(txid) {
            Ok(transaction) => Ok(TransactionSource::Chain(transaction)),
            Err(Error::UnknownTxid) => {
                let (mempool, tip) = Self::lookup_mempool(mempool).ok_or(Error::UnknownTxid)?;
                mempool
                    .transaction(txid, &tip)?
                    .map(TransactionSource::Memory)
                    .ok_or(Error::UnknownTxid)
            }
            Err(error) => Err(error),
        }
    }

    // ── Txid → TxIndex resolution (single source of truth) ─────────

    pub fn txid_by_index(&self, index: TxIndex) -> Result<Txid> {
        self.txid_and_height_by_index(index).map(|(txid, _)| txid)
    }

    /// Resolve a transaction index to its txid and containing block height from
    /// one guarded indexer/mappings snapshot.
    fn txid_and_height_by_index(&self, index: TxIndex) -> Result<(Txid, Height)> {
        let plugins = self.plugins();
        let pin = self.pin_safe_lengths()?;
        if index >= pin.lengths().tx_index {
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

    // ── TxStatus construction (single source of truth) ─────────────

    /// Block hash + timestamp for a height (cached vecs, fast).
    #[inline]
    fn block_hash_and_time(&self, height: Height) -> Result<(BlockHash, Timestamp)> {
        let indexer = self.indexer();
        let hash = indexer.vecs().blocks.blockhash.collect_one(height).data()?;
        let time = indexer.vecs().blocks.timestamp.collect_one(height).data()?;
        Ok((hash, time))
    }

    // ── Transaction queries ────────────────────────────────────────

    pub fn transaction_status(&self, txid: &Txid) -> Result<TxStatus> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let read = self.read_indexer_at(pin);
        match read.resolve_confirmed_position(txid) {
            Ok((_, height)) => self.confirmed_status_at_bounded(height, read.pin().lengths()),
            Err(Error::UnknownTxid) => {
                let (mempool, tip) = Self::lookup_mempool(mempool).ok_or(Error::UnknownTxid)?;
                mempool
                    .contains_txid(txid, &tip)?
                    .then_some(TxStatus::UNCONFIRMED)
                    .ok_or(Error::UnknownTxid)
            }
            Err(error) => Err(error),
        }
    }

    /// Resolve txid to (tx_index, first_txout_index, output_count) within the
    /// published lengths `safe`: the tip transaction's outputs end at
    /// `safe.txout_index`, never at the writer's unpublished length.
    fn resolve_tx_outputs(
        &self,
        txid: &Txid,
        safe: Lengths,
    ) -> Result<(TxIndex, TxOutIndex, usize)> {
        let tx_index = self.resolve_tx_index_bounded(txid, safe)?;
        let first_txout_vec = &self.indexer().vecs().transactions.first_txout_index;
        let first_txout_reader = first_txout_vec.reader();
        let first = first_txout_reader
            .try_get(tx_index)
            .ok_or(Error::Internal(
                "resolve_tx_outputs: first txout index past data",
            ))?
            .get();
        let next_tx = tx_index.incremented();
        let next = if next_tx < safe.tx_index {
            first_txout_reader
                .try_get(next_tx)
                .ok_or(Error::Internal(
                    "resolve_tx_outputs: next first txout index past data",
                ))?
                .get()
        } else {
            safe.txout_index
        };
        let count = output_count::output_count(first, next, safe.txout_index)?;
        Ok((tx_index, first, count))
    }

    // === Helper methods ===

    fn transaction_by_index(&self, tx_index: TxIndex, guard: &SafeLengths) -> Result<Transaction> {
        Ok(self
            .transactions_at_indices(&[tx_index], guard)?
            .into_iter()
            .next()
            .expect("transactions_by_indices returns one tx per input index"))
    }

    fn transaction_raw_by_index(&self, tx_index: TxIndex, pin: &SafeLengths) -> Result<Vec<u8>> {
        indexed_transaction::read_at(self, tx_index, pin.lengths()).map(|(bytes, _)| bytes)
    }

    /// Build a merkleblock proof from a pre-resolved confirmed transaction.
    pub fn merkleblock_proof_resolved(&self, tx: ResolvedConfirmedTx) -> Result<String> {
        let read = self.read_indexer()?;
        let (txid, _, height) = read.revalidate_confirmed_tx(tx)?;
        self.merkleblock_proof_at(txid, height, read)
    }

    fn merkleblock_proof_at(
        &self,
        txid: Txid,
        height: Height,
        read: confirmed::IndexerRead<'_>,
    ) -> Result<String> {
        let header = self.read_block_header_at(height, read.pin())?;
        let txids = self.block_txids_by_height(height, read.pin())?;
        drop(read);

        let target: BitcoinTxid = (&txid).into();
        let mb = MerkleBlock::from_header_txids_with_predicate(
            &header,
            Txid::as_bitcoin_slice(&txids),
            |t| *t == target,
        );
        Ok(serialize_hex(&mb))
    }

    /// Build a merkle proof from a pre-resolved confirmed transaction.
    pub fn merkle_proof_resolved(&self, tx: ResolvedConfirmedTx) -> Result<MerkleProof> {
        let read = self.read_indexer()?;
        let (_, tx_index, height) = read.revalidate_confirmed_tx(tx)?;
        self.merkle_proof_at(tx_index, height, read)
    }

    fn merkle_proof_at(
        &self,
        tx_index: TxIndex,
        height: Height,
        read: confirmed::IndexerRead<'_>,
    ) -> Result<MerkleProof> {
        let first_tx = self
            .indexer()
            .vecs()
            .transactions
            .first_tx_index
            .collect_one(height)
            .data()?;
        let pos = tx_index
            .to_usize()
            .checked_sub(first_tx.to_usize())
            .ok_or(Error::Internal("Transaction precedes its block"))?;
        let txids = self.block_txids_by_height(height, read.pin())?;
        drop(read);
        if pos >= txids.len() {
            return Err(Error::Internal("Transaction exceeds its block"));
        }

        Ok(MerkleProof {
            block_height: height,
            merkle: merkle_path(&txids, pos),
            pos,
        })
    }

    /// Resolve a txid to its internal TxIndex via prefix lookup.
    /// Raw store hit — caller should prefer [`Self::resolve_tx_index_bounded`]
    /// when subsequent reads dereference indexer/plugins vecs by `tx_index`.
    /// Use this raw form only for "is this mined?" probes that don't deref
    /// derived data (mempool merge, cpfp fee-rate fall-through).
    #[inline]
    pub(crate) fn resolve_tx_index(&self, txid: &Txid) -> Result<TxIndex> {
        self.indexer()
            .stores()
            .tx_index(&TxidPrefix::from(txid))?
            .ok_or(Error::UnknownTxid)
    }
    /// Exact txid verification after a prefix lookup, clamped to published data.
    /// Returns `UnknownTxid` for tx_indices the store knows but the snapshot
    /// has not yet covered. Use this from any path that will subsequently
    /// dereference indexer/plugins vecs by `tx_index`.
    #[inline]
    pub(crate) fn resolve_tx_index_bounded(&self, txid: &Txid, bound: Lengths) -> Result<TxIndex> {
        let tx_index = self.resolve_tx_index(txid)?;
        if tx_index >= bound.tx_index
            || self
                .indexer()
                .vecs()
                .transactions
                .txid
                .collect_one(tx_index)
                != Some(*txid)
        {
            return Err(Error::UnknownTxid);
        }
        Ok(tx_index)
    }

    pub(crate) fn confirmed_status_height_bounded(
        &self,
        tx_index: TxIndex,
        bound: Lengths,
    ) -> Result<Height> {
        if tx_index >= bound.tx_index {
            return Err(Error::UnknownTxid);
        }
        self.plugins()
            .mappings
            .tx_heights
            .get_shared(tx_index)
            .data()
    }

    pub(crate) fn confirmed_status_at_bounded(
        &self,
        height: Height,
        bound: Lengths,
    ) -> Result<TxStatus> {
        if height >= bound.height {
            return Err(Error::UnknownTxid);
        }
        let (block_hash, block_time) = self.block_hash_and_time(height)?;
        Ok(TxStatus::confirmed(height, block_hash, block_time))
    }
}

fn merkle_path(txids: &[Txid], pos: usize) -> Vec<String> {
    // Txid bytes are in internal order (same layout as BitcoinTxid)
    let mut hashes: Vec<[u8; 32]> = txids
        .iter()
        .map(|t| <&BitcoinTxid>::from(t).to_byte_array())
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
