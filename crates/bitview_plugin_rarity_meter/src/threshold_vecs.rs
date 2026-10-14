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
