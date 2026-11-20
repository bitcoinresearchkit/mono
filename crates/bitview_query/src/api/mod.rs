mod addr;
pub(crate) mod audit;
mod block;
mod calendar;
mod cpfp;
mod json;
mod mempool;
mod mining;
pub(crate) mod oracle;
mod price;
mod silent_payments;
mod tx;
mod urpd;
mod utxo_set;

pub use addr::{ResolvedAddrChainTxs, ResolvedAddrMempoolTxs, ResolvedAddrTxs, ResolvedAddrUtxos};
pub use bitview_plugin_pools::pools;
pub use bitview_types::{AuditDetail, AuditScore, AuditSummary, TxAudit};
/// The least dust limit the silent payments routes answer for: they hold nothing smaller.
pub use bitview_plugin_silent_payments::DUST_FLOOR;
pub use bitview_plugin_silent_payments::silent_payments_available;
pub(crate) use block::StaleBlocks;
pub use block::{ResolvedBlockTimestamp, ResolvedBlocks, ResolvedBlocksV1};
pub use cpfp::ResolvedCpfp;
pub use mempool::{MempoolBlockSource, MempoolTxids, ResolvedMempoolBlockDiff, ResolvedRbf};
pub use mining::ResolvedPoolBlocks;
pub use oracle::ResolvedOracleBlocks;
pub use price::ResolvedHistoricalPrice;
pub use silent_payments::{BlindbitBlock, BlindbitOutputs};
pub use tx::{ResolvedConfirmedTx, ResolvedTransaction};
pub use urpd::ResolvedUrpd;
pub use utxo_set::UtxoSetPoint;

/// mempool.space reports a block without transactions at 0 sat/vB; the series leave it undefined.
fn mempool_fee_rate(rate: brk_types::FeeRate) -> brk_types::FeeRate {
    if f64::from(rate).is_nan() {
        brk_types::FeeRate::ZERO
    } else {
        rate
    }
}
