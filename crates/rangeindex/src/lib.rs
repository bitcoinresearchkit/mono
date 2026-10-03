//! Ordered range boundaries with shared and cursor-based reverse lookups.

mod base;
mod cursor;
mod share;

pub use base::RangeMap;
pub use cursor::RangeMapCursor;
pub use share::SharedRangeMap;
