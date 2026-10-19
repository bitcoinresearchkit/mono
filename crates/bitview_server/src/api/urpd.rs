use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, Query},
    http::HeaderMap,
    response::Response,
};
use bitview_types::{Cohort, Urpd};
use serde_json::to_vec;

use super::AppState;
use crate::{
    CacheParams,
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::{
        Empty, HeightOrDate, HeightOrDateParam, UrpdCohortParam, UrpdParams, UrpdQuery,
    },
    request_state::RequestState,
};

async fn serve_urpd(
    state: AppState,
    headers: HeaderMap,
    cohort: Cohort,
    point: Option<HeightOrDate>,
    query: UrpdQuery,
) -> Result<Response> {
    let cdn_cache_mode = state.cdn_cache_mode;
    Ok(state
        .respond_admitted(
            &state.urpd_query,
            &state.urpd_bodies,
            headers,
            "application/json",
            move |q| {
                let resolved = match point {
                    Some(HeightOrDate::Date(date)) => {
                        q.resolve_urpd_at(&cohort, date, query.aggregation, query.weight)?
                    }
                    Some(HeightOrDate::Height(height)) => {
                        q.resolve_urpd_height(&cohort, height, query.aggregation, query.weight)?
                    }
                    None => q.resolve_urpd_latest(&cohort, query.aggregation, query.weight)?,
                };
                // The selection alone identifies the body: a match answers 304 unreconstructed.
                let params = CacheParams::live(format!("urpd3-{}", resolved.representation()))
                    .stable_if(resolved.is_settled(), cdn_cache_mode);
                Ok((resolved, params))
            },
            |q, resolved| Ok(to_vec(&q.urpd_resolved(resolved)?)?.into()),
        )
        .await?)
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/urpd",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_content(&headers, |query| query.urpd_cohorts())
                    .await
            },
            |op| {
                op.id("list_urpd_cohorts")
                    .tag(tags::URPD)
                    .summary("Available URPD cohorts")
                    .description(
                        "Cohorts for which URPD data is available. Returns names like \
                        `all`, `sth`, `lth`, `under_4m`, `under_6m`, `over_4m`, `over_6m`, `utxos_under_1h_old`.",
                    )
                    .json_response::<Vec<Cohort>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/urpd/{cohort}",
        get_with(
            async |headers: HeaderMap,
                   Path(params): Path<UrpdCohortParam>,
                   Query(query): Query<UrpdQuery>,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                serve_urpd(state, headers, params.cohort, None, query).await
            },
            |op| {
                op.id("get_urpd")
                    .tag(tags::URPD)
                    .summary("Latest URPD")
                    .description(
                        "URPD for the latest published block; `date` echoes the date served. \
                        Fields, units, price-grid rounding and weighting as in \
                        `GET /api/urpd/{cohort}/{point}`.",
                    )
                    .json_response::<Urpd>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/urpd/{cohort}/{point}",
        get_with(
            async |headers: HeaderMap,
                   Path(params): Path<UrpdParams>,
                   Query(query): Query<UrpdQuery>,
                   RequestState(state): RequestState|
                   -> Result<Response> {
                serve_urpd(state, headers, params.cohort, Some(HeightOrDateParam { point: params.point }.resolve()?), query).await
            },
            |op| {
                op.id("get_urpd_at")
                    .tag(tags::URPD)
                    .summary("URPD at block height or date")
                    .description(
                        "URPD for a cohort at a block height or the last block of a UTC day. Returns \
                        `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }` where \
                        each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. \
                        `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; \
                        `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative. \
                        Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation \
                        price on the price grid (four significant digits of the price in cents, \
                        within 0.05% of the exact price), rounded to the cent per bucket, so the \
                        buckets need not sum to the realized cap series. With a `cointime` or \
                        `coinflow` `weight`, `supply` is the weighted supply (whole satoshis) and \
                        the bucket values follow it.",
                    )
                    .json_response::<Urpd>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
}
