use bitview_primitives::{Lengths, TypeIndex};
use brk_error::Result;
use brk_types::{Height, OutputType, Txid};

use crate::Query;

impl Query {
    pub(crate) fn addr_last_activity_height_bounded(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        before_txid: Option<&Txid>,
        safe: Lengths,
    ) -> Result<Height> {
        if type_index >= safe.to_type_index(output_type) {
            return Err(self.missing_addr());
        }
        let stores = self.indexer().stores();
        let tx_index_len = safe.tx_index;
        let before = before_txid
            .map(|txid| self.resolve_tx_index(txid))
            .transpose()?
            .unwrap_or(tx_index_len)
            .min(tx_index_len);
        let last_tx_index = stores
            .addr_tx_indexes_before(output_type, type_index, before)?
            .next_back()
            .ok_or_else(|| self.missing_addr())?;
        self.confirmed_status_height_bounded(last_tx_index, safe)
    }
}
