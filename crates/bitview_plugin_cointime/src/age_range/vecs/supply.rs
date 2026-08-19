use bitview_traversable::Traversable;

#[derive(Traversable)]
pub struct SupplyVecs<T> {
    /// Supply in each UTXO age range multiplied by that range's wakefulness.
    /// Each result is rounded down to whole satoshis.
    pub awake: T,
    /// Supply in each UTXO age range multiplied by one minus that range's
    /// wakefulness. Each result is rounded down to whole satoshis.
    pub dormant: T,
}
