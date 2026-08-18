//! CPFP queries shared by live mempool and confirmed transactions.

mod confirmed;

use bitview_plugin::Plugin;
use brk_error::{Error, Result};
use brk_types::{FeeRate, Txid, TxidPrefix};
use vecdb::ReadableVec;

use crate::Query;

impl Query {
    /// Returns live mempool information when available, otherwise
    /// reconstructs the confirmed same-block cluster from indexed data.
    pub fn cpfp(&self, txid: &Txid) -> Result<brk_types::CpfpInfo> {
        let prefix = TxidPrefix::from(txid);
        if let Some(info) = self.mempool().and_then(|m| m.cpfp_info(&prefix)) {
            return Ok(info);
        }
        let _guard = self.computer().outputs.gate().read();
        self.confirmed_cpfp(txid)
    }

    /// Effective SFL chunk rate for live, confirmed, or replaced transactions.
    pub fn effective_fee_rate(&self, txid: &Txid) -> Result<FeeRate> {
        let prefix = TxidPrefix::from(txid);

        if let Some(mempool) = self.mempool()
            && let Some(rate) = mempool.live_effective_fee_rate(&prefix)
        {
            return Ok(rate);
        }

        if let Ok(index) = self.resolve_tx_index_bounded(txid)
            && let Some(rate) = self
                .computer()
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
