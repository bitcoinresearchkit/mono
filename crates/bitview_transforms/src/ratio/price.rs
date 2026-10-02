use brk_types::{Cents, PriceRatio};
use vecdb::unlikely;

#[inline]
pub fn price_ratio(close: Cents, price: Cents) -> PriceRatio {
    if unlikely(price == Cents::ZERO) {
        PriceRatio::NAN
    } else {
        PriceRatio::from(f64::from(close) / f64::from(price))
    }
}
