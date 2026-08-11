use std::ops::AddAssign;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

use super::super::{UTXOColumnarMetric, UTXORows};

#[derive(Traversable)]
pub struct CumulativeUTXOColumnarMetric<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[traversable(flatten)]
    pub matrices: UTXOColumnarMetric<T, M>,
    #[traversable(skip)]
    last: Option<(usize, UTXORows<T>)>,
}

impl<T> CumulativeUTXOColumnarMetric<T>
where
    T: PcoVecValue + AddAssign + Copy + Default,
{
    pub(crate) fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            matrices: UTXOColumnarMetric::forced_import(db, name, version)?,
            last: None,
        })
    }

    #[inline(always)]
    pub(crate) fn push_block(&mut self, rows: UTXORows<T>) {
        let len = self.matrices.min_len();
        let mut cumulative = match self.last.take() {
            Some((cached_len, row)) if cached_len == len => row,
            _ => self.matrices.collect_last().unwrap_or_default(),
        };
        cumulative += rows;
        self.matrices.push(cumulative.clone());
        self.last = Some((len + 1, cumulative));
    }

    pub(crate) fn min_len(&self) -> usize {
        self.matrices.min_len()
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.last = None;
        self.matrices.collect_vecs_mut()
    }
}
