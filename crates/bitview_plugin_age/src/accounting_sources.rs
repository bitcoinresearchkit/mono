use bitview_cohort::AgeRange;
use bitview_compute::{CohortAccounting, collect_age_range};
use bitview_primitives::CentsSquaredSats;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, CentsSats, Height, Sats, Version};
use rayon::join;
use vecdb::{AnyVec, LazyVec, ZstdVec};

use crate::Vecs;

/// Read-only accounting inputs for weighted consumers; no cloned source or result cache.
pub struct AccountingSources<'a> {
    supplies: AgeRange<&'a SupplySource>,
    loss_supplies: AgeRange<&'a SupplySource>,
    cap_raw: AgeRange<&'a ZstdVec<Height, CentsSats>>,
    capitalized_cap_raw: AgeRange<&'a ZstdVec<Height, CentsSquaredSats>>,
    unrealized_profits: AgeRange<&'a CachedSeries<Cents>>,
    unrealized_losses: AgeRange<&'a CachedSeries<Cents>>,
    capitals_in_profit: AgeRange<&'a CachedSeries<Cents>>,
    capitals_in_loss: AgeRange<&'a CachedSeries<Cents>>,
}

type SupplySource = LazyVec<Height, Sats, Sats>;

impl Vecs {
    pub fn accounting_sources(&self) -> AccountingSources<'_> {
        let ranges = &self.ranges;
        AccountingSources {
            supplies: AgeRange::from_fn(|id| &id.select(ranges).supply.total.value.sats.height),
            loss_supplies: AgeRange::from_fn(|id| {
                &id.select(ranges).supply.in_loss.value.sats.height
            }),
            cap_raw: AgeRange::from_fn(|id| &id.select(ranges).raw.cap),
            capitalized_cap_raw: AgeRange::from_fn(|id| &id.select(ranges).raw.capitalized_cap),
            unrealized_profits: AgeRange::from_fn(|id| &id.select(ranges).unrealized.profit.stored),
            unrealized_losses: AgeRange::from_fn(|id| &id.select(ranges).unrealized.loss.stored),
            capitals_in_profit: AgeRange::from_fn(|id| &id.select(ranges).capital.in_profit.stored),
            capitals_in_loss: AgeRange::from_fn(|id| &id.select(ranges).capital.in_loss.stored),
        }
    }
}

impl AccountingSources<'_> {
    /// Every input column.
    fn columns(&self) -> impl Iterator<Item = &dyn AnyVec> {
        self.supplies
            .iter()
            .map(|v| *v as &dyn AnyVec)
            .chain(self.loss_supplies.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.cap_raw.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.capitalized_cap_raw.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.unrealized_profits.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.unrealized_losses.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.capitals_in_profit.iter().map(|v| *v as &dyn AnyVec))
            .chain(self.capitals_in_loss.iter().map(|v| *v as &dyn AnyVec))
    }

    pub fn version(&self) -> Version {
        self.columns().map(|v| v.version()).sum::<Version>()
    }

    pub fn min_len(&self) -> usize {
        self.columns()
            .map(|v| v.len())
            .min()
            .expect("age ranges are nonempty")
    }

    pub fn collect_into(&self, start: usize, end: usize, target: &mut CohortAccounting) {
        let CohortAccounting {
            supplies,
            loss_supplies,
            cap_raw,
            capitalized_cap_raw,
            unrealized_profits,
            unrealized_losses,
            capitals_in_profit,
            capitals_in_loss,
        } = target;
        join(
            || {
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
                )
            },
            || {
                join(
                    || {
                        join(
                            || {
                                collect_age_range(
                                    &self.unrealized_profits,
                                    unrealized_profits,
                                    start,
                                    end,
                                )
                            },
                            || {
                                collect_age_range(
                                    &self.unrealized_losses,
                                    unrealized_losses,
                                    start,
                                    end,
                                )
                            },
                        )
                    },
                    || {
                        join(
                            || {
                                collect_age_range(
                                    &self.capitals_in_profit,
                                    capitals_in_profit,
                                    start,
                                    end,
                                )
                            },
                            || {
                                collect_age_range(
                                    &self.capitals_in_loss,
                                    capitals_in_loss,
                                    start,
                                    end,
                                )
                            },
                        )
                    },
                )
            },
        );
    }
}
