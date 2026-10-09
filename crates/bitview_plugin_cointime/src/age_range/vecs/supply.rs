use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct SupplyVecs<T> {
    /// Supply in the age range multiplied by its wakefulness, rounded down to
    /// whole satoshis.
    pub awake: T,
    /// Supply in the age range multiplied by one minus its wakefulness, rounded
    /// down to whole satoshis.
    pub dormant: T,
}
