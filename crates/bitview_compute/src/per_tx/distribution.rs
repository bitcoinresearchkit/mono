//! PerTxDistribution - stored per-tx EagerVec + computed distribution.
//!
//! Like LazyFromTxDistribution, but the per-tx source is eagerly computed
//! and stored rather than lazily derived.

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Lengths, TxIndex, VSize};
use schemars::JsonSchema;
use vecdb::{Database, EagerVec, ImportableVec, PcoVec, ReadableVec, Rw, StorageMode, Version};

use crate::{ComputedVecValue, NumericValue, TxDerivedDistribution};

#[derive(Traversable)]
pub struct PerTxDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub tx_index: M::Stored<EagerVec<PcoVec<TxIndex, T>>>,
    #[traversable(flatten)]
    pub distribution: TxDerivedDistribution<T, M>,
}

impl<T> PerTxDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let tx_index = EagerVec::forced_import(db, name, version)?;
        let distribution = TxDerivedDistribution::forced_import(db, name, version, indexes)?;
        Ok(Self {
            tx_index,
            distribution,
        })
    }

    pub fn derive_from_with_skip(
        &mut self,
        indexes: &crate::IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.distribution.derive_from_with_skip(
            indexes,
            starting_lengths,
            first_tx_index,
            &self.tx_index,
            exit,
            skip_count,
        )
    }

    pub fn derive_from_with_skip_weighted(
        &mut self,
        indexes: &crate::IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        vsize_source: &impl ReadableVec<TxIndex, VSize>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: Copy + Ord + From<f64> + Default,
        f64: From<T>,
    {
        self.distribution.derive_from_with_skip_weighted(
            indexes,
            starting_lengths,
            first_tx_index,
            &self.tx_index,
            vsize_source,
            exit,
            skip_count,
        )
    }
}
