use brk_cohort::AgeRange;
use derive_more::{Deref, DerefMut};

use super::{ModeId, Modes};

#[derive(Deref, DerefMut)]
pub struct ModeWeights(Modes<Option<AgeRange<f64>>>);

impl ModeWeights {
    pub fn from_fn(create: impl FnMut(ModeId) -> Option<AgeRange<f64>>) -> Self {
        Self(Modes::from_fn(create))
    }
}
