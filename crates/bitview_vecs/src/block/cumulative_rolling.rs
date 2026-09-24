//! PerBlockCumulativeRolling - stored cumulative + lazy block and rolling views.
//!
//! The cumulative vector is the sole stored source of truth. Per-block values
//! and rolling sums/averages are all derived lazily from it.

use bitview_collections::Windows;
use bitview_compute::{NumericValue, compute_cumulative_sum_from_indexes};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecIndex,
    VecValue, WritableVec,
};

use crate::{IndexSources, LazyPreviousDeltaVec, PerBlock, RollingTotals};

#[derive(Deref, DerefMut, Traversable)]
pub struct PerBlockCumulativeRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: PerBlock<T, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub rolling: RollingTotals<T>,
    last_cumulative: M::WriteOnly<Option<(usize, T)>>,
}

impl<T> PerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let cumulative =
            PerBlock::forced_import(db, &format!("{name}_cumulative"), version, indexes)?;
        let source = cumulative.resolutions.height_source();
        let block = LazyPreviousDeltaVec::new(name, version, source);
        let rolling = RollingTotals::new(name, version, source, window_starts, indexes);
        let last_cumulative = cumulative
            .height
            .collect_last()
            .map(|value| (cumulative.height.len(), value));

        Ok(Self {
            block,
            cumulative,
            rolling,
            last_cumulative,
        })
    }

    pub fn cumulative_source(&self) -> &(impl ReadableCloneableVec<Height, T> + use<T>) {
        self.cumulative.resolutions.height_source()
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T)
    where
        T: Copy,
    {
        let len = self.cumulative.height.len();
        let mut cumulative = match self.last_cumulative {
            Some((cached_len, value)) if cached_len == len => value,
            _ => self.cumulative.height.collect_last().unwrap_or_default(),
        };
        cumulative += value;
        self.cumulative.height.push(cumulative);
        self.last_cumulative = Some((len + 1, cumulative));
    }

    pub fn compute_cumulative_transformed<S>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<Height, S>,
        mut transform: impl FnMut(S) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
        T: Copy,
    {
        self.last_cumulative = None;
        let mut cumulative = None;
        Ok(self.cumulative.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    height
                        .decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                *cumulative += transform(value);
                (height, *cumulative)
            },
            exit,
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_cumulative_sum_from_indexes<A, B, S>(
        &mut self,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, S>,
        transform: impl FnMut(S) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        S: VecValue,
        usize: From<B>,
        T: Copy,
    {
        self.last_cumulative = None;
        compute_cumulative_sum_from_indexes(
            &mut self.cumulative.height,
            max_from,
            first_indexes,
            indexes_count,
            source,
            transform,
            exit,
        )
    }

    pub fn validate_computed_version_or_reset(&mut self, version: Version) -> Result<()> {
        self.last_cumulative = None;
        self.cumulative
            .height
            .validate_computed_version_or_reset(version)?;
        Ok(())
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.last_cumulative = None;
        Ok(self
            .cumulative
            .height
            .validate_and_truncate(version, height)?)
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.last_cumulative = None;
        Ok(self.cumulative.height.truncate_if_needed_at(len)?)
    }

    pub fn write(&mut self) -> Result<()> {
        self.cumulative.height.write()?;
        Ok(())
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last_cumulative = None;
        &mut self.cumulative.height
    }
}

#[cfg(test)]
mod tests {
    use bitview_transforms::StoredU64ToStoredU32;
    use brk_types::{Height, StoredU32, StoredU64, Version};
    use tempfile::tempdir;
    use vecdb::{
        AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadableVec, WritableVec,
    };

    use crate::LazyPreviousDeltaVec;

    #[test]
    fn lazy_block_is_the_delta_of_cumulative() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut cumulative: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "cumulative", Version::ONE).unwrap();

        for value in [1_u64, 3, 6] {
            cumulative.push(StoredU64::from(value));
        }
        cumulative.write().unwrap();

        let block =
            LazyPreviousDeltaVec::<Height, StoredU64>::new("block", Version::ONE, &cumulative);

        assert_eq!(
            block.collect_range_at(0, 3),
            [1_u64, 2, 3].map(StoredU64::from)
        );
        assert_eq!(
            block.collect_range_at(1, 3),
            [2_u64, 3].map(StoredU64::from)
        );
        assert_eq!(
            block.read_sorted_at(&[0, 2, 2, 3]),
            [1_u64, 3, 3].map(StoredU64::from)
        );

        let transformed =
            LazyPreviousDeltaVec::<Height, StoredU64, StoredU32, StoredU64ToStoredU32>::transformed(
                "transformed",
                Version::ONE,
                &cumulative,
            );
        assert_eq!(
            transformed.collect_range_at(0, 3),
            [1_u32, 2, 3].map(StoredU32::from)
        );
        assert_eq!(
            transformed.collect_range_at(1, 3),
            [2_u32, 3].map(StoredU32::from)
        );
    }
}
