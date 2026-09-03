use brk_error::Result;
use brk_types::{
    CacheClass, Dollars, ExchangeRates, HistoricalPrice, HistoricalPriceEntry, Hour4, INDEX_EPOCH,
    Index, Timestamp,
};
use vecdb::{AnyVec, ReadableVec};

use crate::Query;

/// One historical-price response resolved against its stable Hour4 boundary.
pub struct ResolvedHistoricalPrice {
    value: HistoricalPrice,
    stable: bool,
}

impl ResolvedHistoricalPrice {
    #[inline]
    pub const fn is_stable(&self) -> bool {
        self.stable
    }

    #[inline]
    pub fn into_value(self) -> HistoricalPrice {
        self.value
    }
}

impl Query {
    pub fn historical_price(&self, timestamp: Option<Timestamp>) -> Result<HistoricalPrice> {
        match timestamp {
            Some(timestamp) => self
                .resolve_historical_price(timestamp)
                .map(ResolvedHistoricalPrice::into_value),
            None => self.all_prices(),
        }
    }

    /// Resolve one requested Hour4 price and whether its bucket is outside the
    /// canonical volatile tail.
    pub fn resolve_historical_price(&self, target: Timestamp) -> Result<ResolvedHistoricalPrice> {
        if *target < INDEX_EPOCH {
            return Ok(ResolvedHistoricalPrice {
                value: price_response(vec![]),
                stable: true,
            });
        }

        let price = self.plugins().price;
        let _guard = self.read_plugin(price)?;
        let hour4 = Hour4::from_timestamp(target);
        let values = &price.spot.cents.hour4;
        let cents = values.collect_one(hour4);

        Ok(ResolvedHistoricalPrice {
            value: price_response(vec![HistoricalPriceEntry {
                time: hour4.to_timestamp(),
                usd: Dollars::from(cents.flatten().unwrap_or_default()),
            }]),
            stable: hour4_is_stable(hour4, values.len()),
        })
    }

    fn all_prices(&self) -> Result<HistoricalPrice> {
        let plugins = self.plugins();
        let _guard = self.read_plugin(plugins.price)?;
        let prices = plugins
            .price
            .spot
            .cents
            .hour4
            .collect()
            .into_iter()
            .enumerate()
            .filter_map(|(i, cents)| {
                Some(HistoricalPriceEntry {
                    time: Hour4::from(i).to_timestamp(),
                    usd: Dollars::from(cents?),
                })
            })
            .collect();
        Ok(price_response(prices))
    }
}

#[inline]
fn price_response(prices: Vec<HistoricalPriceEntry>) -> HistoricalPrice {
    HistoricalPrice {
        prices,
        exchange_rates: ExchangeRates {},
    }
}

#[inline]
fn hour4_is_stable(hour4: Hour4, total: usize) -> bool {
    match Index::Hour4.cache_class() {
        CacheClass::Bucket { margin } => usize::from(hour4) < total.saturating_sub(margin),
        CacheClass::Entity | CacheClass::Mutable => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hour4_stability_uses_the_canonical_volatile_tail() {
        assert!(hour4_is_stable(Hour4::from(7usize), 10));
        assert!(!hour4_is_stable(Hour4::from(8usize), 10));
        assert!(!hour4_is_stable(Hour4::from(10usize), 10));
    }
}
