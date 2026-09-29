use bitview_cohort::{
    AgeRange, AgeRangeId, CohortContext, CohortId, UTXOCoreValues, UTXOGroupsWithoutAmountOrType,
};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{
    HalveDollars, HalveSatsToBitcoin, SatsToCents, StoredU64ToCents, StoredU64ToSats,
};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight, LazyValuePerBlock,
    LazyValuePerBlockCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling, SatsCents,
};
use brk_error::Result;
use brk_types::{
    Cents, Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, StoredU64, Version,
};
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, Database, Halve, LazyVec, ReadableBoxedVec, Rw,
    StorageMode,
};

use super::{SupplyBase, SupplyByCohort, SupplySources, SupplyTotal};
use crate::state::UnrealizedState;

const MATURED_VERSION: Version = Version::new(5);

#[derive(Traversable)]
pub struct SupplyVecs<M: StorageMode = Rw> {
    /// Supply: amount of bitcoin held in unspent transaction outputs.
    pub total: SupplyTotal<M>,
    /// Amount of unspent bitcoin that ages out of an exact UTXO age range
    /// during the represented block interval.
    pub matured: AgeRange<LazyValuePerBlockCumulativeRolling>,
    #[traversable(hidden)]
    matured_sources: AgeRange<SatsCents<PerBlockCumulativeRolling<StoredU64, M>>>,
    /// One half of a UTXO cohort's unspent supply.
    pub half: UTXOGroupsWithoutAmountOrType<LazyValuePerBlock>,
    /// Unspent supply in profit: UTXO cohort outputs whose creation price is
    /// less than or equal to the represented block's spot price.
    pub in_profit: SupplyByCohort<M>,
    /// Unspent supply in loss: UTXO cohort outputs whose creation price is
    /// greater than the represented block's spot price.
    pub in_loss: SupplyByCohort<M>,
    /// Change in a UTXO or address-balance cohort's unspent supply over a trailing window, with
    /// the relative change measured against the window's starting value.
    pub delta: UTXOGroupsWithoutAmountOrType<
        LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    >,
    /// Share of all unspent supply held by a UTXO or address-balance cohort.
    pub dominance: UTXOGroupsWithoutAmountOrType<LazyPercentPerBlock<PartsPerMillion32>>,
}

impl SupplyVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Box<Self>> {
        let total = SupplyTotal::forced_import(db, version, mappings, spot_price, all_supply)?;
        let all_supply = total.all_supply();
        let in_profit =
            SupplyByCohort::forced_import(db, "supply_in_profit", version, mappings, spot_price)?;
        let in_loss =
            SupplyByCohort::forced_import(db, "supply_in_loss", version, mappings, spot_price)?;
        let utxo = total.cohorts.map_with_id(|cohort_id, total| {
            if matches!(cohort_id, CohortId::All) {
                SupplyBase::from_all_total(version, total.clone(), mappings, window_starts)
            } else {
                SupplyBase::from_total(
                    CohortContext::Utxo,
                    cohort_id,
                    version,
                    total.clone(),
                    all_supply,
                    mappings,
                    window_starts,
                )
            }
        });
        let bases = utxo;
        let delta = bases.map_with_id(|_, base| base.delta.clone());
        let dominance = bases.map_with_id(|_, base| base.dominance.clone());
        let half = in_profit.cohorts.map_with_id(|cohort_id, _| {
            LazyValuePerBlock::from_spot_block_source::<
                Halve,
                HalveSatsToBitcoin,
                Halve,
                HalveDollars,
            >(
                &CohortContext::Utxo.metric_name(cohort_id, "supply_half"),
                total.get(cohort_id).expect("supported half-supply view"),
                version,
            )
        });
        let matured_version = version + MATURED_VERSION;
        let matured_sources = AgeRange::try_from_fn(|id| -> Result<_> {
            let name = format!(
                "{}_matured_supply",
                CohortContext::Utxo.full_name(id.cohort())
            );
            Ok(SatsCents {
                sats: PerBlockCumulativeRolling::forced_import(
                    db,
                    &format!("{name}_raw_sats"),
                    matured_version + Version::ONE,
                    mappings,
                    window_starts,
                )?,
                cents: PerBlockCumulativeRolling::forced_import(
                    db,
                    &format!("{name}_raw_cents"),
                    matured_version + Version::ONE,
                    mappings,
                    window_starts,
                )?,
            })
        })?;
        let matured = AgeRange::from_fn(|id| {
            let name = format!(
                "{}_matured_supply",
                CohortContext::Utxo.full_name(id.cohort())
            );
            let source = id.select(&matured_sources);
            let sats = LazyVec::transformed::<StoredU64ToSats>(
                &format!("{name}_cumulative_sats"),
                matured_version,
                source.sats.cumulative.height.read_only_boxed_clone(),
            );
            let cents = LazyVec::transformed::<StoredU64ToCents>(
                &format!("{name}_cumulative_cents"),
                matured_version,
                source.cents.cumulative.height.read_only_boxed_clone(),
            );
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name,
                matured_version,
                &sats,
                &cents,
                mappings,
                window_starts,
            )
        });

        Ok(Box::new(Self {
            total,
            matured,
            matured_sources,
            half,
            in_profit,
            in_loss,
            delta,
            dominance,
        }))
    }

    pub fn sources(&self, cohort_id: CohortId) -> Option<SupplySources> {
        Some(SupplySources {
            total: self.total.get(cohort_id)?.clone(),
            in_profit: self.in_profit.get(cohort_id)?.clone(),
        })
    }

    pub fn min_resume_len(&self) -> usize {
        self.total
            .min_len()
            .min(
                self.matured_sources
                    .iter()
                    .flat_map(|v| {
                        [
                            v.sats.cumulative.height.len(),
                            v.cents.cumulative.height.len(),
                        ]
                    })
                    .min()
                    .unwrap_or_default(),
            )
            .min(self.in_profit.min_len())
            .min(self.in_loss.min_len())
    }

    #[inline(always)]
    pub fn push_maturation(&mut self, matured: &AgeRange<Sats>, price: Cents) {
        for id in AgeRangeId::ALL {
            let sats = *id.select(matured);
            let source = id.select_mut(&mut self.matured_sources);
            source.sats.push_block(StoredU64::from(u64::from(sats)));
            source
                .cents
                .push_block(StoredU64::from(u64::from(SatsToCents::apply(sats, price))));
        }
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        total: UTXOCoreValues<Sats>,
        profitability: &UTXOCoreValues<UnrealizedState>,
    ) {
        let in_profit = profitability.map(|state| state.supply_in_profit);
        let in_loss = profitability.map(|state| state.supply_in_loss);

        self.total.push(total);
        self.in_profit.push(in_profit);
        self.in_loss.push(in_loss);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.total.collect_vecs_mut();
        vecs.extend(
            self.matured_sources
                .iter_mut()
                .flat_map(|v| [v.sats.stored_mut(), v.cents.stored_mut()]),
        );
        vecs.extend(self.in_profit.collect_vecs_mut());
        vecs.extend(self.in_loss.collect_vecs_mut());
        vecs
    }
}
use vecdb::ReadableCloneableVec;
