use bitview_cohort::AgeRange;
use bitview_compute::{CohortAccounting, collect_age_range};
use brk_types::{CentsSats, CentsSquaredSats, Height, Sats, Version};
use rayon::join;
use vecdb::ReadableVec;

use crate::Vecs;

/// Read-only accounting inputs for weighted consumers; no cloned source or result cache.
pub struct AccountingSources<'a> {
    supplies: AgeRange<&'a dyn ReadableVec<Height, Sats>>,
    loss_supplies: AgeRange<&'a dyn ReadableVec<Height, Sats>>,
    cap_raw: AgeRange<&'a dyn ReadableVec<Height, CentsSats>>,
    capitalized_cap_raw: AgeRange<&'a dyn ReadableVec<Height, CentsSquaredSats>>,
}

impl Vecs {
    pub fn accounting_sources(&self) -> AccountingSources<'_> {
        AccountingSources {
            supplies: AgeRange::from_fn(|id| {
                &id.select(&self.cohorts.supply.total.cohorts.age)
                    .sats
                    .height as _
            }),
            loss_supplies: AgeRange::from_fn(|id| {
                &id.select(&self.cohorts.supply.in_loss.cohorts.age)
                    .sats
                    .height as _
            }),
            cap_raw: AgeRange::from_fn(|id| id.select(&self.cohorts.realized.cap_raw.age) as _),
            capitalized_cap_raw: AgeRange::from_fn(|id| {
                id.select(&self.cohorts.realized.capitalized_cap_raw.age) as _
            }),
        }
    }
}

impl AccountingSources<'_> {
    pub fn version(&self) -> Version {
        Version::combine_all(
            self.supplies
                .iter()
                .map(|v| v.version())
                .chain(self.loss_supplies.iter().map(|v| v.version()))
                .chain(self.cap_raw.iter().map(|v| v.version()))
                .chain(self.capitalized_cap_raw.iter().map(|v| v.version())),
        )
    }

    pub fn min_len(&self) -> usize {
        self.supplies
            .iter()
            .map(|v| v.len())
            .chain(self.loss_supplies.iter().map(|v| v.len()))
            .chain(self.cap_raw.iter().map(|v| v.len()))
            .chain(self.capitalized_cap_raw.iter().map(|v| v.len()))
            .min()
            .unwrap_or_default()
    }

    pub fn collect_into(&self, start: usize, end: usize, target: &mut CohortAccounting) {
        let CohortAccounting {
            supplies,
            loss_supplies,
            cap_raw,
            capitalized_cap_raw,
        } = target;
        join(
            || {
                join(
                    || collect_age_range(&self.supplies, supplies, start, end),
                    || collect_age_range(&self.loss_supplies, loss_supplies, start, end),
                )
            },
            || {
                join(
                    || collect_age_range(&self.cap_raw, cap_raw, start, end),
                    || {
                        collect_age_range(
                            &self.capitalized_cap_raw,
                            capitalized_cap_raw,
                            start,
                            end,
                        )
                    },
                )
            },
        );
    }
}
