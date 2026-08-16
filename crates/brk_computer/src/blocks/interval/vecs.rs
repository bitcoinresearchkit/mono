use derive_more::{Deref, DerefMut};

use brk_traversable::Traversable;
use brk_types::Timestamp;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeAverage;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw>(
    /// Nonnegative difference in seconds between this block's header timestamp
    /// and the previous block's. Genesis is zero, and a timestamp earlier than
    /// its predecessor is clamped to zero.
    #[traversable(flatten)]
    pub PerBlockCumulativeAverage<Timestamp, Timestamp, M>,
);
