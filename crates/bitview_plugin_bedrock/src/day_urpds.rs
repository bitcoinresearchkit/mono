use bitview_cohort::AgeRangeId;
use bitview_urpd::{accumulate_masses, collect_mass};
use brk_types::{CentsCompact, Sats};

use crate::{ModeId, ModeWeights, WeightedModeId, WeightedModes};

struct Masses {
    raw: Sats,
    weighted: WeightedModes<f64>,
}

impl Default for Masses {
    fn default() -> Self {
        Self {
            raw: Sats::ZERO,
            weighted: WeightedModes::from_fn(|_| 0.0),
        }
    }
}

pub struct DayUrpds {
    raw: Box<[(CentsCompact, Sats)]>,
    weighted: WeightedModes<Box<[(CentsCompact, Sats)]>>,
}

impl DayUrpds {
    pub fn from_age_entries(
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
        weights: &ModeWeights,
    ) -> Self {
        let buckets = accumulate_masses(entries, |bucket: &mut Masses, age, sats| {
            bucket.raw += sats;
            for id in WeightedModeId::ALL {
                if let Some(weights) = weights.select(id.mode()) {
                    *bucket.weighted.select_mut(id) +=
                        u64::from(sats) as f64 * *age.select(weights);
                }
            }
        });
        Self {
            raw: buckets.iter().map(|(&price, b)| (price, b.raw)).collect(),
            weighted: WeightedModes::from_fn(|id| {
                collect_mass(&buckets, |b| *b.weighted.select(id))
            }),
        }
    }
    pub fn mode(&self, mode: ModeId) -> &[(CentsCompact, Sats)] {
        match mode.weighted() {
            None => &self.raw,
            Some(id) => self.weighted.select(id),
        }
    }
    #[cfg(test)]
    pub fn repeated<const N: usize>(entries: [(u32, u64); N]) -> Self {
        let raw = || {
            entries
                .into_iter()
                .map(|(p, s)| (CentsCompact::new(p), Sats::from(s)))
                .collect()
        };
        Self {
            raw: raw(),
            weighted: WeightedModes::from_fn(|_| raw()),
        }
    }
}

#[cfg(test)]
mod tests {
    use bitview_cohort::AgeRange;

    use super::*;

    #[test]
    fn shared_buckets_preserve_raw_supply_and_round_after_weighting() {
        let entries = [
            (AgeRangeId::Under1H, 300, 5),
            (AgeRangeId::From4MTo5M, 100, 1),
            (AgeRangeId::Under1H, 100, 1),
            (AgeRangeId::Over15Y, 200, 3),
            (AgeRangeId::Under1H, 400, 0),
        ]
        .map(|(age, price, sats)| (age, CentsCompact::new(price), Sats::from(sats as u64)));
        let weights = ModeWeights::from_fn(|mode| match mode {
            ModeId::Cointime => Some(AgeRange::from_fn(|_| 0.5)),
            ModeId::Coinflow => Some(AgeRange::from_fn(|age| {
                if age == AgeRangeId::Under1H { 1.0 } else { 0.0 }
            })),
            _ => None,
        });
        let urpds = DayUrpds::from_age_entries(entries, &weights);
        let values = |mode| {
            urpds
                .mode(mode)
                .iter()
                .map(|&(price, sats)| (price.inner(), u64::from(sats)))
                .collect::<Vec<_>>()
        };

        assert_eq!(
            values(ModeId::Raw),
            [(100, 2), (200, 3), (300, 5), (400, 0)]
        );
        assert_eq!(values(ModeId::Cointime), [(100, 1), (200, 1), (300, 2)]);
        assert_eq!(values(ModeId::Coinflow), [(100, 1), (300, 5)]);
        for mode in WeightedModeId::COINFLOW_HORIZONS {
            assert!(urpds.mode(mode.mode()).is_empty());
        }
    }
}
