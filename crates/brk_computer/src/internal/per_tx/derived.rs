use brk_error::Result;
use brk_indexer::{Indexer, Lengths};

use brk_traversable::Traversable;
use brk_types::{TxIndex, VSize};
use schemars::JsonSchema;
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode, Version};

use crate::{
    indexes,
    internal::{ComputedVecValue, NumericValue, PerBlockDistribution},
};

#[derive(Traversable)]
pub struct BlockRollingDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub _6b: PerBlockDistribution<T, M>,
}

impl<T> BlockRollingDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self {
            _6b: PerBlockDistribution::forced_import(db, &format!("{name}_6b"), version, indexes)?,
        })
    }
}

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
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let block = PerBlockDistribution::forced_import(db, name, version, indexes)?;
        let distribution = BlockRollingDistribution::forced_import(db, name, version, indexes)?;

        Ok(Self {
            block,
            distribution,
        })
    }

    pub(crate) fn derive_from(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        starting_lengths: &Lengths,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.derive_from_with_skip(indexer, indexes, starting_lengths, tx_index_source, exit, 0)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn derive_from_with_skip(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        starting_lengths: &Lengths,
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
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            exit,
            skip_count,
        )?;

        self.distribution._6b.compute_from_nblocks(
            starting_lengths.height,
            tx_index_source,
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            6,
            exit,
            skip_count,
        )?;

        Ok(())
    }

    /// Like `derive_from_with_skip` but uses vsize-weighted percentiles for both
    /// the per-block and rolling six-block distributions.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn derive_from_with_skip_weighted(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        starting_lengths: &Lengths,
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
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            exit,
            skip_count,
        )?;

        self.distribution._6b.compute_from_nblocks_weighted(
            starting_lengths.height,
            tx_index_source,
            vsize_source,
            &indexer.vecs().transactions.first_tx_index,
            &indexes.height.tx_index_count,
            6,
            exit,
            skip_count,
        )?;

        Ok(())
    }
}
