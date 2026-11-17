use bitview_cohort::{
    AgeAggregate, AgeAggregateId, BREAK_EVEN_DIVISOR, PROFITABILITY_EDGE_DIVISORS,
    ProfitabilityRange, ProfitabilityRangeId,
};
use bitview_distribution::{
    state::cost_basis::{
        DensityBand, GridCut, PriceIndex, PriceTotals,
        age_index::{self, AgeIndex},
    },
    walk::Block,
};
use bitview_vecs::Density;
use brk_types::{Cents, Version};

use crate::{bucket::Bucket, cost_basis::CostBasisBlockData};

/// Writer-owned derived state, kept only after a complete walk.
pub(crate) struct LiveState {
    pub index: AgeIndex,
    /// The inputs' combined version.
    pub version: Version,
}

/// Each band's supply and capital at `spot`, and the filters' totals.
pub(crate) fn ranges(index: &PriceIndex<4>, spot: Cents) -> (ProfitabilityRange<Bucket>, Bucket) {
    let mut result = ProfitabilityRange::default();
    let total = index.totals();
    if total.sats[0] <= 0 {
        return (result, Bucket::default());
    }
    let mut previous = PriceTotals::default();
    let mut cut = GridCut::default();
    for (i, divisor) in PROFITABILITY_EDGE_DIVISORS.into_iter().enumerate() {
        // Break-even counts the spot's own bucket as profit, like the holders' densities. Cuts never
        // move back: at a zero spot every edge is zero.
        cut = cut.max(if divisor == BREAK_EVEN_DIVISOR {
            GridCut::through(spot)
        } else {
            GridCut::below(spot.as_u128() * 100, u128::from(divisor))
        });
        let current = index.before_cut(cut);
        *ProfitabilityRangeId::ALL[i].select_mut(&mut result) = Bucket::between(previous, current);
        previous = current;
    }
    *ProfitabilityRangeId::ALL
        .last()
        .unwrap()
        .select_mut(&mut result) = Bucket::between(previous, total);
    (result, Bucket::between(PriceTotals::default(), total))
}

/// Every holder group's cost basis after `block`: its percentiles, extremes and densities.
pub(crate) fn cost_basis(live: &AgeIndex, block: &Block<'_>) -> AgeAggregate<CostBasisBlockData> {
    let index = &live.index;
    let total = index.totals();
    // Without a positive spot there is no band, as in the URPD densities.
    let split = DensityBand::new(block.price()).map(|band| index.density_split(band));
    let mut percentiles = index
        .percentiles::<{ AgeAggregateId::ALL.len() }>(|q, n| {
            age_index::selected(AgeAggregateId::ALL[q], n)
        })
        .into_iter();
    AgeAggregate::from_fn(|id| {
        let (sats, cap) = age_index::selected(id, &total);
        // Canonical balances and nested cuts keep every total nonnegative.
        let positive = |value: i128| {
            debug_assert!(value >= 0, "negative cost-basis total {value}");
            value as u128
        };
        let (supply_density, capital_density) = if let Some((profit, loss)) = &split {
            let (profit_sats, profit_cap) = age_index::selected(id, profit);
            let (loss_sats, loss_cap) = age_index::selected(id, loss);
            (
                Density::from_sums(
                    positive(sats.into()),
                    positive(profit_sats.into()),
                    positive(loss_sats.into()),
                ),
                Density::from_sums(positive(cap), positive(profit_cap), positive(loss_cap)),
            )
        } else {
            (Density::NAN, Density::NAN)
        };
        let (min, max) =
            age_index::extremes(&live.held, id, block.timestamps, block.height + 1);
        CostBasisBlockData {
            percentiles: percentiles.next().expect("one result per age filter"),
            min,
            max,
            supply_density,
            capital_density,
        }
    })
}
