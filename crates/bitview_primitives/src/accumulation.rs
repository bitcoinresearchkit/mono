use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// How a series' values combine over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Accumulation {
    /// An amount per period (a block's fees, a day's): sum it over a window.
    Flow,
    /// A state at each point (a price, the supply held, a ratio): read it at a point.
    Level,
    /// Already a running total since the start of the chain: its flow over a window is its difference across it.
    Cumulative,
}

impl Accumulation {
    /// A series no declaration covers is read at a point: the families that sum a base (per
    /// block, cumulative, rolling) declare theirs where they're defined, so every use carries it.
    pub const UNDECLARED: Self = Self::Level;
}
