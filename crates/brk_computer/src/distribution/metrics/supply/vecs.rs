use brk_cohort::{
    AgeRange, AgeRangeId, Amount, CohortContext, Filter, UTXOGroups, UTXOGroupsWithoutAmount,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    Cents, Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, Version,
};
use vecdb::{AnyStoredVec, BinaryTransform, CachedBoxedVec, Database, Rw, StorageMode};

use crate::{
    distribution::{metrics::UTXORows, state::UnrealizedState},
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarValuePerBlockCumulativeRolling, HalveCents, HalveDollars,
        HalveSats, HalveSatsToBitcoin, LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight,
        LazyValuePerBlock, LazyValuePerBlockCumulativeRolling, SatsToCents, Windows,
    },
};

use super::{SupplyBase, SupplyByCohort, SupplySources, SupplyTotal};

const MATURED_VERSION: Version = Version::new(5);

#[derive(Traversable)]
pub struct SupplyVecs<M: StorageMode = Rw> {
    /// Amount of bitcoin held in unspent transaction outputs in the selected
    /// cohort.
    pub total: SupplyTotal<M>,
    /// Amount of unspent bitcoin that crosses out of the selected exact age
    /// range during the represented block interval.
    pub matured: ColumnarValuePerBlockCumulativeRolling<
        AgeRangeId,
        AgeRange<LazyValuePerBlockCumulativeRolling>,
        M,
    >,
    /// One half of the selected cohort's unspent supply.
    pub half: UTXOGroupsWithoutAmount<LazyValuePerBlock>,
    /// Unspent supply whose creation price is less than or equal to the current
    /// spot price.
    pub in_profit: SupplyByCohort<M>,
    /// Unspent supply whose creation price is greater than the current spot
    /// price.
    pub in_loss: SupplyByCohort<M>,
    /// Change in the selected cohort's unspent supply over the named trailing
    /// window, with the percentage change measured against the window's
    /// starting value.
    pub delta:
        UTXOGroups<LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>>,
    #[traversable(wrap = "delta", rename = "addr_balance")]
    /// Change in unspent supply controlled by addresses in the selected balance
    /// cohort over the named trailing window, with the percentage change
    /// measured against the window's starting value.
    pub addr_balance_delta:
        Amount<LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>>,
    /// Share of all unspent supply held by the selected cohort.
    pub dominance: UTXOGroups<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "dominance", rename = "addr_balance")]
    /// Share of all unspent supply controlled by addresses in the selected
    /// balance cohort.
    pub addr_balance_dominance: Amount<LazyPercentPerBlock<PartsPerMillion32>>,
}

impl SupplyVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let total = SupplyTotal::forced_import(db, version, indexes, spot_price)?;
        let all_supply = total.all_supply();
        let in_profit =
            SupplyByCohort::forced_import(db, "supply_in_profit", version, indexes, spot_price)?;
        let in_loss =
            SupplyByCohort::forced_import(db, "supply_in_loss", version, indexes, spot_price)?;
        let bases = total.cohorts.map_named(|filter, cohort_name, total| {
            let full_name = CohortContext::Utxo.full_name(filter, cohort_name);
            if matches!(filter, Filter::All) {
                SupplyBase::from_all_total(
                    &full_name,
                    version,
                    total.clone(),
                    indexes,
                    cached_starts,
                )
            } else {
                SupplyBase::from_total(
                    &full_name,
                    version,
                    total.clone(),
                    all_supply,
                    indexes,
                    cached_starts,
                )
            }
        });
        let delta = bases.map_named(|_, _, base| base.delta.clone());
        let dominance = bases.map_named(|_, _, base| base.dominance.clone());
        let addr_balance_bases = total.addr_balance.series.map_named(|filter, name, total| {
            let full_name = CohortContext::Addr.full_name(filter, name);
            SupplyBase::from_total(
                &full_name,
                version + Version::ONE,
                total.clone(),
                all_supply,
                indexes,
                cached_starts,
            )
        });
        let addr_balance_delta = addr_balance_bases.map_named(|_, _, base| base.delta.clone());
        let addr_balance_dominance =
            addr_balance_bases.map_named(|_, _, base| base.dominance.clone());
        let half = in_profit.cohorts.map_named(|filter, cohort_name, _| {
            let full_name = CohortContext::Utxo.full_name(filter, cohort_name);
            LazyValuePerBlock::from_spot_block_source::<
                HalveSats,
                HalveSatsToBitcoin,
                HalveCents,
                HalveDollars,
            >(
                &SupplyBase::metric_name(&full_name, "supply_half"),
                total.get(filter).expect("supported half-supply view"),
                version,
            )
        });
        let matured_version = version + MATURED_VERSION;
        let matured = ColumnarValuePerBlockCumulativeRolling::forced_import(
            db,
            &format!(
                "{}_age_range_matured_supply_cumulative",
                CohortContext::Utxo.prefix()
            ),
            matured_version,
            |sats, cents| {
                AgeRangeId::series(CohortContext::Utxo, |column, name| {
                    let name = format!("{name}_matured_supply");
                    let (sats, cents) =
                        ColumnarValuePerBlockCumulativeRolling::<AgeRangeId, ()>::sources_from(
                            sats,
                            cents,
                            &format!("{name}_cumulative"),
                            matured_version,
                            [column],
                        );
                    LazyValuePerBlockCumulativeRolling::from_boxed_cumulative_sources(
                        &name,
                        matured_version,
                        sats,
                        cents,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;

        Ok(Self {
            total,
            matured,
            half,
            in_profit,
            in_loss,
            delta,
            addr_balance_delta,
            dominance,
            addr_balance_dominance,
        })
    }

    pub fn sources(&self, filter: &Filter) -> Option<SupplySources> {
        Some(SupplySources {
            total: self.total.get(filter)?.clone(),
            in_profit: self.in_profit.get(filter)?.clone(),
        })
    }

    pub fn min_resume_len(&self) -> usize {
        self.total
            .min_len()
            .min(self.matured.len())
            .min(self.in_profit.min_len())
            .min(self.in_loss.min_len())
    }

    #[inline(always)]
    pub fn push_maturation(&mut self, matured: &AgeRange<Sats>, price: Cents) {
        let cents = AgeRange::from_fn(|column| SatsToCents::apply(*column.select(matured), price));
        self.matured.push_block(matured.clone(), cents);
    }

    #[inline(always)]
    pub fn push(&mut self, total: UTXORows<Sats>, profitability: &UTXORows<UnrealizedState>) {
        let in_profit = profitability.map(|state| state.supply_in_profit);
        let in_loss = profitability.map(|state| state.supply_in_loss);

        self.total.push(total);
        self.in_profit.push(in_profit);
        self.in_loss.push(in_loss);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.total.collect_vecs_mut();
        vecs.extend(self.matured.collect_vecs_mut());
        vecs.extend(self.in_profit.collect_vecs_mut());
        vecs.extend(self.in_loss.collect_vecs_mut());
        vecs
    }
}
