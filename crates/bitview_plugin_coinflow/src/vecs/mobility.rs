use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Mobility<T> {
    /// Supply multiplied by the estimated remaining-lifetime probability of
    /// spending for its UTXO age range. Each age-range contribution is rounded
    /// down to whole satoshis before aggregation.
    pub mobile: T,
    /// Supply multiplied by one minus the estimated remaining-lifetime
    /// probability of spending for its UTXO age range. Each age-range
    /// contribution is rounded down to whole satoshis before aggregation.
    pub immobile: T,
}
