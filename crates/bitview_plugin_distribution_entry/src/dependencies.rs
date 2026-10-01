use brk_types::{Cents, Height, Timestamp};
use statedb::Reader;
use vecdb::ReadableBoxedVec;

/// Completed, read-only history and price inputs. Age owns the anchor series.
pub struct Dependencies<'a> {
    pub history: &'a Reader<'a>,
    pub from: Height,
    pub prices: &'a ReadableBoxedVec<Height, Cents>,
    pub timestamps: &'a ReadableBoxedVec<Height, Timestamp>,
    pub capitalized_price: &'a ReadableBoxedVec<Height, Cents>,
}
