use std::collections::BTreeMap;

use brk_error::{Error, Result};
use brk_types::{Cents, Close, Date, Dollars, High, Low, OHLCCents, Open, Timestamp};
use serde_json::Value;
use tracing::warn;

/// Parse OHLC value from a JSON array element at given index
fn parse_cents(array: &[Value], index: usize) -> Cents {
    let value = array
        .get(index)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            warn!(
                "Failed to parse price at index {index}: {:?}",
                array.get(index)
            );
            0.0
        });
    Cents::from(Dollars::from(value))
}

/// Build OHLCCentsUnsigned from array indices 1-4 (open, high, low, close)
pub(crate) fn ohlc_from_array(array: &[Value]) -> OHLCCents {
    OHLCCents::from((
        Open::new(parse_cents(array, 1)),
        High::new(parse_cents(array, 2)),
        Low::new(parse_cents(array, 3)),
        Close::new(parse_cents(array, 4)),
    ))
}

/// Compute OHLC for a block from a time series of minute data.
/// Aggregates all candles between previous_timestamp and timestamp.
pub fn compute_ohlc_from_range(
    tree: &BTreeMap<Timestamp, OHLCCents>,
    timestamp: Timestamp,
    previous_timestamp: Option<Timestamp>,
    source_name: &str,
) -> Result<OHLCCents> {
    let previous_close = previous_timestamp.map_or(Some(Close::default()), |t| {
        tree.get(&t).map(|ohlc| ohlc.close)
    });

    let missing_timestamp = || Error::NotFound(format!("Couldn't find timestamp in {source_name}"));
    let previous_close = previous_close.ok_or_else(missing_timestamp)?;
    if !tree.contains_key(&timestamp) {
        return Err(missing_timestamp());
    }

    let mut result = OHLCCents::from(previous_close);

    let start = previous_timestamp.unwrap_or(Timestamp::new(0));
    let end = timestamp;

    // Skip if re-org (start >= end)
    if start < end {
        for (_, ohlc) in tree.range(start..=end).skip(1) {
            if ohlc.high > result.high {
                result.high = ohlc.high;
            }
            if ohlc.low < result.low {
                result.low = ohlc.low;
            }
            result.close = ohlc.close;
        }
    }

    Ok(result)
}

/// Parse timestamp from milliseconds (Binance format)
pub(crate) fn timestamp_from_ms(ms: u64) -> Timestamp {
    Timestamp::from((ms / 1_000) as u32)
}

/// Parse timestamp from seconds (Kraken format)
pub(crate) fn timestamp_from_secs(secs: u64) -> Timestamp {
    Timestamp::from(secs as u32)
}

/// Convert timestamp to date
pub(crate) fn date_from_timestamp(timestamp: Timestamp) -> Date {
    Date::from(timestamp)
}
