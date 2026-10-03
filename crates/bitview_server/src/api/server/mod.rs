use std::borrow::Cow;

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Json,
    body::Bytes,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_types::{DiskUsage, Health, SyncStatus};
use jiff::Timestamp;

use super::AppState;
use crate::{
    CacheStrategy, VERSION,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::Empty,
    request_state::RequestState,
};

mod disk;

pub trait ServerRoutes {
    fn add_server_routes(self) -> Self;
}

impl ServerRoutes for ApiRouter<AppState> {
    fn add_server_routes(self) -> Self {
        self.api_route(
            "/health",
            get_with(
                async |_: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let sync = state.read(|q| q.local_sync_status()).await?;
                    let uptime = state.started_instant.elapsed();
                    let timestamp = Timestamp::now().to_string();
                    let mut response = Json(Health {
                        status: Cow::Borrowed("healthy"),
                        service: Cow::Borrowed("brk"),
                        version: Cow::Borrowed(VERSION),
                        timestamp,
                        started_at: state.started_at.to_string(),
                        uptime_seconds: uptime.as_secs(),
                        sync,
                    })
                    .into_response();
                    let h = response.headers_mut();
                    h.insert_cache_control("no-store");
                    h.insert_cdn_cache_control("no-store");
                    Ok(response)
                },
                |op| {
                    op.id("get_health")
                        .server_tag()
                        .mcp_ignore()
                        .summary("Health check")
                        .description("Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index waits until the request deadline, then returns 504. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.")
                        .json_response::<Health>()
                        .bad_request()
                        .server_errors()
                },
            ),
        )
        .api_route(
            "/version",
            get_with(
                version,
                |op| {
                    op.id("get_version")
                        .server_tag()
                        .mcp_ignore()
                        .summary("API version")
                        .description("Returns the current version of the API server")
                        .json_response::<String>()
                        .not_modified()
                        .bad_request()
                },
            ),
        )
        .api_route(
            "/api/server/sync",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| -> Result<Response> {
                    let tip_height = state.node.get_last_height().await?;
                    let sync = state.read_admitted(move |q| q.sync_status(tip_height)).await?;
                    // computed_height and blocks_behind derive from these heights;
                    // last_indexed_at derives from the Unix timestamp.
                    let strategy = CacheStrategy::Live(format!(
                        "sync1-{}-{}-{}",
                        sync.indexed_height, sync.tip_height, *sync.last_indexed_at_unix,
                    ).into());
                    Ok(state.respond_json_value(&headers, strategy, sync))
                },
                |op| {
                    op.id("get_sync_status")
                        .server_tag()
                        .summary("Sync status")
                        .description(
                            "Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. \
                            The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.",
                        )
                        .json_response::<SyncStatus>()
                        .not_modified()
                        .bad_request()
                        .server_errors()
                },
            ),
        )
        .api_route(
            "/api/server/disk",
            get_with(
                disk::get,
                |op| {
                    op.id("get_disk_usage")
                        .server_tag()
                        .mcp_ignore()
                        .summary("Disk usage")
                        .description(
                            "Returns allocated file bytes for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.",
                        )
                        .json_response::<DiskUsage>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                        .gateway_timeout()
                },
            ),
        )
    }
}

async fn version(headers: HeaderMap, _: Empty) -> Response {
    Response::static_json_bytes(
        &headers,
        Bytes::from_static(concat!("\"", env!("CARGO_PKG_VERSION"), "\"").as_bytes()),
    )
}
