use std::collections::BTreeMap;

use brk_error::{Error, Result};
use brk_types::{Cents, Close, Date, Day1, Dollars, Height, High, Low, OHLCCents, Open, Timestamp};
use serde_json::{Value, from_slice};
use tracing::info;
use ureq::Agent;

use crate::{PriceSource, checked_get, default_retry, new_agent};

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
        Self::new_with_agent(new_agent(30))
    }

    pub(crate) fn new_with_agent(agent: Agent) -> Self {
        Self {
            agent,
            height_to_ohlc: BTreeMap::new(),
            day1_to_ohlc: BTreeMap::new(),
        }
    }
}

const API_URL: &str = "https://bitview.space/api/series";
const CHUNK_SIZE: usize = 10_000;

impl BRK {
    pub fn get_from_height(&mut self, height: Height) -> Result<OHLCCents> {
        let (key, offset) = Self::height_chunk(height);

        let needs_fetch = self
            .height_to_ohlc
            .get(&key)
            .is_none_or(|prices| offset >= prices.len());
        if needs_fetch {
            self.height_to_ohlc
                .insert(key, self.fetch_height_prices(key)?);
        }

        self.height_to_ohlc
            .get(&key)
            .and_then(|prices| prices.get(offset))
            .cloned()
            .ok_or_else(|| Error::NotFound("Couldn't find height in BRK".into()))
    }

    fn height_chunk(height: Height) -> (Height, usize) {
        let height = usize::from(height);
        let offset = height % CHUNK_SIZE;
        (Height::from(height - offset), offset)
    }

    fn fetch_height_prices(&self, height: Height) -> Result<Vec<OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = Self::chunk_url("price", "height", u64::from(height));
            info!("Fetching {url}...");

            let bytes = checked_get(agent, &url)?;
            let body: Value = from_slice(&bytes)?;

            body.as_array()
                .ok_or_else(|| Error::Parse("Expected JSON array".into()))?
                .iter()
                .map(Self::value_to_height_ohlc)
                .collect::<Result<Vec<_>>>()
        })
    }

    pub fn get_from_date(&mut self, date: Date) -> Result<OHLCCents> {
        let day1 = Day1::try_from(date)?;
        let (key, offset) = Self::day_chunk(day1);

        let needs_fetch = self
            .day1_to_ohlc
            .get(&key)
            .is_none_or(|prices| offset >= prices.len());
        if needs_fetch {
            self.day1_to_ohlc.insert(key, self.fetch_date_prices(key)?);
        }

        self.day1_to_ohlc
            .get(&key)
            .and_then(|prices| prices.get(offset))
            .cloned()
            .ok_or_else(|| Error::NotFound("Couldn't find date in BRK".into()))
    }

    fn day_chunk(day: Day1) -> (Day1, usize) {
        let day = usize::from(day);
        let offset = day % CHUNK_SIZE;
        (Day1::from(day - offset), offset)
    }

    fn fetch_date_prices(&self, day1: Day1) -> Result<Vec<OHLCCents>> {
        let agent = &self.agent;
        default_retry(|_| {
            let url = Self::chunk_url("price_ohlc", "day1", u64::from(day1));
            info!("Fetching {url}...");

            let bytes = checked_get(agent, &url)?;
            let body: Value = from_slice(&bytes)?;

            body.as_array()
                .ok_or_else(|| Error::Parse("Expected JSON array".into()))?
                .iter()
                .map(Self::value_to_ohlc)
                .collect::<Result<Vec<_>>>()
        })
    }

    fn chunk_url(series: &str, index: &str, start: u64) -> String {
        // The exclusive range end can exceed the index's u16/u32 storage width.
        format!(
            "{API_URL}/{series}/{index}/data?start={start}&end={}",
            start + CHUNK_SIZE as u64
        )
    }

    fn value_to_cents(value: &Value) -> Result<Cents> {
        let dollars = value
            .as_f64()
            .filter(|value| {
                value.is_finite() && *value >= 0.0 && (value * 100.0).round() < u64::MAX as f64
            })
            .ok_or_else(|| {
                Error::Parse("Expected a finite nonnegative price in cents range".into())
            })?;
        Ok(Cents::from(Dollars::from(dollars)))
    }

    fn value_to_height_ohlc(value: &Value) -> Result<OHLCCents> {
        // A block has one spot observation, so its candle has no intrablock range.
        let price = Self::value_to_cents(value)?;
        Ok(OHLCCents::from(Close::new(price)))
    }

    fn value_to_ohlc(value: &Value) -> Result<OHLCCents> {
        let ohlc = value
            .as_array()
            .filter(|values| values.len() == 4)
            .ok_or_else(|| Error::Parse("Expected OHLC array".into()))?;

        let get_value = |index: usize| -> Result<_> { Self::value_to_cents(&ohlc[index]) };

        Ok(OHLCCents::from((
            Open::new(get_value(0)?),
            High::new(get_value(1)?),
            Low::new(get_value(2)?),
            Close::new(get_value(3)?),
        )))
    }

    fn ping(&self) -> Result<()> {
        self.agent.get(API_URL).call()?;
        Ok(())
    }
}

impl PriceSource for BRK {
    fn name(&self) -> &'static str {
        "BRK"
    }

    fn get_date(&mut self, date: Date) -> Option<Result<OHLCCents>> {
        Some(self.get_from_date(date))
    }

    fn get_1mn(
        &mut self,
        _timestamp: Timestamp,
        _previous_timestamp: Option<Timestamp>,
    ) -> Option<Result<OHLCCents>> {
        None // BRK doesn't support timestamp-based queries
    }

    fn get_height(&mut self, height: Height) -> Option<Result<OHLCCents>> {
        Some(self.get_from_height(height))
    }

    fn ping(&self) -> Result<()> {
        self.ping()
    }

    fn clear(&mut self) {
        self.height_to_ohlc.clear();
        self.day1_to_ohlc.clear();
    }
}
