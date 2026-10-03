#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use aide::{axum::ApiRouter, openapi::OpenApi};
use api::*;
use axum::{
    Extension, Router, ServiceExt,
    body::Body,
    http::Request,
    middleware::from_fn,
    response::{IntoResponse, Redirect},
    routing::get,
    serve,
};
use bitview_query::AsyncQuery;
use bitview_website::router as WebsiteRouter;
use brk_error::Result;
use cache::{CacheParams, CacheStrategy};
use error::Error;
use jiff::Timestamp;
use response_size_above::ResponseSizeAbove;
use state::*;
use tokio::{net::TcpListener, sync::Semaphore};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::{
        CompressionLayer, CompressionLevel,
        predicate::{NotForContentType, Predicate},
    },
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
};
use tower_layer::Layer;
use tracing::info;

mod api;
#[cfg(any(feature = "chain", feature = "urpd"))]
mod body_response;
mod cache;
mod config;
mod error;
mod error_body;
mod error_code;
mod etag;
mod extended;
#[cfg(feature = "chain")]
mod historical_price_cache;
mod json_error;
mod params;
#[cfg(any(feature = "series", feature = "chain"))]
mod prepared_json;
#[cfg(any(feature = "chain", feature = "urpd"))]
mod raw_body;
mod request_deadline;
mod request_state;
mod response_size_above;
mod response_time;
#[cfg(feature = "series")]
mod series_bodies;
mod state;
#[cfg(feature = "urpd")]
mod urpd_input;

pub use api::ApiRoutes;

pub use bitview_website::Website;
pub use brk_types::Port;
pub use cache::CdnCacheMode;

pub use config::{DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, ServerConfig};

#[cfg(feature = "chain")]
use raw_body::RawBodyPermit;

#[cfg(feature = "series")]
use series_bodies::SeriesBodies;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Per-request timeout. Hits return 504 Gateway Timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Avoid spending compression work on responses too small to benefit materially.
const MIN_COMPRESSED_RESPONSE_BYTES: u64 = 1024;

fn compression_layer() -> CompressionLayer<impl Predicate> {
    CompressionLayer::new()
        .br(true)
        .gzip(true)
        .zstd(true)
        .quality(CompressionLevel::Fastest)
        .compress_when(
            ResponseSizeAbove(MIN_COMPRESSED_RESPONSE_BYTES)
                .and(NotForContentType::GRPC)
                .and(NotForContentType::IMAGES)
                .and(NotForContentType::SSE),
        )
}

pub struct Server {
    state: AppState,
    listener: TcpListener,
}

impl Server {
    /// Binds the HTTP listener so startup failures are reported before the
    /// caller launches the long-running server task.
    pub async fn bind(query: &AsyncQuery, config: ServerConfig) -> Result<Self> {
        let address = SocketAddr::new(config.bind, config.port.into());
        let listener = TcpListener::bind(address).await?;

        config.website.log();

        #[cfg(feature = "series")]
        let series_bodies = query
            .run(|query| Ok(Arc::new(SeriesBodies::new(query))))
            .await?;
        #[cfg(feature = "chain")]
        let mining_pools_body = Arc::new(prepared_json::PreparedJson::new(
            query.sync(|query| query.all_pools()),
        ));

        Ok(Self {
            state: AppState {
                query: query.clone(),
                sync_query: Arc::new(Semaphore::new(1)),
                disk_query: Arc::new(Semaphore::new(1)),
                #[cfg(feature = "chain")]
                raw_block_bodies: Arc::new(Semaphore::new(RawBodyPermit::CAPACITY)),
                #[cfg(feature = "chain")]
                historical_price_bodies: Arc::new(Semaphore::new(2)),
                #[cfg(feature = "chain")]
                historical_price_cache: Arc::default(),
                #[cfg(feature = "chain")]
                mempool_txid_bodies: Arc::new(Semaphore::new(2)),
                #[cfg(feature = "chain")]
                broadcast_requests: Arc::new(Semaphore::new(
                    api::broadcast::BroadcastPermit::CAPACITY,
                )),
                node: query.run(|query| query.client().asynchronous()).await?,
                #[cfg(feature = "series")]
                series_bodies,
                #[cfg(feature = "urpd")]
                urpd_query: Arc::new(Semaphore::new(2)),
                #[cfg(feature = "urpd")]
                urpd_bodies: Arc::new(Semaphore::new(2)),
                #[cfg(feature = "chain")]
                mining_pools_body,
                data_path: config.data_path,
                website: config.website,
                started_at: Timestamp::now(),
                started_instant: Instant::now(),
                max_weight: config.max_weight,
                max_utxos: config.max_utxos,
                cdn_cache_mode: config.cdn_cache_mode,
            },
            listener,
        })
    }

    pub async fn serve(self) -> Result<()> {
        let Self { state, listener } = self;
        let address = listener.local_addr()?;

        let website_router = WebsiteRouter(state.website.clone());
        let mut router = ApiRouter::new()
            .add_api_routes()
            .layer(from_fn(request_deadline::apply));
        if !state.website.is_enabled() {
            router = router.route("/", get(Redirect::temporary("/api")));
        }
        let router = router
            .with_state(state)
            .merge(website_router)
            .layer(from_fn(json_error::respond))
            .layer(compression_layer())
            .layer(CorsLayer::permissive())
            .layer(CatchPanicLayer::custom(|panic: Box<dyn Any + Send>| {
                let msg = panic
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("Unknown panic");
                Error::internal(msg).into_response()
            }))
            .layer(from_fn(response_time::respond));

        info!("Server listening on http://{address}");

        let (router, openapi) = finish_openapi(router);

        let router = router
            .layer(Extension(OpenApiJson::new(&openapi)))
            .layer(Extension(ApiJson::new(&openapi)));

        // NormalizePath must wrap the router (not be a layer) to run before route matching
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);

        serve(
            listener,
            ServiceExt::<Request<Body>>::into_make_service(app),
        )
        .await?;

        Ok(())
    }
}

/// Finalize a router and extract the OpenAPI spec.
pub fn finish_openapi<S: Clone + Send + Sync + 'static>(
    router: ApiRouter<S>,
) -> (Router<S>, OpenApi) {
    let mut openapi = create_openapi();
    let router = router.finish_api(&mut openapi);
    (router, openapi)
}
