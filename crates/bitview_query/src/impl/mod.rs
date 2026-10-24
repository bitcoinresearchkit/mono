#[cfg(feature = "chain")]
pub mod addr;
#[cfg(feature = "chain")]
pub mod block;
#[cfg(feature = "price")]
mod calendar;
#[cfg(feature = "chain")]
pub mod cpfp;
#[cfg(feature = "chain")]
pub mod mempool;
#[cfg(feature = "chain")]
pub mod mining;
#[cfg(feature = "price")]
pub mod oracle;
#[cfg(feature = "price")]
pub mod price;
#[cfg(feature = "series")]
pub mod series;
#[cfg(feature = "chain")]
pub mod tx;
#[cfg(feature = "urpd")]
pub mod urpd;
#[cfg(feature = "chain")]
pub mod utxo_set;

#[cfg(feature = "urpd")]
pub use urpd::ResolvedUrpd;

#[cfg(feature = "chain")]
pub use addr::{ResolvedAddrChainTxs, ResolvedAddrMempoolTxs, ResolvedAddrTxs, ResolvedAddrUtxos};
#[cfg(feature = "chain")]
pub use block::{ResolvedBlockTimestamp, ResolvedBlocks, ResolvedBlocksV1};
#[cfg(feature = "chain")]
pub use cpfp::ResolvedCpfp;
#[cfg(feature = "chain")]
pub use mempool::{MempoolBlockSource, MempoolTxids, ResolvedMempoolBlockDiff, ResolvedRbf};
#[cfg(feature = "chain")]
pub use mining::ResolvedPoolBlocks;
#[cfg(feature = "series")]
pub use series::{ResolvedQuery, SeriesAnchor, SeriesRead};
#[cfg(feature = "chain")]
pub use tx::{ResolvedConfirmedTx, ResolvedTransaction};
#[cfg(feature = "chain")]
pub use utxo_set::UtxoSetPoint;

/// mempool.space reports a block without transactions at 0 sat/vB; the series leave it undefined.
#[cfg(feature = "chain")]
fn mempool_fee_rate(rate: brk_types::FeeRate) -> brk_types::FeeRate {
    if f64::from(rate).is_nan() {
        brk_types::FeeRate::ZERO
    } else {
        rate
    }
}
