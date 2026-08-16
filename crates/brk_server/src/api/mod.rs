use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Extension,
    http::HeaderMap,
    response::{Html, Redirect, Response},
    routing::get,
};

use crate::{
    Error,
    api::{
        metrics::ApiMetricsLegacyRoutes, series::ApiSeriesRoutes,
        series_legacy::ApiSeriesLegacyRoutes, server::ServerRoutes, urpd::ApiUrpdRoutes,
    },
    extended::{ResponseExtended, TransformResponseExtended},
};

use super::AppState;

mod addrs;
mod blocks;
mod fees;
mod general;
mod mempool;
mod metrics;
mod mining;
mod openapi;
mod oracle;
mod series;
mod series_legacy;
mod server;
mod transactions;
mod urpd;

use addrs::AddrRoutes;
use blocks::BlockRoutes;
use fees::FeesRoutes;
use general::GeneralRoutes;
use mempool::MempoolRoutes;
use mining::MiningRoutes;
pub use openapi::*;
use oracle::OracleRoutes;
use transactions::TxRoutes;

pub trait ApiRoutes {
    fn add_api_routes(self) -> Self;
}

impl ApiRoutes for ApiRouter<AppState> {
    fn add_api_routes(self) -> Self {
        self.add_server_routes()
            .add_series_routes()
            .add_series_legacy_routes()
            .add_urpd_routes()
            .add_metrics_legacy_routes()
            .add_general_routes()
            .add_addr_routes()
            .add_block_routes()
            .add_mining_routes()
            .add_fees_routes()
            .add_mempool_routes()
            .add_oracle_routes()
            .add_tx_routes()
            .api_route(
                "/openapi.json",
                get_with(
                    async |headers: HeaderMap,
                           Extension(api): Extension<OpenApiJson>|
                           -> Response {
                        Response::static_json_bytes(&headers, api.bytes())
                    },
                    |op| {
                        op.id("get_openapi")
                            .server_tag()
                            .mcp_ignore()
                            .summary("OpenAPI specification")
                            .description("Full OpenAPI 3.1 specification for this API.")
                    },
                ),
            )
            .api_route(
                "/api.json",
                get_with(
                    async |headers: HeaderMap,
                           Extension(api): Extension<ApiJson>|
                           -> Response {
                        Response::static_json_bytes(&headers, api.bytes())
                    },
                    |op| {
                        op.id("get_api")
                            .server_tag()
                            .mcp_ignore()
                            .summary("Compact OpenAPI specification")
                            .description(
                                "Compact OpenAPI specification optimized for LLM consumption. \
                                 Removes redundant fields while preserving essential API information. \
                                 The full specification is available at `GET /openapi.json`.",
                            )
                            .json_response::<serde_json::Value>()
                    },
                ),
            )
            .route("/api", get(Html::from(include_str!("./scalar.html"))))
            // Pre-compressed with: brotli -c -q 11 scalar.js > scalar.js.br
            .route("/scalar.js", get(|headers: HeaderMap| async move {
                Response::static_bytes(
                    &headers,
                    include_bytes!("./scalar.js.br").as_slice(),
                    "application/javascript",
                    "br",
                )
            }))
            .route(
                "/.well-known/openapi.json",
                get(|| async { Redirect::permanent("/openapi.json") }),
            )
            .route(
                "/api/{*path}",
                get(|| async {
                    Error::not_found("Unknown API endpoint. See /api for documentation.")
                }),
            )
    }
}
