use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;

use super::{Levels, Percentiles};

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct PriceBands<T> {
    pub floor: Percentiles<T>,
    pub level: Levels<T>,
}
