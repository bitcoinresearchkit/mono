use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::WeightedCohortState;
use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{
    CachedSeries, IndexSources, LazyFiatPerBlock, LazyPerBlock, LazySpotValuePerBlock,
    import_cached, ops::FixedToPercent,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, PcoVecValue, ReadableBoxedVec, Rw, StorageMode, WritableVec};

/// The names of a weighted model's two sides: `awake` gives `sth_awake_supply_sats`.
#[derive(Clone, Copy)]
pub struct WeightedCohortSides {
    pub weighted: &'static str,
    pub complement: &'static str,
}

/// Height-indexed sources of every aggregate cohort split by a weighted model: its weighted side
/// (awake, mobile) and the complement (dormant, immobile). The model owns the weights and the
/// public views.
#[derive(Traversable)]
pub struct WeightedCohortSources<M: StorageMode = Rw> {
    pub weighted_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub complement_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub weighted_realized_cap: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_realized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_capitalized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_supply_in_loss_share: AgeAggregate<CachedSeries<Height, BoundedRatio, M>>,
}

impl WeightedCohortSources {
    pub fn import(db: &Database, sides: WeightedCohortSides, version: Version) -> Result<Self> {
        let WeightedCohortSides {
            weighted,
            complement,
        } = sides;
        let weighted = |metric: &str| format!("{weighted}_{metric}");
        Ok(Self {
            weighted_supply: import_aggregate(db, &weighted("supply_sats"), version)?,
            complement_supply: import_aggregate(db, &format!("{complement}_supply_sats"), version)?,
            weighted_realized_cap: import_aggregate(db, &weighted("realized_cap_cents"), version)?,
            weighted_realized_price: import_aggregate(
                db,
                &weighted("realized_price_cents"),
                version,
            )?,
            weighted_capitalized_price: import_aggregate(
                db,
                &weighted("capitalized_price_cents"),
                version,
            )?,
            weighted_supply_in_loss_share: import_aggregate(
                db,
                &weighted("supply_in_loss_share_bounded"),
                version,
            )?,
        })
    }

    pub fn push(&mut self, states: &AgeAggregate<WeightedCohortState>) {
        for &id in AgeAggregateId::ALL {
            let state = id.select(states);
            id.select_mut(&mut self.weighted_supply)
                .push(state.weighted_supply());
            id.select_mut(&mut self.complement_supply)
                .push(state.complement_supply());
            id.select_mut(&mut self.weighted_realized_cap)
                .push(state.realized_cap());
            id.select_mut(&mut self.weighted_realized_price)
                .push(state.realized_price());
            id.select_mut(&mut self.weighted_capitalized_price)
                .push(state.capitalized_price());
            id.select_mut(&mut self.weighted_supply_in_loss_share)
                .push(state.supply_in_loss_share());
        }
    }
}

/// One aggregate cohort's views over its [`WeightedCohortSources`]. The model moves them into its
/// own public structs, which carry the series' docs and tree shape.
pub struct WeightedCohortViews {
    pub supply: LazySpotValuePerBlock,
    pub supply_in_loss_share: LazyPerBlock<Percent, BoundedRatio>,
    pub capital: LazyFiatPerBlock<Cents>,
    pub realized_cap: LazyFiatPerBlock<Cents>,
    pub cost_basis: CostBasisVecs,
    pub complement_supply: LazySpotValuePerBlock,
}

impl WeightedCohortViews {
    /// `capital` and `realized_cap` are two published names of the same weighted realized cap.
    pub fn new(
        db: &Database,
        id: AgeAggregateId,
        sides: WeightedCohortSides,
        version: Version,
        sources: &WeightedCohortSources,
        indexes: &IndexSources,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let WeightedCohortSides {
            weighted,
            complement,
        } = sides;
        let name = |metric: &str| id.metric_name(&format!("{weighted}_{metric}"));
        Ok(Self {
            supply: LazySpotValuePerBlock::from_sats_source(
                &name("supply"),
                version,
                id.select(&sources.weighted_supply),
                indexes,
                spot_price,
            ),
            supply_in_loss_share: LazyPerBlock::from_height_source::<FixedToPercent>(
                &name("supply_in_loss_share"),
                version,
                id.select(&sources.weighted_supply_in_loss_share),
                indexes,
            ),
            capital: LazyFiatPerBlock::from_cents_source(
                &name("capital"),
                version,
                id.select(&sources.weighted_realized_cap),
                indexes,
            ),
            realized_cap: LazyFiatPerBlock::from_cents_source(
                &name("realized_cap"),
                version,
                id.select(&sources.weighted_realized_cap),
                indexes,
            ),
            cost_basis: CostBasisVecs::import(
                db,
                &id.metric_name(weighted),
                version,
                indexes,
                id.select(&sources.weighted_realized_price),
                id.select(&sources.weighted_capitalized_price),
            )?,
            complement_supply: LazySpotValuePerBlock::from_sats_source(
                &id.metric_name(&format!("{complement}_supply")),
                version,
                id.select(&sources.complement_supply),
                indexes,
                spot_price,
            ),
        })
    }
}

fn import_aggregate<T: PcoVecValue>(
    db: &Database,
    metric: &str,
    version: Version,
) -> Result<AgeAggregate<CachedSeries<Height, T>>> {
    AgeAggregate::try_from_fn(|id| import_cached(db, &id.metric_name(metric), version))
}
