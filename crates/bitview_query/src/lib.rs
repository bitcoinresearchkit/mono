#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

#[cfg(feature = "full-api")]
mod api;
mod r#async;
mod error;
mod output;
mod pin;
mod plugins;
#[cfg(feature = "full-api")]
mod publication_cache;
mod query;
mod representation_id;
#[cfg(feature = "series")]
mod series;

#[cfg(feature = "full-api")]
pub use api::*;
pub use r#async::*;
pub(crate) use error::OptionData;
pub use error::{Error, Result, SeriesNotFound};
pub use output::*;
#[cfg(feature = "series")]
pub(crate) use pin::check_bound;
#[cfg(feature = "full-api")]
pub(crate) use pin::ReadPin;
pub use plugins::QueryPluginSet;
#[cfg(feature = "full-api")]
pub use publication_cache::PublicationCache;
pub use query::Query;
pub use representation_id::RepresentationId;
#[cfg(feature = "series")]
pub use series::{
    ResolvedQuery, ResolvedSeriesInfo, SeriesAnchor, SeriesEntry, SeriesRead, SeriesRegistry,
    SharedSeries,
};
