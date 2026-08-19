use brk_types::Histogram;
use derive_more::{Deref, DerefMut};

use crate::{HistogramEmaCompact, NUM_BINS};

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct HistogramEma(Histogram<f64, NUM_BINS>);

impl HistogramEma {
    #[inline]
    pub fn zeros() -> Self {
        Self(Histogram::zeros())
    }

    #[inline]
    pub fn to_compact(&self) -> HistogramEmaCompact {
        self.0.to_compact().into()
    }

    #[inline]
    pub fn add_from(&mut self, rhs: &Self) {
        self.0.add_from(&rhs.0);
    }

    #[inline]
    pub fn divide_by(&mut self, rhs: f64) {
        self.0.divide_by(rhs);
    }
}
