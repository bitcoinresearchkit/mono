use brk_types::{Height, Weight};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub safe_height: Height,
    pub block_weights: &'a PcoVec<Height, Weight>,
}
