use brk_error::Result;

use brk_error::Error;
use brk_types::{BlockHashPrefix, Timestamp};
use tracing::error;
use vecdb::{WritableVec, unlikely};

use super::{BlockProcessor, transaction::ComputedTx};
use crate::{lengths::IndexerLengths as _, stores::IndexerStores as _};

impl BlockProcessor<'_> {
    pub fn process_block_metadata(&mut self) -> Result<()> {
        let height = self.height;
        let blockhash = self.block.hash();
        let blockhash_prefix = BlockHashPrefix::from(blockhash);

        if unlikely(self.check_collisions)
            && self
                .stores
                .block_height(&blockhash_prefix)?
                .is_some_and(|prev_height| prev_height != height)
        {
            error!("BlockHash: {blockhash}");
            return Err(Error::Internal("BlockHash prefix collision"));
        }

        self.lengths.push(self.vecs);

        self.stores.insert_block_height(blockhash_prefix, height);

        self.vecs
            .blocks
            .blockhash
            .inner
            .debug_checked_push(height, *blockhash);
        self.vecs
            .blocks
            .coinbase_tag
            .debug_checked_push(height, self.block.coinbase_tag());
        self.vecs
            .blocks
            .difficulty
            .debug_checked_push(height, self.block.header.difficulty_float().into());
        self.vecs
            .blocks
            .timestamp
            .inner
            .debug_checked_push(height, Timestamp::from(self.block.header.time));

        Ok(())
    }

    /// Push block total_size and weight, reusing per-tx sizes already computed in ComputedTx.
    /// This avoids redundant tx serialization (base_size + total_size were already computed).
    pub fn push_block_size_and_weight(&mut self, txs: &[ComputedTx]) {
        let overhead = bitcoin::block::Header::SIZE + bitcoin::VarInt::from(txs.len()).size();
        let mut total_size = overhead;
        let mut weight = bitcoin::Weight::from_non_witness_data_size(overhead as u64);
        let mut sw_txs = 0u32;
        let mut sw_size = 0usize;
        let mut sw_weight = bitcoin::Weight::ZERO;

        for (i, tx) in txs.iter().enumerate() {
            total_size += tx.total_size as usize;
            weight += tx.weight();
            if i > 0 && tx.is_segwit() {
                sw_txs += 1;
                sw_size += tx.total_size as usize;
                sw_weight += tx.weight();
            }
        }

        let h = self.height;
        let blocks = &mut self.vecs.blocks;
        blocks.total.debug_checked_push(h, total_size.into());
        blocks.weight.debug_checked_push(h, weight.into());
        blocks.segwit_txs.debug_checked_push(h, sw_txs.into());
        blocks.segwit_size.debug_checked_push(h, sw_size.into());
        blocks.segwit_weight.debug_checked_push(h, sw_weight.into());
    }
}
