use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Sats, Version};
use vecdb::{
    AnyVec, BinaryTransform, Database, EagerVec, Exit, PcoVec, ReadableVec, Rw, StorageMode,
    VecIndex, VecValue, WritableVec,
};

use crate::{
    indexes,
    internal::{LazyValueBlock, SatsToCents, ValuePerBlock},
    price,
};

#[derive(Traversable)]
pub struct ValuePerBlockCumulative<M: StorageMode = Rw> {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: ValuePerBlock<M>,
    #[traversable(skip)]
    last_cumulative_sats: Option<(usize, Sats)>,
}

const VERSION: Version = Version::ONE;

impl ValuePerBlockCumulative {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let v = version + VERSION;
        let cumulative =
            ValuePerBlock::forced_import(db, &format!("{name}_cumulative"), v, indexes)?;
        let last_cumulative_sats = cumulative
            .sats
            .height
            .collect_last()
            .map(|value| (cumulative.sats.height.len(), value));
        let block = LazyValueBlock::from_cumulative(name, v, &cumulative);

        Ok(Self {
            block,
            cumulative,
            last_cumulative_sats,
        })
    }

    pub(crate) fn compute_from<S>(
        &mut self,
        max_from: Height,
        prices: &price::Vecs,
        source: &impl ReadableVec<Height, S>,
        transform: impl FnMut(Height, S) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
    {
        self.compute_sats_from(max_from, source, transform, exit)?;
        self.compute_cents(max_from, prices, exit)
    }

    pub(crate) fn compute_from_pair<S1, S2>(
        &mut self,
        max_from: Height,
        prices: &price::Vecs,
        source1: &impl ReadableVec<Height, S1>,
        source2: &impl ReadableVec<Height, S2>,
        transform: impl FnMut(Height, S1, S2) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S1: VecValue,
        S2: VecValue,
    {
        self.compute_sats_from_pair(max_from, source1, source2, transform, exit)?;
        self.compute_cents(max_from, prices, exit)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_filtered_from_indexes<A, B>(
        &mut self,
        max_from: Height,
        prices: &price::Vecs,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        filter: impl FnMut(&Sats) -> bool,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        self.compute_sats_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            filter,
            exit,
        )?;
        self.compute_cents(max_from, prices, exit)
    }

    fn compute_sats_from<S>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<Height, S>,
        mut transform: impl FnMut(Height, S) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
    {
        let mut cumulative = None;
        self.cumulative.sats.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    height
                        .decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                *cumulative += transform(height, value);
                (height, *cumulative)
            },
            exit,
        )?;
        self.last_cumulative_sats = None;
        Ok(())
    }

    fn compute_sats_from_pair<S1, S2>(
        &mut self,
        max_from: Height,
        source1: &impl ReadableVec<Height, S1>,
        source2: &impl ReadableVec<Height, S2>,
        mut transform: impl FnMut(Height, S1, S2) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S1: VecValue,
        S2: VecValue,
    {
        let mut cumulative = None;
        self.cumulative.sats.height.compute_transform2(
            max_from,
            source1,
            source2,
            |(height, value1, value2, this)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    height
                        .decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                *cumulative += transform(height, value1, value2);
                (height, *cumulative)
            },
            exit,
        )?;
        self.last_cumulative_sats = None;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_sats_from_indexes<A, B>(
        &mut self,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        filter: impl FnMut(&Sats) -> bool,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        Self::compute_sats_height_from_indexes(
            &mut self.cumulative.sats.height,
            max_from,
            first_indexes,
            indexes_count,
            source,
            filter,
            exit,
        )?;
        self.last_cumulative_sats = None;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn compute_sats_height_from_indexes<A, B>(
        target: &mut EagerVec<PcoVec<Height, Sats>>,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        mut filter: impl FnMut(&Sats) -> bool,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        target.validate_computed_version_or_reset(
            first_indexes.version() + indexes_count.version() + source.version(),
        )?;
        target.truncate_if_needed(max_from)?;
        target.repeat_until_complete(exit, |target| {
            let skip = target.len();
            let end = target.batch_end(indexes_count.len());
            if skip >= end || skip >= first_indexes.len() {
                return Ok(());
            }

            let source_start = first_indexes.collect_one_at(skip).unwrap().to_usize();
            let counts: Vec<usize> = indexes_count
                .collect_range_at(skip, end)
                .into_iter()
                .map(usize::from)
                .collect();
            let source_end = source_start + counts.iter().sum::<usize>();
            let mut cumulative = skip
                .checked_sub(1)
                .and_then(|index| target.collect_one_at(index))
                .unwrap_or_default();
            let mut group_index = 0;

            while group_index < counts.len() && counts[group_index] == 0 {
                target.push(cumulative);
                group_index += 1;
            }

            if group_index < counts.len() {
                let mut remaining = counts[group_index];
                source.fold_range_at(source_start, source_end, Sats::ZERO, |sum, value| {
                    let sum = if filter(&value) { sum + value } else { sum };
                    remaining -= 1;
                    if remaining == 0 {
                        cumulative += sum;
                        target.push(cumulative);
                        group_index += 1;
                        while group_index < counts.len() && counts[group_index] == 0 {
                            target.push(cumulative);
                            group_index += 1;
                        }
                        if group_index < counts.len() {
                            remaining = counts[group_index];
                        }
                        Sats::ZERO
                    } else {
                        sum
                    }
                });
            }

            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn compute_cents(
        &mut self,
        max_from: Height,
        prices: &price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.cumulative
            .cents
            .height
            .compute_cumulative_transformed_binary(
                max_from,
                &self.block.sats,
                &prices.spot.cents.height,
                SatsToCents::apply,
                exit,
            )?;

        Ok(())
    }
}
