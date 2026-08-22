use std::ops::AddAssign;

use bitview_cohort::{AgeRangeId, CohortContext, TermId, UTXOAggregateId};
use bitview_compute::{
    AgeBand, CACHE_BUDGET, ColumnarPerBlock, Identity, LazyColumnPerBlock,
    LazyColumnSpotValuePerBlock, LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock,
    LazySpotValuePerBlock,
};
use bitview_plugin::ImportContext;
use brk_error::Result;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{
    CachedBoxedVec, ColumnId, Database, ImportableVec, PcoVec, PcoVecValue, ReadOnlyClone,
    ReadOnlyColumnarVec, ReadableBoxedVec, ReadableCloneableVec, ReadableColumnarVec,
    UnaryTransform,
};

use super::Vecs;
use crate::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonId, HorizonVecs, Mobility, MobilityId,
    STORAGE, SpendingExposureSeries,
};

struct ExposureToMobility;

impl UnaryTransform<StoredF64, StoredF64> for ExposureToMobility {
    #[inline(always)]
    fn apply(exposure: StoredF64) -> StoredF64 {
        StoredF64::from(AgeBand::mobility(*exposure))
    }
}

impl SpendingExposureSeries {
    fn new(
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredF64>, AgeRangeId>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Self {
        let age_range = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            LazyColumnPerBlock::new(
                &format!("{name}_spending_exposure"),
                version,
                source,
                column,
                mappings,
            )
        });
        let mobility = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            let exposure = column.select(&age_range);
            LazyPerBlock::from_resolutions::<ExposureToMobility>(
                &format!("{name}_mobility"),
                version,
                exposure.height.read_only_boxed_clone(),
                &exposure.resolutions,
            )
        });

        Self {
            age_range,
            mobility,
        }
    }
}

impl AggregateSources {
    fn forced_import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            supply: MobilityId::try_from_fn(|side| {
                ImportableVec::forced_import(
                    db,
                    &format!("coinflow_{}_supply_sats_by_term", side.name()),
                    version,
                )
            })?,
            supply_in_loss_share: ImportableVec::forced_import(
                db,
                "coinflow_supply_in_loss_share_by_aggregate",
                version,
            )?,
            horizon: HorizonId::try_from_fn(|horizon| {
                ImportableVec::forced_import(
                    db,
                    &format!(
                        "coinflow_{}_supply_in_loss_share_by_aggregate",
                        horizon.name()
                    ),
                    version,
                )
            })?,
            cap: ImportableVec::forced_import(db, "coinflow_cap_cents_by_term", version)?,
            price: ImportableVec::forced_import(db, "coinflow_price_cents_by_aggregate", version)?,
        })
    }

    fn additive_source<T>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, TermId>,
        name: &str,
        version: Version,
        aggregate: UTXOAggregateId,
    ) -> ReadableBoxedVec<Height, T>
    where
        T: PcoVecValue + AddAssign,
    {
        match aggregate.term() {
            Some(term) => source.column(name, version, term).read_only_boxed_clone(),
            None => CACHE_BUDGET
                .wrap(source.sum_columns(name, version, TermId::ALL.iter().copied()))
                .read_only_boxed_clone(),
        }
    }

    fn exact_source<T>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, UTXOAggregateId>,
        name: &str,
        version: Version,
        aggregate: UTXOAggregateId,
    ) -> ReadableBoxedVec<Height, T>
    where
        T: PcoVecValue,
    {
        source
            .column(name, version, aggregate)
            .read_only_boxed_clone()
    }
}

