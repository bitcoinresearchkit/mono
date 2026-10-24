use brk_oracle::HistogramEma;
use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The oracle's search for one block: each candidate bin it scored, the one it picked, and the
/// histogram it scored them on.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OracleSearch {
    pub height: Height,
    /// The window's first candidate bin, on the oracle's log scale (200 bins a decade; a bin is
    /// where $100 lands in sats: the price is `10^(10 - bin / 200)` dollars).
    pub first_bin: u16,
    /// Each candidate's score, from `first_bin` on: its round-dollar arms' counts, each over that
    /// arm's peak in the window, summed. The highest is the pick's bin.
    pub scores: Vec<f64>,
    /// The bin it picked, refined between bins: the block's price.
    pub bin: f64,
    /// The smoothed histogram it scored: the payments of the block and the 11 before it (and,
    /// live, the mempool's as the next block's), counted by amount in sats on the same scale (a
    /// bin is `round(log10(sats) * 200)`), the newest weighing most.
    pub histogram: HistogramEma,
}
