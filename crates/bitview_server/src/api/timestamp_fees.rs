use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Query, http::HeaderMap, response::IntoResponse};
use bitview_types::BlockFeesEntry;

use super::tags;
use crate::{
    AppState, Error,
    extended::TransformResponseExtended,
    params::TimestampRangeQuery,
    request_state::RequestState,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/mining/blocks/fees",
        get_with(
            async |headers: HeaderMap, Query(params): Query<TimestampRangeQuery>, RequestState(state): RequestState| {
                // Check even on a matching If-None-Match, before history can short-circuit.
                let range = match params.range() {
                    Ok(range) => range,
                    Err(error) => return Error::from(error).into_response(),
                };
                let key = format!("mining-fees-{}-{}", *range.from(), *range.to());
                state.respond_json_history_keyed(headers, key, move |query| {
                    query.block_fees_range(range)
                }).await
            },
            |op| {
                op.id("get_block_fees_by_timestamp")
                    .tag(tags::MINING)
                    .summary("Block fees between inclusive timestamps")
                    .description("Canonical nonnegative integer from/to seconds are required, from <= to. Selects raw header timestamps inclusively and groups by the requested span. Fees, height and time use round-half-up means. USD is the mean on-chain oracle spot price, null if any selected price is unavailable; it is not upstream's external SQL price selection.")
                    .json_response::<Vec<BlockFeesEntry>>()
                    .not_modified()
            },
        ),
    )
}
