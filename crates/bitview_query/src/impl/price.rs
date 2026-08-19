use brk_types::{
    Dollars, ExchangeRates, HistoricalPrice, HistoricalPriceEntry, Hour4, INDEX_EPOCH, Timestamp,
};
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    pub fn historical_price(
        &self,
        timestamp: Option<Timestamp>,
    ) -> brk_error::Result<HistoricalPrice> {
        let prices = match timestamp {
            Some(ts) => self.price_at(ts)?,
            None => self.all_prices()?,
        };
        Ok(HistoricalPrice {
            prices,
            exchange_rates: ExchangeRates {},
        })
    }

    fn price_at(&self, target: Timestamp) -> brk_error::Result<Vec<HistoricalPriceEntry>> {
        if *target < INDEX_EPOCH {
            return Ok(vec![]);
        }
        let h4 = Hour4::from_timestamp(target);
        let cents = self.plugins().price.spot.cents.hour4.collect_one(h4);
        Ok(vec![HistoricalPriceEntry {
            time: h4.to_timestamp(),
            usd: Dollars::from(cents.flatten().unwrap_or_default()),
        }])
    }

    fn all_prices(&self) -> brk_error::Result<Vec<HistoricalPriceEntry>> {
        let plugins = self.plugins();
        Ok(plugins
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
            .collect())
    }
}
