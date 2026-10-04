//! Composable metric vectors and views for Bitview plugins.
//!
//! Storage and cache primitives come from vecdb; range algorithms come from
//! bitview_compute and scalar operations from bitview_transforms. This crate
//! owns metric layout, source caches, and view composition, using shapes from
//! bitview_collections and bitview_cohort. Application composition supplies the
//! shared cache budget and owns its invalidation lifecycle.
#![allow(clippy::type_complexity)]

mod block;
mod cohort;
mod fiat;
mod fixed_ratio;
mod ratio;
mod resolutions;
mod rolling;
mod sources;
mod tx;
mod value;
mod views;

pub use block::*;
pub use cohort::*;
pub use fiat::*;
pub use fixed_ratio::*;
pub use ratio::*;
pub use resolutions::*;
pub use rolling::*;
pub use sources::*;
pub use tx::*;
pub use value::*;
pub use views::*;
