use bitview_traversable::Traversable;

/// Independent amount and fiat sources, composed without coupling their storage.
#[derive(Clone, Traversable)]
pub struct SatsCents<S, C> {
    /// Reported in satoshis.
    pub sats: S,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: C,
}
