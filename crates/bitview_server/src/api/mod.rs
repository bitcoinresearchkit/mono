use std::sync::Arc;

use aide::{
    axum::{ApiRouter, routing::get_with},
    transform::TransformOperation,
};
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
    params::Empty,
};

#[cfg(feature = "chain")]
mod addrs;
#[cfg(feature = "chain")]
mod block_template;
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
                    async |headers: HeaderMap, _: Empty, Extension(api): Extension<Arc<OpenApiBodies>>| {
                        api.full.respond(&headers)
                    },
                    |op| {
                        op.id("get_openapi")
                            .server_tag()
                            .mcp_ignore()
                            .summary("OpenAPI specification")
                            .description("Full OpenAPI 3.1 specification for this API. Build tools from this document.")
                            .json_response::<Value>()
                            .not_modified()
                    },
                ),
            )
            .api_route(
                "/api.json",
                get_with(
                    async |headers: HeaderMap, _: Empty, Extension(api): Extension<Arc<OpenApiBodies>>| {
                        api.compact.respond(&headers)
                    },
                    |op| {
                        op.id("get_api")
                            .server_tag()
                            .mcp_ignore()
                            .summary("Compact API summary")
                            .description(
                                "Compact custom summary of the OpenAPI specification for LLM reading: \
                                 types instead of schemas, `returns` instead of responses, no examples \
                                 or constraints. Not valid OpenAPI: build tools from `GET /openapi.json`.",
                            )
                            .json_response::<Value>()
                            .not_modified()
                    },
                ),
            )
            .route("/api", get(Html::from(include_str!("./scalar.html"))))
            // Encoded per request as the client accepts, by the compression layer.
            .route("/scalar.js", get(|headers: HeaderMap| async move {
                Response::static_bytes(
                    &headers,
                    include_bytes!("./scalar.js").as_slice(),
                    "application/javascript",
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
            // Documented once for every operation instead of per route.
            .with_path_items(|mut item| {
                let path = item.inner_mut();
                for operation in [&mut path.get, &mut path.post].into_iter().flatten() {
                    let _ = TransformOperation::new(operation).universal_errors();
                }
                item
            })
    }
}
