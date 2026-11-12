//! A running total stored per block: the one stored column the flows (`PerBlockCumulativeRolling`,
//! `CumulativeRolling`) derive their per-block and window views from.

use bitview_compute::{NumericValue, Quantity};
use bitview_primitives::Count32;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, ComputeVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
    VecIndex, VecValue,
};

use crate::{CumulativeState, IndexSources, PerBlock};

#[derive(Traversable)]
pub struct PerBlockCumulative<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    #[traversable(accumulation = Cumulative)]
    pub cumulative: PerBlock<T, M>,
    last: M::WriteOnly<CumulativeState<T>>,
}

impl<T> PerBlockCumulative<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            cumulative: PerBlock::import(db, &format!("{name}_cumulative"), version, indexes)?,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &(impl ReadableCloneableVec<Height, T> + use<T>) {
        self.cumulative.resolutions.height_source()
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        self.last.push(&mut self.cumulative.height, value);
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
    {
        let last = &mut self.last;
        Ok(self.cumulative.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let value = transform(value);
                last.add_at(height, this, value)
            },
            exit,
        )?)
    }

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
        self.last = CumulativeState::default();
        Ok(self.cumulative.height.compute_cumulative_sum_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            transform,
            exit,
        )?)
    }

    /// A pure borrow: a length change reloads the cached total.
    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.cumulative.height
    }
}

impl<T> PerBlockCumulative<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T> + From<u64>,
{
    /// Adds up a per-block count column: of things, or of bytes.
    pub fn compute(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<Height, Count32>,
        exit: &Exit,
    ) -> Result<()> {
        self.compute_cumulative_transformed(
            max_from,
            source,
            |count| T::from(u64::from(u32::from(count))),
            exit,
        )
    }
}

/// Structs of counts, each field a flow (a stored running total, with its per-block value and
/// windows read from it) adding up one per-block column of `$source` (the indexer's
/// `BlockCounterVecs`): a plugin's public face of the counters it owns. Each field names its
/// value type (`Count` or `Bytes`) and the column, whose name is also the series'.
#[macro_export]
macro_rules! counter_group {
    (
        $source:ty;
        $(
            $(#[$struct_attr:meta])*
            $vis:vis struct $name:ident {
                $($(#[$attr:meta])* $field:ident: $value:ident => $column:ident,)+
            }
        )+
    ) => {
        $(
            $(#[$struct_attr])*
            #[derive($crate::__macro::Traversable)]
            $vis struct $name<M: $crate::__macro::StorageMode = $crate::__macro::Rw> {
                $($(#[$attr])* pub $field: $crate::PerBlockCumulativeRolling<$crate::__macro::$value, M>,)+
            }

            impl $name {
                $vis fn import(
                    db: &$crate::__macro::Database,
                    version: $crate::__macro::Version,
                    indexes: &$crate::IndexSources,
                    window_starts: &$crate::shapes::Windows<
                        &impl $crate::__macro::ReadableCloneableVec<
                            $crate::__macro::Height,
                            $crate::__macro::Height,
                        >,
                    >,
                ) -> $crate::__macro::Result<Self> {
                    Ok(Self {
                        $($field: $crate::PerBlockCumulativeRolling::import(
                            db,
                            stringify!($column),
                            version,
                            indexes,
                            window_starts,
                        )?,)+
                    })
                }

                $vis fn compute(
                    &mut self,
                    max_from: $crate::__macro::Height,
                    counters: &$source,
                    exit: &$crate::__macro::Exit,
                ) -> $crate::__macro::Result<()> {
                    $(self.$field.compute(max_from, &counters.$column, exit)?;)+
                    Ok(())
                }
            }
        )+
    };
}
