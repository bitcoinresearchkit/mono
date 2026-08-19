use std::time::{Duration, Instant};

use brk_error::Error;
use brk_types::{Date, Height, OHLCCents, Timestamp};
use tracing::info;

/// Default cooldown period for unhealthy sources (5 minutes)
const DEFAULT_COOLDOWN_SECS: u64 = 5 * 60;

/// A price data source that can fetch OHLC data by date or timestamp.
pub trait PriceSource {
    fn name(&self) -> &'static str;

    /// Fetch daily OHLC for a date. Returns None if this source doesn't support date queries.
    fn get_date(&mut self, date: Date) -> Option<brk_error::Result<OHLCCents>>;

    /// Fetch minute OHLC for a timestamp range. Returns None if unsupported.
    fn get_1mn(
        &mut self,
        timestamp: Timestamp,
        previous_timestamp: Option<Timestamp>,
    ) -> Option<brk_error::Result<OHLCCents>>;

    /// Fetch OHLC by block height. Returns None if unsupported.
    fn get_height(&mut self, height: Height) -> Option<brk_error::Result<OHLCCents>>;

    /// Check if the source is reachable
    fn ping(&self) -> brk_error::Result<()>;

    /// Clear cached data
    fn clear(&mut self);
}

/// Wraps a price source with health tracking.
/// Automatically skips blocked/unreachable sources and rechecks after cooldown.
#[derive(Clone)]
pub struct TrackedSource<T> {
    source: T,
    unhealthy_since: Option<Instant>,
    cooldown: Duration,
}

impl<T: PriceSource> TrackedSource<T> {
    pub fn new(source: T) -> Self {
        Self {
            source,
            unhealthy_since: None,
            cooldown: Duration::from_secs(DEFAULT_COOLDOWN_SECS),
        }
    }

    pub fn name(&self) -> &'static str {
        self.source.name()
    }

    fn is_healthy(&self) -> bool {
        match self.unhealthy_since {
            None => true,
            Some(since) => since.elapsed() >= self.cooldown,
        }
    }

    fn remaining_cooldown(&self) -> u64 {
        self.unhealthy_since
            .map(|since| self.cooldown.saturating_sub(since.elapsed()).as_secs())
            .unwrap_or(0)
    }

    /// Try to fetch, tracking health state
    fn try_fetch<R>(
        &mut self,
        fetch: impl FnOnce(&mut T) -> Option<brk_error::Result<R>>,
    ) -> Option<brk_error::Result<R>> {
        if !self.is_healthy() {
            return Some(Err(Error::FetchFailed(format!(
                "{} temporarily disabled (recheck in {}s)",
                self.name(),
                self.remaining_cooldown()
            ))));
        }

        let result = fetch(&mut self.source)?;
        self.update_health(&result);
        Some(result)
    }

    fn update_health<R>(&mut self, result: &brk_error::Result<R>) {
        match result {
            Ok(_) => {
                if self.unhealthy_since.take().is_some() {
                    info!("{} is back online", self.name());
                }
            }
            Err(e) if e.is_network_permanently_blocked() => {
                if self.unhealthy_since.is_none() {
                    info!(
                        "{} marked unhealthy (blocked/unreachable), recheck in {}s",
                        self.name(),
                        self.cooldown.as_secs()
                    );
                }
                self.unhealthy_since = Some(Instant::now());
            }
            Err(_) => {} // Transient - no change
        }
    }

    pub fn reset_health(&mut self) {
        self.unhealthy_since = None;
    }
}

impl<T: PriceSource> PriceSource for TrackedSource<T> {
    fn name(&self) -> &'static str {
        self.source.name()
    }

    fn get_date(&mut self, date: Date) -> Option<brk_error::Result<OHLCCents>> {
        self.try_fetch(|s| s.get_date(date))
    }

    fn get_1mn(
        &mut self,
        timestamp: Timestamp,
        previous_timestamp: Option<Timestamp>,
    ) -> Option<brk_error::Result<OHLCCents>> {
        self.try_fetch(|s| s.get_1mn(timestamp, previous_timestamp))
    }

    fn get_height(&mut self, height: Height) -> Option<brk_error::Result<OHLCCents>> {
        self.try_fetch(|s| s.get_height(height))
    }

    fn ping(&self) -> brk_error::Result<()> {
        self.source.ping()
    }

    fn clear(&mut self) {
        self.source.clear();
        self.reset_health();
    }
}
