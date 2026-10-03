//! CPFP queries shared by live mempool and confirmed transactions.

pub mod confirmed;
pub mod resolved;

pub use resolved::ResolvedCpfp;

use brk_types::{CpfpInfo, Txid};

use crate::{Error, Query, Result, r#impl::tx::ResolvedConfirmedTx};

enum CpfpSource {
    Memory(CpfpInfo),
    Chain(ResolvedConfirmedTx),
}

impl Query {
    fn resolve_cpfp_source(&self, txid: &Txid) -> Result<CpfpSource> {
        let read = self.read_indexer()?;
        match read.resolve_confirmed_tx(txid) {
            Ok(transaction) => Ok(CpfpSource::Chain(transaction)),
            Err(Error::UnknownTxid) => self
                .mempool()
                .ok_or(Error::UnknownTxid)?
                .cpfp_info(txid, &self.tip_blockhash_at(read.pin())?)?
                .map(CpfpSource::Memory)
                .ok_or(Error::UnknownTxid),
            Err(error) => Err(error),
        }
    }
}
