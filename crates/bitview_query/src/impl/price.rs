use bitview_plugin::PublicationReadGuard;
use brk_error::{Error, OptionData, Result};
use brk_types::{
    Cents, Dollars, ExchangeRates, HOUR4_INTERVAL, Height, HistoricalPrice, HistoricalPriceEntry,
    INDEX_EPOCH, Index, Timestamp,
};
use vecdb::{AnyVec, ReadBounds, ReadableVec};

use crate::Query;

// A timestamp is u32 seconds. Reject corrupt lengths before reading the mapping.
const MAX_BUCKETS: usize = ((u32::MAX - INDEX_EPOCH) / HOUR4_INTERVAL) as usize + 1;

/// Validated historical-price sources held stable through response selection.
pub struct ResolvedHistoricalPrice<'a> {
    query: &'a Query,
    source_len: usize,
    _publication: PublicationReadGuard,
}

impl Query {
    pub fn resolve_historical_price(&self) -> Result<ResolvedHistoricalPrice<'_>> {
        let plugins = self.plugins();
        let publication = self.read_publication()?;
        let source_len = usize::from(self.safe_lengths().height);
        let prices = &plugins.price.spot.cents.height;
        if prices.len() < source_len || plugins.mappings.timestamp.monotonic.len() < source_len {
            return Err(Error::StateUpdating);
        }
        if plugins.mappings.first_height.hour4.len() > MAX_BUCKETS {
            return Err(Error::Internal(
                "Historical price mapping exceeds timestamp range",
            ));
        }
        Ok(ResolvedHistoricalPrice {
            query: self,
            source_len,
            _publication: publication,
        })
    }
}

impl ResolvedHistoricalPrice<'_> {
    /// Valid only for the publication held by this resolved read.
    pub fn revision(&self) -> u64 {
        self.query.indexer().publication().revision()
    }

    pub fn get(&self, timestamp: Option<Timestamp>) -> Result<HistoricalPrice> {
        let plugins = self.query.plugins();
        let source_len = self.source_len;
        let prices = &plugins.price.spot.cents.height;
        let mut bounds = ReadBounds::new();
        bounds.set(Index::Height.name(), source_len);
        bounds.scope(|| {
            let first_heights = &plugins.mappings.first_height.hour4;
            let mapping = first_heights.collect_range_dyn(0, first_heights.len());
            // Reuse one decompressed page while walking ordered closes.
            let mut cursor = prices.cursor();
            historical_prices(&mapping, source_len, timestamp, |height| {
                cursor.get(usize::from(height)).data()
            })
        })
    }
}

fn historical_prices(
    mapping: &[Height],
    source_len: usize,
    target: Option<Timestamp>,
    mut price_at: impl FnMut(Height) -> Result<Cents>,
) -> Result<HistoricalPrice> {
    if mapping.len() > MAX_BUCKETS
        || mapping
            .last()
            .is_some_and(|height| usize::from(*height) > source_len)
        || mapping.windows(2).any(|pair| pair[0] > pair[1])
    {
        return Err(Error::Internal("Invalid historical price mapping"));
    }
    // A following boundary proves completion. Never use the current partial close.
    let completed = mapping.len().saturating_sub(1);
    let end = target
        .map_or(completed, |target| {
            ((*target).saturating_sub(INDEX_EPOCH) / HOUR4_INTERVAL) as usize
        })
        .min(completed);
    let mut prices = Vec::with_capacity(if target.is_some() { 1 } else { end });
    let mut push = |index: usize| -> Result<()> {
        let next = usize::from(mapping[index + 1]);
        if usize::from(mapping[index]) == next {
            return Ok(());
        }
        let cents = price_at(Height::from(next - 1))?;
        if cents.is_nan() {
            return Err(Error::Internal("Invalid historical price"));
        }
        prices.push(HistoricalPriceEntry {
            // The mapping ceiling proves this arithmetic fits u32.
            time: Timestamp::new(INDEX_EPOCH + (index as u32 + 1) * HOUR4_INTERVAL),
            usd: Dollars::from(cents),
        });
        Ok(())
    };
    if target.is_some() {
        if let Some(index) = (0..end)
            .rev()
            .find(|&index| mapping[index] != mapping[index + 1])
        {
            push(index)?;
        }
    } else {
        for index in 0..end {
            push(index)?;
        }
    }
    Ok(HistoricalPrice {
        prices,
        exchange_rates: ExchangeRates {},
    })
}

#[cfg(test)]
#[path = "../../tests/unit/impl/price.rs"]
mod tests;
