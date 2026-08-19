use derive_more::Deref;

use super::{ModeId, Modes, Percentiles};

#[derive(Deref)]
pub struct Thresholds(Modes<Option<Percentiles<f64>>>);

impl Thresholds {
    pub fn from_fn(create: impl FnMut(ModeId) -> Option<Percentiles<f64>>) -> Self {
        Self(Modes::from_fn(create))
    }
}
