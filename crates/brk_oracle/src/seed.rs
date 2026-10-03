use brk_types::Cents;

use crate::config::START_HEIGHT_SLOW;

/// Pre-oracle dollar prices, one per line, heights 0..START_HEIGHT_SLOW.
const PRICES: &str = include_str!("prices.txt");

/// Baked pre-oracle prices starting at `start_height`, as a one-pass iterator.
pub fn pre_oracle_prices_from(start_height: usize) -> impl Iterator<Item = Cents> {
    PRICES
        .lines()
        .take(START_HEIGHT_SLOW)
        .skip(start_height.min(START_HEIGHT_SLOW))
        .map(parse_price_cents)
}

fn parse_price_cents(line: &str) -> Cents {
    let dollars: f64 = line.parse().expect("invalid baked oracle price");
    Cents::new((dollars * 100.0).round() as u64)
}
