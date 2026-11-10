use bitview_types::CachedTransaction;
use brk_types::Txid;

use super::serialize_json;
use crate::{Error, Query, RepresentationId, Result};

impl Query {
    /// Native retained cache only: no chain pin, transaction index, RPC or graph traversal.
    /// One publication supplies the body, first sighting and rate. Serialize its borrowed base
    /// body once, hashing the exact bytes, before handing the result back to async code.
    pub fn cached_transaction_json(
        &self,
        txid: &Txid,
    ) -> Result<Option<(Vec<u8>, RepresentationId)>> {
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        let Some((transaction, rate)) = mempool.cached_transaction(txid)? else {
            return Ok(None);
        };
        let value = CachedTransaction::new(transaction.tx.as_ref(), transaction.first_seen, rate);
        Ok(Some(serialize_json(&value)))
    }
}
