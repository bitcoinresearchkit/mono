#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    net::SocketAddr,
    num::NonZero,
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

use aide::{axum::ApiRouter, openapi::OpenApi};
use axum::{
    Extension, Router,
    middleware::{from_fn, map_response},
    response::{IntoResponse, Response},
    serve,
};
#[cfg(feature = "series")]
use bitview_query::SharedSeries;
use bitview_query::{AsyncQuery, Result};
use jiff::Timestamp;
use tokio::{
    net::{TcpListener, TcpSocket},
    sync::Semaphore,
};
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};
use tracing::info;

use api::*;
use stall_limit::StallLimitListener;
use state::*;

mod answer;
mod api;
mod canonical_url;
mod config;
mod error;
mod etag;
mod extended;
mod gates;
mod json_error;
#[cfg(feature = "full-api")]
mod mcp;
mod params;
mod port;
mod prepared_json;
mod raw_body;
mod request_deadline;
mod request_state;
mod response_time;
#[cfg(feature = "series")]
mod series_bodies;
mod stall_limit;
mod state;

pub use answer::CdnCacheMode;
#[cfg(feature = "full-api")]
pub use mcp::DESCRIPTION as MCP_DESCRIPTION;
pub use api::ApiRoutes;

pub use error::Error;
pub use port::Port;

pub use config::{DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, PublicUrl, ServerConfig, SiteUrl};

use raw_body::RawBodyPermit;

#[cfg(feature = "series")]
use series_bodies::SeriesBodies;

static VERSION: OnceLock<&'static str> = OnceLock::new();

/// Names what the application was built from: the version `/version` and `/health` report and
/// the digest of its sources that prefixes every ETag. Call before starting the server. It
/// lives with the application, not here, so a new commit rebuilds only the binary: embedders
/// that don't call it report the crate's version and tag by their executable's hash.
pub fn set_build(version: &'static str, tag: &'static str) {
    VERSION.set(version).expect("Build is already set");
    etag::set_build_tag(tag);
}

fn build_version() -> &'static str {
    VERSION.get().copied().unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}

/// Per-request timeout. Hits return 504 Gateway Timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

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
    etag::initialize()?;
    #[cfg(feature = "full-api")]
    let site_url = config.site_url();

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
        chain_bodies: Arc::new(Semaphore::new(RawBodyPermit::CAPACITY)),
        #[cfg(feature = "full-api")]
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
        started_at: Timestamp::now(),
        started_instant: Instant::now(),
        max_weight: config.max_weight,
        max_utxos: config.max_utxos,
        cdn_cache_mode: config.cdn_cache_mode,
    };
    let state = AppState::new(query.clone(), shared);

    let cdn_purged = state.cdn_cache_mode == CdnCacheMode::Aggressive;
    let router = ApiRouter::new()
        .add_api_routes()
        .route_layer(from_fn(canonical_url::refuse_respelled))
        .with_state(state)
        .merge(bitview_website::router(cdn_purged))
        .layer(from_fn(request_deadline::apply))
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
    // Outermost: every answer names this server, errors and preflights too.
    let router = match config.public_url {
        Some(PublicUrl(url)) => router.layer(map_response(move |mut response: Response| {
            response.headers_mut().insert(bitview_http::BITVIEW_ORIGIN, url.clone());
            async { response }
        })),
        None => router,
    };

    let (router, openapi) = finish_openapi(router);
    let router = router.layer(Extension(Arc::new(OpenApiBodies::new(&openapi))));
    // Beside the API's layers, not under them: a tool call's API request is the one line it logs.
    #[cfg(feature = "full-api")]
    let router = router.clone().merge(mcp::router(router, &site_url));

    Ok(router)
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
