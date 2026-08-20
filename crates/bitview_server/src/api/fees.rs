use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::State, http::HeaderMap};
use brk_types::{MempoolBlock, RecommendedFees};

use crate::{AppState, extended::TransformResponseExtended, params::Empty};

pub trait FeesRoutes {
    fn add_fees_routes(self) -> Self;
}

impl FeesRoutes for ApiRouter<AppState> {
    fn add_fees_routes(self) -> Self {
        self.api_route(
            "/api/v1/fees/mempool-blocks",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), |q| {
                            q.mempool_blocks()
                        })
                        .await
                },
                |op| {
                    op.id("get_mempool_blocks")
                        .fees_tag()
                        .summary("Projected mempool blocks")
                        .description("Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*")
                        .json_response::<Vec<MempoolBlock>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/fees/recommended",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), |q| {
                            q.recommended_fees()
                        })
                        .await
                },
                |op| {
                    op.id("get_recommended_fees")
                        .fees_tag()
                        .mcp_ignore()
                        .summary("Recommended fees")
                        .description("Recommended fee rates by confirmation target.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*")
                        .json_response::<RecommendedFees>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/fees/precise",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), |q| {
                            q.recommended_fees()
                        })
                        .await
                },
                |op| {
                    op.id("get_precise_fees")
                        .fees_tag()
                        .summary("Recommended fee rates (precise)")
                        .description("Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*")
                        .json_response::<RecommendedFees>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
    }
}
