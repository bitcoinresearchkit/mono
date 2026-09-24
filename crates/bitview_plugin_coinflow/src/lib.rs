#![allow(clippy::type_complexity)]

use bitview_cohort::AGE_RANGE_COUNT;
use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;
use vecs::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonVecs, Mobility, MobilityId,
    SpendingExposureSeries,
};

mod dependencies;
mod has;
mod horizon;
mod vecs;
mod weights;

pub use dependencies::Dependencies;
pub use has::HasCoinflow;
pub use horizon::{HorizonId, Horizons};

pub use vecs::Vecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("coinflow"), Version::new(15));
pub const ID: PluginId = STORAGE.id();

const AGE_COHORT_COUNT: usize = AGE_RANGE_COUNT;
