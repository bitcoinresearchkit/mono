use std::{borrow::Cow, future::ready, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::State,
    http::{
        HeaderValue, Method, Request, StatusCode,
        header::{ACCEPT, ORIGIN, RETRY_AFTER},
        request::Parts,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post_service},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use http_body_util::BodyExt;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        DiscoverResult, ErrorCode, Icon, Implementation, InitializeRequestParams, InitializeResult,
        ListToolsResult, MetaObject, PaginatedRequestParams, ProtocolVersion, ResourceContents,
        ServerCapabilities, ServerConfig, Tool,
    },
    service::RequestContext,
    transport::{
        common::http_header::HEADER_MCP_PROTOCOL_VERSION,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
        },
    },
};
use serde_json::{Value, from_slice, json};
use tower_http::timeout::RequestBodyTimeoutLayer;
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::spawn_blocking,
    time::timeout,
};
use tracing::debug;

use crate::{
    discovery, logo, manifest::Catalog, page, page::Pages, upstream::Upstream,
    upstream_response::UpstreamResponse,
};

const CACHE_META_KEY: &str = "org.bitcoinresearchkit/upstreamCache";

#[cfg(test)]
mod optional_response_tests;
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_CONCURRENCY: usize = 16;
const CONCURRENCY_WAIT: Duration = Duration::from_secs(5);
const CATALOG_TTL_MS: u64 = 3_600_000;
const MAX_UPSTREAM_ERROR_BYTES: usize = 2_048;
/// The newest revision, back to the oldest one a real client still opens with: Codex and Gemini CLI
/// at 2025-06-18. Every revision is served without sessions: `initialize` is one more request.
pub(crate) static SUPPORTED_PROTOCOL_VERSIONS: [ProtocolVersion; 3] = [
    ProtocolVersion::V_2026_07_28,
    ProtocolVersion::V_2025_11_25,
    ProtocolVersion::V_2025_06_18,
];

struct AppState {
    catalog: Catalog,
    upstream: Upstream,
    display_name: String,
    public_url: String,
    server_name: String,
}

#[derive(Clone)]
struct BitviewMcp {
    state: Arc<AppState>,
}

pub fn router(
    api_url: String,
    catalog: Catalog,
    display_name: String,
    public_url: String,
    pages: Pages,
) -> Router {
    let transport_config = StreamableHttpServerConfig::default()
        .disable_allowed_hosts()
        .with_sse_keep_alive(None)
        .with_sse_retry(None)
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BYTES);

    let discovery_routes = discovery::router(&display_name, &public_url);
    let server_name = discovery::server_name(&display_name, &public_url);
    let state = Arc::new(AppState {
        catalog,
        upstream: Upstream::new(api_url),
        display_name,
        public_url,
        server_name,
    });
    let handler = BitviewMcp {
        state: state.clone(),
    };
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(NeverSessionManager::default()),
        transport_config,
    );

    Router::new()
        .route(
            "/",
            page::route(pages.home).merge(post_service(service).layer(
                middleware::from_fn_with_state(Arc::new(Semaphore::new(MAX_CONCURRENCY)), admit),
            )),
        )
        .route("/logo.png", get(logo::get))
        .route("/privacy", page::route(pages.privacy))
        .route("/terms", page::route(pages.terms))
        .route("/support", page::route(pages.support))
        .merge(discovery_routes)
        .layer(middleware::from_fn(gateway_guard))
        // A POST's body may not stall: it would hold its admission permit.
        .layer(RequestBodyTimeoutLayer::new(REQUEST_BODY_TIMEOUT))
}

/// The longest a request body may go without sending more.
const REQUEST_BODY_TIMEOUT: Duration = Duration::from_secs(10);

impl BitviewMcp {
    fn server_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(
                Implementation::new(&self.state.server_name, env!("CARGO_PKG_VERSION"))
                    .with_title(&self.state.display_name)
                    .with_icons(vec![
                        Icon::new(format!("{}{}", self.state.public_url, logo::PATH))
                            .with_mime_type("image/png")
                            .with_sizes(vec!["512x512".to_owned()]),
                    ])
                    .with_website_url(&self.state.public_url),
            )
            .with_instructions(
                "Live and historical Bitcoin data from Bitview, read from a full node: on-chain \
                 metrics (MVRV, realized price, SOPR, holder cohorts, supply by age), blocks, \
                 transactions, addresses, mempool and fees, mining and difficulty, and the BTC/USD \
                 price computed from the chain. Free and read-only, with no API key.\n\n\
                 For a metric, `search_series` with plain words returns series ids, which \
                 `get_series_latest` and `get_series` read; only ids it returned exist. Source of \
                 this data: Bitview (https://bitview.space). Binary responses are returned as \
                 embedded MCP resources.",
            )
    }
}

