use crate::state::{AddrCohortState, RealizedOps};
use bitview_cohort::{AmountRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_distribution_common::metrics::SupplyBase;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::{
    AmountSources, AmountValueSources, LazyFiatPerBlock, LazyFiatPerBlockCumulativeWithSums,
    LazyPerBlockWithDeltas, LazySpotValuePerBlock, LazyValuePerBlockCumulativeRolling,
    LazyWindowStartVec,
};
use brk_error::Result;
use brk_types::{Cents, Height, PartsPerMillionSigned64, Sats, StoredI64, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct BalanceMetrics<M: StorageMode = Rw> {
    pub supply: AmountRange<SupplyBase>,
    #[traversable(hidden)]
    pub supply_source: AmountSources<Sats, LazySpotValuePerBlock, M>,
    pub utxo_count: AmountSources<
        StoredU64,
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
        M,
    >,
    pub transfer_volume: AmountValueSources<LazyValuePerBlockCumulativeRolling, M>,
    pub realized_cap: AmountSources<Cents, LazyFiatPerBlock<Cents>, M>,
    pub realized_profit: AmountSources<Cents, LazyFiatPerBlockCumulativeWithSums<Cents>, M>,
    pub realized_loss: AmountSources<Cents, LazyFiatPerBlockCumulativeWithSums<Cents>, M>,
}

impl BalanceMetrics {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Box<Self>> {
        let balance_version = version + Version::ONE;
        let cumulative_version = version + Version::TWO;
        let supply_source = AmountSources::forced_import(
            db,
            "addrs_supply_sats_by_balance_range",
            CohortContext::Addr,
            "supply",
            balance_version,
            |name, source| {
                LazySpotValuePerBlock::from_sats_source(
                    name,
                    balance_version,
                    source,
                    mappings,
                    spot,
                )
            },
        )?;
        let supply = AmountRange::from_fn(|id| {
            SupplyBase::from_total(
                CohortContext::Addr,
                id.cohort(),
                balance_version,
                id.select(&supply_source.series).clone(),
                all_supply,
                mappings,
                windows,
            )
        });
        let utxo_count = AmountSources::forced_import(
            db,
            "addrs_utxo_count_by_balance_range",
            CohortContext::Addr,
            "utxo_count",
            balance_version,
            |name, source| {
                LazyPerBlockWithDeltas::from_height_source(
                    name,
                    balance_version,
                    source,
                    Version::TWO,
                    mappings,
                    windows,
                )
            },
        )?;
        let transfer_volume = AmountValueSources::forced_import(
            db,
            "addrs_transfer_volume_cumulative_by_balance_range",
            CohortContext::Addr,
            "transfer_volume",
            cumulative_version,
            |name, sats, cents| {
                LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                    name,
                    cumulative_version,
                    &sats,
                    &cents,
                    mappings,
                    windows,
                )
            },
        )?;
        let realized_cap = AmountSources::forced_import(
            db,
            "addrs_realized_cap_cents_by_balance_range",
            CohortContext::Addr,
            "realized_cap",
            balance_version,
            |name, source| {
                LazyFiatPerBlock::from_cents_source(name, balance_version, source, mappings)
            },
        )?;
        let cumulative = |metric: &str| {
            AmountSources::forced_import(
                db,
                &format!("addrs_{metric}_cumulative_cents_by_balance_range"),
                CohortContext::Addr,
                metric,
                cumulative_version,
                |name, source| {
                    LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                        name,
                        cumulative_version,
                        source,
                        mappings,
                        windows,
                    )
                },
            )
        };
        Ok(Box::new(Self {
            supply,
            supply_source,
            utxo_count,
            transfer_volume,
            realized_cap,
            realized_profit: cumulative("realized_profit")?,
            realized_loss: cumulative("realized_loss")?,
        }))
    }

    pub fn push(&mut self, states: &AmountRange<AddrCohortState>, price: Cents) {
        self.supply_source.push(AmountRange::from_fn(|id| {
            id.select(states).inner.supply.value
        }));
        self.utxo_count.push(AmountRange::from_fn(|id| {
            StoredU64::from(id.select(states).inner.supply.utxo_count)
        }));
        let sats = AmountRange::from_fn(|id| id.select(states).inner.sent);
        let cents = AmountRange::from_fn(|id| SatsToCents::apply(*id.select(&sats), price));
        self.transfer_volume.push_cumulative(&sats, &cents);
        self.realized_cap.push(AmountRange::from_fn(|id| {
            id.select(states).inner.realized.cap()
        }));
        self.realized_profit
            .push_cumulative(&AmountRange::from_fn(|id| {
                id.select(states).inner.realized.profit()
            }));
        self.realized_loss
            .push_cumulative(&AmountRange::from_fn(|id| {
                id.select(states).inner.realized.loss()
            }));
    }

    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.supply_source
            .stored_vecs_mut()
            .chain(self.utxo_count.stored_vecs_mut())
            .chain(self.transfer_volume.stored_vecs_mut())
            .chain(self.realized_cap.stored_vecs_mut())
            .chain(self.realized_profit.stored_vecs_mut())
            .chain(self.realized_loss.stored_vecs_mut())
            .collect::<Vec<_>>()
            .into_par_iter()
    }
}
