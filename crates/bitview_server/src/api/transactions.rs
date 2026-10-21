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
use bitview_types::{MerkleProof, RbfResponse};
use brk_types::{
    CpfpInfo, Hex, Timestamp, Transaction, TxOutspend, TxStatus, Txid, TxidPrefix, Version,
};
use serde_json::to_vec;
use tower_http::limit::RequestBodyLimitLayer;

use super::broadcast;
use crate::{
    AppState, CacheParams,
    api::tags,
    error::Result,
    extended::{ResponseExtended, TransformResponseExtended},
    params::{Empty, TxIndexParam, TxidParam, TxidVout, TxidsParam},
    request_state::RequestState,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router
        .api_route(
        "/api/tx-index/{index}",
        get_with(
            async |headers: HeaderMap,
                   Path(param): Path<TxIndexParam>,
                   _: Empty,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                let txid = state.read_admitted(move |q| q.txid_by_index(param.index)).await?;
                let params = CacheParams::live(format!("tx-index3-{}-{txid}", Version::ONE));
                Ok(Response::cached_bytes(&headers, &params, "text/plain", || Bytes::from(txid.to_string())))
            },
            |op| op
                .id("get_tx_by_index")
                .tag(tags::TRANSACTIONS)
                .summary("Txid by index")
                .description("Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.")
                .text_response::<Txid>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/v1/cpfp/{txid}",
        get_with(
            async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "application/json", move |q| {
                    let source = q.resolve_cpfp(&param.txid)?;
                    let params = CacheParams::representation(Version::TWO, source.identity())
                        .stable_if(source.is_settled(), cdn_cache_mode);
                    Ok((source, params))
                }, |q, source| q.cpfp_json_resolved(source).map(Bytes::from)).await
            },
            |op| op
                .id("get_cpfp")
                .tag(tags::TRANSACTIONS)
                .summary("CPFP info")
                .description("Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*")
                .json_response::<CpfpInfo>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/v1/tx/{txid}/rbf",
        get_with(
            async |headers: HeaderMap, Path(TxidParam { txid }): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                state.respond_json_bound(headers, Version::ONE, move |q| q.tx_rbf_json_resolved(q.resolve_rbf(&txid)?)).await
            },
            |op| op
                .id("get_tx_rbf")
                .tag(tags::TRANSACTIONS)
                .summary("RBF replacement history")
                .description("Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*")
                .json_response::<RbfResponse>()
                .not_modified()
                .not_found(),
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
            | {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "application/json", move |q| {
                    let source = q.resolve_transaction(&param.txid)?;
                    let params = CacheParams::representation(Version::ONE, source.identity())
                        .stable_if(source.is_settled(), cdn_cache_mode);
                    Ok((source, params))
                }, |q, source| q.transaction_json_resolved(source).map(Bytes::from)).await
            },
            |op| op
                .id("get_tx")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction information")
                .description(
                    "*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*",
                )
                .json_response::<Transaction>()
                .not_modified()
                .not_found(),
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
            | {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "text/plain", move |q| {
                    let source = q.resolve_transaction(&param.txid)?;
                    let params = CacheParams::representation(Version::ONE, source.identity())
                        .stable_if(source.is_settled(), cdn_cache_mode);
                    Ok((source, params))
                }, |q, source| q.transaction_hex_resolved(source).map(Bytes::from)).await
            },
            |op| op
                .id("get_tx_hex")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction hex")
                .description(
                    "Hex-encoded serialized transaction.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*",
                )
                .text_response::<Hex>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/tx/{txid}/merkleblock-proof",
        get_with(
            async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "text/plain", move |q| {
                    let tx = q.resolve_confirmed_tx(&param.txid)?;
                    let params = CacheParams::representation(Version::ONE, tx.identity())
                        .stable_if(tx.is_settled(), cdn_cache_mode);
                    Ok((tx, params))
                }, |q, tx| q.merkleblock_proof_resolved(tx).map(Bytes::from)).await
            },
            |op| op
                .id("get_tx_merkleblock_proof")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction merkleblock proof")
                .description("Get the merkleblock proof for a transaction (BIP37 format, hex encoded).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*")
                .text_response::<Hex>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/tx/{txid}/merkle-proof",
        get_with(
            async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "application/json", move |q| {
                    let tx = q.resolve_confirmed_tx(&param.txid)?;
                    let params = CacheParams::representation(Version::ONE, tx.identity())
                        .stable_if(tx.is_settled(), cdn_cache_mode);
                    Ok((tx, params))
                }, |q, tx| Ok(to_vec(&q.merkle_proof_resolved(tx)?)?.into())).await
            },
            |op| op
                .id("get_tx_merkle_proof")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction merkle proof")
                .description("*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*")
                .json_response::<MerkleProof>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/tx/{txid}/outspend/{vout}",
        get_with(
            async |
                headers: HeaderMap,
                Path(TxidVout { txid, vout }): Path<TxidVout>,
                _: Empty,
                RequestState(state): RequestState
            | {
                state
                    .respond_json_bound(headers, Version::ONE, move |q| {
                        q.outspend_json(&txid, vout)
                    })
                    .await
            },
            |op| op
                .id("get_tx_outspend")
                .tag(tags::TRANSACTIONS)
                .summary("Output spend status")
                .description(
                    "Whether the output has been spent and, if so, the spending transaction details.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*",
                )
                .json_response::<TxOutspend>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/tx/{txid}/outspends",
        get_with(
            async |
                headers: HeaderMap,
                Path(TxidParam { txid }): Path<TxidParam>,
                _: Empty,
                RequestState(state): RequestState
            | {
                state
                    .respond_json_bound(headers, Version::ONE, move |q| {
                        q.outspends_json(&txid)
                    })
                    .await
            },
            |op| op
                .id("get_tx_outspends")
                .tag(tags::TRANSACTIONS)
                .summary("All output spend statuses")
                .description(
                    "Spend statuses in output-index order.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*",
                )
                .json_response::<Vec<TxOutspend>>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/tx/{txid}/raw",
        get_with(
            async |headers: HeaderMap, Path(param): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                let cdn_cache_mode = state.cdn_cache_mode;
                state.respond_read(headers, "application/octet-stream", move |q| {
                    let source = q.resolve_transaction(&param.txid)?;
                    let params = CacheParams::representation(Version::ONE, source.identity())
                        .stable_if(source.is_settled(), cdn_cache_mode);
                    Ok((source, params))
                }, |q, source| q.transaction_raw_resolved(source).map(Bytes::from)).await
            },
            |op| op
                .id("get_tx_raw")
                .tag(tags::TRANSACTIONS)
                .mcp_ignore()
                .summary("Transaction raw")
                .description("Returns a transaction as binary data.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*")
                .binary_response()
                .not_modified()
                .not_found(),
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
                let cdn_cache_mode = state.cdn_cache_mode;
                let (status, settled) =
                    state.read_admitted(move |q| q.transaction_status(&txid)).await?;
                let params = match status.block_hash {
                    Some(hash) => CacheParams::live(format!("tx-status3-{}-{hash}", Version::ONE))
                        .stable_if(settled, cdn_cache_mode),
                    None => CacheParams::live_hash(*TxidPrefix::from(txid)),
                };
                Ok(Response::cached_json(&headers, &params, || status))
            },
            |op| op
                .id("get_tx_status")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction status")
                .description(
                    "Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*",
                )
                .json_response::<TxStatus>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/v1/transaction-times",
        get_with(
            async |headers: HeaderMap, params: TxidsParam, RequestState(state): RequestState| -> Result<Response> {
                let (times, hash) = state.query.read(move |q| q.transaction_times_with_hash(&params.txids)).await?;
                Ok(Response::cached_json(&headers, &CacheParams::live_hash(hash), || times))
            },
            |op| op
                .id("get_transaction_times")
                .tag(tags::TRANSACTIONS)
                .summary("Transaction first-seen times")
                .description("First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*")
                .json_response::<Vec<Timestamp>>()
                .not_modified(),
        ),
    )
    .api_route(
        "/api/tx",
        post_with(
            broadcast::serve,
            |op| {
                op.id("post_tx")
                    .tag(tags::TRANSACTIONS)
                    .mcp_ignore()
                    .summary("Broadcast transaction")
                    .description("Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*")
                    .text_response::<Txid>()
                    .error_response::<413>("Request body too large")
            },
        )
        .layer((
            RequestBodyLimitLayer::new(broadcast::MAX_BODY_BYTES),
            DefaultBodyLimit::disable(),
        )),
    )
}
