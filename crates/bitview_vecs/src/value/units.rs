use bitview_traversable::Traversable;

/// A Bitcoin amount and its fiat value, with independently chosen representations.
/// Series expose BTC and USD; sats and cents are their exact storage.
#[derive(Clone, Traversable)]
pub struct Value<S, C, B, U> {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars.
    pub usd: U,
    /// Reported in US cents; 100 cents equal one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}

/// A Bitcoin amount valued at the spot price: BTC and USD, with sats as its exact storage.
#[derive(Clone, Traversable)]
pub struct SpotValue<S, B, U> {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars.
    pub usd: U,
}
