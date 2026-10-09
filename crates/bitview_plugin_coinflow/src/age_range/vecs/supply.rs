use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct SupplyVecs<T> {
    /// Supply in the age range multiplied by its mobility, rounded down to
    /// whole satoshis.
    pub mobile: T,
    /// Supply in the age range multiplied by one minus its mobility, rounded
    /// down to whole satoshis.
    pub immobile: T,
}
