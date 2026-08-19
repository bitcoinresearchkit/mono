use std::str::FromStr;

use brk_error::Error;
use brk_types::{Addr, AddrBytes, Transaction};

use crate::Query;

impl Query {
    pub fn addr_mempool_hash(&self, addr: &Addr) -> Option<u64> {
        let mempool = self.mempool()?;
        let bytes = AddrBytes::from_str(addr).ok()?;
        mempool.addr_state_hash(&bytes)
    }

    pub fn addr_mempool_txs(
        &self,
        addr: &Addr,
        limit: usize,
    ) -> brk_error::Result<Vec<Transaction>> {
        let bytes = AddrBytes::from_str(addr)?;
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        Ok(mempool.addr_txs(&bytes, limit))
    }
}
