use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_primitives::Day1;
use brk_oracle::{HistogramEmaCompact, HistogramRaw};
use brk_types::Dollars;

use crate::{
    AppState,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, HeightOrDate, HeightOrDateParam},
    request_state::RequestState,
};

pub async fn serve_live_price(
    headers: HeaderMap,
    _: Empty,
    RequestState(state): RequestState,
) -> Response {
    state
        .respond_json_content(&headers, |query| query.live_price())
        .await
}

pub trait OracleRoutes {
    fn add_oracle_routes(self) -> Self;
}

impl OracleRoutes for ApiRouter<AppState> {
    fn add_oracle_routes(self) -> Self {
        self.api_route(
            "/api/oracle/price",
            get_with(serve_live_price, |op| {
                op.id("get_oracle_price")
                    .oracle_tag()
                    .mcp_ignore()
                    .summary("Live BTC/USD price")
                    .description(
                        "Current BTC/USD price in dollars. Same value as \
                            `GET /api/mempool/price`. Confirmed per-height history is available at \
                            `GET /api/series/price/height`.",
                    )
                    .json_response::<Dollars>()
                    .not_modified()
                    .server_error()
            }),
        )
        .api_route(
            "/api/oracle/histogram/payments/live",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |query| query.live_payment_histogram())
                        .await
                },
                |op| {
                    op.id("get_oracle_histogram_payments_live")
                        .oracle_tag()
                        .summary("Live payment output histogram")
                        .description(
                            "Live smoothed histogram of oracle-eligible payment outputs, binned \
                            by output value on the oracle log scale. It combines the committed \
                            oracle window with the complete mempool's eligible outputs from \
                            a matching chain publication. A flat array of \
                            log-scale bins.",
                        )
                        .json_response::<HistogramEmaCompact>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/oracle/histogram/payments/{point}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let point = path.resolve()?;
                    Ok(state
                        .respond_json_content(&headers, move |q| match point {
                            HeightOrDate::Date(date) => {
                                q.confirmed_payment_histogram_day(Day1::try_from(date)?)
                            }
                            HeightOrDate::Height(height) => {
                                q.confirmed_payment_histogram(usize::from(height))
                            }
                        })
                        .await)
                },
                |op| {
                    op.id("get_oracle_histogram_payments")
                        .oracle_tag()
                        .summary("Payment output histogram at height or day")
                        .description(
                            "Smoothed histogram of oracle-eligible payment outputs for a \
                            confirmed point. A block height (`840000`) gives that block's oracle \
                            payment histogram; a calendar date (`YYYY-MM-DD`) gives the average \
                            of that day's per-block payment histograms. A flat array of log-scale \
                            bins.",
                        )
                        .json_response::<HistogramEmaCompact>()
                        .not_modified()
                        .bad_request()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/oracle/histogram/outputs/live",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_json_content(&headers, |query| query.live_output_histogram())
                        .await
                },
                |op| {
                    op.id("get_oracle_histogram_outputs_live")
                        .oracle_tag()
                        .summary("Live output value histogram")
                        .description(
                            "Live unfiltered output value histogram for the complete published \
                            mempool. Every live output is binned by value on the oracle log scale; \
                            no oracle payment filters are applied. A flat array of log-scale \
                            bins, all zero when no mempool is configured.",
                        )
                        .json_response::<HistogramRaw>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/oracle/histogram/outputs/{point}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       RequestState(state): RequestState|
                       -> Result<Response> {
                    let point = path.resolve()?;
                    Ok(state
                        .respond_json_content(&headers, move |q| match point {
                            HeightOrDate::Date(date) => {
                                q.confirmed_output_histogram_day(Day1::try_from(date)?)
                            }
                            HeightOrDate::Height(height) => {
                                q.confirmed_output_histogram(usize::from(height))
                            }
                        })
                        .await)
                },
                |op| {
                    op.id("get_oracle_histogram_outputs")
                        .oracle_tag()
                        .summary("Output value histogram at height or day")
                        .description(
                            "Unfiltered output value histogram for a confirmed point. A block \
                            height (`840000`) gives every output in that block, coinbase \
                            included, binned by value on the oracle log scale; a calendar date \
                            (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale \
                            bins.",
                        )
                        .json_response::<HistogramRaw>()
                        .not_modified()
                        .bad_request()
                        .not_found()
                        .server_error()
                },
            ),
        )
    }
}
