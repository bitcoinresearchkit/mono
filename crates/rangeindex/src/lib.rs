//! Ordered range boundaries in shared pages, with published and cursor-based reverse lookups.

mod base;
mod cursor;
mod pages;
mod share;

pub use base::RangeMap;
pub use cursor::RangeMapCursor;
pub use pages::Pages;
pub use share::{RangeMapWriter, SharedRangeMap};
