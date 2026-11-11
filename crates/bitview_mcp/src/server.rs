use std::{
    borrow::Cow,
    future::ready,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    http::{
        HeaderValue, Method, Request, StatusCode,
        header::{ACCEPT, ALLOW, CACHE_CONTROL, ORIGIN, VARY},
        request::Parts,
    },
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post_service},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use http_body_util::BodyExt;
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
use serde_json::{Value, from_str, json};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::spawn_blocking,
    time::timeout,
};
use tower_http::timeout::RequestBodyTimeoutLayer;
use tracing::debug;

use crate::{
    discovery, logo,
    manifest::Catalog,
    page,
    page::Pages,
    upstream::{MAX_RESPONSE_BYTES, Upstream},
    upstream_response::UpstreamResponse,
};

const CACHE_META_KEY: &str = "org.bitcoinresearchkit/upstreamCache";
const MAX_REQUEST_BYTES: usize = 64 * 1024;
/// Bytes of large upstream bodies that calls may hold at once, from reading the body until the response is written.
/// Rendering peaks at about ten times the body (the parsed `Value` and the serialized response), so this bounds them
/// near 1.3 GB, as 16 calls of the largest bodies did. Small bodies, which end within the first read, take none:
/// their memory is bounded by the calls in flight.
const BODY_BUDGET: u64 = 16 * MAX_RESPONSE_BYTES;
/// The longest a call waits for its body's share of the budget.
const BUDGET_WAIT: Duration = Duration::from_secs(5);
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
    budget: Arc<Semaphore>,
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

    // One privacy policy and one set of terms cover every Bitview service: the website's, beside the
    // API it calls.
    let redirect = |page: &str| {
        let to = Redirect::permanent(&format!("{api_url}/{page}/"));
        get(move || ready(to.clone()))
    };
    let [privacy, terms] = ["privacy", "terms"].map(redirect);
    let discovery_routes = discovery::router(&display_name, &public_url);
    let server_name = discovery::server_name(&display_name, &public_url);
    let state = Arc::new(AppState {
        catalog,
        upstream: Upstream::new(api_url),
        budget: Arc::new(Semaphore::new(BODY_BUDGET as usize)),
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
            page::route(pages.home)
                .merge(post_service(service).layer(middleware::from_fn(hold_charge))),
        )
        .route("/logo.png", get(logo::get))
        .route("/privacy", privacy)
        .route("/terms", terms)
        .route("/support", page::route(pages.support))
        .merge(discovery_routes)
        .layer(middleware::from_fn(gateway_guard))
        // A POST's body may not stall: it would hold its connection.
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

        let charge = context
            .extensions
            .get::<Parts>()
            .and_then(|parts| parts.extensions.get::<Charge>())
            .cloned();
        debug_assert!(
            charge.is_some(),
            "MCP POSTs get a charge slot before dispatch"
        );
        let state = Arc::clone(&self.state);
        let fail = |message| Ok(tool_error(message, &self.state.server_name));

        let fetched = {
            let state = Arc::clone(&state);
            spawn_blocking(move || state.upstream.fetch(url)).await
        };
        let partial = match fetched {
            Ok(Ok(partial)) => partial,
            Ok(Err(error)) => return fail(error),
            Err(_) => return fail("Bitview API request task failed".to_string()),
        };
        // A large body reserves the most it can take before the rest is read, and waits for that room here, holding
        // no thread.
        let permit = if partial.is_large() {
            let reserve = Arc::clone(&state.budget).acquire_many_owned(MAX_RESPONSE_BYTES as u32);
            let Ok(Ok(permit)) = timeout(BUDGET_WAIT, reserve).await else {
                return fail("Bitview MCP is busy with large responses; retry shortly".to_string());
            };
            Some(permit)
        } else {
            None
        };
        Ok(spawn_blocking(move || {
            let response = match partial.finish() {
                Ok(response) => response,
                Err(error) => return tool_error(error, &state.server_name),
            };
            // The reservation shrinks to the body's length, held until the response is written.
            let permit = permit.map(|mut permit| {
                drop(permit.split(permit.num_permits() - response.body.len()));
                permit
            });
            log_cache_status(&name, &response);
            let result = render(response, has_output_schema, &state.server_name);
            if let (Some(charge), Some(permit)) = (charge, permit) {
                let _ = charge.set(permit);
            }
            result
        })
        .await
        .unwrap_or_else(|_| tool_error("Bitview API request task failed", &self.state.server_name)))
    }

    fn get_info(&self) -> ServerConfig {
        self.server_info()
    }
}

/// A call's share of `BODY_BUDGET`, set once its body is rendered.
type Charge = Arc<OnceLock<OwnedSemaphorePermit>>;

/// Gives each MCP POST a `Charge` slot, held by every data frame of its response: Hyper may still be writing the last
/// one after the body is dropped, and the bytes it writes are what the charge pays for.
async fn hold_charge(mut request: Request<Body>, next: Next) -> Response {
    let charge = Charge::default();
    request.extensions_mut().insert(charge.clone());
    next.run(request).await.map(|body| {
        Body::new(body.map_frame(move |frame| {
            let charge = charge.clone();
            frame.map_data(|bytes| {
                Bytes::from_owner(ChargedBytes {
                    bytes,
                    _charge: charge,
                })
            })
        }))
    })
}

struct ChargedBytes {
    bytes: Bytes,
    _charge: Charge,
}

impl AsRef<[u8]> for ChargedBytes {
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

    let is_json = content_type == "application/json" || content_type.ends_with("+json");
    if is_json && has_output_schema {
        // The text content is the upstream body itself, so integers beyond u64 (u128 hashrates) stay
        // exact there. The structured `Value` holds those as f64: exact numbers would need serde_json's
        // arbitrary_precision, which feature unification would leak into every workspace build.
        // Checked as UTF-8 once: parsing a `str` skips serde_json's own check of each string.
        let parsed = String::from_utf8(response.body)
            .ok()
            .and_then(|text| Some((from_str::<Value>(&text).ok()?, text)));
        return match parsed {
            Some((value, text)) => {
                let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
                result.structured_content = Some(value);
                result.with_meta(meta).into()
            }
            None => tool_error("Bitview API returned invalid JSON", server_name),
        };
    }

    // JSON without an output schema is passed on as the text it is, unparsed.
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

    // `GET /` (and its `HEAD`) is the page. As an event stream it is MCP, which has no stream.
    let landing = matches!(*request.method(), Method::GET | Method::HEAD) && request.uri().path() == "/";
    if landing
        && request
            .headers()
            .get(ACCEPT)
            .and_then(|accept| accept.to_str().ok())
            .is_some_and(|accept| accept.contains("text/event-stream"))
    {
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
    if landing {
        response.headers_mut().append(VARY, HeaderValue::from_static("Accept"));
    }
    response
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
