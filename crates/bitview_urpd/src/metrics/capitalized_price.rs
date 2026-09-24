use brk_types::{Cents, CentsCompact, Sats};

/// Capital-weighted mean of the already-rounded URPD buckets. The moments
/// exist only during this scan; no second-moment history is retained.
pub(crate) fn capitalized_price(entries: impl IntoIterator<Item = (CentsCompact, Sats)>) -> Cents {
    let mut first = 0_u128;
    let mut second = 0_u128;
    for (price, sats) in entries {
        let Some(price) = price.finite_inner().map(u128::from) else {
            return Cents::NAN;
        };
        let mass = price * sats.as_u128();
        let Some(next_first) = first.checked_add(mass) else {
            return Cents::NAN;
        };
        let Some(next_second) = mass
            .checked_mul(price)
            .and_then(|value| second.checked_add(value))
        else {
            return Cents::NAN;
        };
        first = next_first;
        second = next_second;
    }
    second
        .checked_div(first)
        .map(Cents::from)
        .unwrap_or(Cents::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn price(entries: &[(u32, u64)]) -> Cents {
        capitalized_price(
            entries
                .iter()
                .map(|&(p, s)| (CentsCompact::new(p), Sats::from(s))),
        )
    }

    #[test]
    fn weights_by_capital_not_coins_or_cohort_means() {
        // Coin-weighted average is 150; capital-weighted is 166.666... cents.
        assert_eq!(price(&[(100, 1), (200, 1)]), Cents::new(166));
        assert_eq!(price(&[(100, 3), (200, 1)]), Cents::new(140));
        // Zero-price and zero-mass buckets do not move either moment.
        assert_eq!(price(&[(0, 999), (100, 1), (200, 0)]), Cents::new(100));
    }

    #[test]
    fn undefined_and_tiny_denominators() {
        assert!(price(&[]).is_nan());
        assert!(price(&[(0, 100), (100, 0)]).is_nan());
        assert_eq!(price(&[(1, 1)]), Cents::new(1));
        assert_eq!(
            price(&[(u32::MAX - 1, 1)]),
            Cents::new(u64::from(u32::MAX - 1))
        );
    }

    #[test]
    fn bitcoin_supply_at_largest_bucket_fits_without_float_rounding() {
        assert_eq!(
            price(&[(u32::MAX - 1, 2_100_000_000_000_000)]),
            Cents::new(u64::from(u32::MAX - 1))
        );
    }
}
