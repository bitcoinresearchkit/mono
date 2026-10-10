use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{BinaryTransform, Database, ReadableVec, Rw, StorageMode, VecValue};

use crate::{IndexSources, LazyValueBlock, ValuePerBlock};

#[derive(Traversable)]
pub struct ValuePerBlockCumulative<M: StorageMode = Rw> {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: ValuePerBlock<M>,
}

const VERSION: Version = Version::ONE;

impl ValuePerBlockCumulative {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let v = version + VERSION;
        let cumulative = ValuePerBlock::import(db, &format!("{name}_cumulative"), v, indexes)?;
        let block = LazyValueBlock::from_cumulative(name, v, &cumulative);

        Ok(Self { block, cumulative })
    }

    pub fn compute_from<S>(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        source: &impl ReadableVec<Height, S>,
        transform: impl FnMut(Height, S) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
    {
        self.compute_sats_from(max_from, source, transform, exit)?;
        self.compute_cents(max_from, price_cents, exit)
    }

    pub fn compute_from_pair<S1, S2>(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
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
        self.compute_cents(max_from, price_cents, exit)
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
        Ok(())
    }
}

impl ValuePerBlockCumulative {
    pub fn compute_cents(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.cumulative
            .cents
            .height
            .compute_cumulative_transformed_binary(
                max_from,
                &self.block.sats,
                price_cents,
                SatsToCents::apply,
                exit,
            )?;

        Ok(())
    }
}
