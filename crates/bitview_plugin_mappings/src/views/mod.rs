//! Lazy vecs over the resident range maps: reverse lookups and per-index counts.

mod count;
mod cumulative;
mod range_map_lookup;
mod terminal_len;

pub use count::LazyIndexCountVec;
pub use cumulative::LazyCumulativeIndexVec;
pub use range_map_lookup::RangeMapLookupVec;
