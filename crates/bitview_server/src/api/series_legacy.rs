//! Deprecated series-format infrastructure. Sunset date: 2027-01-01.
//!
//! Two responsibilities, deletable as a unit when the sunset arrives:
//! - `handler` / `SUNSET`: the shared legacy series handler used by `/api/series`
//!   in legacy mode (registered by metrics endpoints that emit the old format).
//! - `add_series_legacy_routes`: the deprecated `/api/series/cost-basis/*` URLs.

use std::collections::BTreeMap;

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};
use bitview_query::Query as BrkQuery;
use brk_error::Error;
use brk_types::{
    Bitcoin, Cents, Cohort, Date, Day1, Dollars, OutputLegacy, Sats, SeriesSelection,
    UrpdAggregation, Version,
};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::ReadableOptionVec;

use crate::{
    AppState, CacheStrategy,
    extended::{HeaderMapExtended, TransformResponseExtended},
    params::Empty,
};

pub const SUNSET: &str = "2027-01-01T00:00:00Z";

/// Legacy series handler. Emits the pre-2027 `OutputLegacy` format and tags
/// the response with `Deprecation` / `Sunset` headers. Reused by `metrics/*`
/// for endpoints that must stay on the old format until sunset.
pub async fn handler(
    uri: Uri,
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    State(state): State<AppState>,
) -> std::result::Result<Response, crate::Error> {
    let mut response = super::series::serve(state, uri, headers, params, legacy_bytes).await?;
    if response.status() == StatusCode::OK {
        response.headers_mut().insert_deprecation(SUNSET);
    }
    Ok(response)
}

fn legacy_bytes(q: &BrkQuery, r: bitview_query::ResolvedQuery) -> brk_error::Result<Bytes> {
    Ok(match q.format_legacy(r)?.output {
        OutputLegacy::CSV(s) => Bytes::from(s),
        OutputLegacy::Json(v) => Bytes::from(v.to_vec()),
    })
}

#[derive(Deserialize, JsonSchema)]
struct CostBasisParams {
    cohort: Cohort,
    #[schemars(with = "String", example = &"2024-01-01")]
    date: Date,
}

#[derive(Deserialize, JsonSchema)]
struct CostBasisCohortParam {
    cohort: Cohort,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CostBasisQuery {
    #[serde(default)]
    bucket: UrpdAggregation,
    #[serde(default)]
    value: CostBasisValue,
}

/// Value type for the deprecated cost-basis distribution output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum CostBasisValue {
    #[default]
    Supply,
    Realized,
    Unrealized,
}

fn cost_basis_formatted(
    q: &BrkQuery,
    cohort: &Cohort,
    date: Date,
    agg: UrpdAggregation,
    value: CostBasisValue,
) -> brk_error::Result<BTreeMap<Dollars, f64>> {
    let raw = q.urpd_raw(cohort, date)?;
    let day1 = Day1::try_from(date)?;
    let spot_cents = q
        .price()
        .split
        .close
        .cents
        .day1
        .collect_one_flat(day1)
        .ok_or_else(|| Error::NotFound(format!("No price data for {date}")))?;
    let spot = Dollars::from(spot_cents);
    let needs_realized = value == CostBasisValue::Realized;

    let mut bucketed: FxHashMap<Cents, (Sats, Dollars)> =
        FxHashMap::with_capacity_and_hasher(raw.map.len(), Default::default());
    for (&price_cents, &sats) in &raw.map {
        let price = Cents::from(price_cents);
        let key = agg.bucket_floor(price);
        let entry = bucketed.entry(key).or_insert((Sats::ZERO, Dollars::ZERO));
        entry.0 += sats;
        if needs_realized {
            entry.1 += Dollars::from(price) * sats;
        }
    }

    Ok(bucketed
        .into_iter()
        .map(|(cents, (sats, realized))| {
            let k = Dollars::from(cents);
            let v = match value {
                CostBasisValue::Supply => f64::from(Bitcoin::from(sats)),
                CostBasisValue::Realized => f64::from(realized),
                CostBasisValue::Unrealized => f64::from((spot - k) * sats),
            };
            (k, v)
        })
        .collect())
}

pub trait ApiSeriesLegacyRoutes {
    fn add_series_legacy_routes(self) -> Self;
}

impl ApiSeriesLegacyRoutes for ApiRouter<AppState> {
    fn add_series_legacy_routes(self) -> Self {
        self.api_route(
            "/api/series/cost-basis",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, CacheStrategy::Deploy, &uri, |q| q.urpd_cohorts())
                        .await
                },
                |op| {
                    op.id("get_cost_basis_cohorts")
                        .series_tag()
                        .deprecated()
                        .summary("Available cost basis cohorts (deprecated)")
                        .description(
                            "**DEPRECATED** - Use `GET /api/urpd` instead.\n\n\
                            Sunset date: 2027-01-01.",
                        )
                        .json_response::<Vec<Cohort>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/series/cost-basis/{cohort}/dates",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(params): Path<CostBasisCohortParam>,
                       _: Empty,
                       State(state): State<AppState>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.urpd_dates(&params.cohort)
                        })
                        .await
                },
                |op| {
                    op.id("get_cost_basis_dates")
                        .series_tag()
                        .deprecated()
                        .summary("Available cost basis dates (deprecated)")
                        .description(
                            "**DEPRECATED** - Use `GET /api/urpd/{cohort}/dates` instead.\n\n\
                            Sunset date: 2027-01-01.",
                        )
                        .json_response::<Vec<Date>>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/series/cost-basis/{cohort}/{date}",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(params): Path<CostBasisParams>,
                       Query(query): Query<CostBasisQuery>,
                       State(state): State<AppState>| {
                    let strategy = state.date_strategy(Version::ONE, params.date);
                    state
                        .respond_json(&headers, strategy, &uri, move |q| {
                            cost_basis_formatted(
                                q,
                                &params.cohort,
                                params.date,
                                query.bucket,
                                query.value,
                            )
                        })
                        .await
                },
                |op| {
                    op.id("get_cost_basis")
                        .series_tag()
                        .deprecated()
                        .summary("Cost basis distribution (deprecated)")
                        .description(
                            "**DEPRECATED** - Use `GET /api/urpd/{cohort}/{date}` instead. \
                            The new endpoint returns supply, realized cap, and unrealized P&L \
                            per bucket in one response.\n\n\
                            Sunset date: 2027-01-01.",
                        )
                        .json_response::<BTreeMap<Dollars, f64>>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
    }
}
