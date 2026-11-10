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
    #[serde(default, deserialize_with = "bitview_types::de_plain")]
    pub max_txs: Option<usize>,
}

/// A page of an address's confirmed transactions.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressChainTxsQuery {
    /// Transactions to return, at most 25 (the default); larger values return 25.
    #[serde(default, deserialize_with = "bitview_types::de_plain")]
    pub max_txs: Option<usize>,
}

/// A page of an address's mempool transactions.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressMempoolTxsQuery {
    /// Transactions to return, at most 50 (the default); larger values return 50.
    #[serde(default, deserialize_with = "bitview_types::de_plain")]
    pub max_txs: Option<usize>,
}

/// `max_txs` within `page`, the route's default and maximum, as mempool.space's Esplora caps it.
pub fn page_size(max_txs: Option<usize>, page: usize) -> usize {
    max_txs.map_or(page, |max_txs| max_txs.min(page))
}

/// A page of confirmed address summaries; the cursor is supplied only in the path.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressTxsSummaryQuery {
    /// Transactions to return, at most 5,000 (the default); zero returns an empty array.
    #[serde(default, deserialize_with = "bitview_types::de_plain")]
    pub max_txs: Option<usize>,
}

#[cfg(test)]
mod tests {
use axum::{extract::Query, http::Uri};

use crate::params::{AddressTxsSummaryQuery, page_size};

#[test]
fn summary_query_is_bounded_and_has_one_numeric_spelling() {
    for (query, expected) in [("", 5_000), ("?max_txs=0", 0), ("?max_txs=1", 1),
        ("?max_txs=25", 25), ("?max_txs=5000", 5_000), ("?max_txs=5001", 5_000)] {
        let uri: Uri = format!("/api/address/a/txs/summary{query}").parse().unwrap();
        let Query(query) = Query::<AddressTxsSummaryQuery>::try_from_uri(&uri).unwrap();
        assert_eq!(page_size(query.max_txs, 5_000), expected);
    }
    for query in ["max_txs=00", "max_txs=01", "max_txs=-1", "max_txs=+1", "max_txs=1.0",
        "max_txs=", "max_txs=1&max_txs=2", "after_txid=abc", "limit=1", "max_txs=1&extra=0"] {
        let uri: Uri = format!("/api/address/a/txs/summary?{query}").parse().unwrap();
        assert!(Query::<AddressTxsSummaryQuery>::try_from_uri(&uri).is_err(), "{query}");
    }
}
}