impl ServerHandler for BitviewMcp {
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&SUPPORTED_PROTOCOL_VERSIONS)
    }

    fn discover(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<DiscoverResult, McpError>> + Send + '_ {
        ready(Ok(DiscoverResult::from_server_info(
            SUPPORTED_PROTOCOL_VERSIONS.to_vec(),
            self.server_info(),
        )
        .with_ttl_ms(CATALOG_TTL_MS)
        .with_cache_scope(CacheScope::Public)))
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if !SUPPORTED_PROTOCOL_VERSIONS.contains(&request.protocol_version) {
            return Err(McpError::unsupported_protocol_version(
                request.protocol_version,
                &SUPPORTED_PROTOCOL_VERSIONS,
            ));
        }
        context.peer.set_peer_info(request.clone());
        self.negotiate_initialize(&request)
    }

    fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let result = if request.and_then(|request| request.cursor).is_some() {
            Err(McpError::invalid_params(
                "The tool catalog does not accept a cursor",
                None,
            ))
        } else {
            let mut result = ListToolsResult::with_all_items(self.state.catalog.tools().to_vec())
                .with_ttl_ms(CATALOG_TTL_MS)
                .with_cache_scope(CacheScope::Public);
            result.meta = Some(server_meta(&self.state.server_name));
            Ok(result)
        };
        ready(result)
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.state
            .catalog
            .operation(name)
            .map(|operation| operation.tool.clone())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let name = request.name;
        let operation = self
            .state
            .catalog
            .operation(&name)
            .ok_or_else(|| McpError::invalid_params("Unknown tool name", None))?;
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        operation
            .validate_arguments(&arguments)
            .map_err(|error| McpError::invalid_params(error, None))?;
        let has_output_schema = operation.tool.output_schema.is_some();
        let url = self
            .state
            .upstream
            .prepare(operation, &arguments)
            .map_err(|error| McpError::invalid_params(error, None))?;

        let permit = context
            .extensions
            .get::<Parts>()
            .and_then(|parts| parts.extensions.get::<Arc<OwnedSemaphorePermit>>())
            .cloned();
        debug_assert!(permit.is_some(), "MCP POSTs are admitted before dispatch");
        let state = Arc::clone(&self.state);
        Ok(spawn_blocking(move || {
            let _permit = permit;
            match state.upstream.fetch(url) {
                Ok(response) => {
                    log_cache_status(&name, &response);
                    render(response, has_output_schema, &state.server_name)
                }
                Err(error) => tool_error(error, &state.server_name),
            }
        })
        .await
        .unwrap_or_else(|_| tool_error("Bitview API request task failed", &self.state.server_name)))
    }

    fn get_info(&self) -> ServerConfig {
        self.server_info()
    }
}

/// Admits each MCP POST under one of `MAX_CONCURRENCY` permits, held by the response body and by the request's
/// blocking upstream work, so fetching, rendering, serialization and transmission stay bounded, cancelled requests
/// included.
async fn admit(
    State(concurrency): State<Arc<Semaphore>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let Ok(Ok(permit)) = timeout(CONCURRENCY_WAIT, concurrency.acquire_owned()).await else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(RETRY_AFTER, "1")],
            "MCP proxy is temporarily busy; retry shortly",
        )
            .into_response();
    };
    let permit = Arc::new(permit);
    request.extensions_mut().insert(permit.clone());
    // Every data frame co-owns the permit: Hyper may still be writing the last one after the body is dropped.
    next.run(request).await.map(|body| {
        Body::new(body.map_frame(move |frame| {
            let permit = permit.clone();
            frame.map_data(|bytes| {
                Bytes::from_owner(PermitBytes {
                    bytes,
                    _permit: permit,
                })
            })
        }))
    })
}

struct PermitBytes {
    bytes: Bytes,
    _permit: Arc<OwnedSemaphorePermit>,
}

impl AsRef<[u8]> for PermitBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

fn tool_error(message: impl Into<String>, server_name: &str) -> CallToolResponse {
    CallToolResult::error(vec![ContentBlock::text(message)])
        .with_meta(Some(server_meta(server_name)))
        .into()
}

