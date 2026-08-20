use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use brk_types::{
    CpfpInfo, Hex, MerkleProof, RbfResponse, Transaction, TxOutspend, TxStatus, Txid, Version,
};

use crate::{
    AppState, CacheStrategy, Error,
    extended::TransformResponseExtended,
    params::{Empty, TxIndexParam, TxidParam, TxidVout, TxidsParam},
};

pub trait TxRoutes {
    fn add_tx_routes(self) -> Self;
}

impl TxRoutes for ApiRouter<AppState> {
    fn add_tx_routes(self) -> Self {
        self
            .api_route(
            "/api/tx-index/{index}",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxIndexParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_text(&headers, CacheStrategy::Immutable(Version::ONE), move |q| q.txid_by_index(param.index).map(|t| t.to_string())).await
                },
                |op| op
                    .id("get_tx_by_index")
                    .transactions_tag()
                    .summary("Txid by index")
                    .description("Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.")
                    .text_response::<Txid>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/v1/cpfp/{txid}",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.cpfp(&param.txid)).await
                },
                |op| op
                    .id("get_cpfp")
                    .transactions_tag()
                    .summary("CPFP info")
                    .description("Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*")
                    .json_response::<CpfpInfo>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/v1/tx/{txid}/rbf",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, state.mempool_strategy(), move |q| q.tx_rbf(&param.txid)).await
                },
                |op| op
                    .id("get_tx_rbf")
                    .transactions_tag()
                    .summary("RBF replacement history")
                    .description("Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*")
                    .json_response::<RbfResponse>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}",
            get_with(
                async |
                    headers: HeaderMap,
                    Path(param): Path<TxidParam>,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    state.respond_json(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.transaction(&param.txid)).await
                },
                |op| op
                    .id("get_tx")
                    .transactions_tag()
                    .summary("Transaction information")
                    .description(
                        "Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*",
                    )
                    .json_response::<Transaction>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/hex",
            get_with(
                async |
                    headers: HeaderMap,
                    Path(param): Path<TxidParam>,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    state.respond_text(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.transaction_hex(&param.txid)).await
                },
                |op| op
                    .id("get_tx_hex")
                    .transactions_tag()
                    .summary("Transaction hex")
                    .description(
                        "Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*",
                    )
                    .text_response::<Hex>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/merkleblock-proof",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_text(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.merkleblock_proof(&param.txid)).await
                },
                |op| op
                    .id("get_tx_merkleblock_proof")
                    .transactions_tag()
                    .summary("Transaction merkleblock proof")
                    .description("Get the merkleblock proof for a transaction (BIP37 format, hex encoded).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*")
                    .text_response::<Hex>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/merkle-proof",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.merkle_proof(&param.txid)).await
                },
                |op| op
                    .id("get_tx_merkle_proof")
                    .transactions_tag()
                    .summary("Transaction merkle proof")
                    .description("Get the merkle inclusion proof for a transaction.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*")
                    .json_response::<MerkleProof>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/outspend/{vout}",
            get_with(
                async |
                    headers: HeaderMap,
                    Path(path): Path<TxidVout>,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    let v = Version::ONE;
                    state.respond_json_optimistic(&headers, CacheStrategy::Immutable(v), move |q| {
                        let outspend = q.outspend(&path.txid, path.vout)?;
                        let strategy = if outspend.is_deeply_spent(q.height()) {
                            CacheStrategy::Immutable(v)
                        } else {
                            CacheStrategy::Tip
                        };
                        Ok((outspend, strategy))
                    }).await
                },
                |op| op
                    .id("get_tx_outspend")
                    .transactions_tag()
                    .summary("Output spend status")
                    .description(
                        "Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*",
                    )
                    .json_response::<TxOutspend>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/outspends",
            get_with(
                async |
                    headers: HeaderMap,
                    Path(param): Path<TxidParam>,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    let v = Version::ONE;
                    state.respond_json_optimistic(&headers, CacheStrategy::Immutable(v), move |q| {
                        let outspends = q.outspends(&param.txid)?;
                        let height = q.height();
                        let all_deep = outspends.iter().all(|o| o.is_deeply_spent(height));
                        let strategy = if all_deep { CacheStrategy::Immutable(v) } else { CacheStrategy::Tip };
                        Ok((outspends, strategy))
                    }).await
                },
                |op| op
                    .id("get_tx_outspends")
                    .transactions_tag()
                    .summary("All output spend statuses")
                    .description(
                        "Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*",
                    )
                    .json_response::<Vec<TxOutspend>>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/raw",
            get_with(
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_bytes(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.transaction_raw(&param.txid)).await
                },
                |op| op
                    .id("get_tx_raw")
                    .transactions_tag()
                    .mcp_ignore()
                    .summary("Transaction raw")
                    .description("Returns a transaction as binary data.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*")
                    .binary_response()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx/{txid}/status",
            get_with(
                async |
                    headers: HeaderMap,
                    Path(param): Path<TxidParam>,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    state.respond_json(&headers, state.tx_strategy(Version::ONE, &param.txid), move |q| q.transaction_status(&param.txid)).await
                },
                |op| op
                    .id("get_tx_status")
                    .transactions_tag()
                    .summary("Transaction status")
                    .description(
                        "Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*",
                    )
                    .json_response::<TxStatus>()
                    .not_modified()
                    .bad_request()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/v1/transaction-times",
            get_with(
                async |headers: HeaderMap, params: TxidsParam, State(state): State<AppState>| -> std::result::Result<Response, Error> {
                    Ok(state.respond_json(&headers, state.mempool_strategy(), move |q| q.transaction_times(&params.txids)).await)
                },
                |op| op
                    .id("get_transaction_times")
                    .transactions_tag()
                    .summary("Transaction first-seen times")
                    .description("Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*")
                    .json_response::<Vec<u64>>()
                    .not_modified()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/tx",
            post_with(
                async |_: Empty, State(state): State<AppState>, body: String| {
                    let hex = body.trim().to_string();
                    state.run(move |q| q.broadcast_transaction(&hex))
                        .await
                        .map(|txid| txid.to_string())
                        .map_err(Error::from)
                },
                |op| {
                    op.id("post_tx")
                        .transactions_tag()
                        .mcp_ignore()
                        .summary("Broadcast transaction")
                        .description("Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*")
                        .json_response::<Txid>()
                        .bad_request()
                        .server_error()
                },
            ),
        )
    }
}
