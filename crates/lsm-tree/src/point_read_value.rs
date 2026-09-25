use crate::{Slice, ValueType};

pub struct PointReadValue<V = Slice> {
    pub value_type: ValueType,
    pub value: V,
}
