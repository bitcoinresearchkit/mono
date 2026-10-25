use brk_oracle::{HistogramEma, Tally};
use brk_types::{BlockHash, Dollars, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The oracle at one block: what it counted (`histogram`, `outputs`), where it looked (`first_bin`,
/// `scores`) and what it found (`bin`, `price`).
///
/// Bins are the oracle's log scale of amounts in sats, 200 a decade: amount `sats` is in bin
/// `round(log10(sats) * 200)`, bin 0 at 1 sat to bin 2399 just under 10^12. Read as where $100
/// lands, a bin is a price: `10^(10 - bin / 200)` dollars, higher bins lower prices.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OracleBlock {
    /// The block's height; live with the mempool, the next block's.
    pub height: Height,
    /// The confirmed block it was read at: this block's hash; live with the mempool, the tip's
    /// (block `height - 1`), the one the mempool is blended onto.
    pub hash: BlockHash,
    /// Whether it's the live reading with the mempool blended in as the next block (`outputs` the
    /// mempool's), rather than a confirmed block's.
    pub mempool: bool,
    /// The price it found, in dollars: `bin` as a price, to the cent.
    pub price: Dollars,
    /// The bin it picked, refined between bins by a parabola through the best score and its
    /// neighbours.
    pub bin: f64,
    /// The search window's first candidate bin: 12 below the block before's answer, rounded, 24
    /// candidates in all.
    pub first_bin: u16,
    /// Each candidate's score, from `first_bin` on, unitless: for each of the 19 round-dollar
    /// amounts ($1, 2, 3, 5, 10, 15, 20, 25, 30, 50, 100, 150, 200, 300, 500, 1k, 2k, 5k, 10k),
    /// the histogram's count where that amount lands if $100 is at the candidate, over that
    /// amount's peak in the window, summed. The highest is the pick's bin; all zero when the window
    /// held no payments at all, and the oracle kept the block before's answer.
    pub scores: Vec<f64>,
    /// The smoothed histogram it scored, 2,400 bins: the block's payments and the 11 blocks'
    /// before it (live, the mempool's as the next block's), each block's counts weighted
    /// `(2/7) * (5/7)^age`, the newest at age 0.
    pub histogram: HistogramEma,
    /// The block's own outputs (live, the mempool's) as the oracle's filter judged them.
    pub outputs: OracleOutputs,
}

/// A block's outputs as the oracle's filter judged them: each skipped one under the first rule
/// that skips it, in the filter's order, and the payments it kept. They add up to the block's
/// outputs.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OracleOutputs {
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

impl From<Tally> for OracleOutputs {
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
