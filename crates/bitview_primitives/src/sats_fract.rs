use crate::scalar::scalar;

scalar! {
    /// Fractional satoshis (f64): mean amounts in sats, and USD prices expressed in sats.
    ///
    /// A USD price in sats is `usd_value * 100_000_000 / btc_price`
    ///
    /// When BTC is $100,000:
    /// - $1 = 1,000 sats
    /// - $0.001 = 1 sat
    /// - $0.0001 = 0.1 sats (fractional)
    float SatsFract(f64)
}
