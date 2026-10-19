use bitview_traversable::Traversable;

/// A fiat value represented in cents and US dollars.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct Fiat<C, U> {
    /// Reported in US dollars.
    pub usd: U,
    /// Reported in US cents; 100 cents equal one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}
