use std::ops::Range;

use bitview_plugin_indexer::SafeLengths;
use bitview_plugin_price::priced_range;
use bitview_primitives::{HOUR4_INTERVAL, INDEX_EPOCH, Index};
use bitview_types::{ExchangeRates, HistoricalPrice, HistoricalPriceEntry};
use brk_types::{Cents, Dollars, Timestamp};
use vecdb::{AnyVec, ReadBounds, ReadableVec};

use crate::{Error, OptionData, Query, Result};

// A timestamp is u32 seconds. Reject corrupt lengths before reading the mapping.
const MAX_BUCKETS: usize = ((u32::MAX - INDEX_EPOCH) / HOUR4_INTERVAL) as usize + 1;

/// Validated historical-price sources held stable through response selection.
pub struct ResolvedHistoricalPrice<'a> {
    query: &'a Query,
    source_len: usize,
    pin: SafeLengths,
}

impl Query {
    pub fn resolve_historical_price(&self) -> Result<ResolvedHistoricalPrice<'_>> {
        let plugins = self.plugins();
        let pin = self.pin_safe_lengths()?;
        let source_len = usize::from(pin.lengths().height);
        let prices = &plugins.price.spot.cents.height;
        if prices.len() < source_len || plugins.mappings.timestamp.monotonic.len() < source_len {
            return Err(Error::Internal(
                "Historical price is behind the published state",
            ));
        }
        if plugins.mappings.first_height.hour4.len() > MAX_BUCKETS {
            return Err(Error::Internal(
                "Historical price mapping exceeds timestamp range",
            ));
        }
        Ok(ResolvedHistoricalPrice {
            query: self,
            source_len,
            pin: pin.into_inner(),
        })
    }
}

impl ResolvedHistoricalPrice<'_> {
    /// The publication this read is pinned to.
    pub fn revision(&self) -> u64 {
        self.pin.revision()
    }

    /// Every completed four-hour close from the first priced one, or the last one before
    /// `timestamp`'s bucket (none before the first price).
    pub fn get(&self, timestamp: Option<Timestamp>) -> Result<HistoricalPrice> {
        let plugins = self.query.plugins();
        let source_len = self.source_len;
        let mut bounds = ReadBounds::new();
        bounds.set(Index::Height.name(), source_len);
        bounds.scope(|| {
            let first_heights = &plugins.mappings.first_height.hour4;
            let first = |index: usize| first_heights.collect_one_at(index).map(usize::from).data();
            // Only buckets whose first block is published: an update appends past them.
            let (mut len, mut high) = (0, first_heights.len());
            while len < high {
                let middle = len + (high - len) / 2;
                if first(middle)? < source_len {
                    len = middle + 1;
                } else {
                    high = middle;
                }
            }
            // A following boundary proves completion. Never use the current partial close.
            let completed = len.saturating_sub(1);
            // Reuse one decompressed page while walking ordered closes.
            let mut cursor = plugins.price.spot.cents.height.cursor();
            // Bucket `index` over `heights`, closed by its last block; none while it holds no
            // price.
            let mut entry =
                |index: usize, heights: Range<usize>| -> Result<Option<HistoricalPriceEntry>> {
                    let Some(close) = priced_range(heights).next_back() else {
                        return Ok(None);
                    };
                    let cents: Cents = cursor.get(close).data()?;
                    if cents.is_nan() {
                        return Err(Error::Internal("Invalid historical price"));
                    }
                    Ok(Some(HistoricalPriceEntry {
                        // MAX_BUCKETS proves this arithmetic fits u32.
                        time: Timestamp::new(INDEX_EPOCH + (index as u32 + 1) * HOUR4_INTERVAL),
                        usd: Dollars::from(cents),
                    }))
                };
            let prices = match timestamp {
                Some(target) => {
                    let end = (((*target).saturating_sub(INDEX_EPOCH) / HOUR4_INTERVAL) as usize)
                        .min(completed);
                    // The last bucket holding a block, walking back over empty ones.
                    let mut prices = Vec::with_capacity(1);
                    if end > 0 {
                        let mut next = first(end)?;
                        for index in (0..end).rev() {
                            let start = first(index)?;
                            if start != next {
                                prices.extend(entry(index, start..next)?);
                                break;
                            }
                            next = start;
                        }
                    }
                    prices
                }
                None => {
                    let mapping = first_heights.collect_range_dyn(0, len);
                    debug_assert!(mapping.is_sorted(), "bucket boundaries never decrease");
                    let mut prices = Vec::with_capacity(completed);
                    for (index, pair) in mapping.windows(2).enumerate() {
                        if pair[0] != pair[1] {
                            prices
                                .extend(entry(index, usize::from(pair[0])..usize::from(pair[1]))?);
                        }
                    }
                    prices
                }
            };
            Ok(HistoricalPrice {
                prices,
                exchange_rates: ExchangeRates {},
            })
        })
    }
}
