use brk_traversable::Traversable;
use derive_more::{Deref, DerefMut};

use super::WeightedModes;

#[derive(Deref, DerefMut, Traversable)]
pub struct Modes<T> {
    pub raw: T,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub weighted: WeightedModes<T>,
}
