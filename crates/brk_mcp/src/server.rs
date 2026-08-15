use std::{borrow::Cow, sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header::ORIGIN},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        DiscoverResult, Implementation, ListToolsResult, MetaObject, PaginatedRequestParams,
        ProtocolVersion, ResourceContents, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    },
};
use serde_json::{Value, json};
use tokio::sync::Semaphore;
use tracing::{info, warn};

use crate::{
    manifest::Catalog,
    upstream::{Upstream, UpstreamResponse},
};

const CACHE_META_KEY: &str = "space.bitview/upstreamCache";
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_CONCURRENCY: usize = 16;
const CONCURRENCY_WAIT: Duration = Duration::from_secs(5);
const CATALOG_TTL_MS: u64 = 3_600_000;
const MAX_UPSTREAM_ERROR_BYTES: usize = 2_048;
static SUPPORTED_PROTOCOL_VERSIONS: [ProtocolVersion; 1] = [ProtocolVersion::V_2026_07_28];

struct AppState {
    catalog: Catalog,
    upstream: Upstream,
    concurrency: Arc<Semaphore>,
}

#[derive(Clone)]
struct BrkMcp {
    state: Arc<AppState>,
}

impl AppState {
    fn new(api_bases: Vec<String>, catalog: Catalog) -> Self {
        let upstream = Upstream::new(api_bases);
        Self {
            concurrency: Arc::new(Semaphore::new(MAX_CONCURRENCY)),
            catalog,
            upstream,
        }
    }
}

pub fn router(api_bases: Vec<String>, catalog: Catalog) -> Router {
    let transport_config = StreamableHttpServerConfig::default()
        .disable_allowed_hosts()
        .with_sse_keep_alive(None)
        .with_sse_retry(None)
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BYTES)
        .with_stateless_protocol_metadata_required(true);

    let state = Arc::new(AppState::new(api_bases, catalog));
    let handler = BrkMcp {
        state: state.clone(),
    };
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(NeverSessionManager::default()),
        transport_config,
    );

    Router::new()
        .route("/", get(crate::page::get).post_service(service))
        .layer(middleware::from_fn(gateway_guard))
}

impl BrkMcp {
    fn server_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(Implementation::new("brk_mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Read-only Bitcoin analytics tools backed by BRK REST GET endpoints. \
                 Calls use the configured public Cloudflare-cached API and never mutate BRK state. \
                 Tool descriptions preserve the OpenAPI documentation and identify their underlying \
                 REST operation. Use available list, search, and info operations to discover \
                 identifiers before querying dynamic datasets. Binary responses are returned as \
                 embedded MCP resources.",
            )
    }

    fn tool_error(&self, message: impl Into<String>) -> CallToolResponse {
        CallToolResult::error(vec![ContentBlock::text(message)])
            .with_meta(Some(server_meta()))
            .into()
    }

    fn render_upstream(
        &self,
        response: UpstreamResponse,
        has_output_schema: bool,
    ) -> CallToolResponse {
        if !(200..300).contains(&response.status) {
            let detail = String::from_utf8_lossy(
                &response.body[..response.body.len().min(MAX_UPSTREAM_ERROR_BYTES)],
            );
            let detail = detail.trim();
            let message = if detail.is_empty() {
                format!("BRK API returned HTTP {}", response.status)
            } else {
                format!("BRK API returned HTTP {}: {detail}", response.status)
            };
            return self.tool_error(message);
        }

        let content_type = response
            .content_type
            .split(';')
            .next()
            .unwrap_or("application/octet-stream")
            .trim()
            .to_ascii_lowercase();
        let meta = Some(upstream_meta(&response));

        if content_type == "application/json" || content_type.ends_with("+json") {
            return match serde_json::from_slice::<Value>(&response.body) {
                Ok(value) => CallToolResult::structured(value).with_meta(meta).into(),
                Err(_) => self.tool_error("BRK API returned invalid JSON"),
            };
        }

        if content_type.starts_with("text/") {
            return match String::from_utf8(response.body) {
                Ok(text) => {
                    let mut result =
                        CallToolResult::success(vec![ContentBlock::text(text.clone())]);
                    if has_output_schema {
                        result.structured_content = Some(Value::String(text));
                    }
                    result.with_meta(meta).into()
                }
                Err(_) => self.tool_error("BRK API returned invalid UTF-8 text"),
            };
        }

        let resource = ResourceContents::blob(BASE64_STANDARD.encode(response.body), response.url)
            .with_mime_type(content_type);
        CallToolResult::success(vec![ContentBlock::resource(resource)])
            .with_meta(meta)
            .into()
    }
}

