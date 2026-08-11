use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct Levels<T> {
    pub pct10: T,
    pub pct20: T,
    pub pct30: T,
    pub pct40: T,
    pub pct50: T,
    pub pct60: T,
    pub pct70: T,
    pub pct80: T,
    pub pct90: T,
}
