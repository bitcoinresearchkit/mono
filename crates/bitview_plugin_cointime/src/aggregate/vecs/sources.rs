use bitview_cohort::AgeAggregate;
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height, Sats};
use vecdb::{Rw, StorageMode};

/// Height-indexed sources of every cohort's awake and dormant metrics.
#[derive(Traversable)]
pub struct Sources<M: StorageMode = Rw> {
    pub awake_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub dormant_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub awake_realized_cap: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub awake_realized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub awake_capitalized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub awake_supply_in_loss_share: AgeAggregate<CachedSeries<Height, BoundedRatio, M>>,
}
