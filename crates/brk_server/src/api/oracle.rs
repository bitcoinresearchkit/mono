use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Uri},
    response::IntoResponse,
};
use brk_oracle::{HistogramEmaCompact, HistogramRaw};
use brk_types::{Day1, Dollars, Version};

use crate::{
    AppState,
    extended::TransformResponseExtended,
    params::{Empty, HeightOrDate, HeightOrDateParam},
};

pub trait OracleRoutes {
    fn add_oracle_routes(self) -> Self;
}

impl OracleRoutes for ApiRouter<AppState> {
    fn add_oracle_routes(self) -> Self {
        self.api_route(
            "/api/oracle/price",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| q.live_price())
                        .await
                },
                |op| {
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
                },
            ),
        )
        .api_route(
            "/api/oracle/histogram/payments/live",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| {
                            q.live_payment_histogram()
                        })
                        .await
                },
                |op| {
                    op.id("get_oracle_histogram_payments_live")
                        .oracle_tag()
                        .summary("Live payment output histogram")
                        .description(
                            "Live smoothed histogram of oracle-eligible payment outputs, binned \
                            by output value on the oracle log scale. It combines the committed \
                            oracle window with the forming mempool block. A flat array of \
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
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       State(state): State<AppState>| {
                    let version = Version::new(brk_oracle::VERSION);
                    match path.resolve() {
                        Ok(HeightOrDate::Date(date)) => {
                            let strategy = state.date_strategy(version, date);
                            state
                                .respond_json(&headers, strategy, &uri, move |q| {
                                    q.confirmed_payment_histogram_day(Day1::try_from(date)?)
                                })
                                .await
                        }
                        Ok(HeightOrDate::Height(height)) => {
                            let strategy = state.height_strategy(version, height);
                            state
                                .respond_json(&headers, strategy, &uri, move |q| {
                                    q.confirmed_payment_histogram(usize::from(height))
                                })
                                .await
                        }
                        Err(e) => e.into_response(),
                    }
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
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json(&headers, state.mempool_strategy(), &uri, |q| {
                            q.live_output_histogram()
                        })
                        .await
                },
                |op| {
                    op.id("get_oracle_histogram_outputs_live")
                        .oracle_tag()
                        .summary("Live output value histogram")
                        .description(
                            "Live unfiltered output value histogram for the forming mempool \
                            block. Every live output is binned by value on the oracle log scale; \
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
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(path): Path<HeightOrDateParam>,
                       _: Empty,
                       State(state): State<AppState>| {
                    let version = Version::new(brk_oracle::VERSION);

                    match path.resolve() {
                        Ok(HeightOrDate::Date(date)) => {
                            let strategy = state.date_strategy(version, date);
                            state
                                .respond_json(&headers, strategy, &uri, move |q| {
                                    q.confirmed_output_histogram_day(Day1::try_from(date)?)
                                })
                                .await
                        }
                        Ok(HeightOrDate::Height(height)) => {
                            let strategy = state.height_strategy(version, height);
                            state
                                .respond_json(&headers, strategy, &uri, move |q| {
                                    q.confirmed_output_histogram(usize::from(height))
                                })
                                .await
                        }
                        Err(e) => e.into_response(),
                    }
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
