use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, Query},
    http::HeaderMap,
    response::Response,
};
use bitview_types::{HeightOrDate, Urpd, UrpdCohort};

use super::AppState;
use crate::{
    answer::{Answer, Content, JSON},
    api::tags,
    extended::TransformResponseExtended,
    params::{Empty, UrpdCohortParam, UrpdParams, UrpdQuery},
    request_state::RequestState,
};

async fn serve_urpd(
    state: AppState,
    headers: &HeaderMap,
    cohort: UrpdCohort,
    point: Option<HeightOrDate>,
    query: UrpdQuery,
) -> Response {
    state
        .respond(headers, state.urpd(), move |q| {
            let resolved = match point {
                Some(HeightOrDate::Date(date)) => {
                    q.resolve_urpd_at(cohort, date, query.aggregation, query.weight)?
                }
                Some(HeightOrDate::Height(height)) => {
                    q.resolve_urpd_height(cohort, height, query.aggregation, query.weight)?
                }
                None => q.resolve_urpd_latest(cohort, query.aggregation, query.weight)?,
            };
            // The selection alone identifies the body: a match answers 304 unreconstructed.
            let tag = format!("urpd3-{}", resolved.representation());
            let settled = resolved.is_settled();
            let answer = Answer::new(tag, move |q| {
                Ok(Content::new(JSON, q.urpd_resolved(resolved)?))
            });
            Ok(answer.settled(settled))
        })
        .await
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/urpd",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond(&headers, state.chain(), |q| {
                        Ok(Answer::built(Content::json(&q.urpd_cohorts()?)?))
                    })
                    .await
            },
            |op| {
                op.id("list_urpd_cohorts")
                    .tag(tags::URPD)
                    .summary("Available URPD cohorts")
                    .description(
                        "Every URPD cohort once a UTXO set is published (none before): the age aggregates \
                        (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest \
                        (`age_under_1h`, `age_1h_to_1d`, ...).",
                    )
                    .json_response::<Vec<UrpdCohort>>()
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
                   RequestState(state): RequestState| {
                serve_urpd(state, &headers, params.cohort, None, query).await
            },
            |op| {
                op.id("get_urpd_latest")
                    .tag(tags::URPD)
                    .summary("Latest URPD")
                    .description(
                        "URPD (UTXO realized price distribution: how much supply last moved at each price) for the latest published block; `date` echoes the date served. \
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
                   RequestState(state): RequestState| {
                serve_urpd(state, &headers, params.cohort, Some(params.point), query).await
            },
            |op| {
                op.id("get_urpd")
                    .tag(tags::URPD)
                    .summary("URPD at block height or date")
                    .description(
                        "URPD (UTXO realized price distribution: how much supply last moved at each price) for a cohort at a block height or the last block of a UTC day. Returns \
                        `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where \
                        `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry \
                        per bucket in ascending price: the same block header and columnar layout as \
                        `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and \
                        `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. \
                        Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation \
                        price on the price grid (four significant digits of the price in cents, \
                        within 0.05% of the exact price), rounded to the cent per bucket, so the \
                        buckets need not sum to the realized cap series. With a `wakefulness` or \
                        `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and \
                        the bucket values follow it.",
                    )
                    .json_response::<Urpd>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
}
