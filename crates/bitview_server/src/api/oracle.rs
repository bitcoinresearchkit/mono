use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_types::{OracleBlock, OraclePayments};
use brk_types::Dollars;

use crate::{
    AppState,
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, HeightParam, StartParam},
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
                        outputs blended in when available, to the cent: `price` of `GET /api/oracle/live`. \
                        Confirmed per-height history is available at `GET /api/series/price/height`. Not \
                        found before the oracle's first prices.",
                    )
                    .json_response::<Dollars>()
                    .not_modified()
                    .not_found()
            }),
        )
        .api_route(
            "/api/oracle/live",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |query| query.live_block())
                        .await
                },
                |op| {
                    op.id("get_oracle_live")
                        .tag(tags::ORACLE)
                        .mcp_ignore()
                        .summary("The oracle now")
                        .description(
                            "The oracle's reading now, as `GET /api/oracle/blocks/{height}` reads a block: with \
                        the mempool's outputs blended in as the next block (`mempool` true, `height` the next \
                        block's, `outputs` the mempool's), its search from the tip's answer, `price` the live \
                        price; without a mempool at the tip, the tip block's own. One snapshot: its parts never \
                        from two moments, `hash` the confirmed tip it was read at. Not found before the oracle's \
                        first prices, nor before height 508,000.",
                        )
                        .json_response::<OracleBlock>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/oracle/blocks/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let height = usize::from(path.height);
                    Ok(state
                        .respond_json_settled_on(&state.oracle_replays, &headers, move |q| {
                            q.oracle_block(height)
                        })
                        .await)
                },
                |op| {
                    op.id("get_oracle_block")
                        .tag(tags::ORACLE)
                        .mcp_ignore()
                        .summary("The oracle at a block")
                        .description(
                            "How the oracle read a confirmed block, after height 508,000: what it counted (the \
                        smoothed histogram of the payments of the block and the 11 before it, by amount on its \
                        log scale; the block's outputs as its filter judged them, each skipped one under the \
                        first rule that skips it), where it looked (every candidate bin in its window, 12 below \
                        the block before's answer to 11 above, with its score) and what it found (the bin it \
                        picked and its price). Bins: 200 a decade, a bin `round(log10(sats) * 200)`; as where \
                        $100 lands, a bin is the price `10^(10 - bin / 200)` dollars. A settled block's never \
                        changes.",
                        )
                        .json_response::<OracleBlock>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/oracle/payments/{start}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<StartParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let start = usize::from(path.start);
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
                        .summary("The oracle's payments, 144 blocks at a time")
                        .description(
                            "For each of the 144 blocks from `start` (a multiple of 144, else a bad request; from \
                        508,032, earlier ones out of range): its payments as the oracle filters them, each read as each of its 19 \
                        round-dollar amounts ($1 to $10,000) at its exact amount in sats, counted by the price \
                        each reading implies, in buckets a tenth of a bin wide (0.115% of price) over the \
                        block's search window: finer than the oracle's bins, the evidence behind its prices. \
                        The chunk at the tip holds its blocks so far, `hash` its last block's; a whole, settled \
                        chunk never changes.",
                        )
                        .json_response::<OraclePayments>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
}
