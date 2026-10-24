use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_types::{OraclePayments, OracleSearch};
use brk_types::Dollars;

use crate::{
    AppState,
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, HeightParam},
    request_state::RequestState,
};

async fn serve_live_price(
    headers: HeaderMap,
    _: Empty,
    RequestState(state): RequestState,
) -> Response {
    state
        .respond_json_content(&headers, |query| query.live_price())
        .await
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router
        .api_route(
            "/api/oracle/price",
            get_with(serve_live_price, |op| {
                op.id("get_live_price")
                    .tag(tags::ORACLE)
                    .mcp_ignore()
                    .summary("Live BTC/USD price")
                    .description(
                        "Current BTC/USD price in dollars, derived from on-chain round-dollar \
                        output patterns in the oracle's recent-block window, with the mempool's \
                        outputs blended in when available. Confirmed per-height history is \
                        available at `GET /api/series/price/height`.",
                    )
                    .json_response::<Dollars>()
                    .not_modified()
            }),
        )
        .api_route(
            "/api/oracle/searches/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let height = usize::from(path.height);
                    Ok(state
                        .respond_json_settled_on(&state.oracle_replays, &headers, move |q| {
                            q.oracle_search(height)
                        })
                        .await)
                },
                |op| {
                    op.id("get_oracle_search")
                        .tag(tags::ORACLE)
                        .mcp_ignore()
                        .summary("The oracle's search at a block")
                        .description(
                            "The oracle's search behind a block's price, after height 508,000: every candidate \
                        bin in its window, 12 below the block before's answer to 11 above, with its score (each \
                        round-dollar arm's count over that arm's peak in the window, summed), the bin it picked, \
                        and the smoothed histogram it scored them on.",
                        )
                        .json_response::<OracleSearch>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/oracle/payments/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let start = usize::from(path.height);
                    Ok(state
                        .respond_json_settled_on(&state.oracle_replays, &headers, move |q| {
                            q.oracle_payments_chunk(start)
                        })
                        .await)
                },
                |op| {
                    op.id("get_oracle_payments")
                        .tag(tags::ORACLE)
                        .mcp_ignore()
                        .summary("The oracle's payments, a day of blocks at a time")
                        .description(
                            "For each of the 144 blocks from `height` (a multiple of 144, from 508,032): its payments as the oracle filters them, each read \
                        as each of its 19 round-dollar amounts ($1 to $10,000) at its exact amount in sats, \
                        counted by the price each reading implies, in buckets a tenth of an oracle bin wide \
                        (0.115% of price) over the block's search window. The chunk at the tip holds the blocks \
                        so far; a complete, settled chunk never changes.",
                        )
                        .json_response::<OraclePayments>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/oracle/searches/live",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |query| query.live_search())
                        .await
                },
                |op| {
                    op.id("get_oracle_search_live")
                        .tag(tags::ORACLE)
                        .mcp_ignore()
                        .summary("The oracle's live search")
                        .description(
                            "With the mempool's eligible outputs blended in as the next block, the oracle's \
                        search for it from the tip's answer, as in `GET /api/oracle/searches/{height}`: its height is \
                        the next block's, its pick the live price. Without a mempool publication at the tip, \
                        the tip block's own search.",
                        )
                        .json_response::<OracleSearch>()
                        .not_modified()
                },
            ),
        )
}
