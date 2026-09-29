use std::ops::AddAssign;

use bitview_cohort::UTXOValues;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, AnyVec, Database, PcoVecValue, Rw, StorageMode};

use super::super::UTXOSources;
use crate::CumulativeState;

#[derive(Traversable)]
pub struct CumulativeUTXOSources<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[traversable(flatten)]
    pub stored: UTXOSources<T, M>,
    last: M::WriteOnly<CumulativeState<UTXOValues<T>>>,
}

impl<T> CumulativeUTXOSources<T>
where
    T: PcoVecValue + AddAssign + Copy + Default,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: UTXOSources::forced_import(db, name, version)?,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, cohort_values: UTXOValues<T>) {
        let len = self.stored.min_len();
        let cumulative = self.last.accumulate(
            len,
            || self.stored.collect_last(),
            |values| *values += cohort_values,
        );
        self.stored.push(cumulative);
    }

    pub fn push_partition<const ORIGIN: bool>(&mut self, values: UTXOValues<T>) {
        let len = if ORIGIN {
            self.stored.typed.core.min_len()
        } else {
            self.stored.amount.iter().map(AnyVec::len).min().unwrap()
        };
        let cumulative = self.last.accumulate(
            len,
            || self.stored.collect_last(),
            |last| {
                if ORIGIN {
                    last.core += values.core;
                } else {
                    last.amount_range += values.amount_range;
                    last.type_ += values.type_;
                }
            },
        );
        self.stored.push_partition::<ORIGIN>(cumulative, None);
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len()
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.last = Default::default();
        self.stored.collect_vecs_mut()
    }
}
