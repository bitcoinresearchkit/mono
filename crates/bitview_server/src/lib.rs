#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    net::SocketAddr,
    time::{Duration, Instant},
};

#[cfg(feature = "bindgen")]
use std::path::PathBuf;

use aide::axum::ApiRouter;
use axum::{
    Extension, ServiceExt,
    body::Body,
    http::{
        Request, Response, StatusCode, Uri,
        header::{ALLOW, CONTENT_TYPE},
    },
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::get,
    serve,
};
use bitview_query::AsyncQuery;
use brk_error::Result;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer,
    classify::ServerErrorsFailureClass,
    compression::{
        CompressionLayer, CompressionLevel,
        predicate::{DefaultPredicate, Predicate, SizeAbove},
    },
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_layer::Layer;
use tracing::{debug, error, info};

mod api;
mod cache;
mod config;
mod error;
mod error_body;
mod etag;
mod extended;
mod params;
mod state;

pub use api::ApiRoutes;
use api::*;
pub use bitview_website::Website;
pub use brk_types::Port;
pub use cache::CdnCacheMode;
use cache::{CacheParams, CacheStrategy};
pub use config::{DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, ServerConfig};
pub use error::Error;
use state::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cap for buffering an upstream error body before re-wrapping it as JSON.
/// Larger bodies are truncated; the bound only affects the message we surface.
const MAX_ERROR_BODY_BYTES: usize = 4096;

/// Per-request timeout. Hits return 504 Gateway Timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Avoid spending compression work on responses too small to benefit materially.
const MIN_COMPRESSED_RESPONSE_BYTES: u64 = 1024;

