use bitview_traversable::Traversable;

/// A Bitcoin amount and its fiat value, with independently chosen representations.
/// Series expose BTC and USD over their sats and cents.
#[derive(Clone, Traversable)]
pub struct Value<S, C, B, U> {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars, each block's amount valued at that block's spot price (zero before
    /// the first exchange price, block 74,641).
    pub usd: U,
    /// Reported in US cents; 100 cents equal one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}

/// A Bitcoin amount at the spot price, in BTC, USD and satoshis.
#[derive(Clone, Traversable)]
pub struct SpotValue<S, B, U> {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars: the amount at the spot price (its market cap), zero before the
    /// first exchange price (block 74,641, August 2010).
    pub usd: U,
}
