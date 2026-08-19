use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Bytes, Height, PartsPerMillion32, Sats, StoredU64, VSize, Version};
use vecdb::{AnyVec, CachedBoxedVec, Database, ReadOnlyClone, Rw, StorageMode};

use super::breakdown::BlockMetrics;
use bitview_compute::{
    CachedPerBlockCumulativeRolling, CachedWindowStartVec, LazyPercentCumulativeRolling,
    LazyPercentPerBlock, PerBlockCumulativeRolling, RatioBytes, RatioSats, Windows,
};

#[derive(Traversable)]
pub struct Total<M: StorageMode = Rw> {
    /// Number of script bytes following the `OP_RETURN` opcode across all
    /// `OP_RETURN` outputs.
    pub data_bytes: CachedPerBlockCumulativeRolling<Bytes, M>,
    /// Number of transactions containing at least one `OP_RETURN` output; each
    /// transaction is counted once regardless of how many such outputs it has.
    pub tx_count: PerBlockCumulativeRolling<StoredU64, M>,
    /// Sum of the full virtual sizes of transactions containing at least one
    /// `OP_RETURN` output; each transaction is included once.
    pub tx_vsize: PerBlockCumulativeRolling<VSize, M>,
    /// Sum of the full fees of transactions containing at least one `OP_RETURN`
    /// output; each transaction is included once.
    pub fees: PerBlockCumulativeRolling<Sats, M>,
    /// Cumulative `OP_RETURN` data bytes divided by cumulative serialized block
    /// bytes through the represented block.
    pub chain_share: LazyPercentPerBlock<PartsPerMillion32>,
    /// Fees of transactions carrying `OP_RETURN` divided by all transaction
    /// fees over the same cumulative or trailing window.
    pub fee_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl Total {
    pub fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        block_size: &CachedBoxedVec<Height, StoredU64>,
        chain_fees: &CachedBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let data_bytes = CachedPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{prefix}_data_bytes"),
            version,
            indexes,
            cached_starts,
        )?;
        let tx_count = PerBlockCumulativeRolling::forced_import(
            db,
            &format!("{prefix}_tx_count"),
            version,
            indexes,
            cached_starts,
        )?;
        let tx_vsize = PerBlockCumulativeRolling::forced_import(
            db,
            &format!("{prefix}_tx_vsize"),
            version,
            indexes,
            cached_starts,
        )?;
        let fees = PerBlockCumulativeRolling::forced_import(
            db,
            &format!("{prefix}_fees"),
            version,
            indexes,
            cached_starts,
        )?;

        Ok(Self {
            chain_share: Self::lazy_chain_share(
                prefix,
                version,
                &data_bytes,
                block_size.clone(),
                indexes,
            ),
            fee_share: Self::lazy_fee_share(
                prefix,
                version,
                &fees,
                chain_fees.clone(),
                cached_starts,
                indexes,
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
        data_bytes: &CachedPerBlockCumulativeRolling<Bytes>,
        block_size: CachedBoxedVec<Height, StoredU64>,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> LazyPercentPerBlock<PartsPerMillion32> {
        let data_bytes = data_bytes.cumulative.height.read_only_clone();
        LazyPercentPerBlock::from_cached_ratio::<Bytes, StoredU64, RatioBytes<PartsPerMillion32>>(
            &format!("{prefix}_chain_share"),
            version,
            &data_bytes,
            block_size,
            indexes,
        )
    }

    fn lazy_fee_share(
        prefix: &str,
        version: Version,
        fees: &PerBlockCumulativeRolling<Sats>,
        chain_fees: CachedBoxedVec<Height, Sats>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> LazyPercentCumulativeRolling<PartsPerMillion32> {
        LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            RatioSats<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_share"),
            version,
            &fees.cumulative.height,
            chain_fees,
            cached_starts,
            indexes,
        )
    }

    pub fn cached_data_bytes(&self) -> CachedBoxedVec<Height, Bytes> {
        self.data_bytes.cached_cumulative()
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
