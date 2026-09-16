use bitview_cohort::{AgeRange, AgeRangeId, CohortContext, UTXOAggregate, UTXOAggregateId};
use bitview_plugin::ImportContext;
use bitview_plugin_distribution::Vecs as DistributionVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_transforms::BoundedToF64;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock,
    LazySpotValuePerBlock, PerBlock, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, PcoVecValue, ReadableBoxedVec};

use super::Vecs;
use crate::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonId, HorizonVecs, Mobility, MobilityId,
    STORAGE, SpendingExposureSeries,
};

impl AggregateSources {
    fn forced_import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            under_4m_price: import_cached(db, "under_4m_coinflow_price_cents", version)?,
            under_4m_capitalized_price: import_cached(
                db,
                "under_4m_coinflow_capitalized_price_cents",
                version,
            )?,
            under_6m_price: import_cached(db, "under_6m_coinflow_price_cents", version)?,
            under_6m_capitalized_price: import_cached(
                db,
                "under_6m_coinflow_capitalized_price_cents",
                version,
            )?,
            over_4m_price: import_cached(db, "over_4m_coinflow_price_cents", version)?,
            over_4m_capitalized_price: import_cached(
                db,
                "over_4m_coinflow_capitalized_price_cents",
                version,
            )?,
            over_6m_price: import_cached(db, "over_6m_coinflow_price_cents", version)?,
            over_6m_capitalized_price: import_cached(
                db,
                "over_6m_coinflow_capitalized_price_cents",
                version,
            )?,
            supply: MobilityId::try_from_fn(|side| {
                import_aggregate(
                    db,
                    &format!("coinflow_{}_supply_sats", side.name()),
                    version,
                )
            })?,
            supply_in_loss_share: import_aggregate(
                db,
                "coinflow_supply_in_loss_share_bounded",
                version,
            )?,
            horizon: HorizonId::try_from_fn(|h| {
                import_aggregate(
                    db,
                    &format!("coinflow_{}_supply_in_loss_share_bounded", h.name()),
                    version,
                )
            })?,
            cap: import_aggregate(db, "coinflow_cap_cents", version)?,
            price: import_aggregate(db, "coinflow_price_cents", version)?,
            capitalized_price: import_aggregate(db, "coinflow_capitalized_price_cents", version)?,
        })
    }
}

fn import_aggregate<T: PcoVecValue>(
    db: &Database,
    metric: &str,
    version: Version,
) -> Result<UTXOAggregate<CachedSeries<Height, T>>> {
    UTXOAggregate::try_from_fn(|id| import_cached(db, &id.metric_name(metric), version))
}

