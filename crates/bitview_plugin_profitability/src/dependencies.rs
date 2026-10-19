use brk_types::{Cents, Height, Timestamp};
use statedb::Reader;
use vecdb::ReadableBoxedVec;

/// Published immutable sources; no dependency on Age's mutable working state.
pub struct Dependencies<'a> {
    pub history: &'a Reader<'a>,
    pub from: Height,
    pub prices: &'a ReadableBoxedVec<Height, Cents>,
    pub timestamps: &'a ReadableBoxedVec<Height, Timestamp>,
}
