use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_primitives::BlockHashPrefix;
use bitview_query::{Query, Result as QueryResult, UtxoSetPoint};
use bitview_types::{UtxoSet, UtxoSetDiff};
use brk_types::Version;
use serde::Serialize;
use serde_json::to_vec;

use super::AppState;
use crate::{
    CacheParams, CacheStrategy,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::{Empty, HeightOrDate, HeightOrDateParam},
    raw_body::RawBodyPermit,
    request_state::RequestState,
};

/// Bumped when a response's format changes.
const VERSION: Version = Version::ONE;

impl AppState {
    /// The point resolves and validates first; the set is rebuilt only for a body.
    async fn respond_utxo_set<T: Serialize + 'static>(
        &self,
        headers: HeaderMap,
        point: Option<HeightOrDate>,
        build: fn(&Query, &UtxoSetPoint) -> QueryResult<T>,
    ) -> Result<Response> {
        let bodies = self.utxo_set_bodies.clone();
        let mode = self.cdn_cache_mode;
        Ok(self
            .read_body(
                &self.utxo_set_query,
                &self.utxo_set_bodies,
                move |q, permit| {
                    let point = match point {
                        None => q.utxo_set_point_latest()?,
                        Some(HeightOrDate::Height(height)) => q.utxo_set_point_height(height)?,
                        Some(HeightOrDate::Date(date)) => q.utxo_set_point_date(date)?,
                    };
                    let strategy =
                        CacheStrategy::ActivityBound(VERSION, BlockHashPrefix::from(&point.hash));
                    let params = CacheParams::resolve(&strategy, mode);
                    if params.matches_etag(&headers) {
                        return Ok(Some(Response::new_not_modified(&params)));
                    }
                    let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&bodies))
                    else {
                        return Ok(None);
                    };
                    let bytes = to_vec(&build(q, &point)?)?.into();
                    Ok(Some(permit.response(
                        params,
                        bytes,
                        HeaderMapExtended::insert_content_type_application_json,
                    )))
                },
            )
            .await?)
    }
}

pub trait UtxoSetRoutes {
    fn add_utxo_set_routes(self) -> Self;
}

impl UtxoSetRoutes for ApiRouter<AppState> {
    fn add_utxo_set_routes(self) -> Self {
        self.api_route(
            "/api/utxo-set",
            get_with(
                async |headers: HeaderMap,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    state.respond_utxo_set(headers, None, Query::utxo_set).await
                },
                |op| {
                    op.id("get_utxo_set_latest")
                        .utxo_set_tag()
                        .mcp_ignore()
                        .summary("Latest UTXO set")
                        .description(
                            "The UTXO set after the latest published block, by creation height. \
                            Returns `{ height, hash, date, count, supply, origins }`; entry `i` of \
                            `origins.count` and `origins.supply` (BTC) is what remains unspent of \
                            block `i`'s outputs.",
                        )
                        .json_response::<UtxoSet>()
                        .not_modified()
                        .not_found()
                        .server_errors()
                },
            ),
        )
        .api_route(
            "/api/utxo-set/{point}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let point = path.resolve()?;
                    state
                        .respond_utxo_set(headers, Some(point), Query::utxo_set)
                        .await
                },
                |op| {
                    op.id("get_utxo_set")
                        .utxo_set_tag()
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
                        .bad_request()
                        .not_found()
                        .server_errors()
                },
            ),
        )
        .api_route(
            "/api/utxo-set/{point}/diff",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let point = path.resolve()?;
                    state
                        .respond_utxo_set(headers, Some(point), Query::utxo_set_diff)
                        .await
                },
                |op| {
                    op.id("get_utxo_set_diff")
                        .utxo_set_tag()
                        .mcp_ignore()
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
                        .bad_request()
                        .not_found()
                        .server_errors()
                },
            ),
        )
    }
}