impl AggregateVecs {
    fn new(
        aggregate: UTXOAggregateId,
        version: Version,
        sources: &AggregateSources,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let metric_name = |metric: &str| aggregate.metric_name(metric);
        let supply = Mobility {
            mobile: LazySpotValuePerBlock::from_sats_source(
                &metric_name("mobile_supply"),
                version,
                aggregate.select(&sources.supply.mobile),
                mappings,
                spot_price,
            ),
            immobile: LazySpotValuePerBlock::from_sats_source(
                &metric_name("immobile_supply"),
                version,
                aggregate.select(&sources.supply.immobile),
                mappings,
                spot_price,
            ),
        };
        let supply_in_loss_share = LazyPerBlock::from_height_source::<BoundedToF64>(
            &metric_name("coinflow_supply_in_loss_share"),
            version,
            aggregate.select(&sources.supply_in_loss_share),
            mappings,
        );
        let horizon = HorizonId::from_fn(|horizon| {
            let name = metric_name(&format!("coinflow_{}_supply_in_loss_share", horizon.name()));
            HorizonVecs {
                supply_in_loss_share: LazyPerBlock::from_height_source::<BoundedToF64>(
                    &name,
                    version,
                    aggregate.select(horizon.select(&sources.horizon)),
                    mappings,
                ),
            }
        });
        let cap = LazyFiatPerBlock::from_cents_source(
            &metric_name("coinflow_cap"),
            version,
            aggregate.select(&sources.cap),
            mappings,
        );
        let price = LazyPriceWithRatioPerBlock::from_height_source(
            &metric_name("coinflow_price"),
            version,
            aggregate.select(&sources.price),
            mappings,
            spot_price,
        );

        Self {
            supply,
            supply_in_loss_share,
            horizon,
            cap,
            price,
            capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                &metric_name("coinflow_capitalized_price"),
                version,
                aggregate.select(&sources.capitalized_price),
                mappings,
                spot_price,
            ),
        }
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        prices: &PriceVecs,
        distribution: &DistributionVecs,
    ) -> Result<Self> {
        let database = STORAGE.open_database(context, 250_000)?;

        let db = &database;
        let version = STORAGE.schema_version() + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_boxed_clone();
        let spending_rate = AgeRange::try_from_fn(|id| {
            let name = format!(
                "{}_spending_rate",
                CohortContext::Utxo.full_name(id.cohort())
            );
            PerBlock::forced_import(db, &name, version, mappings)
        })?;
        let mobility_source = AgeRange::try_from_fn(|id| {
            let name = format!(
                "{}_mobility_bounded_source",
                CohortContext::Utxo.full_name(id.cohort())
            );
            import_cached(db, &name, version)
        })?;
        let spending_exposure = SpendingExposureSeries {
            age_range: AgeRange::try_from_fn(|id| {
                let name = format!(
                    "{}_spending_exposure",
                    CohortContext::Utxo.full_name(id.cohort())
                );
                PerBlock::forced_import(db, &name, version, mappings)
            })?,
            mobility: AgeRangeId::series(CohortContext::Utxo, |id, name| {
                LazyPerBlock::from_height_source::<BoundedToF64>(
                    &format!("{name}_mobility"),
                    version,
                    id.select(&mobility_source),
                    mappings,
                )
            }),
        };
        let supply_for = |side: MobilityId| {
            let side = side.name();
            AgeRangeId::series(CohortContext::Utxo, |id, name| {
                let name = format!("{name}_{side}_supply");
                let supply = id.select(&distribution.cohorts.supply.total.stored.cohorts.age);
                let weight = id.select(&mobility_source);
                if side == "immobile" {
                    LazySpotValuePerBlock::from_weighted_supply::<true>(
                        &name,
                        version,
                        supply,
                        weight,
                        mappings,
                        &spot_price,
                    )
                } else {
                    LazySpotValuePerBlock::from_weighted_supply::<false>(
                        &name,
                        version,
                        supply,
                        weight,
                        mappings,
                        &spot_price,
                    )
                }
            })
        };
        let supply = Mobility {
            mobile: supply_for(MobilityId::Mobile),
            immobile: supply_for(MobilityId::Immobile),
        };

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
            db: database,
            age_range: AgeRangeVecs {
                spending_rate,
                spending_exposure,
                mobility_source,
                supply,
            },
            all,
            sth,
            lth,
            under_4m_price: LazyPriceWithRatioPerBlock::from_height_source(
                "under_4m_coinflow_price",
                version,
                &aggregate_sources.under_4m_price,
                mappings,
                &spot_price,
            ),
            under_4m_capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                "under_4m_coinflow_capitalized_price",
                version,
                &aggregate_sources.under_4m_capitalized_price,
                mappings,
                &spot_price,
            ),
            under_6m_price: LazyPriceWithRatioPerBlock::from_height_source(
                "under_6m_coinflow_price",
                version,
                &aggregate_sources.under_6m_price,
                mappings,
                &spot_price,
            ),
            under_6m_capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                "under_6m_coinflow_capitalized_price",
                version,
                &aggregate_sources.under_6m_capitalized_price,
                mappings,
                &spot_price,
            ),
            over_4m_price: LazyPriceWithRatioPerBlock::from_height_source(
                "over_4m_coinflow_price",
                version,
                &aggregate_sources.over_4m_price,
                mappings,
                &spot_price,
            ),
            over_4m_capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                "over_4m_coinflow_capitalized_price",
                version,
                &aggregate_sources.over_4m_capitalized_price,
                mappings,
                &spot_price,
            ),
            over_6m_price: LazyPriceWithRatioPerBlock::from_height_source(
                "over_6m_coinflow_price",
                version,
                &aggregate_sources.over_6m_price,
                mappings,
                &spot_price,
            ),
            over_6m_capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                "over_6m_coinflow_capitalized_price",
                version,
                &aggregate_sources.over_6m_capitalized_price,
                mappings,
                &spot_price,
            ),
            aggregate_sources,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
use vecdb::ReadableCloneableVec;
