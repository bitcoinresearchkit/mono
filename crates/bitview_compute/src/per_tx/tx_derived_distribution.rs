use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Lengths, TxIndex, VSize};
use schemars::JsonSchema;
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode, Version};

use crate::{BlockRollingDistribution, ComputedVecValue, NumericValue, PerBlockDistribution};

#[derive(Traversable)]
pub struct TxDerivedDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub block: PerBlockDistribution<T, M>,
    #[traversable(flatten)]
    pub distribution: BlockRollingDistribution<T, M>,
}

impl<T> TxDerivedDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let block = PerBlockDistribution::forced_import(db, name, version, indexes)?;
        let distribution = BlockRollingDistribution::forced_import(db, name, version, indexes)?;

        Ok(Self {
            block,
            distribution,
        })
    }

    pub fn derive_from(
        &mut self,
        indexes: &crate::IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.derive_from_with_skip(
            indexes,
            starting_lengths,
            first_tx_index,
            tx_index_source,
            exit,
            0,
        )
    }

    pub fn derive_from_with_skip(
        &mut self,
        indexes: &crate::IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.block.compute_with_skip(
            starting_lengths.height,
            tx_index_source,
            first_tx_index,
            &indexes.height_tx_index_count,
            exit,
            skip_count,
        )?;

        self.distribution._6b.compute_from_nblocks(
            starting_lengths.height,
            tx_index_source,
            first_tx_index,
            &indexes.height_tx_index_count,
            6,
            exit,
            skip_count,
        )?;

        Ok(())
    }

    /// Like `derive_from_with_skip` but uses vsize-weighted percentiles for both
    /// the per-block and rolling six-block distributions.
    #[allow(clippy::too_many_arguments)]
    pub fn derive_from_with_skip_weighted(
        &mut self,
        indexes: &crate::IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        vsize_source: &impl ReadableVec<TxIndex, VSize>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.block.compute_with_skip_weighted(
            starting_lengths.height,
            tx_index_source,
            vsize_source,
            first_tx_index,
            &indexes.height_tx_index_count,
            exit,
            skip_count,
        )?;

        self.distribution._6b.compute_from_nblocks_weighted(
            starting_lengths.height,
            tx_index_source,
            vsize_source,
            first_tx_index,
            &indexes.height_tx_index_count,
            6,
            exit,
            skip_count,
        )?;

        Ok(())
    }
}
