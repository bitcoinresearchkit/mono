use brk_types::Histogram;
use derive_more::{Deref, DerefMut};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::NUM_BINS;

#[derive(Clone, Debug, Default, Deref, DerefMut, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", schemars(transparent))]
pub struct HistogramRaw(Histogram<u32, NUM_BINS>);

impl HistogramRaw {
    #[inline]
    pub fn zeros() -> Self {
        Self(Histogram::zeros())
    }

    #[inline]
    pub(crate) fn increment(&mut self, bin: usize) {
        self.0.increment(bin);
    }
}
