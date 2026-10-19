use crate::{AnyVec, VecIndex, VecValue};

/// A vector with statically known index and value types.
pub trait TypedVec: AnyVec {
    type I: VecIndex;
    type T: VecValue;
}
