use brk_types::{Height, Weight};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub(crate) safe_height: Height,
    pub(crate) block_weights: &'a PcoVec<Height, Weight>,
}
