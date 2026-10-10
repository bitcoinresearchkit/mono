use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct ThresholdVecs<T> {
    /// Source-value boundary for a 0.1% historical tail share.
    pub tail0_1: T,
    /// Source-value boundary for a 0.05% historical tail share.
    pub tail0_05: T,
    /// Source-value boundary for a 0.025% historical tail share.
    pub tail0_025: T,
}

impl<T> ThresholdVecs<T> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.tail0_1, &mut self.tail0_05, &mut self.tail0_025].into_iter()
    }
}