fn log_cache_status(tool: &str, response: &UpstreamResponse) {
    if let Some(cache_status) = &response.cache_status {
        debug!(
            tool,
            cache_status,
            cache_age = response.cache_age.as_deref().unwrap_or(""),
            "Bitview API response"
        );
    }
}

fn render(
    response: UpstreamResponse,
    has_output_schema: bool,
    server_name: &str,
) -> CallToolResponse {
    if !(200..300).contains(&response.status) {
        let detail = String::from_utf8_lossy(
            &response.body[..response.body.len().min(MAX_UPSTREAM_ERROR_BYTES)],
        );
        let detail = detail.trim();
        let message = if detail.is_empty() {
            format!("Bitview API returned HTTP {}", response.status)
        } else {
            format!("Bitview API returned HTTP {}: {detail}", response.status)
        };
        return tool_error(message, server_name);
    }

    let content_type = response
        .content_type
        .split(';')
        .next()
        .unwrap()
        .trim()
        .to_ascii_lowercase();
    let meta = Some(upstream_meta(&response, server_name));

    // An empty 204 is success regardless of Content-Type; never attempt JSON decoding.
    if response.status == 204 {
        return CallToolResult::success(vec![]).with_meta(meta).into();
    }

    if content_type == "application/json" || content_type.ends_with("+json") {
        // The text content is the upstream body itself, so integers beyond u64 (u128 hashrates) stay
        // exact there. The structured `Value` holds those as f64: exact numbers would need serde_json's
        // arbitrary_precision, which feature unification would leak into every workspace build.
        return match (
            from_slice::<Value>(&response.body),
            String::from_utf8(response.body),
        ) {
            (Ok(value), Ok(text)) => {
                let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
                result.structured_content = has_output_schema.then_some(value);
                result.with_meta(meta).into()
            }
            _ => tool_error("Bitview API returned invalid JSON", server_name),
        };
    }

    if content_type.starts_with("text/") {
        return match String::from_utf8(response.body) {
            Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)])
                .with_meta(meta)
                .into(),
            Err(_) => tool_error("Bitview API returned invalid UTF-8 text", server_name),
        };
    }

    let resource = ResourceContents::blob(BASE64_STANDARD.encode(response.body), response.url)
        .with_mime_type(content_type);
    CallToolResult::success(vec![ContentBlock::resource(resource)])
        .with_meta(meta)
        .into()
}

async fn gateway_guard(request: Request<Body>, next: Next) -> Response {
    if request.method() == Method::POST && request.headers().contains_key(ORIGIN) {
        return (StatusCode::FORBIDDEN, "Forbidden origin").into_response();
    }

    // rmcp checks only a version named in `_meta`, and drops notifications unread.
    if request.method() == Method::POST
        && let Some(version) = request.headers().get(HEADER_MCP_PROTOCOL_VERSION)
        && !is_supported(version)
    {
        return unsupported_version(version);
    }

    // `GET /` is the page. Asked for as an event stream it's the MCP endpoint, which has no stream.
    if request.method() == Method::GET
        && request.uri().path() == "/"
        && request
            .headers()
            .get(ACCEPT)
            .and_then(|accept| accept.to_str().ok())
            .is_some_and(|accept| accept.contains("text/event-stream"))
    {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    next.run(request).await
}

fn is_supported(version: &HeaderValue) -> bool {
    SUPPORTED_PROTOCOL_VERSIONS
        .iter()
        .any(|supported| supported.as_str().as_bytes() == version.as_bytes())
}

fn unsupported_version(version: &HeaderValue) -> Response {
    let error = McpError::new(
        ErrorCode::UNSUPPORTED_PROTOCOL_VERSION,
        "Unsupported protocol version",
        Some(json!({
            "requested": String::from_utf8_lossy(version.as_bytes()),
            "supported": SUPPORTED_PROTOCOL_VERSIONS,
        })),
    );
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "jsonrpc": "2.0", "id": null, "error": error })),
    )
        .into_response()
}

fn server_meta(server_name: &str) -> MetaObject {
    let mut meta = MetaObject::new();
    meta.insert(
        "io.modelcontextprotocol/serverInfo".to_string(),
        json!({
            "name": server_name,
            "version": env!("CARGO_PKG_VERSION"),
        }),
    );
    meta
}

fn upstream_meta(response: &UpstreamResponse, server_name: &str) -> MetaObject {
    let mut meta = server_meta(server_name);
    meta.insert(
        CACHE_META_KEY.to_string(),
        json!({
            "status": response.cache_status,
            "age": response.cache_age,
        }),
    );
    meta
}
