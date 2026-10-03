use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path},
    http::HeaderMap,
    response::Response,
};
use bitview_types::{Hex, MerkleProof, RbfResponse};
use brk_types::{CpfpInfo, Transaction, TxOutspend, TxStatus, Txid, TxidPrefix, Version};
use serde_json::to_vec;
use tower_http::limit::RequestBodyLimitLayer;

use super::broadcast;
use crate::{
    AppState, CacheParams, CacheStrategy,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, TxIndexParam, TxidParam, TxidVout, TxidsParam},
    request_state::RequestState,
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
                async |headers: HeaderMap,
                       Path(param): Path<TxIndexParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let txid = state.read_admitted(move |q| q.txid_by_index(param.index)).await?;
                    let strategy = CacheStrategy::Live(format!("tx-index3-{}-{txid}", Version::ONE).into());
                    Ok(state.respond_text_value(&headers, strategy, txid.to_string()))
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
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "application/json", move |q| {
                        let source = q.resolve_cpfp(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, source.identity()), mode);
                        Ok((source, params))
                    }, |q, source| q.cpfp_json_resolved(source).map(Bytes::from)).await)
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
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let rbf = state.read_admitted(move |q| q.resolve_rbf(&param.txid)).await?;
                    let strategy = rbf.identity().map(|identity| AppState::representation_strategy(Version::ONE, identity));
                    if let Some(strategy) = strategy {
                        return Ok(state.respond_json_value(&headers, strategy, RbfResponse::EMPTY));
                    }
                    Ok(state.respond_json_bound(&headers, Version::ONE, move |q| q.tx_rbf_json_resolved(q.resolve_rbf(&param.txid)?)).await)
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
                    RequestState(state): RequestState
                | -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "application/json", move |q| {
                        let source = q.resolve_transaction(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, source.identity()), mode);
                        Ok((source, params))
                    }, |q, source| q.transaction_json_resolved(source).map(Bytes::from)).await)
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
                    RequestState(state): RequestState
                | -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "text/plain", move |q| {
                        let source = q.resolve_raw_transaction(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, source.identity()), mode);
                        Ok((source, params))
                    }, |q, source| q.transaction_hex_resolved(source).map(Bytes::from)).await)
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
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "text/plain", move |q| {
                        let tx = q.resolve_confirmed_tx(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, tx.identity()), mode);
                        Ok((tx, params))
                    }, |q, tx| q.merkleblock_proof_resolved(tx).map(Bytes::from)).await)
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
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "application/json", move |q| {
                        let tx = q.resolve_confirmed_tx(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, tx.identity()), mode);
                        Ok((tx, params))
                    }, |q, tx| Ok(to_vec(&q.merkle_proof_resolved(tx)?)?.into())).await)
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
                    RequestState(state): RequestState
                | {
                    state
                        .respond_json_bound(&headers, Version::ONE, move |q| {
                            q.outspend_json(&path.txid, path.vout)
                        })
                        .await
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
                    RequestState(state): RequestState
                | {
                    state
                        .respond_json_bound(&headers, Version::ONE, move |q| {
                            q.outspends_json(&param.txid)
                        })
                        .await
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
                async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let mode = state.cdn_cache_mode;
                    Ok(state.respond_read(headers, "application/octet-stream", move |q| {
                        let source = q.resolve_raw_transaction(&param.txid)?;
                        let params = CacheParams::resolve(&AppState::representation_strategy(Version::ONE, source.identity()), mode);
                        Ok((source, params))
                    }, |q, source| q.transaction_raw_resolved(source).map(Bytes::from)).await)
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
                    RequestState(state): RequestState
                | -> Result<Response> {
                    let txid = param.txid;
                    let status = state.read_admitted(move |q| q.transaction_status(&txid)).await?;
                    let strategy = match status.block_hash {
                        Some(hash) => CacheStrategy::Live(format!("tx-status3-{}-{hash}", Version::ONE).into()),
                        None => CacheStrategy::LiveHash(*TxidPrefix::from(txid)),
                    };
                    Ok(state.respond_json_value(&headers, strategy, status))
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
                async |headers: HeaderMap, params: TxidsParam, RequestState(state): RequestState| -> Result<Response> {
                    let (times, hash) = state.read(move |q| q.transaction_times_with_hash(&params.txids)).await?;
                    Ok(state.respond_json_value(&headers, CacheStrategy::LiveHash(hash), times))
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
                broadcast::serve,
                |op| {
                    op.id("post_tx")
                        .transactions_tag()
                        .mcp_ignore()
                        .summary("Broadcast transaction")
                        .description("Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*")
                        .text_response::<Txid>()
                        .bad_request()
                        .error_response::<413>("Request body too large")
                        .server_errors()
                },
            )
            .layer((
                RequestBodyLimitLayer::new(broadcast::MAX_BODY_BYTES),
                DefaultBodyLimit::disable(),
            )),
        )
    }
}
