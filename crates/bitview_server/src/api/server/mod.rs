use std::borrow::Cow;

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Json,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_http::CachePolicy;
use bitview_types::{DiskUsage, Health, HealthStatus, Logs, ServerVersion, SyncStatus};
use jiff::Timestamp;

use super::AppState;
use crate::{
    VERSION,
    answer::{Answer, Content, JSON},
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::Empty,
    request_state::RequestState,
};

mod disk;
mod logs;

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/health",
        get_with(
            async |_: Empty, RequestState(state): RequestState| -> Result<Response> {
                let sync = state.query.read(|q| q.sync_status()).await?;
                let uptime = state.started_instant.elapsed();
                let timestamp = Timestamp::now().to_string();
                let mut response = Json(Health {
                    status: HealthStatus::Healthy,
                    service: Cow::Borrowed("brk"),
                    version: ServerVersion(Cow::Borrowed(VERSION.as_str())),
                    timestamp,
                    started_at: state.started_at.to_string(),
                    uptime_seconds: uptime.as_secs(),
                    sync,
                })
                .into_response();
                CachePolicy::NoStore.apply(response.headers_mut());
                Ok(response)
            },
            |op| {
                op.id("get_health")
                    .tag(tags::SERVER)
                    .mcp_ignore()
                    .summary("Health check")
                    .description("Local health and query-readiness check. Returns server identity, uptime, and the sync status of `GET /api/server/sync`, without a bitcoind round-trip. `status` says the server answers, whatever `blocks_behind` is. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached.")
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
                    .json_response::<ServerVersion>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/server/sync",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| -> Response {
                state
                    .respond(&headers, state.chain(), |q| {
                        let sync = q.sync_status()?;
                        // computed_height and blocks_behind derive from these heights;
                        // last_indexed_at derives from the Unix timestamp.
                        let tag = format!(
                            "sync-{}-{}-{}",
                            sync.indexed_height, sync.tip_height, *sync.last_indexed_at_unix,
                        );
                        Ok(Answer::ready(tag, Content::json(&sync)?))
                    })
                    .await
            },
            |op| {
                op.id("get_sync_status")
                    .tag(tags::SERVER)
                    .summary("Sync status")
                    .description(
                        "Returns a coherent local index snapshot and the Bitcoin Core tip height the server last saw: \
                        it looks for a new tip every second and no request asks the node. The two heights differ while a new block is being indexed.",
                    )
                    .json_response::<SyncStatus>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/server/logs",
        get_with(logs::get, |op| {
            op.id("get_logs")
                .tag(tags::SERVER)
                .mcp_ignore()
                .summary("Logs")
                .description(
                    "The server's last log lines, at most 1,000, oldest first: requests (status, method, path and duration; never who made them) \
                    and everything else it logs. To read on, send the answer's `next` back as `after`: the lines since, each once. \
                    A jump in `seq` shows lines that left memory before the next request; another `started_unix_ms` shows the \
                    server restarted (and its answer has every kept line). Answers are cached for a second.",
                )
                .json_response::<Logs>()
                .not_modified()
        }),
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
                        "Returns the bytes Bitview's data takes on disk and the bytes of the node's block files. Bitview's total is its indexer stores' table files as the stores track them, plus the allocated bytes (file lengths on non-Unix platforms) of every other file in its data directory, the server's own logs left out, measured when the server starts and after each indexed block. The node's total is its own estimate (`getblockchaininfo`'s `size_on_disk`: block and undo files, without its block index), as of the node's last new tip. No request reads the filesystem or asks the node. The ETag is made from both totals.",
                    )
                    .json_response::<DiskUsage>()
                    .not_modified()
            },
        ),
    )
}

/// The release, which every tag starts with: `d` after it.
async fn version(headers: HeaderMap, _: Empty, RequestState(state): RequestState) -> Response {
    let content = Content::new(JSON, format!("\"{}\"", *VERSION));
    state.respond_now(&headers, Answer::ready("d".to_owned(), content))
}
