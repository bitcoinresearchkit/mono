use bitview_cohort::AgeAggregate;
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height, Sats};
use vecdb::{Rw, StorageMode};

/// Height-indexed sources of every cohort's mobile and immobile metrics.
#[derive(Traversable)]
pub struct Sources<M: StorageMode = Rw> {
    pub mobile_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub immobile_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub mobile_realized_cap: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub mobile_realized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub mobile_capitalized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub mobile_supply_in_loss_share: AgeAggregate<CachedSeries<Height, BoundedRatio, M>>,
}
