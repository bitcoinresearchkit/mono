use brk_types::{Cents, CentsCompact, Sats, StoredF64, UrpdRaw};

use super::{
    DayUrpds, LEVEL_IDS, Levels, LossPercentileId, ModeId, ModeResult, Modes, Percentiles,
    PriceBands, Thresholds,
};

const LEVEL_PERCENTILES: Levels<f64> = Levels {
    pct10: 0.1,
    pct20: 0.2,
    pct30: 0.3,
    pct40: 0.4,
    pct50: 0.5,
    pct60: 0.6,
    pct70: 0.7,
    pct80: 0.8,
    pct90: 0.9,
};

pub struct DayResult {
    pub by_mode: Modes<ModeResult>,
}

impl DayResult {
    pub fn from_thresholds(thresholds: &Thresholds) -> Self {
        Self {
            by_mode: Modes::from_fn(|mode| ModeResult {
                loss_threshold: match thresholds.select(mode) {
                    Some(values) => Percentiles::from_fn(|percentile| {
                        StoredF64::from(*percentile.select(values))
                    }),
                    None => Percentiles::from_fn(|_| StoredF64::NAN),
                },
                prices: PriceBands::from_fn(|_| Cents::NAN),
            }),
        }
    }

    pub fn evaluate(&mut self, urpds: &DayUrpds, thresholds: &Thresholds) {
        for mode in ModeId::ALL {
            let urpd = urpds.mode(mode);
            let denominator = urpd.map.values().copied().map(u64::from).sum::<u64>();
            let Some(thresholds) = thresholds.select(mode) else {
                continue;
            };
            if denominator == 0
                || !urpd
                    .map
                    .iter()
                    .any(|(price, sats)| price.inner() != 0 && *sats != Sats::ZERO)
            {
                continue;
            }

            let mut remaining_loss = denominator;
            let mut floors = Percentiles::from_fn(|_| Cents::NAN);
            let mut p95_floor = None;
            for (price, sats) in &urpd.map {
                remaining_loss -= u64::from(*sats);
                let remaining_share = remaining_loss as f64 / denominator as f64;
                for percentile in LossPercentileId::ALL {
                    let floor = percentile.select_mut(&mut floors);
                    if floor.is_nan() && remaining_share <= *percentile.select(thresholds) {
                        *floor = Cents::from(*price);
                        if percentile == LossPercentileId::Pct95 {
                            p95_floor = Some(*price);
                        }
                    }
                }
                if floors.iter().all(|floor| !floor.is_nan()) {
                    break;
                }
            }
            let mode_result = self.by_mode.select_mut(mode);
            mode_result.prices.floor = floors;
            if let Some(p95_floor) = p95_floor {
                mode_result.prices.level = Self::conditional_levels(urpd, p95_floor);
            }
        }
    }

    fn conditional_levels(urpd: &UrpdRaw, lower: CentsCompact) -> Levels<Cents> {
        let mut levels = Levels::from_fn(|_| Cents::NAN);
        let total = urpd
            .map
            .range(lower..)
            .map(|(_, sats)| u64::from(*sats))
            .sum::<u64>();
        if total == 0 {
            return levels;
        }

        let mut cumulative = 0_u64;
        let mut percentiles = LEVEL_IDS.iter().copied().peekable();
        for (price, sats) in urpd.map.range(lower..) {
            let sats = u64::from(*sats);
            if sats == 0 {
                continue;
            }
            cumulative += sats;
            while let Some(percentile) = percentiles.peek().copied()
                && cumulative as f64 >= total as f64 * *percentile.select(&LEVEL_PERCENTILES)
            {
                *percentile.select_mut(&mut levels) = Cents::from(*price);
                percentiles.next();
            }
            if percentiles.peek().is_none() {
                break;
            }
        }
        levels
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Cents, StoredF64};

    use super::DayResult;
    use crate::{DayUrpds, Levels, Percentiles, Thresholds};

    fn repeated_urpds<const N: usize>(entries: [(u32, u64); N]) -> DayUrpds {
        DayUrpds::repeated(entries)
    }

    #[test]
    fn calibrated_loss_share_sets_floor_and_levels() {
        let urpds = repeated_urpds([(100, 50), (200, 50)]);
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 0.5)));
        let mut result = DayResult::from_thresholds(&thresholds);
        result.evaluate(&urpds, &thresholds);
        let result = &result.by_mode.coinflow;

        assert_eq!(
            result.loss_threshold,
            Percentiles::from_fn(|_| StoredF64::from(0.5))
        );
        assert_eq!(
            result.prices.floor,
            Percentiles::from_fn(|_| Cents::new(100))
        );
        assert_eq!(
            result.prices.level,
            Levels {
                pct10: Cents::new(100),
                pct20: Cents::new(100),
                pct30: Cents::new(100),
                pct40: Cents::new(100),
                pct50: Cents::new(100),
                pct60: Cents::new(200),
                pct70: Cents::new(200),
                pct80: Cents::new(200),
                pct90: Cents::new(200),
            }
        );
    }

    #[test]
    fn zero_cost_distribution_stays_missing() {
        let urpds = repeated_urpds([(0, 100)]);
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 1.0)));
        let mut result = DayResult::from_thresholds(&thresholds);
        result.evaluate(&urpds, &thresholds);
        assert!(result.by_mode.raw.prices.floor.pct95.is_nan());
    }
}
