use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Uri},
};
use brk_types::{
    BlockTemplate, BlockTemplateDiff, Dollars, MempoolInfo, MempoolRecentTx, NextBlockHash,
    ReplacementNode, Txid,
};

use crate::{
    AppState,
    extended::TransformResponseExtended,
    params::{Empty, NextBlockHashParam},
};

pub trait MempoolRoutes {
    fn add_mempool_routes(self) -> Self;
}

impl MempoolRoutes for ApiRouter<AppState> {
    fn add_mempool_routes(self) -> Self {
        self.api_route(
            "/api/mempool",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.mempool_info())
                        .await
                },
                |op| {
                    op.id("get_mempool")
                        .mempool_tag()
                        .summary("Mempool statistics")
                        .description("Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*")
                        .json_response::<MempoolInfo>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/mempool/hash",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.mempool_hash())
                        .await
                },
                |op| {
                    op.id("get_mempool_hash")
                        .mempool_tag()
                        .summary("Mempool content hash")
                        .description("Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.")
                        .json_response::<NextBlockHash>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/mempool/txids",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.mempool_txids())
                        .await
                },
                |op| {
                    op.id("get_mempool_txids")
                        .mempool_tag()
                        .mcp_ignore()
                        .summary("Mempool transaction IDs")
                        .description("Get all transaction IDs currently in the mempool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*")
                        .json_response::<Vec<Txid>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/mempool/recent",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.mempool_recent())
                        .await
                },
                |op| {
                    op.id("get_mempool_recent")
                        .mempool_tag()
                        .summary("Recent mempool transactions")
                        .description("Get the last 10 transactions to enter the mempool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*")
                        .json_response::<Vec<MempoolRecentTx>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/replacements",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| {
                            q.recent_replacements(false)
                        })
                        .await
                },
                |op| {
                    op.id("get_replacements")
                        .mempool_tag()
                        .summary("Recent RBF replacements")
                        .description("Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*")
                        .json_response::<Vec<ReplacementNode>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/fullrbf/replacements",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| {
                            q.recent_replacements(true)
                        })
                        .await
                },
                |op| {
                    op.id("get_fullrbf_replacements")
                        .mempool_tag()
                        .summary("Recent full-RBF replacements")
                        .description("Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*")
                        .json_response::<Vec<ReplacementNode>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mempool/block-template",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| {
                            q.block_template()
                        })
                        .await
                },
                |op| {
                    op.id("get_block_template")
                        .mempool_tag()
                        .mcp_ignore()
                        .summary("Projected next block template")
                        .description("Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.")
                        .json_response::<BlockTemplate>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mempool/block-template/diff/{hash}",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(path): Path<NextBlockHashParam>,
                       _: Empty,
                       State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, move |q| {
                            q.block_template_diff(path.hash)
                        })
                        .await
                },
                |op| {
                    op.id("get_block_template_diff")
                        .mempool_tag()
                        .mcp_ignore()
                        .summary("Block template diff since hash")
                        .description("Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.")
                        .json_response::<BlockTemplateDiff>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/mempool/price",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.live_price())
                        .await
                },
                |op| {
                    op.id("get_live_price")
                        .mempool_tag()
                        .mcp_ignore()
                        .summary("Live BTC/USD price")
                        .description(
                            "Returns the current BTC/USD price in dollars, derived from \
                            on-chain round-dollar output patterns in the last 12 blocks \
                            plus mempool.",
                        )
                        .json_response::<Dollars>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
    }
}
