use crate::request_state::RequestState;
use aide::axum::{ApiRouter, routing::get_with};
use axum::{http::HeaderMap, response::Response};
use brk_types::{MempoolBlock, PreciseRecommendedFees, RecommendedFees};

use crate::{
    AppState, api::tags, error::Result, extended::TransformResponseExtended, params::Empty,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/fees/mempool-blocks",
        get_with(
            async |headers: HeaderMap,
                   _: Empty,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                let blocks = state.query.read(|q| q.mempool_blocks()).await?;
                Ok(state.respond_json_content_value(&headers, blocks))
            },
            |op| {
                op.id("get_mempool_blocks")
                    .tag(tags::FEES)
                    .summary("Projected mempool blocks")
                    .description("Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*")
                    .json_response::<Vec<MempoolBlock>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/fees/recommended",
        get_with(
            async |headers: HeaderMap,
                   _: Empty,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                let fees = state.query.read(|q| q.recommended_fees()).await?;
                Ok(state.respond_json_content_value(&headers, fees))
            },
            |op| {
                op.id("get_recommended_fees")
                    .tag(tags::FEES)
                    .mcp_ignore()
                    .summary("Recommended fees")
                    .description("Recommended fee rates by confirmation target, in whole sat/vB.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*")
                    .json_response::<RecommendedFees>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/fees/precise",
        get_with(
            async |headers: HeaderMap,
                   _: Empty,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                let fees = state.query.read(|q| q.precise_fees()).await?;
                Ok(state.respond_json_content_value(&headers, fees))
            },
            |op| {
                op.id("get_precise_fees")
                    .tag(tags::FEES)
                    .summary("Recommended fee rates (precise)")
                    .description("Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*")
                    .json_response::<PreciseRecommendedFees>()
                    .not_modified()
            },
        ),
    )
}
