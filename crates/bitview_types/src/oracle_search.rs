use brk_oracle::{HistogramEma, Tally};
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
    /// The block's own outputs (live, the mempool's) as the oracle's filter judged them.
    pub outputs: FilteredOutputs,
    /// Whether it's the live search with the mempool blended in as the next block (`height` the
    /// next block's, `outputs` the mempool's), rather than a block's.
    pub mempool: bool,
}

/// A block's outputs as the oracle's filter judged them: each skipped one under the first rule
/// that skips it, in the filter's order, and the payments it kept. They add up to the block's
/// outputs.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FilteredOutputs {
    /// The coinbase's: new coins, not payments.
    pub coinbase: u32,
    /// Those of transactions with more outputs than the fan-out cap (250; 100 before height
    /// 630,000): batch payouts.
    pub fan_out: u32,
    /// Those of transactions carrying an OP_RETURN output: data, not payments.
    pub op_return: u32,
    /// Taproot outputs: mostly protocol traffic.
    pub taproot: u32,
    /// Under 1,000 sats.
    pub dust: u32,
    /// Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).
    pub round_btc: u32,
    /// The rest: the payments it counted.
    pub payments: u32,
}

impl From<Tally> for FilteredOutputs {
    fn from(t: Tally) -> Self {
        Self {
            coinbase: t.coinbase,
            fan_out: t.fan_out,
            op_return: t.op_return,
            taproot: t.taproot,
            dust: t.dust,
            round_btc: t.round_btc,
            payments: t.payments,
        }
    }
}
