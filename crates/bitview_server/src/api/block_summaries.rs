use aide::axum::{ApiRouter, routing::get_with};
use axum::{body::Bytes, extract::Path, http::HeaderMap, response::Response};
use bitview_types::TransactionSummary;
use brk_types::{BlockHash, Txid};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AppState, CacheParams,
    api::tags,
    extended::TransformResponseExtended,
    params::{BlockHashParam, Empty},
    request_state::RequestState,
};

#[derive(Deserialize, JsonSchema)]
struct BlockTxSummaryParam {
    hash: BlockHash,
    txid: Txid,
}

const POLICY: &str = "Canonical chain only. All outputs contribute to value; vsize is BIP-141 weight/4 rounded up. Rate is the stored local policy-size chunk rate, rounded up to 0.001 sat/vB. Flags use mempool bit positions with Bitview's documented native standardness, key/signature and CoinJoin heuristics, broad opcode-envelope detection, structural same-block CPFP and payout ratios including coinbase. Time, acceleration and replacement observations are unavailable: optional fields are omitted and observation bits are not populated. This is a native-policy adapter, not exact mempool classifier parity.";

impl AppState {
    async fn respond_block_summary(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
        txid: Option<Txid>,
    ) -> Response {
        let cdn_cache_mode = self.cdn_cache_mode;
        self.respond_read(headers, "application/json", move |q| {
            let source = q.resolve_block_summary(&hash, txid.as_ref())?;
            // The selector includes the exact supplied canonical block and tx membership.
            // No observer facts: append-only publication changes cannot change this body.
            let selector = txid.map(|id| id.to_string()).unwrap_or_else(|| "all".into());
            let params = CacheParams::live(format!("block-summary-native1-{}-{selector}", source.block_hash()))
                .stable_if(source.is_settled(), cdn_cache_mode);
            Ok((source, params))
        }, move |q, source| {
            let rows = source.build(q)?;
            // The pin has dropped before JSON serialization/body-budget accounting. Both
            // wire forms use the same computation; a point never invokes the full form.
            let bytes = if txid.is_some() {
                let row = rows.into_iter().next().ok_or(bitview_query::Error::Internal("Missing point summary"))?;
                serde_json::to_vec(&row)?
            } else {
                serde_json::to_vec(&rows)?
            };
            Ok(Bytes::from(bytes))
        }).await
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/block/{hash}/summary",
        get_with(
            async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty,
                   RequestState(state): RequestState| {
                state.respond_block_summary(headers, path.hash, None).await
            },
            |op| op.id("get_block_summary").tag(tags::BLOCKS)
                .summary("Classified block transactions").description(POLICY)
                .json_response::<Vec<TransactionSummary>>().not_modified().not_found(),
        ),
    ).api_route(
        "/api/v1/block/{hash}/tx/{txid}/summary",
        get_with(
            async |headers: HeaderMap, Path(path): Path<BlockTxSummaryParam>, _: Empty,
                   RequestState(state): RequestState| {
                state.respond_block_summary(headers, path.hash, Some(path.txid)).await
            },
            |op| op.id("get_block_transaction_summary").tag(tags::BLOCKS)
                .summary("Classified transaction in block").description(POLICY)
                .json_response::<TransactionSummary>().not_modified().not_found(),
        ),
    )
}
