use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct Percentiles<T> {
    pub pct95: T,
    pub pct98: T,
    pub pct99: T,
    pub pct99_5: T,
    pub pct99_9: T,
}
