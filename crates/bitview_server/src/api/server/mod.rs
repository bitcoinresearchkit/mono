use std::borrow::Cow;

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Json,
    body::Bytes,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_types::{DiskUsage, Health, HealthStatus, SyncStatus};
use jiff::Timestamp;

use super::AppState;
use crate::{
    CacheParams, VERSION,
    api::tags,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::Empty,
    request_state::RequestState,
};

mod disk;

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/health",
        get_with(
            async |_: Empty, RequestState(state): RequestState| -> Result<Response> {
                let sync = state.query.read(|q| q.local_sync_status()).await?;
                let uptime = state.started_instant.elapsed();
                let timestamp = Timestamp::now().to_string();
                let mut response = Json(Health {
                    status: HealthStatus::Healthy,
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
                    .tag(tags::SERVER)
                    .mcp_ignore()
                    .summary("Health check")
                    .description("Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.")
                    .json_response::<Health>()
            },
        ),
    )
    .api_route(
        "/version",
        get_with(
            version,
            |op| {
                op.id("get_version")
                    .tag(tags::SERVER)
                    .mcp_ignore()
                    .summary("API version")
                    .json_response::<String>()
                    .not_modified()
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
                let params = CacheParams::live(format!(
                    "sync1-{}-{}-{}",
                    sync.indexed_height, sync.tip_height, *sync.last_indexed_at_unix,
                ));
                Ok(Response::cached_json(&headers, &params, || sync))
            },
            |op| {
                op.id("get_sync_status")
                    .tag(tags::SERVER)
                    .summary("Sync status")
                    .description(
                        "Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. \
                        The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.",
                    )
                    .json_response::<SyncStatus>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/server/disk",
        get_with(
            disk::get,
            |op| {
                op.id("get_disk_usage")
                    .tag(tags::SERVER)
                    .mcp_ignore()
                    .summary("Disk usage")
                    .description(
                        "Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.",
                    )
                    .json_response::<DiskUsage>()
                    .not_modified()
            },
        ),
    )
}

async fn version(headers: HeaderMap, _: Empty) -> Response {
    Response::cached_bytes(&headers, &CacheParams::deploy(), "application/json", || {
        Bytes::from_static(concat!("\"", env!("CARGO_PKG_VERSION"), "\"").as_bytes())
    })
}
