use brk_types::Cents;

use crate::{config::START_HEIGHT_SLOW, scale::cents_to_bin};

/// Pre-oracle dollar prices, one per line, heights 0..START_HEIGHT_SLOW.
const PRICES: &str = include_str!("prices.txt");

/// Baked pre-oracle price at `height`, or `None` once on-chain oracle prices
/// start.
fn pre_oracle_price_cents(height: usize) -> Option<Cents> {
    if height >= START_HEIGHT_SLOW {
        return None;
    }
    PRICES.lines().nth(height).map(parse_price_cents)
}

/// Baked pre-oracle prices starting at `start_height`, as a one-pass iterator.
pub fn pre_oracle_prices_from(start_height: usize) -> impl Iterator<Item = Cents> {
    PRICES
        .lines()
        .take(START_HEIGHT_SLOW)
        .skip(start_height.min(START_HEIGHT_SLOW))
        .map(parse_price_cents)
}

/// Baked exchange price for the block immediately before on-chain oracle prices
/// start.
fn seed_price_cents() -> Cents {
    pre_oracle_price_cents(START_HEIGHT_SLOW - 1)
        .expect("prices.txt must cover height START_HEIGHT_SLOW - 1")
}

/// Initial reference bin for processing height START_HEIGHT_SLOW.
pub(super) fn seed_bin() -> f64 {
    cents_to_bin(seed_price_cents().inner() as f64)
}

fn parse_price_cents(line: &str) -> Cents {
    let dollars: f64 = line.parse().expect("invalid baked oracle price");
    Cents::new((dollars * 100.0).round() as u64)
}
