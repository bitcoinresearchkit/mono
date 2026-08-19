use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, Sats};

use bitview_compute::Resolutions;

#[derive(Clone, Traversable)]
pub struct SplitCloseByUnit {
    /// Reported in USD per BTC.
    pub usd: Resolutions<Dollars>,
    /// Reported in cents per BTC.
    pub cents: Resolutions<Cents>,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: Resolutions<Sats>,
}
