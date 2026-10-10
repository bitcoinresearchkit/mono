use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count, PartsPerMillion32};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentPerBlock, LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, VSize, Version};
use vecdb::{AnyVec, Database, ReadOnlyClone, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use super::breakdown::{BlockMetrics, FeesSeries};

#[derive(Traversable)]
pub struct Total<M: StorageMode = Rw> {
    /// Number of `OP_RETURN` outputs.
    pub output_count: PerBlockCumulativeRolling<Count, M>,
    /// Number of script bytes following the `OP_RETURN` opcode across all
    /// `OP_RETURN` outputs.
    pub data_bytes: PerBlockCumulativeRolling<Bytes, M>,
    /// Cumulative `OP_RETURN` data bytes divided by cumulative serialized block
    /// bytes through the represented block.
    #[traversable(wrap = "data_bytes")]
    pub chain_share: LazyPercentPerBlock<PartsPerMillion32>,
    /// Number of transactions containing at least one `OP_RETURN` output; each
    /// transaction is counted once regardless of how many such outputs it has.
    pub tx_count: PerBlockCumulativeRolling<Count, M>,
    /// Sum of the full virtual sizes of transactions containing at least one
    /// `OP_RETURN` output; each transaction is included once.
    pub tx_vsize: PerBlockCumulativeRolling<VSize, M>,
    /// Sum of the full fees of transactions containing at least one `OP_RETURN`
    /// output; each transaction is included once.
    pub fees: FeesSeries<M>,
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
        macro_rules! import {
            ($name:literal) => {
                PerBlockCumulativeRolling::import(
                    db,
                    &format!("{prefix}_{}", $name),
                    version,
                    mappings,
                    window_starts,
                )
            };
        }
        let data_bytes = import!("data_bytes")?;
        let chain_share =
            LazyPercentPerBlock::from_ratio::<Bytes, Bytes, Quotient<PartsPerMillion32>>(
                &format!("{prefix}_data_chain_share"),
                version,
                &data_bytes.cumulative.height.read_only_clone(),
                block_size,
                mappings,
            );

        Ok(Self {
            output_count: import!("output_count")?,
            data_bytes,
            chain_share,
            tx_count: import!("tx_count")?,
            tx_vsize: import!("tx_vsize")?,
            fees: FeesSeries::import(db, prefix, version, chain_fees, window_starts, mappings)?,
        })
    }

    pub fn data_bytes_source(&self) -> &(impl ReadableCloneableVec<Height, Bytes> + use<>) {
        self.data_bytes.cumulative_source()
    }

    pub fn len(&self) -> usize {
        self.output_count
            .block
            .len()
            .min(self.data_bytes.block.len())
            .min(self.tx_count.block.len())
            .min(self.tx_vsize.block.len())
            .min(self.fees.len())
    }

    pub fn push(&mut self, block: BlockMetrics) {
        self.output_count.push_block(block.output_count.into());
        self.data_bytes.push_block(block.data_bytes);
        self.tx_count.push_block(block.tx_count.into());
        self.tx_vsize.push_block(block.tx_vsize);
        self.fees.push_block(block.fees);
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.output_count.validate_and_truncate(version, height)?;
        self.data_bytes.validate_and_truncate(version, height)?;
        self.tx_count.validate_and_truncate(version, height)?;
        self.tx_vsize.validate_and_truncate(version, height)?;
        let fees = self.fees.stored_mut();
        fees.any_validate_computed_version_or_reset(version)?;
        fees.any_truncate_if_needed_at(usize::from(height))?;
        Ok(())
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.output_count.truncate_if_needed_at(len)?;
        self.data_bytes.truncate_if_needed_at(len)?;
        self.tx_count.truncate_if_needed_at(len)?;
        self.tx_vsize.truncate_if_needed_at(len)?;
        self.fees.stored_mut().any_truncate_if_needed_at(len)?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.output_count.write()?;
        self.data_bytes.write()?;
        self.tx_count.write()?;
        self.tx_vsize.write()?;
        self.fees.stored_mut().write()?;
        Ok(())
    }

    pub fn compute_cents(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.fees.compute_cents(max_from, price_cents, exit)
    }
}
