use bitview_distribution::state::cost_basis::{PriceTotals, age_index};
use brk_types::{Cents, CentsSats, Sats};

use crate::terms::Terms;

#[derive(Default)]
pub(crate) struct Bucket {
    pub(crate) supply: Terms<Sats>,
    pub(crate) cap: Terms<Cents>,
    /// The exact capital, for shares that stay proportional at a few cents.
    pub(crate) cap_raw: Terms<CentsSats>,
}

impl Bucket {
    pub(crate) fn between(before: PriceTotals<4>, after: PriceTotals<4>) -> Self {
        // Canonical balances and nested cuts: every total is nonnegative and grows with the cut.
        // Rounding the running totals, not each band, keeps a filter's bands summing to its total.
        let cents = |cap: i128| {
            debug_assert!(cap >= 0, "negative capital {cap}");
            CentsSats::new(cap as u128).to_cents().inner()
        };
        let values = Terms::from_fn(|id| {
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
            supply: Terms::from_fn(|id| values.select(id).0),
            cap: Terms::from_fn(|id| values.select(id).1),
            cap_raw: Terms::from_fn(|id| values.select(id).2),
        }
    }
}
