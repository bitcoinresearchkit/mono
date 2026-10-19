use brk_types::{Height, Weight32};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub(crate) from: Height,
    pub(crate) block_weights: &'a PcoVec<Height, Weight32>,
}
