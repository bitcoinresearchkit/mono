use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use brk_types::{Cohort, Date, Urpd, Version};

use crate::{
    CacheStrategy,
    error::RouteResult,
    extended::TransformResponseExtended,
    params::{Empty, UrpdCohortParam, UrpdParams, UrpdQuery, UrpdWeightQuery},
};

use super::AppState;

pub(super) fn serve_cohorts(state: AppState, headers: HeaderMap) -> Response {
    state.respond_json_bytes_value(&headers, CacheStrategy::Deploy, &state.urpd_cohorts_body)
}

pub trait ApiUrpdRoutes {
    fn add_urpd_routes(self) -> Self;
}

impl ApiUrpdRoutes for ApiRouter<AppState> {
    fn add_urpd_routes(self) -> Self {
        self.api_route(
            "/api/urpd",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    serve_cohorts(state, headers)
                },
                |op| {
                    op.id("list_urpd_cohorts")
                        .urpd_tag()
                        .summary("Available URPD cohorts")
                        .description(
                            "Cohorts for which URPD data is available. Returns names like \
                            `all`, `sth`, `lth`, `utxos_under_1h_old`.",
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
                       State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.tip_strategy(), move |q| {
                            q.urpd_dates_with_weight(&params.cohort, query.weight)
                        })
                        .await
                },
                |op| {
                    op.id("list_urpd_dates")
                        .urpd_tag()
                        .summary("Available URPD dates")
                        .description(
                            "Dates for which a URPD snapshot is available for the cohort and \
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
                       State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.tip_strategy(), move |q| {
                            q.urpd_latest_with_weight(
                                &params.cohort,
                                query.aggregation,
                                query.weight,
                            )
                        })
                        .await
                },
                |op| {
                    op.id("get_urpd")
                        .urpd_tag()
                        .summary("Latest URPD")
                        .description(
                            "URPD for the most recent available date in the cohort. \
                            The response's `date` field echoes which date was served. Returns \
                            `{ cohort, date, weight, aggregation, close, total_supply, buckets }`. \
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
            "/api/urpd/{cohort}/{date}",
            get_with(
                async |headers: HeaderMap,
                       Path(params): Path<UrpdParams>,
                       Query(query): Query<UrpdQuery>,
                       State(state): State<AppState>|
                       -> RouteResult<Response> {
                    let strategy = state.date_strategy(Version::TWO, params.date).await?;
                    Ok(state
                        .respond_json(&headers, strategy, move |q| {
                            q.urpd_at_with_weight(
                                &params.cohort,
                                params.date,
                                query.aggregation,
                                query.weight,
                            )
                        })
                        .await)
                },
                |op| {
                    op.id("get_urpd_at")
                        .urpd_tag()
                        .summary("URPD at date")
                        .description(
                            "URPD for a (cohort, date) pair. Returns \
                            `{ cohort, date, weight, aggregation, close, total_supply, buckets }` where \
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
