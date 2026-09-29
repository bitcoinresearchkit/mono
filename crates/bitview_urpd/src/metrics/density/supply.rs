use bitview_traversable::Traversable;
use brk_types::PartsPerMillion32;

#[cfg(test)]
use brk_types::{Cents, CentsCompact, Sats};

#[derive(Clone, Copy, Debug, PartialEq, Traversable)]
pub struct SupplyDensity<T> {
    /// Share of total weighted supply with cost basis within 5% below or above
    /// per-block closing spot, using rounded URPD creation-price buckets.
    pub total: T,
    /// Share of total weighted supply from the lower band boundary through spot.
    pub in_profit: T,
    /// Share of total weighted supply above spot through the upper band boundary.
    pub in_loss: T,
}

impl<T> SupplyDensity<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(&str) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            total: create("_total")?,
            in_profit: create("_in_profit")?,
            in_loss: create("_in_loss")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.total, &self.in_profit, &self.in_loss].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.total, &mut self.in_profit, &mut self.in_loss].into_iter()
    }
}

impl SupplyDensity<PartsPerMillion32> {
    pub const NAN: Self = Self {
        total: PartsPerMillion32::NAN,
        in_profit: PartsPerMillion32::NAN,
        in_loss: PartsPerMillion32::NAN,
    };

    #[cfg(test)]
    pub fn from_entries(
        entries: impl IntoIterator<Item = (CentsCompact, Sats)>,
        spot: Cents,
    ) -> Self {
        let Some(spot) = spot
            .finite_inner()
            .filter(|&price| price > 0)
            .map(u128::from)
        else {
            return Self::NAN;
        };
        let mut total = 0_u128;
        let mut profit = 0_u128;
        let mut loss = 0_u128;
        for (price, sats) in entries {
            let Some(price) = price.finite_inner().map(u128::from) else {
                return Self::NAN;
            };
            let sats = sats.as_u128();
            total += sats;
            // Integer hundredths of a cent keep the inclusive 5% boundaries exact.
            if price * 100 >= spot * 95 && price * 100 <= spot * 105 {
                if price <= spot {
                    profit += sats;
                } else {
                    loss += sats;
                }
            }
        }
        Self::from_sums(total, profit, loss)
    }

    pub fn from_sums(total: u128, profit: u128, loss: u128) -> Self {
        if total == 0 {
            return Self::NAN;
        }
        Self {
            total: PartsPerMillion32::from((profit + loss) as f64 / total as f64),
            in_profit: PartsPerMillion32::from(profit as f64 / total as f64),
            in_loss: PartsPerMillion32::from(loss as f64 / total as f64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn density(entries: &[(u32, u64)], spot: Cents) -> SupplyDensity<PartsPerMillion32> {
        SupplyDensity::from_entries(
            entries
                .iter()
                .map(|&(price, sats)| (CentsCompact::new(price), Sats::from(sats))),
            spot,
        )
    }

    #[test]
    fn includes_outer_bounds_and_assigns_spot_to_profit_once() {
        let result = density(
            &[(94, 10), (95, 20), (100, 30), (105, 40), (106, 100)],
            Cents::new(100),
        );
        assert!((f64::from(result.total) - 0.45).abs() < 1e-9);
        assert!((f64::from(result.in_profit) - 0.25).abs() < 1e-9);
        assert!((f64::from(result.in_loss) - 0.2).abs() < 1e-9);
        assert!(
            (f64::from(result.total) - f64::from(result.in_profit) - f64::from(result.in_loss))
                .abs()
                < 1e-9
        );
        // 95% of 101 is 95.95 cents: a 95-cent bucket is outside the band.
        let fractional = density(&[(95, 10), (96, 10), (106, 10), (107, 10)], Cents::new(101));
        assert!((f64::from(fractional.total) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn distinguishes_undefined_from_empty_band() {
        for entries in [&[][..], &[(100, 0)][..]] {
            assert_eq!(density(entries, Cents::new(100)), SupplyDensity::NAN);
        }
        for spot in [Cents::ZERO, Cents::NAN] {
            assert_eq!(density(&[(100, 10)], spot), SupplyDensity::NAN);
        }
        let outside = density(&[(200, 10)], Cents::new(100));
        assert!(
            outside
                .iter()
                .all(|value| *value == PartsPerMillion32::ZERO)
        );
        let largest = density(
            &[(u32::MAX - 1, 2_100_000_000_000_000)],
            Cents::new(u64::from(u32::MAX - 1)),
        );
        assert_eq!(largest.total, PartsPerMillion32::ONE);
        assert_eq!(largest.in_profit, PartsPerMillion32::ONE);
    }
}
