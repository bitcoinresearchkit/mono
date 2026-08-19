#![allow(clippy::type_complexity)]

mod dependencies;
mod horizon;
mod vecs;

pub use dependencies::Dependencies;
pub use horizon::HorizonId;
use horizon::Horizons;
pub use vecs::Vecs;
use vecs::{
    AgeRangeVecs, AggregateSources, AggregateVecs, HorizonVecs, Mobility, MobilityId,
    SpendingExposureSeries,
};

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("coinflow");
const DB_NAME: &str = ID.as_str();

const AGE_COHORT_COUNT: usize = brk_cohort::AGE_RANGE_COUNT;
