//! Composable metric vectors and views for Bitview plugins.
//!
//! Storage and cache primitives come from vecdb; range algorithms come from
//! bitview_compute. This crate owns typed shapes (`shapes`), scalar transforms
//! (`ops`), metric layout, source caches, and view composition. Application
//! composition supplies the shared cache budget and owns its invalidation lifecycle.
#![allow(clippy::type_complexity)]

mod block;
mod fiat;
pub mod ops;
mod percent;
mod priced;
mod ratio;
mod resolutions;
mod rolling;
pub mod shapes;
mod sources;
mod value;
mod views;

pub use block::*;
pub use fiat::*;
pub use percent::*;
pub use priced::*;
pub use ratio::*;
pub use resolutions::*;
pub use rolling::*;
pub use sources::*;
pub use value::*;
pub use views::*;

/// What `counter_group!` names, for crates that don't depend on these themselves.
#[doc(hidden)]
pub mod __macro {
    pub use bitview_primitives::{Bytes, Count};
    pub use bitview_traversable::Traversable;
    pub use brk_error::Result;
    pub use brk_exit::Exit;
    pub use brk_types::{Height, Version};
    pub use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};
}
