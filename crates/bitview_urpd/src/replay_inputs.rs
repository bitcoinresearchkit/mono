use brk_types::{Cents, Height, Timestamp};
use statedb::Reader;
use vecdb::ReadableVec;

/// Read-only sources needed to reconstruct block price distributions.
#[derive(Clone, Copy)]
pub struct ReplayInputs<'a> {
    pub history: &'a Reader<'a>,
    pub prices: &'a dyn ReadableVec<Height, Cents>,
    pub timestamps: &'a dyn ReadableVec<Height, Timestamp>,
}
