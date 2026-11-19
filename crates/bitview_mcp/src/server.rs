use std::{borrow::Cow, future::ready, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    http::{
        HeaderMap, HeaderValue, Method, Request, StatusCode,
        header::{ACCEPT, ALLOW, CACHE_CONTROL, VARY},
    },
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post_service},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        DiscoverResult, ErrorCode, Icon, Implementation, ListToolsResult, MetaObject,
        PaginatedRequestParams, ProtocolVersion, ResourceContents, ServerCapabilities,
        ServerConfig, Tool,
    },
    service::RequestContext,
    transport::{
        common::http_header::HEADER_MCP_PROTOCOL_VERSION,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
        },
    },
};
use serde_json::json;
use tokio::{sync::Semaphore, time::timeout};
use tower_http::{cors::CorsLayer, timeout::RequestBodyTimeoutLayer};
use tracing::debug;

use crate::{
    discovery, logo,
    manifest::Catalog,
    page,
    upstream::{MAX_CALLS, Upstream},
    upstream_response::UpstreamResponse,
};

const CACHE_META_KEY: &str = "org.bitcoinresearchkit/upstreamCache";
const MAX_REQUEST_BYTES: usize = 64 * 1024;
/// The longest a call waits for its turn among the `MAX_CALLS` in flight.
const TURN_WAIT: Duration = Duration::from_secs(5);
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
    calls: Semaphore,
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
    home: Bytes,
) -> Router {
    let transport_config = StreamableHttpServerConfig::default()
        .disable_allowed_hosts()
        .with_sse_keep_alive(None)
        .with_sse_retry(None)
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BYTES);

    // One privacy policy, one set of terms and one support page cover every Bitview service: the
    // website's, beside the API it calls.
    let site_pages =
        ["privacy", "terms", "support"]
            .into_iter()
            .fold(Router::new(), |router, page| {
                let to = Redirect::permanent(&format!("{api_url}/{page}/"));
                router.route(&format!("/{page}"), get(move || ready(to.clone())))
            });
    let home = page::route(home, &api_url);
    let discovery_routes = discovery::router(&display_name, &public_url);
    let server_name = discovery::server_name(&display_name, &public_url);
    let state = Arc::new(AppState {
        catalog,
        upstream: Upstream::new(api_url),
        calls: Semaphore::new(MAX_CALLS),
        display_name,
        public_url,
        server_name,
    });
    let handler = BitviewMcp { state };
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(NeverSessionManager::default()),
        transport_config,
    );

    Router::new()
        .route("/", home.merge(post_service(service)))
        .route("/logo.png", get(logo::get))
        .merge(site_pages)
        .merge(discovery_routes)
        .layer(middleware::from_fn(gateway_guard))
        // A POST's body may not stall: it would hold its connection.
        .layer(RequestBodyTimeoutLayer::new(REQUEST_BODY_TIMEOUT))
        // Any page may call and read any route: the data is public and the server takes no credentials.
        // Outermost, so it answers every preflight itself.
        .layer(CorsLayer::permissive().max_age(PREFLIGHT_MAX_AGE))
}

/// The longest a request body may go without sending more.
const REQUEST_BODY_TIMEOUT: Duration = Duration::from_secs(10);
/// How long browsers may reuse a CORS preflight.
const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

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
            .with_instructions(self.state.catalog.instructions())
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
        // rmcp signals disconnect through the context of its spawned handler.
        context
            .ct
            .run_until_cancelled(self.call(request))
            .await
            .unwrap_or_else(|| Ok(tool_error("Tool call cancelled", &self.state.server_name)))
    }

    fn get_info(&self) -> ServerConfig {
        self.server_info()
    }
}

impl BitviewMcp {
    async fn call(&self, request: CallToolRequestParams) -> Result<CallToolResponse, McpError> {
        let name = request.name;
        let operation = self
            .state
            .catalog
            .operation(&name)
            .ok_or_else(|| McpError::invalid_params("Unknown tool name", None))?;
        let fail = |message| Ok(tool_error(message, &self.state.server_name));
        let url = match self
            .state
            .upstream
            .prepare(operation, &request.arguments.unwrap_or_default())
        {
            Ok(url) => url,
            Err(error) => return fail(error),
        };

        let Ok(Ok(_turn)) = timeout(TURN_WAIT, self.state.calls.acquire()).await else {
            return fail("Bitview MCP is busy; retry shortly".to_string());
        };
        let response = match self.state.upstream.fetch(url).await {
            Ok(response) => response,
            Err(error) => return fail(error),
        };
        log_cache_status(&name, &response);
        Ok(render(response, &self.state.server_name))
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

fn render(response: UpstreamResponse, server_name: &str) -> CallToolResponse {
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

    let is_json = content_type == "application/json" || content_type.ends_with("+json");
    // Pass JSON through unparsed so every upstream number stays exact.
    if is_json || content_type.starts_with("text/") {
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
    // rmcp checks only a version named in `_meta`, and drops notifications unread.
    if request.method() == Method::POST
        && let Some(version) = request.headers().get(HEADER_MCP_PROTOCOL_VERSION)
        && !is_supported(version)
    {
        return unsupported_version(version);
    }

    // `GET /` (and its `HEAD`) is the page. As an event stream it is MCP, which has no stream.
    let landing =
        matches!(*request.method(), Method::GET | Method::HEAD) && request.uri().path() == "/";
    if landing && accepts_event_stream(request.headers()) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            [
                (CACHE_CONTROL, "no-store"),
                (ALLOW, "GET, HEAD, POST"),
                (VARY, "Accept"),
            ],
        )
            .into_response();
    }

    let mut response = next.run(request).await;
    if landing && !varies_by_accept(response.headers()) {
        response
            .headers_mut()
            .append(VARY, HeaderValue::from_static("Accept"));
    }
    response
}

/// Whether a response already says it varies by `Accept` (any `Vary` header naming it, any case).
fn varies_by_accept(headers: &HeaderMap) -> bool {
    headers
        .get_all(VARY)
        .iter()
        .filter_map(|vary| vary.to_str().ok())
        .flat_map(|vary| vary.split(','))
        .any(|name| name.trim().eq_ignore_ascii_case("accept"))
}

/// Whether any `Accept` header names `text/event-stream` as a media range (any case, its parameters aside), not refused
/// with `q=0`.
fn accepts_event_stream(headers: &HeaderMap) -> bool {
    headers
        .get_all(ACCEPT)
        .iter()
        .filter_map(|accept| accept.to_str().ok())
        .flat_map(|accept| split_unquoted(accept, b','))
        .any(|range| {
            let mut parts = split_unquoted(range, b';').into_iter();
            parts
                .next()
                .is_some_and(|media| media.trim().eq_ignore_ascii_case("text/event-stream"))
                && !parts.any(|parameter| {
                    parameter.trim().split_once('=').is_some_and(|(name, q)| {
                        name.trim().eq_ignore_ascii_case("q") && q.trim().parse::<f32>() == Ok(0.0)
                    })
                })
        })
}

/// A header's text split at an ASCII `delimiter` outside its quoted strings (a `"`-quoted value, `\` escaping within it),
/// as RFC 9110 reads a list and its parameters.
fn split_unquoted(text: &str, delimiter: u8) -> Vec<&str> {
    let (mut parts, mut start, mut quoted, mut escaped) = (Vec::new(), 0, false, false);
    for (at, byte) in text.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if quoted && byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            quoted = !quoted;
        } else if !quoted && byte == delimiter {
            parts.push(&text[start..at]);
            start = at + 1;
        }
    }
    parts.push(&text[start..]);
    parts
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
