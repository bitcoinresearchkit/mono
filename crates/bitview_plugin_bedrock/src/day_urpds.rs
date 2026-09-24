use bitview_cohort::AgeRangeId;
use bitview_urpd::{UrpdRaw, accumulate_masses, collect_mass};
use brk_types::{CentsCompact, Sats};

use crate::{ModeId, ModeWeights, WeightedModeId, WeightedModes};

struct Masses(WeightedModes<f64>);
impl Default for Masses {
    fn default() -> Self {
        Self(WeightedModes::from_fn(|_| 0.0))
    }
}

pub struct DayUrpds {
    raw: UrpdRaw,
    weighted: WeightedModes<UrpdRaw>,
}

impl DayUrpds {
    pub fn from_age_entries(
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
        weights: &ModeWeights,
    ) -> Self {
        let mut raw = UrpdRaw::default();
        let buckets = accumulate_masses(entries, |bucket: &mut Masses, age, price, sats| {
            *raw.map.entry(price).or_default() += sats;
            for id in WeightedModeId::ALL {
                if let Some(weights) = weights.select(id.mode()) {
                    *bucket.0.select_mut(id) += u64::from(sats) as f64 * *age.select(weights);
                }
            }
        });
        Self {
            raw,
            weighted: WeightedModes::from_fn(|id| collect_mass(&buckets, |b| *b.0.select(id))),
        }
    }
    pub fn mode(&self, mode: ModeId) -> &UrpdRaw {
        match mode.weighted() {
            None => &self.raw,
            Some(id) => self.weighted.select(id),
        }
    }
    #[cfg(test)]
    pub fn repeated<const N: usize>(entries: [(u32, u64); N]) -> Self {
        let raw = || UrpdRaw {
            map: entries
                .into_iter()
                .map(|(p, s)| (CentsCompact::new(p), Sats::from(s)))
                .collect(),
        };
        Self {
            raw: raw(),
            weighted: WeightedModes::from_fn(|_| raw()),
        }
    }
}
