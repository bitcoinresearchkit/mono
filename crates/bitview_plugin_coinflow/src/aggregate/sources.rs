use bitview_cohort::{UTXOAggregate, UTXOAggregateId};
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height, Sats};
use vecdb::{AnyStoredVec, Rw, StorageMode, WritableVec};

use crate::{Mobility, model::PrimaryValues};

#[derive(Traversable)]
pub struct AggregateSources<M: StorageMode = Rw> {
    pub supply: Mobility<UTXOAggregate<CachedSeries<Height, Sats, M>>>,
    /// Share of estimated mobile supply that is in loss: the sum of supply in
    /// loss multiplied by remaining-lifetime spending probability divided by
    /// the sum of total supply multiplied by that probability. Returns NaN
    /// when the weighted supply is zero.
    pub supply_in_loss_share: UTXOAggregate<CachedSeries<Height, BoundedRatio, M>>,
    /// Sum of creation-date USD value multiplied by remaining-lifetime spending
    /// probability across a set of UTXO age ranges. Creation-date value is each
    /// unspent output's BTC value multiplied by Bitcoin's spot price when it was
    /// created.
    pub cap: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Mobility-weighted mean creation price of supply estimated to move
    /// eventually: coinflow capitalization divided by estimated mobile supply
    /// in BTC. Returns zero when mobile supply is zero.
    pub price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Creation price weighted by invested value and remaining-lifetime spending probability:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub capitalized_price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    pub under_4m_price: CachedSeries<Height, Cents, M>,
    pub under_4m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub under_6m_price: CachedSeries<Height, Cents, M>,
    pub under_6m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_4m_price: CachedSeries<Height, Cents, M>,
    pub over_4m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_6m_price: CachedSeries<Height, Cents, M>,
    pub over_6m_capitalized_price: CachedSeries<Height, Cents, M>,
}

impl AggregateSources {
    pub(crate) fn push(&mut self, values: PrimaryValues) {
        self.under_4m_price.push(values.under_4m.realized_price());
        self.under_4m_capitalized_price
            .push(values.under_4m.capitalized_price.value());
        self.under_6m_price.push(values.under_6m.realized_price());
        self.under_6m_capitalized_price
            .push(values.under_6m.capitalized_price.value());
        self.over_4m_price.push(values.over_4m.realized_price());
        self.over_4m_capitalized_price
            .push(values.over_4m.capitalized_price.value());
        self.over_6m_price.push(values.over_6m.realized_price());
        self.over_6m_capitalized_price
            .push(values.over_6m.capitalized_price.value());
        let all = values.terms.short.merged(values.terms.long);
        for (id, state) in [
            (UTXOAggregateId::All, all),
            (UTXOAggregateId::Sth, values.terms.short),
            (UTXOAggregateId::Lth, values.terms.long),
        ] {
            id.select_mut(&mut self.supply.mobile)
                .push(state.weighted_supply);
            id.select_mut(&mut self.supply.immobile)
                .push(state.complement_supply);
            id.select_mut(&mut self.supply_in_loss_share)
                .push(state.supply_in_loss.value());
            id.select_mut(&mut self.cap).push(state.weighted_cap);
            id.select_mut(&mut self.capitalized_price)
                .push(state.capitalized_price.value());
            id.select_mut(&mut self.price).push(state.realized_price());
        }
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        [&mut self.supply.mobile, &mut self.supply.immobile]
            .into_iter()
            .flat_map(|group| group.iter_mut())
            .map(|v| v as &mut dyn AnyStoredVec)
            .chain(
                self.supply_in_loss_share
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain(
                [&mut self.cap, &mut self.price, &mut self.capitalized_price]
                    .into_iter()
                    .flat_map(|group| group.iter_mut())
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain([
                &mut self.under_4m_price as &mut dyn AnyStoredVec,
                &mut self.under_4m_capitalized_price,
                &mut self.under_6m_price,
                &mut self.under_6m_capitalized_price,
                &mut self.over_4m_price,
                &mut self.over_4m_capitalized_price,
                &mut self.over_6m_price,
                &mut self.over_6m_capitalized_price,
            ])
    }
}
