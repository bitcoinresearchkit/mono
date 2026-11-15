mod addr;
mod block;
mod calendar;
mod cpfp;
mod mempool;
mod mining;
pub(crate) mod oracle;
mod price;
mod silent_payments;
mod tx;
mod urpd;
mod utxo_set;

pub use addr::{ResolvedAddrChainTxs, ResolvedAddrMempoolTxs, ResolvedAddrTxs, ResolvedAddrUtxos};
/// The least dust limit the silent payments routes take: they hold nothing smaller.
pub use bitview_plugin_silent_payments::DUST_FLOOR;
pub(crate) use block::StaleBlocks;
pub use block::{ResolvedBlockTimestamp, ResolvedBlocks, ResolvedBlocksV1};
pub use cpfp::ResolvedCpfp;
pub use mempool::{MempoolBlockSource, MempoolTxids, ResolvedMempoolBlockDiff, ResolvedRbf};
pub use mining::ResolvedPoolBlocks;
pub use price::ResolvedHistoricalPrice;
pub use silent_payments::{BlindbitBlock, BlindbitSpends};
pub use tx::{ResolvedConfirmedTx, ResolvedTransaction};
pub use urpd::ResolvedUrpd;
pub(crate) use urpd::UrpdGrid;
pub use utxo_set::UtxoSetPoint;

/// mempool.space reports a block without transactions at 0 sat/vB; the series leave it undefined.
fn mempool_fee_rate(rate: brk_types::FeeRate) -> brk_types::FeeRate {
    if f64::from(rate).is_nan() {
        brk_types::FeeRate::ZERO
    } else {
        rate
    }
}
