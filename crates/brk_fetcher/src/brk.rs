use std::collections::BTreeMap;

use brk_error::Error;
use brk_types::{Cents, Close, Date, Day1, Dollars, Height, High, Low, OHLCCents, Open, Timestamp};
use serde_json::Value;
use tracing::info;
use ureq::Agent;

use crate::{PriceSource, checked_get, default_retry};

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct BRK {
    agent: Agent,
    height_to_ohlc: BTreeMap<Height, Vec<OHLCCents>>,
    day1_to_ohlc: BTreeMap<Day1, Vec<OHLCCents>>,
}

impl BRK {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::new_with_agent(crate::new_agent(30))
    }

    pub fn new_with_agent(agent: Agent) -> Self {
        Self {
            agent,
            height_to_ohlc: BTreeMap::new(),
            day1_to_ohlc: BTreeMap::new(),
        }
    }
}

const API_URL: &str = "https://bitview.space/api/vecs";
const CHUNK_SIZE: usize = 10_000;

impl BRK {
    pub fn get_from_height(&mut self, height: Height) -> brk_error::Result<OHLCCents> {
        let (key, offset) = Self::height_chunk(height);

        let needs_fetch = self
            .height_to_ohlc
            .get(&key)
            .is_none_or(|prices| key + prices.len() <= height);
        if needs_fetch {
            self.height_to_ohlc
                .insert(key, self.fetch_height_prices(key)?);
        }

        self.height_to_ohlc
            .get(&key)
            .and_then(|prices| prices.get(offset))
            .cloned()
            .ok_or(Error::NotFound("Couldn't find height in BRK".into()))
    }

    fn height_chunk(height: Height) -> (Height, usize) {
        let height = usize::from(height);
        let offset = height % CHUNK_SIZE;
        (Height::from(height - offset), offset)
    }

    fn fetch_height_prices(&self, height: Height) -> brk_error::Result<Vec<OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = format!(
                "{API_URL}/height-to-price-ohlc?from={}&to={}",
                height,
                height + CHUNK_SIZE
            );
            info!("Fetching {url} ...");

            let bytes = checked_get(agent, &url)?;
            let body: Value = serde_json::from_slice(&bytes)?;

            body.as_array()
                .ok_or(Error::Parse("Expected JSON array".into()))?
                .iter()
                .map(Self::value_to_ohlc)
                .collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get_from_date(&mut self, date: Date) -> brk_error::Result<OHLCCents> {
        let day1 = Day1::try_from(date)?;
        let (key, offset) = Self::day_chunk(day1);

        let needs_fetch = self
            .day1_to_ohlc
            .get(&key)
            .is_none_or(|prices| key + prices.len() <= day1);
        if needs_fetch {
            self.day1_to_ohlc.insert(key, self.fetch_date_prices(key)?);
        }

        self.day1_to_ohlc
            .get(&key)
            .and_then(|prices| prices.get(offset))
            .cloned()
            .ok_or(Error::NotFound("Couldn't find date in BRK".into()))
    }

    fn day_chunk(day: Day1) -> (Day1, usize) {
        let day = usize::from(day);
        let offset = day % CHUNK_SIZE;
        (Day1::from(day - offset), offset)
    }

    fn fetch_date_prices(&self, day1: Day1) -> brk_error::Result<Vec<OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = format!(
                "{API_URL}/day1-to-price-ohlc?from={}&to={}",
                day1,
                day1 + CHUNK_SIZE
            );
            info!("Fetching {url}...");

            let bytes = checked_get(agent, &url)?;
            let body: Value = serde_json::from_slice(&bytes)?;

            body.as_array()
                .ok_or(Error::Parse("Expected JSON array".into()))?
                .iter()
                .map(Self::value_to_ohlc)
                .collect::<Result<Vec<_>, _>>()
        })
    }

    fn value_to_ohlc(value: &Value) -> brk_error::Result<OHLCCents> {
        let ohlc = value
            .as_array()
            .ok_or(Error::Parse("Expected OHLC array".into()))?;

        let get_value = |index: usize| -> brk_error::Result<_> {
            Ok(Cents::from(Dollars::from(
                ohlc.get(index)
                    .ok_or(Error::Parse("Missing OHLC value at index".into()))?
                    .as_f64()
                    .ok_or(Error::Parse("Invalid OHLC value type".into()))?,
            )))
        };

        Ok(OHLCCents::from((
            Open::new(get_value(0)?),
            High::new(get_value(1)?),
            Low::new(get_value(2)?),
            Close::new(get_value(3)?),
        )))
    }

    pub fn ping(&self) -> brk_error::Result<()> {
        self.agent.get(API_URL).call()?;
        Ok(())
    }
}

impl PriceSource for BRK {
    fn name(&self) -> &'static str {
        "BRK"
    }

    fn get_date(&mut self, date: Date) -> Option<brk_error::Result<OHLCCents>> {
        Some(self.get_from_date(date))
    }

    fn get_1mn(
        &mut self,
        _timestamp: Timestamp,
        _previous_timestamp: Option<Timestamp>,
    ) -> Option<brk_error::Result<OHLCCents>> {
        None // BRK doesn't support timestamp-based queries
    }

    fn get_height(&mut self, height: Height) -> Option<brk_error::Result<OHLCCents>> {
        Some(self.get_from_height(height))
    }

    fn ping(&self) -> brk_error::Result<()> {
        self.ping()
    }

    fn clear(&mut self) {
        self.height_to_ohlc.clear();
        self.day1_to_ohlc.clear();
    }
}
