use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_primitives::BlockHashPrefix;
use bitview_query::{Query, Result as QueryResult, UtxoSetPoint};
use bitview_types::{HeightOrDate, UtxoSet, UtxoSetDiff};
use brk_types::Version;

use super::AppState;
use crate::{
    answer::{Answer, Content, JSON},
    api::tags,
    extended::TransformResponseExtended,
    params::{Empty, HeightOrDateParam},
    request_state::RequestState,
};

/// Bumped when a response's format changes.
const VERSION: Version = Version::ONE;

impl AppState {
    /// The set at a point, named by the point's block (`a{version}-{block hash prefix:x}`)
    /// before any of it is built.
    async fn respond_utxo_set(
        &self,
        headers: &HeaderMap,
        point: Option<HeightOrDate>,
        build: fn(&Query, &UtxoSetPoint) -> QueryResult<Vec<u8>>,
    ) -> Response {
        self.respond(headers, self.utxo_set(), move |q| {
            let point = match point {
                None => q.utxo_set_point_latest()?,
                Some(HeightOrDate::Height(height)) => q.utxo_set_point_height(height)?,
                Some(HeightOrDate::Date(date)) => q.utxo_set_point_date(date)?,
            };
            let tag = format!("a{VERSION}-{:x}", *BlockHashPrefix::from(point.hash()));
            let settled = point.is_settled();
            let answer = Answer::new(tag, move |q| Ok(Content::new(JSON, build(q, &point)?)));
            Ok(answer.settled(settled))
        })
        .await
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/utxo-set",
        get_with(
            async |headers: HeaderMap,
                   _: Empty,
                   RequestState(state): RequestState| {
                state.respond_utxo_set(&headers, None, Query::utxo_set).await
            },
            |op| {
                op.id("get_utxo_set_latest")
                    .tag(tags::UTXO_SET)
                    .mcp_ignore()
                    .summary("Latest UTXO set")
                    .description(
                        "The UTXO set after the latest published block, in the layout of \
                        `GET /api/utxo-set/{point}`.",
                    )
                    .json_response::<UtxoSet>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/utxo-set/{point}",
        get_with(
            async |headers: HeaderMap,
                   Path(path): Path<HeightOrDateParam>,
                   _: Empty,
                   RequestState(state): RequestState| {
                let point = path.point;
                state
                    .respond_utxo_set(&headers, Some(point), Query::utxo_set)
                    .await
            },
            |op| {
                op.id("get_utxo_set")
                    .tag(tags::UTXO_SET)
                    .mcp_ignore()
                    .summary("UTXO set at block height or date")
                    .description(
                        "The UTXO set after a block (`840000`) or after the last published \
                        block of a UTC day (`YYYY-MM-DD`), by creation height. Returns \
                        `{ height, hash, date, count, supply, origins }`; entry `i` of \
                        `origins.count` and `origins.supply` (BTC) is what remains unspent of \
                        block `i`'s outputs.",
                    )
                    .json_response::<UtxoSet>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/utxo-set/{point}/diff",
        get_with(
            async |headers: HeaderMap,
                   Path(path): Path<HeightOrDateParam>,
                   _: Empty,
                   RequestState(state): RequestState| {
                let point = path.point;
                state
                    .respond_utxo_set(&headers, Some(point), Query::utxo_set_diff)
                    .await
            },
            |op| {
                op.id("get_utxo_set_diff")
                    .tag(tags::UTXO_SET)
                    .summary("UTXO set changes of a block or date")
                    .description(
                        "What a block (`840000`) or a UTC day's published blocks \
                        (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: \
                        `created` has one row per block, `spent` one row per creation height, \
                        each as columnar `height`, `count` and `supply` (BTC). The set \
                        after block `first - 1` plus `created` minus `spent` is the set after \
                        block `last`.",
                    )
                    .json_response::<UtxoSetDiff>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
}
