use brk_types::{Cents, Height, Sats};
use vecdb::ReadableBoxedVec;

/// The totals a cohort's shares divide: all supply and all capital.
#[derive(Clone, Copy)]
pub struct ShareTotals<'a> {
    pub supply: &'a ReadableBoxedVec<Height, Sats>,
    pub capital: &'a ReadableBoxedVec<Height, Cents>,
}
