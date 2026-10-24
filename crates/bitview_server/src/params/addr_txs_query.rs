use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Txid;

/// A page of an address's transactions, mempool ones first.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressTxsQuery {
    /// Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
    /// cursor skips the mempool). One that is neither answers 422.
    pub after_txid: Option<Txid>,
    /// Transactions to return, at most 50 (the default); larger values return 50.
    pub max_txs: Option<usize>,
}

/// A page of an address's confirmed transactions.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressChainTxsQuery {
    /// Transactions to return, at most 25 (the default); larger values return 25.
    pub max_txs: Option<usize>,
}

/// A page of an address's mempool transactions.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressMempoolTxsQuery {
    /// Transactions to return, at most 50 (the default); larger values return 50.
    pub max_txs: Option<usize>,
}

/// `max_txs` within `page`, the route's default and maximum, as mempool.space's Esplora caps it.
pub fn page_size(max_txs: Option<usize>, page: usize) -> usize {
    max_txs.map_or(page, |max_txs| max_txs.min(page))
}
