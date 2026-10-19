use crate::{ReadPin, check_bound};
use bitcoin::{
    MerkleBlock, Txid as BitcoinTxid,
    consensus::encode::serialize_hex,
    hashes::{Hash, sha256d},
    hex::DisplayHex,
};
use bitview_plugin_indexer::{OutputRange, SafeLengths};
use bitview_primitives::Lengths;
use bitview_types::MerkleProof;
use brk_types::{BlockHash, Height, Timestamp, Transaction, TxIndex, TxStatus, Txid, TxidPrefix};
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result};

pub(crate) mod body;
pub mod confirmed;
pub mod info;
pub mod outspend;

pub use confirmed::ResolvedConfirmedTx;
pub use info::ResolvedTransaction;

impl Query {
    pub fn txid_by_index(&self, index: TxIndex) -> Result<Txid> {
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, index, pin.lengths().tx_index)?;
        if index >= pin.lengths().tx_index {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        self.indexer()
            .vecs()
            .transactions
            .txid
            .collect_one(index)
            .ok_or_else(|| Error::OutOfRange("Transaction index out of range".into()))
    }

    #[inline]
    fn block_hash_and_time(&self, height: Height) -> Result<(BlockHash, Timestamp)> {
        let indexer = self.indexer();
        let hash = indexer.vecs().blocks.blockhash.collect_one(height).data()?;
        let time = indexer.vecs().blocks.timestamp.collect_one(height).data()?;
        Ok((hash, time))
    }

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

    /// Resolve txid to its outputs as published at `pin`: the tip transaction's outputs end at
    /// the published output length, never at the writer's unpublished one.
    fn resolve_tx_outputs(&self, txid: &Txid, pin: &SafeLengths) -> Result<OutputRange> {
        let tx_index = self.resolve_tx_index_bounded(txid, pin)?;
        self.indexer()
            .output_ranges(pin.lengths())
            .get(tx_index)
            .ok_or(Error::Internal(
                "Transaction outputs outside published data",
            ))
    }

    fn transaction_by_index(&self, tx_index: TxIndex, guard: &ReadPin) -> Result<Transaction> {
        Ok(self
            .transactions_at_indices(&[tx_index], guard)?
            .into_iter()
            .next()
            .expect("transactions_at_indices returns one transaction per index"))
    }

    fn transaction_raw_by_index(&self, tx_index: TxIndex, pin: &ReadPin) -> Result<Vec<u8>> {
        debug_assert!(
            tx_index < pin.lengths().tx_index,
            "a resolved transaction is published"
        );
        Ok(self.indexer().read_tx(tx_index)?.0)
    }

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
        // The revalidated token's transaction is in its block.
        debug_assert!(tx_index >= first_tx);
        let pos = tx_index.to_usize() - first_tx.to_usize();
        let txids = self.block_txids_by_height(height, read.pin())?;
        drop(read);
        debug_assert!(pos < txids.len());

        Ok(MerkleProof {
            block_height: height,
            merkle: merkle_path(&txids, pos),
            pos,
        })
    }

    /// The txid's index, verified against the full txid after the prefix lookup.
    /// `StateUpdating` past a lowered publication; `UnknownTxid` past the published data or on
    /// a prefix collision.
    #[inline]
    pub(crate) fn resolve_tx_index_bounded(
        &self,
        txid: &Txid,
        pin: &SafeLengths,
    ) -> Result<TxIndex> {
        let tx_index = self
            .indexer()
            .stores()
            .tx_index(&TxidPrefix::from(txid))?
            .ok_or(Error::UnknownTxid)?;
        let bound = pin.lengths();
        check_bound(pin, tx_index, bound.tx_index)?;
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
        self.plugins().mappings.tx_heights.get(tx_index).data()
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
