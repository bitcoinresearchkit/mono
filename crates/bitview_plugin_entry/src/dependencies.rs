use bitview_distribution::walk::{Sources, Walk};
use brk_types::{Cents, Height};
use vecdb::ReadableBoxedVec;

/// Completed, read-only history and price inputs. Holders owns the anchor series.
pub struct Dependencies<'a> {
    pub sources: Sources<'a>,
    /// The completed walk of `sources`, which holds each block's price and time.
    pub walk: &'a Walk,
    pub from: Height,
    pub capitalized_price: &'a ReadableBoxedVec<Height, Cents>,
}
