use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::internal::{CentsUnsignedToDollars, LazyPreviousDeltaVec, SatsToBitcoin, ValuePerBlock};

/// Per-block amount data derived from stored cumulative sats and cents.
#[derive(Clone, Traversable)]
pub struct LazyValueBlock {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyVec<Height, Bitcoin, Height, Sats>,
    /// Reported in satoshis.
    pub sats: LazyPreviousDeltaVec<Height, Sats>,
    /// Reported in US dollars.
    pub usd: LazyVec<Height, Dollars, Height, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyPreviousDeltaVec<Height, Cents>,
}

impl LazyValueBlock {
    pub(crate) fn from_cumulative(
        name: &str,
        version: Version,
        cumulative: &ValuePerBlock,
    ) -> Self {
        Self::from_cumulative_sources(
            name,
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
        )
    }

    pub(crate) fn from_cumulative_sources(
        name: &str,
        version: Version,
        cumulative_sats: &(impl ReadableCloneableVec<Height, Sats> + 'static),
        cumulative_cents: &(impl ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Self {
        let sats = LazyPreviousDeltaVec::new(
            &format!("{name}_sats"),
            version,
            cumulative_sats.read_only_boxed_clone(),
        );
        let btc =
            LazyVec::transformed::<SatsToBitcoin>(name, version, sats.read_only_boxed_clone());
        let cents = LazyPreviousDeltaVec::new(
            &format!("{name}_cents"),
            version,
            cumulative_cents.read_only_boxed_clone(),
        );
        let usd = LazyVec::transformed::<CentsUnsignedToDollars>(
            &format!("{name}_usd"),
            version,
            cents.read_only_boxed_clone(),
        );

        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
