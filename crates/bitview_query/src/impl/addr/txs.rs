use brk_types::{Addr, Transaction, TxIndex, Txid};

use crate::Query;

impl Query {
    pub fn addr_txs_chain(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> brk_error::Result<Vec<Transaction>> {
        let txindices = self.addr_txindices(addr, after_txid, limit)?;
        self.transactions_by_indices(&txindices)
    }

    pub fn addr_txids(
        &self,
        addr: Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> brk_error::Result<Vec<Txid>> {
        let txindices = self.addr_txindices(&addr, after_txid, limit)?;
        let txid_reader = self.indexer().vecs().transactions.txid.reader();
        Ok(txindices
            .into_iter()
            .map(|tx_index| txid_reader.get(tx_index))
            .collect())
    }

    fn addr_txindices(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> brk_error::Result<Vec<TxIndex>> {
        let stores = self.indexer().stores();

        let (output_type, type_index) = super::resolve::resolve_addr(self, addr)?;
        let tx_index_len = self.safe_lengths().tx_index;

        if let Some(after_txid) = after_txid {
            let after_tx_index = crate::r#impl::tx::resolve_tx_index(self, &after_txid)?;
            Ok(stores
                .addr_tx_indexes_before(output_type, type_index, after_tx_index)?
                .rev()
                .filter(|tx_index| *tx_index < tx_index_len)
                .take(limit)
                .collect())
        } else {
            Ok(stores
                .addr_tx_indexes(output_type, type_index)?
                .rev()
                .filter(|tx_index| *tx_index < tx_index_len)
                .take(limit)
                .collect())
        }
    }
}
