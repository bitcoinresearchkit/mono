use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_types::ReplacementNode;
use brk_types::{
    Dollars, MempoolBlockContent, MempoolBlockDiff, MempoolInfo, MempoolRecentTx, Txid, Version,
};

use super::{mempool_block, mempool_txids};
use crate::{
    AppState,
    api::oracle::serve_live_price,
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, MempoolBlockDiffParam, MempoolBlockParam},
    request_state::RequestState,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/mempool",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_bound(headers, Version::ONE, |q| q.mempool_info_json())
                    .await
            },
            |op| {
                op.id("get_mempool")
                    .tag(tags::MEMPOOL)
                    .summary("Mempool statistics")
                    .description("*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*")
                    .json_response::<MempoolInfo>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/mempool/txids",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                mempool_txids::serve(state, headers).await
            },
            |op| {
                op.id("get_mempool_txids")
                    .tag(tags::MEMPOOL)
                    .mcp_ignore()
                    .summary("Mempool transaction IDs")
                    .description("Get all transaction IDs currently in the mempool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*")
                    .json_response::<Vec<Txid>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/mempool/recent",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_bound(headers, Version::ONE, |q| q.mempool_recent_json())
                    .await
            },
            |op| {
                op.id("get_mempool_recent")
                    .tag(tags::MEMPOOL)
                    .summary("Recent mempool transactions")
                    .description("Get the last 10 transactions to enter the mempool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*")
                    .json_response::<Vec<MempoolRecentTx>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/replacements",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_bound(headers, Version::ONE, |q| {
                        q.recent_replacements_json(false)
                    })
                    .await
            },
            |op| {
                op.id("get_replacements")
                    .tag(tags::MEMPOOL)
                    .summary("Recent RBF replacements")
                    .description("Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*")
                    .json_response::<Vec<ReplacementNode>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/fullrbf/replacements",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_bound(headers, Version::ONE, |q| {
                        q.recent_replacements_json(true)
                    })
                    .await
            },
            |op| {
                op.id("get_fullrbf_replacements")
                    .tag(tags::MEMPOOL)
                    .summary("Recent full-RBF replacements")
                    .description("Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*")
                    .json_response::<Vec<ReplacementNode>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/mempool-blocks/{index}",
        get_with(
            async |headers: HeaderMap,
                   Path(path): Path<MempoolBlockParam>,
                   _: Empty,
                   RequestState(state): RequestState| -> Result<Response> {
                mempool_block::serve(state, headers, path.index).await
            },
            |op| {
                op.id("get_mempool_block")
                    .tag(tags::MEMPOOL)
                    .mcp_ignore()
                    .summary("Projected mempool block")
                    .description("A projected mempool block with its transactions in order and its stats, by position as `GET /api/v1/fees/mempool-blocks` lists them: 0 is the next block, Bitcoin Core's `getblocktemplate` selection, and the only one served for now (others answer 404). The returned `hash` is an opaque content token: poll `GET /api/mempool-blocks/{index}/diff/{hash}` to follow changes instead of refetching the block.")
                    .json_response::<MempoolBlockContent>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/mempool-blocks/{index}/diff/{since}",
        get_with(
            async |headers: HeaderMap,
                   Path(path): Path<MempoolBlockDiffParam>,
                   _: Empty,
                   RequestState(state): RequestState| -> Result<Response> {
                mempool_block::serve_diff(state, headers, path.index, path.since).await
            },
            |op| {
                op.id("get_mempool_block_diff")
                    .tag(tags::MEMPOOL)
                    .mcp_ignore()
                    .summary("Projected mempool block changes")
                    .description("The projected mempool block's changes since the client's copy at `since`. `order` is the new block in order: each entry is either `[start, count]`, the client's transactions `start..start + count` kept in order, or a transaction object, a body to place there. Walk `order` once to rebuild, and replace the cached `stats` with the response's; `removed` lists the txids that left, so clients can evict cached bodies. Use the response `hash` as the next `since`; with `If-None-Match` an unchanged block answers 304. Returns `404` when `since` has aged out of server history: fetch `GET /api/mempool-blocks/{index}` again.")
                    .json_response::<MempoolBlockDiff>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/mempool/price",
        get_with(
            serve_live_price,
            |op| {
                op.id("get_live_price")
                    .tag(tags::MEMPOOL)
                    .mcp_ignore()
                    .summary("Live BTC/USD price")
                    .description(
                        "Current BTC/USD price in dollars, derived from on-chain round-dollar \
                        output patterns in the oracle's recent-block window, with the mempool's \
                        outputs blended in when available.",
                    )
                    .json_response::<Dollars>()
                    .not_modified()
            },
        ),
    )
}
