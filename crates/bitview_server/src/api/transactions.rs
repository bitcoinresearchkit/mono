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
use bitview_query::{
    Query, RepresentationId, ResolvedConfirmedTx, ResolvedTransaction, Result as QueryResult,
};
use bitview_types::{MerkleProof, RbfResponse};
use brk_types::{
    Cpfp, Hex, Timestamp, Transaction, TxOutspend, TxStatus, Txid, TxidPrefix, Version,
};
use serde_json::to_vec;
use tower_http::limit::RequestBodyLimitLayer;

use super::broadcast;
use crate::{
    AppState,
    answer::{Answer, Content, JSON, Settle},
    api::tags,
    etag::Etag,
    extended::TransformResponseExtended,
    params::{Empty, TxIndexParam, TxidParam, TxidVout, TxidsParam},
    request_state::RequestState,
};

const TEXT: &str = "text/plain";

impl AppState {
    /// [`Self::respond_resolved`] for a transaction in the mempool or on the chain.
    async fn respond_transaction<B: Into<Bytes> + 'static>(
        &self,
        headers: &HeaderMap,
        txid: Txid,
        content_type: &'static str,
        settle: Settle,
        build: fn(&Query, ResolvedTransaction) -> QueryResult<B>,
    ) -> Response {
        self.respond_resolved(
            headers,
            Version::ONE,
            content_type,
            settle,
            move |q| {
                let source = q.resolve_transaction(&txid)?;
                let (identity, settled) = (source.identity(), source.is_settled());
                Ok((source, identity, settled))
            },
            build,
        )
        .await
    }

    /// [`Self::respond_resolved`] for a confirmed transaction's proof. Settled, never committed:
    /// the txid in its address doesn't say which block holds the transaction, so a deeper reorg
    /// can replace even the merkleblock proof's bytes, and browsers are never purged.
    async fn respond_proof<B: Into<Bytes> + 'static>(
        &self,
        headers: &HeaderMap,
        txid: Txid,
        content_type: &'static str,
        build: fn(&Query, ResolvedConfirmedTx) -> QueryResult<B>,
    ) -> Response {
        self.respond_resolved(
            headers,
            Version::ONE,
            content_type,
            Answer::settled,
            move |q| {
                let tx = q.resolve_confirmed_tx(&txid)?;
                let (identity, settled) = (tx.identity(), tx.is_settled());
                Ok((tx, identity, settled))
            },
            build,
        )
        .await
    }

    /// A transaction's outspends, which the query builds with what names them and whether that
    /// is settled.
    async fn respond_outspends(
        &self,
        headers: &HeaderMap,
        built: impl FnOnce(&Query) -> QueryResult<(Vec<u8>, RepresentationId, bool)>
        + Clone
        + Send
        + 'static,
    ) -> Response {
        let built_bytes = |_: &Query, bytes| Ok(bytes);
        self.respond_resolved(headers, Version::ONE, JSON, Answer::settled, built, built_bytes)
            .await
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router
        .api_route(
        "/api/tx-index/{index}",
        get_with(
            async |headers: HeaderMap,
                   Path(param): Path<TxIndexParam>,
                   _: Empty,
                   RequestState(state): RequestState| {
                state
                    .respond(&headers, state.chain(), move |q| {
                        let (txid, settled) = q.txid_by_index(param.index)?;
                        let tag = format!("tx-index3-{}-{txid}", Version::ONE);
                        Ok(Answer::ready(tag, Content::text(txid.to_string())).settled(settled))
                    })
                    .await
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
                state
                    .respond_resolved(
                        &headers,
                        Version::TWO,
                        JSON,
                        Answer::settled,
                        move |q| {
                            let source = q.resolve_cpfp(&param.txid)?;
                            let (identity, settled) = (source.identity(), source.is_settled());
                            Ok((source, identity, settled))
                        },
                        |q, source| q.cpfp_json_resolved(source),
                    )
                    .await
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
        "/api/v1/tx/{txid}/rbf",
        get_with(
            async |headers: HeaderMap, Path(TxidParam { txid }): Path<TxidParam>, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_built(&headers, state.chain(), move |q| {
                        q.tx_rbf_json_resolved(q.resolve_rbf(&txid)?)
                    })
                    .await
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
                state
                    .respond_transaction(
                        &headers,
                        param.txid,
                        JSON,
                        Answer::settled,
                        Query::transaction_json_resolved,
                    )
                    .await
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
                state
                    .respond_transaction(
                        &headers,
                        param.txid,
                        TEXT,
                        Answer::committed,
                        Query::transaction_hex_resolved,
                    )
                    .await
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
                state
                    .respond_proof(
                        &headers,
                        param.txid,
                        TEXT,
                        Query::merkleblock_proof_resolved,
                    )
                    .await
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
                state
                    .respond_proof(&headers, param.txid, JSON, |q, tx| {
                        Ok(to_vec(&q.merkle_proof_resolved(tx)?)?)
                    })
                    .await
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
                    .respond_outspends(&headers, move |q| q.outspend_json(&txid, vout))
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
        "/api/tx/{txid}/outspends",
        get_with(
            async |
                headers: HeaderMap,
                Path(TxidParam { txid }): Path<TxidParam>,
                _: Empty,
                RequestState(state): RequestState
            | {
                state
                    .respond_outspends(&headers, move |q| q.outspends_json(&txid))
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
                state
                    .respond_transaction(
                        &headers,
                        param.txid,
                        "application/octet-stream",
                        Answer::committed,
                        Query::transaction_raw_resolved,
                    )
                    .await
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
            | {
                let txid = param.txid;
                state
                    .respond(&headers, state.chain(), move |q| {
                        let (status, settled) = q.transaction_status(&txid)?;
                        let tag = match status.block_hash {
                            Some(hash) => format!("tx-status3-{}-{hash}", Version::ONE).into(),
                            None => Etag::content(*TxidPrefix::from(txid)),
                        };
                        Ok(Answer::ready(tag, Content::json(&status)?).settled(settled))
                    })
                    .await
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
            async |headers: HeaderMap, params: TxidsParam, RequestState(state): RequestState| {
                state
                    .respond(&headers, state.mempool(), move |q| {
                        let (times, hash) = q.transaction_times_with_hash(&params.txids)?;
                        Ok(Answer::ready(Etag::content(hash), Content::json(&times)?))
                    })
                    .await
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
                    .description("Submit a raw transaction as hexadecimal text (at most 800,000 request bytes, including whitespace: a standard transaction's largest size). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*")
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
