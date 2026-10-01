use std::collections::BTreeMap;

use brk_error::{Error, Result};
use brk_types::{Date, Height, OHLCCents, Timestamp};
use serde_json::{Value, from_slice};
use tracing::info;
use ureq::Agent;

use crate::{
    PriceSource, checked_get, default_retry, new_agent,
    ohlc::{compute_ohlc_from_range, date_from_timestamp, ohlc_from_array, timestamp_from_secs},
};

#[derive(Clone)]
pub struct Kraken {
    agent: Agent,
    _1mn: Option<BTreeMap<Timestamp, OHLCCents>>,
    _1d: Option<BTreeMap<Date, OHLCCents>>,
}

impl Kraken {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::new_with_agent(new_agent(30))
    }

    pub(crate) fn new_with_agent(agent: Agent) -> Self {
        Self {
            agent,
            _1mn: None,
            _1d: None,
        }
    }
}

impl Kraken {
    fn get_from_1mn(
        &mut self,
        timestamp: Timestamp,
        previous_timestamp: Option<Timestamp>,
    ) -> Result<OHLCCents> {
        if self
            ._1mn
            .as_ref()
            .and_then(|m| m.last_key_value())
            .is_none_or(|(k, _)| k <= &timestamp)
        {
            self._1mn.replace(self.fetch_1mn()?);
        }
        compute_ohlc_from_range(
            self._1mn.as_ref().unwrap(),
            timestamp,
            previous_timestamp,
            "Kraken 1mn",
        )
    }

    pub fn fetch_1mn(&self) -> Result<BTreeMap<Timestamp, OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = Self::url(1);
            info!("Fetching {url}...");
            let bytes = checked_get(agent, &url)?;
            let json: Value = from_slice(&bytes)?;
            Self::parse_ohlc_response(&json)
        })
    }

    fn get_from_1d(&mut self, date: &Date) -> Result<OHLCCents> {
        if self
            ._1d
            .as_ref()
            .and_then(|m| m.last_key_value())
            .is_none_or(|(k, _)| k <= date)
        {
            self._1d.replace(self.fetch_1d()?);
        }
        self._1d
            .as_ref()
            .unwrap()
            .get(date)
            .cloned()
            .ok_or_else(|| Error::NotFound("Couldn't find date".into()))
    }

    pub fn fetch_1d(&self) -> Result<BTreeMap<Date, OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = Self::url(1440);
            info!("Fetching {url}...");
            let bytes = checked_get(agent, &url)?;
            let json: Value = from_slice(&bytes)?;
            Self::parse_date_ohlc_response(&json)
        })
    }

    /// Parse Kraken's nested JSON response: { result: { XXBTZUSD: [...] } }
    fn parse_ohlc_response(json: &Value) -> Result<BTreeMap<Timestamp, OHLCCents>> {
        let result = json
            .get("result")
            .and_then(|r| r.get("XXBTZUSD"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::Parse("Invalid Kraken response format".into()))?
            .iter()
            .filter_map(|v| v.as_array())
            .map(|arr| {
                let ts = arr.first().and_then(|v| v.as_u64()).unwrap_or(0);
                (timestamp_from_secs(ts), ohlc_from_array(arr))
            })
            .collect();
        Ok(result)
    }

    fn parse_date_ohlc_response(json: &Value) -> Result<BTreeMap<Date, OHLCCents>> {
        Self::parse_ohlc_response(json).map(|map| {
            map.into_iter()
                .map(|(ts, ohlc)| (date_from_timestamp(ts), ohlc))
                .collect()
        })
    }

    fn url(interval: usize) -> String {
        format!("https://api.kraken.com/0/public/OHLC?pair=XBTUSD&interval={interval}")
    }

    fn ping(&self) -> Result<()> {
        self.agent
            .get("https://api.kraken.com/0/public/Time")
            .call()?;
        Ok(())
    }
}

impl PriceSource for Kraken {
    fn name(&self) -> &'static str {
        "Kraken"
    }

    fn get_date(&mut self, date: Date) -> Option<Result<OHLCCents>> {
        Some(self.get_from_1d(&date))
    }

    fn get_1mn(
        &mut self,
        timestamp: Timestamp,
        previous_timestamp: Option<Timestamp>,
    ) -> Option<Result<OHLCCents>> {
        Some(self.get_from_1mn(timestamp, previous_timestamp))
    }

    fn get_height(&mut self, _height: Height) -> Option<Result<OHLCCents>> {
        None // Kraken doesn't support height-based queries
    }

    fn ping(&self) -> Result<()> {
        self.ping()
    }

    fn clear(&mut self) {
        self._1d.take();
        self._1mn.take();
    }
}

#[cfg(test)]
#[path = "../tests/unit/kraken.rs"]
mod tests;
