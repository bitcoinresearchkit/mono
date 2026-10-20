use std::sync::Arc;

use aide::{
    axum::{ApiRouter, routing::get_with},
    transform::TransformOperation,
};
use axum::{Extension, http::HeaderMap, response::Redirect, routing::get};
use serde_json::Value;

use super::AppState;
use crate::{Error, extended::TransformResponseExtended, params::Empty};

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

pub use openapi::*;

pub trait ApiRoutes {
    fn add_api_routes(self) -> Self;
}

impl ApiRoutes for ApiRouter<AppState> {
    fn add_api_routes(self) -> Self {
        let router = server::add_routes(self);
        #[cfg(feature = "series")]
        let router = series::add_routes(router);
        #[cfg(feature = "urpd")]
        let router = urpd::add_routes(router);
        #[cfg(feature = "chain")]
        let router = [
            general::add_routes,
            addrs::add_routes,
            blocks::add_routes,
            mining::add_routes,
            fees::add_routes,
            mempool::add_routes,
            transactions::add_routes,
            utxo_set::add_routes,
        ]
        .into_iter()
        .fold(router, |router, add| add(router));
        #[cfg(feature = "price")]
        let router = oracle::add_routes(router);

        router
            .api_route(
                "/openapi.json",
                get_with(
                    async |headers: HeaderMap, _: Empty, Extension(api): Extension<Arc<OpenApiBodies>>| {
                        api.full.respond(&headers)
                    },
                    |op| {
                        op.id("get_openapi")
                            .tag(tags::SERVER)
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
                            .tag(tags::SERVER)
                            .mcp_ignore()
                            .summary("Compact API summary")
                            .description(
                                "Compact custom summary of the OpenAPI specification for LLM reading: \
                                 types instead of schemas, `returns` instead of responses, no examples; \
                                 parameters keep their enum, default and bounds. Not valid OpenAPI: \
                                 build tools from `GET /openapi.json`.",
                            )
                            .json_response::<Value>()
                            .not_modified()
                    },
                ),
            )
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
