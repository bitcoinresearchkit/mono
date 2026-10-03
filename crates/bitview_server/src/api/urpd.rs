use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, Query},
    http::HeaderMap,
    response::Response,
};
use bitview_primitives::Date;
use bitview_types::{Cohort, Urpd};
use serde_json::to_vec;

use super::AppState;
use crate::{
    CacheParams, CacheStrategy, CdnCacheMode,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::{
        Empty, HeightOrDate, HeightOrDateParam, UrpdCohortParam, UrpdParams, UrpdQuery,
        UrpdWeightQuery,
    },
    raw_body::RawBodyPermit,
    request_state::RequestState,
    urpd_input,
};

pub async fn serve_cohorts(state: AppState, headers: HeaderMap) -> Response {
    state
        .respond_json_content(&headers, |query| query.urpd_cohorts())
        .await
}

async fn serve_urpd(
    state: AppState,
    headers: HeaderMap,
    cohort: Cohort,
    point: Option<HeightOrDate>,
    query: UrpdQuery,
) -> Result<Response> {
    let bodies = state.urpd_bodies.clone();
    let response = state
        .read_body(&state.urpd_query, &state.urpd_bodies, move |q, permit| {
            let input = match point {
                Some(HeightOrDate::Date(date)) => {
                    q.resolve_urpd_at(&cohort, date, query.aggregation, query.weight)?
                }
                Some(HeightOrDate::Height(height)) => {
                    q.resolve_urpd_height(&cohort, height, query.aggregation, query.weight)?
                }
                None => q.resolve_urpd_latest(&cohort, query.aggregation, query.weight)?,
            };
            let id = urpd_input::identity(&input)?;
            let params = CacheParams::resolve(
                &CacheStrategy::Live(format!("urpd2-{id}").into()),
                CdnCacheMode::Live,
            );
            if params.matches_etag(&headers) {
                return Ok(Some(Response::new_not_modified(&params)));
            }
            let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&bodies)) else {
                return Ok(None);
            };
            let bytes = to_vec(&input.build()?)?.into();
            Ok(Some(permit.response(
                params,
                bytes,
                HeaderMapExtended::insert_content_type_application_json,
            )))
        })
        .await?;
    Ok(response)
}

pub trait ApiUrpdRoutes {
    fn add_urpd_routes(self) -> Self;
}

impl ApiUrpdRoutes for ApiRouter<AppState> {
    fn add_urpd_routes(self) -> Self {
        self.api_route(
            "/api/urpd",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    serve_cohorts(state, headers).await
                },
                |op| {
                    op.id("list_urpd_cohorts")
                        .urpd_tag()
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
            "/api/urpd/{cohort}/dates",
            get_with(
                async |headers: HeaderMap,
                       Path(params): Path<UrpdCohortParam>,
                       Query(query): Query<UrpdWeightQuery>,
                       RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, move |q| {
                            q.urpd_dates_with_weight(&params.cohort, query.weight)
                        })
                        .await
                },
                |op| {
                    op.id("list_urpd_dates")
                        .urpd_tag()
                        .summary("Available URPD dates")
                        .description(
                            "Dates for which a published block is available for the cohort and \
                            selected `weight`. One entry per UTC day, sorted ascending.",
                        )
                        .json_response::<Vec<Date>>()
                        .not_modified()
                        .not_found()
                        .server_error()
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
                        .urpd_tag()
                        .summary("Latest URPD")
                        .description(
                            "URPD for the latest published block. \
                            The response's `date` field echoes which date was served. Returns \
                            `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }`. \
                            `close` and each bucket's `price_floor`, `realized_cap`, and \
                            `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. \
                            `unrealized_pnl` can be negative.",
                        )
                        .json_response::<Urpd>()
                        .not_modified()
                        .not_found()
                        .server_error()
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
                        .urpd_tag()
                        .summary("URPD at block height or date")
                        .description(
                            "URPD for a cohort at a block height or the last block of a UTC day. Returns \
                            `{ cohort, height, date, weight, aggregation, close, total_supply, buckets }` where \
                            each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. \
                            `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; \
                            `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.",
                        )
                        .json_response::<Urpd>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
    }
}
