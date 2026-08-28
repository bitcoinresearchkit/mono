#![doc = include_str!("../README.md")]

use std::io::Read as _;
use std::{path::Path, thread::sleep, time::Duration};

use brk_error::Error;
use brk_types::{Date, Height, OHLCCents, Timestamp};
use tracing::warn;
use ureq::Agent;

mod binance;
mod brk;
mod kraken;
mod ohlc;
mod retry;
mod source;

pub use binance::*;
pub use brk::*;
pub use kraken::*;
pub use ohlc::compute_ohlc_from_range;
use retry::*;
pub use source::{PriceSource, TrackedSource};

const MAX_RETRIES: usize = 12 * 60; // 12 hours of retrying

/// Create a shared HTTP agent with connection pooling and default timeout.
/// Status codes are not treated as errors — callers use `checked_get` for status handling.
pub fn new_agent(timeout_secs: u64) -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(timeout_secs)))
        .http_status_as_error(false)
        .build()
        .into()
}

/// Perform a GET request and check the response status.
pub fn checked_get(agent: &Agent, url: &str) -> brk_error::Result<Vec<u8>> {
    let mut response = agent.get(url).call()?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(Error::HttpStatus {
            status,
            url: url.to_string(),
        });
    }
    let mut bytes = Vec::new();
    response.body_mut().as_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[derive(Clone)]
pub struct Fetcher {
    pub agent: Agent,
    pub binance: TrackedSource<Binance>,
    pub kraken: TrackedSource<Kraken>,
    pub brk: TrackedSource<BRK>,
}

impl Fetcher {
    pub fn import(hars_path: Option<&Path>) -> brk_error::Result<Self> {
        Self::new(hars_path)
    }

    pub fn new(hars_path: Option<&Path>) -> brk_error::Result<Self> {
        let agent = new_agent(30);
        Ok(Self {
            binance: TrackedSource::new(Binance::new_with_agent(hars_path, agent.clone())),
            kraken: TrackedSource::new(Kraken::new_with_agent(agent.clone())),
            brk: TrackedSource::new(BRK::new_with_agent(agent.clone())),
            agent,
        })
    }

    /// Iterate over all active sources in priority order
    fn for_each_source<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn PriceSource),
    {
        f(&mut self.binance);
        f(&mut self.kraken);
        f(&mut self.brk);
    }

    /// Try fetching from each source in order, return first success
    fn try_sources<F>(&mut self, mut fetch: F) -> Option<brk_error::Result<OHLCCents>>
    where
        F: FnMut(&mut dyn PriceSource) -> Option<brk_error::Result<OHLCCents>>,
    {
        match fetch(&mut self.binance) {
            Some(Ok(ohlc)) => return Some(Ok(ohlc)),
            Some(Err(e)) => warn!("Binance fetch failed: {e}"),
            None => {}
        }
        match fetch(&mut self.kraken) {
            Some(Ok(ohlc)) => return Some(Ok(ohlc)),
            Some(Err(e)) => warn!("Kraken fetch failed: {e}"),
            None => {}
        }
        match fetch(&mut self.brk) {
            Some(Ok(ohlc)) => return Some(Ok(ohlc)),
            Some(Err(e)) => warn!("Brk fetch failed: {e}"),
            None => {}
        }
        None
    }

    pub fn get_date(&mut self, date: Date) -> brk_error::Result<OHLCCents> {
        self.fetch_with_retry(
            |source| source.get_date(date),
            || format!("Failed to fetch price for date {date}"),
        )
    }

    pub fn get_height(
        &mut self,
        height: Height,
        timestamp: Timestamp,
        previous_timestamp: Option<Timestamp>,
    ) -> brk_error::Result<OHLCCents> {
        let timestamp = timestamp.floor_seconds();
        let previous_timestamp = previous_timestamp.map(|t| t.floor_seconds());

        if previous_timestamp.is_none() && height != Height::ZERO {
            panic!("previous_timestamp required for non-genesis blocks");
        }

        self.fetch_with_retry(
            |source| {
                // Try 1mn data first, fall back to height-based
                source
                    .get_1mn(timestamp, previous_timestamp)
                    .or_else(|| source.get_height(height))
            },
            || {
                let date = Date::from(timestamp);
                format!(
                    "
Can't find the price for: height: {height} - date: {date}
1mn APIs are limited to the last 16 hours for Binance's and the last 10 hours for Kraken's
How to fix this:
0. Try rerunning the program first, it usually fixes the problem
1. If it didn't, go to https://www.binance.com/en/trade/BTC_USDT?type=spot
2. Select 1mn interval
3. Open the inspector/dev tools
4. Go to the Network Tab
5. Filter URLs by 'uiKlines'
6. Go back to the chart and scroll until you pass the date mentioned few lines ago
7. Go back to the dev tools
8. Export to a har file (if there is no explicit button, click on the cog button)
9. Move the file to 'parser/imports/binance.har'
"
                )
            },
        )
    }

    /// Try each source in order, with retries on total failure
    fn fetch_with_retry<F, E>(
        &mut self,
        mut fetch: F,
        error_message: E,
    ) -> brk_error::Result<OHLCCents>
    where
        F: FnMut(&mut dyn PriceSource) -> Option<brk_error::Result<OHLCCents>>,
        E: Fn() -> String,
    {
        for retry in 0..=MAX_RETRIES {
            if let Some(ohlc) = self.try_sources(&mut fetch) {
                return ohlc;
            }

            // All sources failed
            if retry < MAX_RETRIES {
                warn!("All price sources failed; retrying in 60s...");
                sleep(Duration::from_secs(60));
                self.clear_caches();
            }
        }

        Err(Error::FetchFailed(error_message()))
    }

    fn clear_caches(&mut self) {
        self.for_each_source(|s| s.clear());
    }

    /// Clear caches and reset health state for all sources
    pub fn clear(&mut self) {
        self.binance.clear();
        self.binance.reset_health();
        self.kraken.clear();
        self.kraken.reset_health();
        self.brk.clear();
        self.brk.reset_health();
    }

    /// Ping all sources and return results for each
    pub fn ping(&self) -> Vec<(&'static str, brk_error::Result<()>)> {
        vec![
            (self.binance.name(), self.binance.ping()),
            (self.kraken.name(), self.kraken.ping()),
            (self.brk.name(), self.brk.ping()),
        ]
    }
}
