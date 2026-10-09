use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Extension,
    http::HeaderMap,
    response::{Html, Redirect, Response},
    routing::get,
};
use serde_json::Value;

use super::AppState;
#[cfg(feature = "series")]
use crate::api::series::ApiSeriesRoutes;
#[cfg(feature = "urpd")]
use crate::api::urpd::ApiUrpdRoutes;
use crate::{
    Error,
    api::server::ServerRoutes,
    extended::{ResponseExtended, TransformResponseExtended},
};

#[cfg(feature = "chain")]
mod addrs;
#[cfg(feature = "chain")]
mod blocks;
#[cfg(feature = "chain")]
pub mod broadcast;
#[cfg(feature = "chain")]
mod fees;
#[cfg(feature = "chain")]
mod general;
#[cfg(feature = "chain")]
mod historical_price;
#[cfg(feature = "chain")]
mod mempool;
#[cfg(feature = "chain")]
mod mempool_txids;
#[cfg(feature = "chain")]
mod mining;
mod openapi;
#[cfg(feature = "price")]
mod oracle;
#[cfg(feature = "series")]
mod series;
mod server;
#[cfg(feature = "chain")]
mod transactions;
#[cfg(feature = "urpd")]
mod urpd;
#[cfg(feature = "chain")]
mod utxo_set;

#[cfg(feature = "chain")]
use addrs::AddrRoutes;
#[cfg(feature = "chain")]
use blocks::BlockRoutes;
#[cfg(feature = "chain")]
use fees::FeesRoutes;
#[cfg(feature = "chain")]
use general::GeneralRoutes;
#[cfg(feature = "chain")]
use mempool::MempoolRoutes;
#[cfg(feature = "chain")]
use mining::MiningRoutes;
pub use openapi::*;
#[cfg(feature = "price")]
use oracle::OracleRoutes;
#[cfg(feature = "chain")]
use transactions::TxRoutes;
#[cfg(feature = "chain")]
use utxo_set::UtxoSetRoutes;

pub trait ApiRoutes {
    fn add_api_routes(self) -> Self;
}

impl ApiRoutes for ApiRouter<AppState> {
    fn add_api_routes(self) -> Self {
        let router = self.add_server_routes();
        #[cfg(feature = "series")]
        let router = router.add_series_routes();
        #[cfg(feature = "urpd")]
        let router = router.add_urpd_routes();
        #[cfg(feature = "chain")]
        let router = router
            .add_general_routes()
            .add_addr_routes()
            .add_block_routes()
            .add_mining_routes()
            .add_fees_routes()
            .add_mempool_routes()
            .add_tx_routes()
            .add_utxo_set_routes();
        #[cfg(feature = "price")]
        let router = router.add_oracle_routes();

        router
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
                            .json_response::<Value>()
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
