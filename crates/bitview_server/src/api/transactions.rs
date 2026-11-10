use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query},
    http::HeaderMap,
    response::Response,
};
use bitview_types::{CachedTransaction, MerkleProof, RbfResponse};
use brk_types::{
    Cpfp, Hex, Timestamp, Transaction, TxOutspend, TxStatus, Txid, TxidPrefix, Version,
};
use serde_json::to_vec;
use schemars::JsonSchema;
use serde::Deserialize;
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
                .id("get_txid_by_index")
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
                .description("Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package. A mempool transaction answers its live package; a confirmed one its package among its block's transactions; an unknown transaction, or a confirmed one without relatives in its block, `{\"ancestors\":[]}`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*")
                .json_response::<Cpfp>()
                .not_modified(),
        ),
    )
    .api_route(
        "/api/v1/tx/{txid}/cached",
        get_with(
            async |headers: HeaderMap, Path(TxidParam { txid }): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                state.respond_optional_json_bound(headers, Version::ONE, move |q| q.cached_transaction_json(&txid)).await
            },
            |op| op
                .id("get_cached_tx")
                .tag(tags::TRANSACTIONS)
                .summary("Retained transaction")
                .description("Returns a retained transaction with all mandatory mempool enrichment fields, or an empty 204 when absent. Native cache membership includes published pool records and all retained graveyard bodies, beyond upstream's RBF-only admission. Lone tombstones normally remain one hour; replacement trees one day after their root leaves, subject to the existing memory policy. Status and prevouts are the retained mempool view, including after mining; no chain or RPC fallback. firstSeen is the retained first sighting. effectiveFeePerVsize is the native Core chunk rate (milli-sat/vB precision), frozen at burial for tombstones. Sigops use the existing base-body count, which omits missing-prevout components. Unknown graph, acceleration, position and bookkeeping fields are omitted. Found bodies use the live cache tier; absence is not stored.")
                .json_response::<CachedTransaction>()
                .response_with::<204, (), _>(|res| res.description("Not present in the published native retained cache; empty body"))
                .not_modified(),
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
                .description("Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window. A history past 63 versions or 4,096 transactions comes cut to its newest part, its root marked `\"truncated\": true`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*")
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
                    "Full details of a transaction by txid: inputs, outputs, fee, size, and confirmation status.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*",
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
                .description("Merkle inclusion proof of a confirmed transaction in its block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*")
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
                    "Whether the output has been spent and, if so, the spending transaction details. An output no known transaction holds is unspent.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*",
                )
                .json_response::<TxOutspend>()
                .not_modified(),
        ),
    )
    .api_route(
        "/api/txs/outspends",
        get_with(
            async |headers: HeaderMap, Query(query): Query<OutspendsQuery>, RequestState(state): RequestState| -> Result<Response> {
                let txids = query.parse()?;
                Ok(state.respond_json_bound(headers, Version::ONE, move |q| {
                    q.batch_outspends_json(&txids)
                }).await)
            },
            |op| op
                .id("get_txs_outspends")
                .tag(tags::TRANSACTIONS)
                .summary("Output spend statuses for several transactions")
                .description("Outspends for 1–50 comma-separated lowercase transaction IDs in `txids`, in request order and then output order. Unknown transactions have an empty array; repeated IDs retain their positions. Invalid IDs, empty elements and unknown query parameters answer 400. The complete batch uses one chain and mempool publication.")
                .json_response::<Vec<Vec<TxOutspend>>>()
                .not_modified(),
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
                    "Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp. A transaction not in the chain, pending or unknown, is `{\"confirmed\":false}`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*",
                )
                .json_response::<TxStatus>()
                .not_modified(),
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
        "/api/broadcast",
        get_with(broadcast::serve_query, |op| {
            op.id("get_broadcast")
                .tag(tags::TRANSACTIONS)
                .mcp_ignore()
                .summary("Broadcast transaction from query")
                .description("Submit `tx` as hexadecimal transaction text through the same admission and Core RPC as `POST /api/tx`. This GET changes node state. Submission responses are not cached. HTTP request-line limits apply; use POST for large transactions. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.")
                .text_response::<Txid>()
        }),
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

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct OutspendsQuery {
    /// One to fifty comma-separated, lowercase transaction IDs.
    txids: String,
}

impl OutspendsQuery {
    fn parse(self) -> Result<Vec<Txid>> {
        let parts = self.txids.split(',');
        if parts.clone().count() > 50 {
            return Err(crate::Error::bad_request("at most 50 transaction IDs are accepted"));
        }
        parts.map(|part| {
            if part.len() != 64 || !part.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')) {
                return Err(crate::Error::bad_request("txids must be comma-separated lowercase transaction IDs"));
            }
            part.parse().map_err(|_| crate::Error::bad_request("invalid transaction ID"))
        }).collect()
    }
}