impl ServerHandler for BrkMcp {
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&SUPPORTED_PROTOCOL_VERSIONS)
    }

    fn discover(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<DiscoverResult, McpError>> + Send + '_ {
        std::future::ready(Ok(DiscoverResult::from_server_info(
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
                "This complete tool catalog does not accept a cursor",
                None,
            ))
        } else {
            let mut result = ListToolsResult::with_all_items(self.state.catalog.tools().to_vec())
                .with_ttl_ms(CATALOG_TTL_MS)
                .with_cache_scope(CacheScope::Public);
            result.meta = Some(server_meta());
            Ok(result)
        };
        std::future::ready(result)
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
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let name = request.name;
        let operation = self
            .state
            .catalog
            .operation(&name)
            .ok_or_else(|| McpError::invalid_params("Unknown tool name", None))?;
        let arguments = request.arguments.unwrap_or_default();
        let arguments = operation
            .validate_arguments(&arguments)
            .map_err(|error| McpError::invalid_params(error, None))?;
        let has_output_schema = operation.tool.output_schema.is_some();
        let prepared = self
            .state
            .upstream
            .prepare(operation, arguments)
            .map_err(|error| McpError::invalid_params(error, None))?;
        let permit = match tokio::time::timeout(
            CONCURRENCY_WAIT,
            self.state.concurrency.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Ok(self.tool_error("MCP proxy is unavailable")),
            Err(_) => {
                return Ok(
                    self.tool_error("MCP proxy is temporarily busy; retry this tool call shortly")
                );
            }
        };

        let upstream = self.state.upstream.clone();
        let response = match tokio::task::spawn_blocking(move || {
            let _permit = permit;
            upstream.fetch(prepared)
        })
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => return Ok(self.tool_error(error)),
            Err(_) => return Ok(self.tool_error("BRK API request task failed")),
        };
        if let Some(cache_status) = &response.cache_status {
            info!(
                tool = name.as_ref(),
                cache_status,
                cache_age = response.cache_age.as_deref().unwrap_or(""),
                "BRK API response"
            );
        } else {
            warn!(
                tool = name.as_ref(),
                "BRK API response did not include CF-Cache-Status"
            );
        }
        Ok(self.render_upstream(response, has_output_schema))
    }

    fn get_info(&self) -> ServerInfo {
        self.server_info()
    }
}

async fn gateway_guard(request: Request<Body>, next: Next) -> Response {
    if request.method() == Method::POST && request.headers().contains_key(ORIGIN) {
        return (StatusCode::FORBIDDEN, "Forbidden origin").into_response();
    }

    next.run(request).await
}

fn server_meta() -> MetaObject {
    let mut meta = MetaObject::new();
    meta.insert(
        "io.modelcontextprotocol/serverInfo".to_string(),
        json!({
            "name": "brk_mcp",
            "version": env!("CARGO_PKG_VERSION"),
        }),
    );
    meta
}

fn upstream_meta(response: &UpstreamResponse) -> MetaObject {
    let mut meta = server_meta();
    meta.insert(
        CACHE_META_KEY.to_string(),
        json!({
            "status": response.cache_status,
            "age": response.cache_age,
        }),
    );
    meta
}
