use std::ops::AddAssign;

use brk_types::{Cents, CentsSats, Sats};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RealizedTotals {
    pub(super) cap_raw: CentsSats,
    pub(super) supply: Sats,
}

impl RealizedTotals {
    pub(crate) fn price(&self) -> Cents {
        self.cap_raw
            .as_u128()
            .checked_div(self.supply.as_u128())
            .map(|price| Cents::new(price as u64))
            .unwrap_or_default()
    }
}

impl AddAssign for RealizedTotals {
    fn add_assign(&mut self, rhs: Self) {
        self.cap_raw += rhs.cap_raw;
        self.supply += rhs.supply;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_uses_total_raw_cap_and_supply() {
        let mut totals = RealizedTotals {
            cap_raw: CentsSats::new(100),
            supply: Sats::new(3),
        };
        totals += RealizedTotals {
            cap_raw: CentsSats::new(100),
            supply: Sats::new(7),
        };

        assert_eq!(totals.price(), Cents::new(20));
        assert_ne!(totals.price(), Cents::new(100 / 3 + 100 / 7));
    }

    #[test]
    fn price_is_zero_without_supply() {
        assert_eq!(RealizedTotals::default().price(), Cents::ZERO);
    }
}
