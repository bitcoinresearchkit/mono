use crate::TypedVec;

use super::MutableVec;

impl<V: TypedVec> TypedVec for MutableVec<V> {
    type I = V::I;
    type T = V::T;
}
