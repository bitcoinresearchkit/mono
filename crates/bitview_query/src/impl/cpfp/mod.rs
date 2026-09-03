//! CPFP queries shared by live mempool and confirmed transactions.

mod confirmed;
mod resolved;

pub use resolved::ResolvedCpfp;

use brk_error::{Error, Result};
use brk_types::{CpfpInfo, FeeRate, Txid};
use vecdb::ReadableVec;

use crate::Query;

use resolved::CpfpSource;

impl Query {
    /// Returns live mempool information when available, otherwise
    /// reconstructs the confirmed same-block cluster from indexed data.
    pub fn cpfp(&self, txid: &Txid) -> Result<CpfpInfo> {
        match self.resolve_cpfp_source(txid)? {
            CpfpSource::Memory(info) => Ok(info),
            CpfpSource::Chain(transaction) => self.confirmed_cpfp_resolved(transaction),
        }
    }

    /// Effective SFL chunk rate for live, confirmed, or replaced transactions.
    pub fn effective_fee_rate(&self, txid: &Txid) -> Result<FeeRate> {
        if let Some(mempool) = self.mempool()
            && let Some(rate) = mempool.live_effective_fee_rate(txid)
        {
            return Ok(rate);
        }

        if let Ok(index) = super::tx::resolve_tx_index_bounded(self, txid)
            && let Some(rate) = self
                .plugins()
                .transactions
                .fees
                .effective_fee_rate
                .tx_index
                .collect_one(index)
        {
            return Ok(rate);
        }

        if let Some(mempool) = self.mempool()
            && let Some(rate) = mempool.graveyard_fee_rate(txid)
        {
            return Ok(rate);
        }

        Err(Error::UnknownTxid)
    }
}
