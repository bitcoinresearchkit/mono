use aide::axum::{ApiRouter, routing::get_with};
use axum::http::HeaderMap;
use brk_types::{ChainTip, Version};

use crate::{
    AppState, api::tags, extended::TransformResponseExtended, params::Empty,
    request_state::RequestState,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/chain-tips",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_bound(headers, Version::ONE, |q| q.chain_tips_json())
                    .await
            },
            |op| {
                op.id("get_chain_tips")
                    .tag(tags::BLOCKS)
                    .summary("Known chain tips")
                    .description("All tips known to this server's Bitcoin Core node, including the active tip and every fork status. Returns the last complete background observation in Core's order (descending height; tie order unspecified). Refreshed at startup and after indexed chain-tip changes; requests never call Core. Before the first successful observation, returns 503. No query parameters.")
                    .json_response::<Vec<ChainTip>>()
                    .not_modified()
            },
        ),
    )
}
