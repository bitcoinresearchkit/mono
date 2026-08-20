#![allow(clippy::type_complexity)]

mod dependencies;
mod has;
mod horizon;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasCoinflow;
pub use horizon::HorizonId;
use horizon::Horizons;
pub use vecs::Vecs;
use vecs::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonVecs, Mobility, MobilityId,
    SpendingExposureSeries,
};

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("coinflow"), Version::new(14));
pub const ID: PluginId = STORAGE.id();

const AGE_COHORT_COUNT: usize = bitview_cohort::AGE_RANGE_COUNT;
