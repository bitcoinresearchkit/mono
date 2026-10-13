use bitview_cohort::AgeAggregate;
use bitview_distribution::state::cost_basis::{PriceTotals, age_index};
use brk_types::{Cents, CentsSats, Sats};

#[derive(Default)]
pub(crate) struct Bucket {
    pub(crate) supply: AgeAggregate<Sats>,
    pub(crate) cap: AgeAggregate<Cents>,
}

impl Bucket {
    pub(crate) fn between(before: PriceTotals<4>, after: PriceTotals<4>) -> Self {
        // Rounding the running totals, not each band, keeps a filter's bands summing to its total.
        let cents = |cap: i128| CentsSats::new(cap.max(0) as u128).to_cents_rounded();
        let values = AgeAggregate::from_fn(|id| {
            let (before_sats, before_cap) = age_index::selected(id, &before);
            let (after_sats, after_cap) = age_index::selected(id, &after);
            (
                Sats::new((after_sats - before_sats).max(0) as u64),
                cents(after_cap).saturating_sub(cents(before_cap)),
            )
        });
        Self {
            supply: AgeAggregate::from_fn(|id| id.select(&values).0),
            cap: AgeAggregate::from_fn(|id| id.select(&values).1),
        }
    }
}
