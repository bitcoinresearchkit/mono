#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    borrow::Cow,
    fs::{self, DirEntry, Metadata},
    path::Path,
};

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_types::{Health, SyncStatus};
use brk_error::Result;
use brk_types::DiskUsage;
use rayon::{
    iter::{ParallelBridge, ParallelIterator},
    join,
};

use crate::{
    CacheStrategy, VERSION,
    error::RouteResult,
    extended::{HeaderMapExtended, TransformResponseExtended},
    params::Empty,
};

use super::AppState;

pub trait ServerRoutes {
    fn add_server_routes(self) -> Self;
}

impl ServerRoutes for ApiRouter<AppState> {
    fn add_server_routes(self) -> Self {
        self.api_route(
            "/health",
            get_with(
                async |_: Empty, State(state): State<AppState>| -> RouteResult<Response> {
                    let uptime = state.started_instant.elapsed();
                    let started_at = state.started_at.to_string();
                    let sync = state.run(|q| q.local_sync_status()).await?;
                    let mut response = axum::Json(Health {
                        status: Cow::Borrowed("healthy"),
                        service: Cow::Borrowed("brk"),
                        version: Cow::Borrowed(VERSION),
                        timestamp: jiff::Timestamp::now().to_string(),
                        started_at,
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
                        .description("Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, request `GET /api/server/sync`.")
                        .json_response::<Health>()
                },
            ),
        )
        .api_route(
            "/version",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json_value(
                        &headers,
                        CacheStrategy::Deploy,
                        env!("CARGO_PKG_VERSION"),
                    )
                },
                |op| {
                    op.id("get_version")
                        .server_tag()
                        .mcp_ignore()
                        .summary("API version")
                        .description("Returns the current version of the API server")
                        .json_response::<String>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/server/sync",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state
                        .respond_json_content(&headers, move |q| {
                            let tip_height = q.client().get_last_height()?;
                            q.sync_status(tip_height)
                        })
                        .await
                },
                |op| {
                    op.id("get_sync_status")
                        .server_tag()
                        .summary("Sync status")
                        .description(
                            "Returns the sync status of the indexer, including indexed height, \
                            tip height, blocks behind, and last indexed timestamp.",
                        )
                        .json_response::<SyncStatus>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/server/disk",
            get_with(
                async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    let brk_path = state.data_path.clone();
                    state
                        .respond_json_content(&headers, move |q| {
                            let bitcoin_path = q.blocks_dir();
                            let (brk_bytes, bitcoin_bytes) = join(
                                || dir_size(&brk_path),
                                || dir_size(bitcoin_path),
                            );
                            Ok(DiskUsage::new(brk_bytes?, bitcoin_bytes?))
                        })
                        .await
                },
                |op| {
                    op.id("get_disk_usage")
                        .server_tag()
                        .mcp_ignore()
                        .summary("Disk usage")
                        .description(
                            "Returns the disk space used by BRK and Bitcoin data.",
                        )
                        .json_response::<DiskUsage>()
                        .not_modified()
                },
            ),
        )
    }
}

fn dir_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        dir_contents_size(path)
    } else {
        Ok(allocated_bytes(&metadata))
    }
}

fn dir_contents_size(path: &Path) -> Result<u64> {
    fs::read_dir(path)?
        .par_bridge()
        .map(|entry| dir_entry_size(entry?))
        .try_reduce(|| 0, |left, right| Ok(left + right))
}

fn dir_entry_size(entry: DirEntry) -> Result<u64> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return dir_contents_size(&entry.path());
    }

    if file_type.is_symlink() {
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        return if metadata.is_dir() {
            dir_contents_size(&path)
        } else {
            Ok(allocated_bytes(&metadata))
        };
    }

    Ok(allocated_bytes(&entry.metadata()?))
}

#[cfg(unix)]
fn allocated_bytes(metadata: &Metadata) -> u64 {
    // POSIX st_blocks units are always 512 bytes.
    metadata.blocks() * 512
}

#[cfg(not(unix))]
fn allocated_bytes(metadata: &Metadata) -> u64 {
    metadata.len()
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn directory_size_counts_nested_allocated_bytes() -> Result<()> {
        let directory = tempdir()?;
        let nested = directory.path().join("nested");
        fs::create_dir(&nested)?;

        let first = directory.path().join("first");
        let second = nested.join("second");
        fs::write(&first, [0; 1])?;
        fs::write(&second, [0; 8192])?;

        let first_bytes = allocated_bytes(&fs::metadata(&first)?);
        let second_bytes = allocated_bytes(&fs::metadata(&second)?);
        let mut expected = first_bytes + second_bytes;

        #[cfg(unix)]
        {
            symlink(&second, directory.path().join("second-link"))?;
            expected += second_bytes;
        }

        assert_eq!(dir_size(directory.path())?, expected);
        assert_eq!(dir_size(&first)?, first_bytes);
        Ok(())
    }
}
