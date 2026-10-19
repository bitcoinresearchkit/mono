use brk_types::{Bitcoin, Sats};

use crate::scalar::scalar;

scalar! {
    /// A mean amount in sats.
    float SatsFract(f64)
}

impl From<SatsFract> for Bitcoin {
    #[inline]
    fn from(value: SatsFract) -> Self {
        Self::from(f64::from(value) / Sats::ONE_BTC_U128 as f64)
    }
}
