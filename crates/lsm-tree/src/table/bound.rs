use crate::Slice;

pub enum Bound {
    Included(Slice),
    Excluded(Slice),
}