impl AggregateVecs {
    fn new(
        aggregate: UTXOAggregateId,
        version: Version,
        sources: &AggregateSources,
        mappings: &bitview_plugin_mappings::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let metric_name = |metric: &str| aggregate.metric_name(metric);
        let supply = Mobility {
            mobile: LazySpotValuePerBlock::from_boxed_sats_source(
                &metric_name("mobile_supply"),
                version,
                AggregateSources::additive_source(
                    &sources.supply.mobile.read_only_clone(),
                    &metric_name("mobile_supply_sats"),
                    version,
                    aggregate,
                ),
                mappings,
                spot_price,
            ),
            immobile: LazySpotValuePerBlock::from_boxed_sats_source(
                &metric_name("immobile_supply"),
                version,
                AggregateSources::additive_source(
                    &sources.supply.immobile.read_only_clone(),
                    &metric_name("immobile_supply_sats"),
                    version,
                    aggregate,
                ),
                mappings,
                spot_price,
            ),
        };
        let supply_in_loss_share = LazyPerBlock::from_boxed_height_source::<Identity<StoredF64>>(
            &metric_name("coinflow_supply_in_loss_share"),
            version,
            AggregateSources::exact_source(
                &sources.supply_in_loss_share.read_only_clone(),
                &metric_name("coinflow_supply_in_loss_share"),
                version,
                aggregate,
            ),
            mappings,
        );
        let horizon = HorizonId::from_fn(|horizon| {
            let name = metric_name(&format!("coinflow_{}_supply_in_loss_share", horizon.name()));
            HorizonVecs {
                supply_in_loss_share: LazyPerBlock::from_boxed_height_source::<Identity<StoredF64>>(
                    &name,
                    version,
                    AggregateSources::exact_source(
                        &horizon.select(&sources.horizon).read_only_clone(),
                        &name,
                        version,
                        aggregate,
                    ),
                    mappings,
                ),
            }
        });
        let cap = LazyFiatPerBlock::from_boxed_cents_source(
            &metric_name("coinflow_cap"),
            version,
            AggregateSources::additive_source(
                &sources.cap.read_only_clone(),
                &metric_name("coinflow_cap_cents"),
                version,
                aggregate,
            ),
            mappings,
        );
        let price = LazyPriceWithRatioPerBlock::from_boxed_height_source(
            &metric_name("coinflow_price"),
            version,
            AggregateSources::exact_source(
                &sources.price.read_only_clone(),
                &metric_name("coinflow_price_cents"),
                version,
                aggregate,
            ),
            mappings,
            spot_price,
        );

        Self {
            supply,
            supply_in_loss_share,
            horizon,
            cap,
            price,
        }
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        prices: &bitview_plugin_price::Vecs,
    ) -> Result<Self> {
        let database = STORAGE.open_database(context, 250_000)?;
        let db = &database;
        let version = STORAGE.schema_version();
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let spending_rate = ColumnarPerBlock::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_spending_rate"),
            version,
            |source| {
                AgeRangeId::series(CohortContext::Utxo, |column, name| {
                    LazyColumnPerBlock::new(
                        &format!("{name}_spending_rate"),
                        version,
                        source,
                        column,
                        mappings,
                    )
                })
            },
        )?;
        let spending_exposure = ColumnarPerBlock::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_spending_exposure"),
            version,
            |source| SpendingExposureSeries::new(version, source, mappings),
        )?;
        let supply = MobilityId::try_from_fn(|side| {
            let side = side.name();
            ColumnarPerBlock::forced_import(
                db,
                &CohortContext::Utxo.prefixed(&format!("age_range_{side}_supply_sats")),
                version,
                |source| {
                    AgeRangeId::series(CohortContext::Utxo, |column, name| {
                        LazyColumnSpotValuePerBlock::new(
                            &format!("{name}_{side}_supply"),
                            version,
                            source,
                            column,
                            mappings,
                            &spot_price,
                        )
                    })
                },
            )
        })?;

        let aggregate_sources = AggregateSources::forced_import(db, version)?;
        let all = AggregateVecs::new(
            UTXOAggregateId::All,
            version,
            &aggregate_sources,
            mappings,
            &spot_price,
        );
        let sth = AggregateVecs::new(
            UTXOAggregateId::Sth,
            version,
            &aggregate_sources,
            mappings,
            &spot_price,
        );
        let lth = AggregateVecs::new(
            UTXOAggregateId::Lth,
            version,
            &aggregate_sources,
            mappings,
            &spot_price,
        );

        let this = Self {
            plugin_gate: Default::default(),
            db: database,
            age_range: AgeRangeVecs {
                spending_rate,
                spending_exposure,
                supply,
            },
            all,
            sth,
            lth,
            aggregate_sources,
        };
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }
}
