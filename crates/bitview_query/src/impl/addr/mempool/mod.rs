use std::sync::Arc;

use brk_types::{Addr, Transaction};

use crate::{Error, Query, Result, r#impl::addr::parse_addr};

impl Query {
    /// Capture shared transaction bodies only when mempool and indexed chain
    /// refer to the same completed publication.
    pub fn addr_mempool_txs(&self, addr: &Addr, limit: usize) -> Result<Vec<Arc<Transaction>>> {
        let bytes = parse_addr(addr)?;
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        let pin = self.pin_safe_lengths()?;
        Ok(mempool.addr_txs(&bytes, limit, &self.tip_blockhash_at(&pin)?)?)
    }
}
