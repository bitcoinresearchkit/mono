use bitview_cohort::AgeAggregate;
use bitview_distribution::state::cost_basis::{PriceTotals, age_index};
use brk_types::{Cents, CentsSats, Sats};

#[derive(Default)]
pub(crate) struct Bucket {
    pub(crate) supply: AgeAggregate<Sats>,
    pub(crate) cap: AgeAggregate<Cents>,
    /// The exact capital, for shares that stay proportional at a few cents.
    pub(crate) cap_raw: AgeAggregate<CentsSats>,
}

impl Bucket {
    pub(crate) fn between(before: PriceTotals<4>, after: PriceTotals<4>) -> Self {
        // Canonical balances and nested cuts: every total is nonnegative and grows with the cut.
        // Rounding the running totals, not each band, keeps a filter's bands summing to its total.
        let cents = |cap: i128| {
            debug_assert!(cap >= 0, "negative capital {cap}");
            CentsSats::new(cap as u128).to_cents_rounded().inner()
        };
        let values = AgeAggregate::from_fn(|id| {
            let (before_sats, before_cap) = age_index::selected(id, &before);
            let (after_sats, after_cap) = age_index::selected(id, &after);
            debug_assert!(
                before_sats <= after_sats && before_cap <= after_cap,
                "a cut's totals shrank"
            );
            (
                Sats::new((after_sats - before_sats) as u64),
                Cents::new(cents(after_cap) - cents(before_cap)),
                CentsSats::new((after_cap - before_cap) as u128),
            )
        });
        Self {
            supply: AgeAggregate::from_fn(|id| id.select(&values).0),
            cap: AgeAggregate::from_fn(|id| id.select(&values).1),
            cap_raw: AgeAggregate::from_fn(|id| id.select(&values).2),
        }
    }
}
