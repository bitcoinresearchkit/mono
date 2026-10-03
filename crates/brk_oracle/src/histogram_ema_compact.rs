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
pub struct HistogramEmaCompact(Histogram<u16, NUM_BINS>);

impl From<Histogram<u16, NUM_BINS>> for HistogramEmaCompact {
    fn from(histogram: Histogram<u16, NUM_BINS>) -> Self {
        Self(histogram)
    }
}
