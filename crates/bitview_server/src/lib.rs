#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    fs,
    net::SocketAddr,
    num::NonZero,
    path::Path,
    sync::{Arc, LazyLock},
    thread,
    time::{Duration, Instant},
};

use aide::{axum::ApiRouter, openapi::OpenApi};
use axum::{
    Extension, Router,
    middleware::from_fn,
    response::{IntoResponse, Redirect},
    routing::get,
    serve,
};
#[cfg(feature = "series")]
use bitview_query::SharedSeries;
use bitview_query::{AsyncQuery, Result};
use bitview_website::router as WebsiteRouter;
use jiff::Timestamp;
use tokio::{
    net::{TcpListener, TcpSocket},
    sync::Semaphore,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    timeout::RequestBodyTimeoutLayer,
};
use tracing::info;

use api::*;
use cache::CacheParams;
use stall_limit::StallLimitListener;
use state::*;

mod api;
#[cfg(feature = "full-api")]
mod body_response;
mod cache;
mod canonical_url;
mod config;
mod error;
mod etag;
mod extended;
mod json_error;
mod params;
mod port;
mod prepared_json;
#[cfg(feature = "full-api")]
mod raw_body;
mod request_action;
mod request_deadline;
mod request_state;
mod response_time;
#[cfg(feature = "series")]
mod series_bodies;
mod stall_limit;
mod state;

pub use api::ApiRoutes;

pub use bitview_website::Website;
pub use cache::CdnCacheMode;
pub use error::Error;
pub use port::Port;

pub use config::{DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, ServerConfig};

#[cfg(feature = "full-api")]
use raw_body::RawBodyPermit;

#[cfg(feature = "series")]
use series_bodies::SeriesBodies;

