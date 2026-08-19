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

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("coinflow");

const AGE_COHORT_COUNT: usize = brk_cohort::AGE_RANGE_COUNT;