/// Matches `application/json` and `application/...+json`, ignoring parameters
/// like `; charset=utf-8`. Used to skip JSON-error rewriting for already-JSON bodies.
fn is_json_content_type(s: &str) -> bool {
    let mime = s.split(';').next().unwrap_or("").trim();
    mime == "application/json" || (mime.starts_with("application/") && mime.ends_with("+json"))
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
        cache::init(config.cdn_cache_mode);

        Ok(Self {
            state: AppState {
                query: query.clone(),
                data_path: config.data_path,
                website: config.website,
                started_at: jiff::Timestamp::now(),
                started_instant: Instant::now(),
                max_weight: config.max_weight,
                max_utxos: config.max_utxos,
            },
            listener,
        })
    }

    pub async fn serve(self) -> Result<()> {
        let Self { state, listener } = self;
        let address = listener.local_addr()?;

        #[cfg(feature = "bindgen")]
        let vecs = state.query.inner().vecs();

        let compression_layer = CompressionLayer::new()
            .br(true)
            .gzip(true)
            .zstd(true)
            .quality(CompressionLevel::Fastest)
            .compress_when(
                DefaultPredicate::new().and(SizeAbove::new(MIN_COMPRESSED_RESPONSE_BYTES)),
            );

        let response_time_layer = axum::middleware::from_fn(
            async |request: Request<Body>, next: Next| -> Response<Body> {
                let uri = request.uri().clone();
                let method = request.method().clone();
                let start = Instant::now();
                let mut response = next.run(request).await;
                response.extensions_mut().insert(uri);
                response.extensions_mut().insert(method);
                response.headers_mut().insert(
                    "X-Response-Time",
                    format!("{}us", start.elapsed().as_micros())
                        .parse()
                        .unwrap(),
                );
                response
            },
        );

        // Wrap non-JSON error responses in structured JSON
        let json_error_layer = axum::middleware::from_fn(
            async |request: Request<Body>, next: Next| -> Response<Body> {
                let response = next.run(request).await;
                let status = response.status();
                if status.is_success()
                    || status.is_redirection()
                    || status.is_informational()
                    || response
                        .headers()
                        .get(CONTENT_TYPE)
                        .is_some_and(|v| v.to_str().is_ok_and(is_json_content_type))
                {
                    return response;
                }

                let (parts, body) = response.into_parts();
                let bytes = axum::body::to_bytes(body, MAX_ERROR_BODY_BYTES)
                    .await
                    .unwrap_or_default();
                let msg = String::from_utf8_lossy(&bytes);
                let (code, msg) = match parts.status {
                    StatusCode::NOT_FOUND => (
                        "not_found",
                        if msg.is_empty() {
                            "Not found".into()
                        } else {
                            msg
                        },
                    ),
                    StatusCode::METHOD_NOT_ALLOWED => (
                        "method_not_allowed",
                        "Only GET requests are supported".into(),
                    ),
                    StatusCode::GATEWAY_TIMEOUT => ("timeout", "Request timed out".into()),
                    s if s.is_client_error() => (
                        "bad_request",
                        if msg.is_empty() {
                            "Bad request".into()
                        } else {
                            msg
                        },
                    ),
                    _ => (
                        "internal_error",
                        if msg.is_empty() {
                            "Internal server error".into()
                        } else {
                            msg
                        },
                    ),
                };
                let msg = msg.into_owned();
                let mut response = error::new(parts.status, code, msg).into_response();
                response.extensions_mut().extend(parts.extensions);
                if let Some(allow) = parts.headers.get(ALLOW) {
                    response.headers_mut().insert(ALLOW, allow.clone());
                }
                response
            },
        );

        let trace_layer = TraceLayer::new_for_http()
            .on_request(())
            .on_response(
                |response: &Response<Body>, latency: Duration, _: &tracing::Span| {
                    let status = response.status().as_u16();
                    let unknown_uri = Uri::from_static("/unknown");
                    let unknown_method = axum::http::Method::default();
                    let uri = response.extensions().get::<Uri>().unwrap_or(&unknown_uri);
                    let method = response
                        .extensions()
                        .get::<axum::http::Method>()
                        .unwrap_or(&unknown_method);
                    match response.status() {
                        StatusCode::NOT_MODIFIED | StatusCode::BAD_REQUEST => {
                            debug!(%method, status, %uri, ?latency)
                        }
                        status_code
                            if status_code.is_informational()
                                || status_code.is_success()
                                || status_code.is_redirection() =>
                        {
                            info!(%method, status, %uri, ?latency)
                        }
                        _ => error!(%method, status, %uri, ?latency),
                    }
                },
            )
            .on_body_chunk(())
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _: &tracing::Span| {
                    let latency = format!("{latency:.2?}");
                    error!(?error, %latency, "Request failed");
                },
            )
            .on_eos(());

        let website_router = bitview_website::router(state.website.clone());
        let mut router = ApiRouter::new()
            .add_api_routes()
            .layer(TimeoutLayer::with_status_code(
                StatusCode::GATEWAY_TIMEOUT,
                REQUEST_TIMEOUT,
            ));
        if !state.website.is_enabled() {
            router = router.route("/", get(Redirect::temporary("/api")));
        }
        let router = router
            .with_state(state)
            .merge(website_router)
            .layer(json_error_layer)
            .layer(compression_layer)
            .layer(CorsLayer::permissive())
            .layer(CatchPanicLayer::custom(|panic: Box<dyn Any + Send>| {
                let msg = panic
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("Unknown panic");
                Error::internal(msg).into_response()
            }))
            .layer(response_time_layer)
            .layer(trace_layer);

        info!("Server listening on http://{address}");

        let (router, openapi) = finish_openapi(router);

        #[cfg(feature = "bindgen")]
        {
            let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|p| p.parent())
                .unwrap()
                .to_path_buf();

            let output_paths = bitview_bindgen::ClientOutputPaths::new()
                .rust(workspace_root.join("crates/bitview_client/src/generated.rs"))
                .javascript(workspace_root.join("modules/bitview-client/index.js"))
                .python(workspace_root.join("packages/bitview_client/bitview_client/__init__.py"))
                .llm(workspace_root.join("website"))
                .llm(workspace_root.join("website_next"));

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                generate_bindings(vecs, &openapi, &output_paths)
            }));

            match result {
                Ok(Ok(())) => debug!("Generated clients"),
                Ok(Err(e)) => error!("Failed to generate clients: {e}"),
                Err(_) => error!("Client generation panicked"),
            }
        }

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
) -> (axum::Router<S>, aide::openapi::OpenApi) {
    let mut openapi = create_openapi();
    let router = router.finish_api(&mut openapi);
    (router, openapi)
}

#[cfg(feature = "bindgen")]
pub fn generate_bindings(
    vecs: &bitview_query::Vecs,
    openapi: &aide::openapi::OpenApi,
    output_paths: &bitview_bindgen::ClientOutputPaths,
) -> std::io::Result<()> {
    let openapi_json = serde_json::to_string(openapi)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let output_paths = if output_paths.llm_manifest.is_some() {
        output_paths.clone()
    } else {
        output_paths.clone().llm_manifest(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../bitview_mcp/generated/manifest.json"),
        )
    };
    bitview_bindgen::generate_clients(vecs, &openapi_json, &output_paths)
}

#[cfg(test)]
mod tests {
    use super::is_json_content_type;

    #[test]
    fn json_content_type_matches() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/json; charset=utf-8"));
        assert!(is_json_content_type("  application/json  "));
        assert!(is_json_content_type("application/problem+json"));
        assert!(is_json_content_type(
            "application/vnd.api+json; charset=utf-8"
        ));
    }

    #[test]
    fn json_content_type_rejects_non_json() {
        assert!(!is_json_content_type("text/plain"));
        assert!(!is_json_content_type("application/xml"));
        assert!(!is_json_content_type("application/json+xml"));
        assert!(!is_json_content_type(""));
        assert!(!is_json_content_type("text/json"));
    }
}
