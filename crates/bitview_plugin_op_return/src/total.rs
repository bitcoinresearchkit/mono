use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count, PartsPerMillion32};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFixedRatioCumulativeRolling, LazyFixedRatioPerBlock, LazyWindowStartVec,
    PerBlockCumulativeRolling,
};
use brk_error::Result;
use brk_types::{Height, Sats, VSize, Version};
use vecdb::{AnyVec, Database, ReadOnlyClone, ReadableCloneableVec, Rw, StorageMode};

use super::breakdown::BlockMetrics;

#[derive(Traversable)]
pub struct Total<M: StorageMode = Rw> {
    /// Number of script bytes following the `OP_RETURN` opcode across all
    /// `OP_RETURN` outputs.
    pub data_bytes: PerBlockCumulativeRolling<Bytes, M>,
    /// Number of transactions containing at least one `OP_RETURN` output; each
    /// transaction is counted once regardless of how many such outputs it has.
    pub tx_count: PerBlockCumulativeRolling<Count, M>,
    /// Sum of the full virtual sizes of transactions containing at least one
    /// `OP_RETURN` output; each transaction is included once.
    pub tx_vsize: PerBlockCumulativeRolling<VSize, M>,
    /// Sum of the full fees of transactions containing at least one `OP_RETURN`
    /// output; each transaction is included once.
    pub fees: PerBlockCumulativeRolling<Sats, M>,
    /// Cumulative `OP_RETURN` data bytes divided by cumulative serialized block
    /// bytes through the represented block.
    pub chain_share: LazyFixedRatioPerBlock<PartsPerMillion32>,
    /// Fees of transactions carrying `OP_RETURN` divided by all transaction
    /// fees over the same cumulative or trailing window.
    pub fee_share: LazyFixedRatioCumulativeRolling<PartsPerMillion32>,
}

impl Total {
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        let data_bytes = PerBlockCumulativeRolling::import(
            db,
            &format!("{prefix}_data_bytes"),
            version,
            mappings,
            window_starts,
        )?;
        let tx_count = PerBlockCumulativeRolling::import(
            db,
            &format!("{prefix}_tx_count"),
            version,
            mappings,
            window_starts,
        )?;
        let tx_vsize = PerBlockCumulativeRolling::import(
            db,
            &format!("{prefix}_tx_vsize"),
            version,
            mappings,
            window_starts,
        )?;
        let fees = PerBlockCumulativeRolling::import(
            db,
            &format!("{prefix}_fees"),
            version,
            mappings,
            window_starts,
        )?;

        Ok(Self {
            chain_share: Self::lazy_chain_share(prefix, version, &data_bytes, block_size, mappings),
            fee_share: Self::lazy_fee_share(
                prefix,
                version,
                &fees,
                chain_fees,
                window_starts,
                mappings,
            ),
            data_bytes,
            tx_count,
            tx_vsize,
            fees,
        })
    }

    fn lazy_chain_share(
        prefix: &str,
        version: Version,
        data_bytes: &PerBlockCumulativeRolling<Bytes>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
        mappings: &MappingsVecs,
    ) -> LazyFixedRatioPerBlock<PartsPerMillion32> {
        let data_bytes = data_bytes.cumulative.height.read_only_clone();
        LazyFixedRatioPerBlock::from_ratio::<Bytes, Bytes, Quotient<PartsPerMillion32>>(
            &format!("{prefix}_chain_share"),
            version,
            &data_bytes,
            block_size,
            mappings,
        )
    }

    fn lazy_fee_share(
        prefix: &str,
        version: Version,
        fees: &PerBlockCumulativeRolling<Sats>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> LazyFixedRatioCumulativeRolling<PartsPerMillion32> {
        LazyFixedRatioCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_share"),
            version,
            &fees.cumulative.height,
            chain_fees,
            window_starts,
            mappings,
        )
    }

    pub fn data_bytes_source(&self) -> &(impl ReadableCloneableVec<Height, Bytes> + use<>) {
        self.data_bytes.cumulative_source()
    }

    pub fn len(&self) -> usize {
        self.data_bytes
            .block
            .len()
            .min(self.tx_count.block.len())
            .min(self.tx_vsize.block.len())
            .min(self.fees.block.len())
    }

    pub fn push(&mut self, block: BlockMetrics) {
        self.data_bytes.push_block(block.data_bytes);
        self.tx_count.push_block(block.tx_count.into());
        self.tx_vsize.push_block(block.tx_vsize);
        self.fees.push_block(block.fees);
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.data_bytes.validate_and_truncate(version, height)?;
        self.tx_count.validate_and_truncate(version, height)?;
        self.tx_vsize.validate_and_truncate(version, height)?;
        self.fees.validate_and_truncate(version, height)?;
        Ok(())
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.data_bytes.truncate_if_needed_at(len)?;
        self.tx_count.truncate_if_needed_at(len)?;
        self.tx_vsize.truncate_if_needed_at(len)?;
        self.fees.truncate_if_needed_at(len)?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.data_bytes.write()?;
        self.tx_count.write()?;
        self.tx_vsize.write()?;
        self.fees.write()?;
        Ok(())
    }
}