/// The version the server says it runs (`/version`, `/health`'s, every ETag's prefix): built
/// from the repository, its commit, so each build is told apart (by `cargo install --git`,
/// next's, the one it was built from; in a checkout, `cargo dev`'s, the one checked out as it
/// starts); a crates.io release, `v` and the release.
pub(crate) static VERSION: LazyLock<String> = LazyLock::new(|| {
    installed_commit()
        .or_else(|| checkout_commit(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
        .unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")))
});

/// The short commit checked out in the repository at `root` (this crate's workspace, two up
/// from its source), read from its `.git` as the server starts: its HEAD, the branch it names
/// (loose or packed, a worktree's in its repository's), or the commit itself. None where it's no
/// repository (a crates.io release's source, in cargo's registry).
fn checkout_commit(root: &Path) -> Option<String> {
    let dot = root.join(".git");
    let git = if dot.is_dir() {
        dot
    } else {
        // (A worktree's: a file naming its own.)
        root.join(
            fs::read_to_string(&dot)
                .ok()?
                .strip_prefix("gitdir:")?
                .trim(),
        )
    };
    let head = fs::read_to_string(git.join("HEAD")).ok()?;
    let commit = match head.trim().strip_prefix("ref:") {
        None => head.trim().to_string(),
        Some(name) => {
            let name = name.trim();
            let common = fs::read_to_string(git.join("commondir"))
                .map_or_else(|_| git.clone(), |dir| git.join(dir.trim()));
            match fs::read_to_string(common.join(name)) {
                Ok(commit) => commit.trim().to_string(),
                Err(_) => fs::read_to_string(common.join("packed-refs"))
                    .ok()?
                    .lines()
                    .find_map(|line| {
                        let (commit, packed) = line.split_once(' ')?;
                        (packed == name).then(|| commit.to_string())
                    })?,
            }
        }
    };
    (commit.len() >= 7 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| commit[..7].to_string())
}

/// The short commit `cargo install --git` built this binary from, as cargo recorded it: its record
/// of installs (`.crates2.json`, in the root whose `bin/` holds the binary, its links followed)
/// keys each by its package id, a git one ending `#` and the full commit. None for any other
/// install, or none.
fn installed_commit() -> Option<String> {
    let exe = std::fs::canonicalize(std::env::current_exe().ok()?).ok()?;
    let bin = exe.parent()?;
    if bin.file_name()? != "bin" {
        return None;
    }
    let record = std::fs::read(bin.parent()?.join(".crates2.json")).ok()?;
    commit_installed(&record, exe.file_name()?.to_str()?)
}

/// In cargo's record of installs, the short commit this release's binary `name` was built from by
/// `cargo install --git`; None if it came from anywhere else (another release's record, a source
/// not a commit, a record not cargo's).
fn commit_installed(record: &[u8], name: &str) -> Option<String> {
    let record: serde_json::Value = serde_json::from_slice(record).ok()?;
    record
        .get("installs")?
        .as_object()?
        .iter()
        .find_map(|(id, install)| {
            let bins = install.get("bins")?.as_array()?;
            if !bins.iter().any(|bin| bin.as_str() == Some(name)) {
                return None;
            }
            // (`name version (source)`, this release's, a git source's commit in hex digits.)
            let (package, source) = id.strip_suffix(')')?.split_once(" (")?;
            if package.split_once(' ')?.1 != env!("CARGO_PKG_VERSION") {
                return None;
            }
            let (_, commit) = source.strip_prefix("git+")?.rsplit_once('#')?;
            (commit.len() >= 7 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
                .then(|| commit[..7].to_string())
        })
}

/// Per-request timeout. Hits return 504 Gateway Timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// The longest a request body may go without sending more (a transaction broadcast): a client
/// stalling one would hold its admission permit.
const REQUEST_BODY_TIMEOUT: Duration = Duration::from_secs(10);

/// How long browsers may reuse a CORS preflight.
const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Connections that may wait to be accepted, past which a burst's are reset. The kernel caps it
/// without saying (`kern.ipc.somaxconn`, `net.core.somaxconn`).
const ACCEPT_QUEUE: u32 = 4096;

pub struct Server {
    app: Router,
    listener: TcpListener,
}

impl Server {
    /// Binds the HTTP listener so startup failures are reported before the
    /// caller launches the long-running server task.
    pub async fn bind(query: &AsyncQuery, config: ServerConfig) -> Result<Self> {
        let address = SocketAddr::new(config.bind, config.port.into());
        let socket = match address {
            SocketAddr::V4(_) => TcpSocket::new_v4()?,
            SocketAddr::V6(_) => TcpSocket::new_v6()?,
        };
        // As `TcpListener::bind`: on Windows this would let another process take the port.
        #[cfg(not(windows))]
        socket.set_reuseaddr(true)?;
        socket.bind(address)?;
        let listener = socket.listen(ACCEPT_QUEUE)?;

        config.website.log();
        #[cfg(feature = "series")]
        query.sync(|q| {
            log_shared_series(q.registry().shared_series(), q.registry().value_type_conflicts())
        });

        Ok(Self {
            app: app(query, config).await?,
            listener,
        })
    }

    pub async fn serve(self) -> Result<()> {
        let Self { app, listener } = self;
        info!("Server listening on http://{}", listener.local_addr()?);

        serve(StallLimitListener(listener), app).await?;

        Ok(())
    }
}

/// Builds the HTTP application for `query` without binding a socket: every route plus the
/// response middleware.
pub async fn app(query: &AsyncQuery, config: ServerConfig) -> Result<Router> {
    #[cfg(feature = "series")]
    let series_bodies = query
        .run(|query| Ok(Arc::new(SeriesBodies::new(query))))
        .await?;
    #[cfg(feature = "full-api")]
    let mining_pools_body = Arc::new(prepared_json::PreparedJson::new(
        query.sync(|query| query.all_pools()),
    ));

    let shared = SharedState {
        chain_query: Arc::new(Semaphore::new(
            thread::available_parallelism().map_or(4, NonZero::get),
        )),
        #[cfg(feature = "full-api")]
        history_query: Arc::new(Semaphore::new(2)),
        disk_query: Arc::new(Semaphore::new(1)),
        disk_walk: Default::default(),
        #[cfg(feature = "full-api")]
        oracle_replays: Arc::new(Semaphore::new(1)),
        #[cfg(feature = "full-api")]
        raw_block_bodies: Arc::new(Semaphore::new(RawBodyPermit::CAPACITY)),
        #[cfg(feature = "full-api")]
        historical_price_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        historical_price_cache: Arc::default(),
        #[cfg(feature = "full-api")]
        mempool_txid_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        mempool_block_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        mempool_block_diff_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        broadcast_requests: Arc::new(Semaphore::new(api::broadcast::BroadcastPermit::CAPACITY)),
        #[cfg(feature = "full-api")]
        chain_bodies: Arc::new(Semaphore::new(RawBodyPermit::CAPACITY)),
        node: query
            .run(|query| Ok(query.client().asynchronous()?))
            .await?,
        #[cfg(feature = "series")]
        series_bodies,
        #[cfg(feature = "full-api")]
        utxo_set_query: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        utxo_set_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        urpd_query: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        urpd_bodies: Arc::new(Semaphore::new(2)),
        #[cfg(feature = "full-api")]
        mining_pools_body,
        data_path: config.data_path,
        website: config.website,
        started_at: Timestamp::now(),
        started_instant: Instant::now(),
        max_weight: config.max_weight,
        max_utxos: config.max_utxos,
        cdn_cache_mode: config.cdn_cache_mode,
    };
    let state = AppState::new(query.clone(), shared);

    let website_router = WebsiteRouter(state.website.clone());
    let mut router = ApiRouter::new()
        .add_api_routes()
        .route_layer(from_fn(canonical_url::refuse_respelled));
    if !state.website.is_enabled() {
        router = router.route("/", get(Redirect::temporary("/openapi.json")));
    }
    let router = router
        .with_state(state)
        .merge(website_router)
        .layer(from_fn(request_deadline::apply))
        .layer(RequestBodyTimeoutLayer::new(REQUEST_BODY_TIMEOUT))
        .layer(from_fn(json_error::respond))
        .layer(CatchPanicLayer::custom(|panic: Box<dyn Any + Send>| {
            let msg = panic
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("Unknown panic");
            Error::internal(msg).into_response()
        }))
        // Outside the panic catcher, so its 500 reaches browsers too; preflights hold a day.
        .layer(CorsLayer::permissive().max_age(PREFLIGHT_MAX_AGE))
        .layer(from_fn(response_time::respond));

    let (router, openapi) = finish_openapi(router);

    Ok(router.layer(Extension(Arc::new(OpenApiBodies::new(&openapi)))))
}

pub fn finish_openapi<S: Clone + Send + Sync + 'static>(
    router: ApiRouter<S>,
) -> (Router<S>, OpenApi) {
    let mut openapi = create_openapi();
    let router = router.finish_api(&mut openapi);
    (router, openapi)
}

/// A composition may publish one id from several plugins: say which plugin serves it.
#[cfg(feature = "series")]
fn log_shared_series(shared: &[SharedSeries], value_type_conflicts: &[&str]) {
    for series in shared {
        for (plugin, matches) in &series.also {
            if *matches {
                info!(
                    "Series {} is published by {} and {plugin}; {} serves it",
                    series.name, series.served_by, series.served_by
                );
            } else {
                tracing::warn!(
                    "Series {} from {plugin} has another value type than {}'s and is left out",
                    series.name,
                    series.served_by
                );
            }
        }
    }
    for series in value_type_conflicts {
        tracing::warn!("Series {series} is published with more than one value type");
    }
}
