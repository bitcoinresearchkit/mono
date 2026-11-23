//! Bitview's MCP server at `/mcp`: stateless and read-only, one tool per MCP-visible API operation. A tool call is
//! the API request it names, sent to this server's own router: the same handler, admission and answer as over HTTP,
//! without a socket.

use std::{future::ready, sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    http::{Request, header::ACCEPT},
    middleware,
    response::Redirect,
    routing::{any, get, post},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use serde_json::{Map, Value, json};
use tower::ServiceExt;
use tower_http::{cors::CorsLayer, timeout::RequestBodyTimeoutLayer};

use catalog::Catalog;
use protocol::{Results, ToolResult, server_meta};
use transport::{BodyError, read_capped};

mod catalog;
mod discovery;
mod logo;
mod page;
mod protocol;
mod tools;
mod transport;

pub use protocol::SUPPORTED_PROTOCOL_VERSIONS;

/// Where the server is mounted. `/mcp/` is not a second spelling: it redirects here.
pub const PATH: &str = "/mcp";
/// The name the server goes by on its page, its card and in its info.
const DISPLAY_NAME: &str = "Bitview";
/// The largest answer a call returns: every default answer fits (the largest, a raw URPD of all coins, is 1.3 MB),
/// and more is past what an assistant reads.
const MAX_ANSWER_BYTES: usize = 2 * 1024 * 1024;
const MAX_API_ERROR_BYTES: usize = 2_048;
/// The longest a request body may go without sending more.
const REQUEST_BODY_TIMEOUT: Duration = Duration::from_secs(10);
/// How long browsers may reuse a CORS preflight.
const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub struct AppState {
    catalog: Catalog,
    /// The API, with every layer it answers through.
    api: Router,
    /// The public origin, which names a binary answer's resource.
    site_url: String,
    server_meta: Map<String, Value>,
    results: Results,
}

/// The MCP routes, for `site_url` (the public origin, no trailing slash), calling `api`.
pub fn router(api: Router, site_url: &str) -> Router {
    let catalog = Catalog::embedded();
    let public_url = format!("{site_url}{PATH}");
    let logo_url = format!("{public_url}/{}", logo::PATH);
    let server_name = discovery::server_name(DISPLAY_NAME, site_url);
    let server_info = json!({
        "name": server_name,
        "title": DISPLAY_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "icons": [{ "src": logo_url, "mimeType": "image/png", "sizes": ["512x512"] }],
        "websiteUrl": public_url,
    });
    let server_meta = server_meta(&server_name);
    let results = Results::new(server_info, &server_meta, catalog.instructions(), catalog.tools_json());
    let home = page::route(
        page::render(DISPLAY_NAME, &public_url, &logo_url, site_url, catalog.tool_count()),
        site_url,
    );
    // One privacy policy, one set of terms and one support page cover every Bitview service: the site's.
    let site_pages = ["privacy", "terms", "support"]
        .into_iter()
        .fold(Router::new(), |router, page| {
            let to = Redirect::permanent(&format!("{site_url}/{page}/"));
            router.route(&format!("{PATH}/{page}"), get(move || ready(to.clone())))
        });
    let state = Arc::new(AppState {
        catalog,
        api,
        site_url: site_url.to_owned(),
        server_meta,
        results,
    });

    Router::new()
        .route(PATH, home.merge(post(transport::post).with_state(state)))
        .route(&format!("{PATH}/"), any(|| ready(Redirect::permanent(PATH))))
        .route(&format!("{PATH}/logo.png"), get(logo::get))
        .merge(site_pages)
        .merge(discovery::router(DISPLAY_NAME, site_url, &public_url, &logo_url))
        .layer(middleware::from_fn(transport::page_guard))
        // A POST's body may not stall: it would hold its connection.
        .layer(RequestBodyTimeoutLayer::new(REQUEST_BODY_TIMEOUT))
        // Any page may call and read any route: the data is public and the server takes no credentials.
        .layer(CorsLayer::permissive().max_age(PREFLIGHT_MAX_AGE))
}

impl AppState {
    /// Calls a tool; `None` for a name the catalog doesn't have.
    async fn call(&self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult> {
        let operation = self.catalog.operation(name)?;
        let path = match tools::api_path(operation, arguments) {
            Ok(path) => path,
            Err(error) => return Some(self.tool_error(error)),
        };
        let request = Request::get(&path)
            .header(ACCEPT, "application/json, text/plain, text/csv, application/octet-stream")
            .body(Body::empty())
            .expect("a percent-encoded path makes a request");
        let response = self.api.clone().oneshot(request).await.unwrap_or_else(|never| match never {});
        let status = response.status();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .split(';')
            .next()
            .unwrap()
            .trim()
            .to_ascii_lowercase();
        let body = match read_capped(response.into_body(), MAX_ANSWER_BYTES).await {
            Ok(body) => body,
            Err(BodyError::TooLarge) => return Some(self.tool_error(format!(
                "This answer is over {} MiB, the most a call returns: ask for less, with a shorter range, a smaller \
                 limit or a coarser resolution",
                MAX_ANSWER_BYTES / (1024 * 1024)
            ))),
            Err(BodyError::Failed(error)) => {
                return Some(self.tool_error(format!("Bitview API response read failed: {error}")));
            }
        };

        if !status.is_success() {
            let detail = String::from_utf8_lossy(&body[..body.len().min(MAX_API_ERROR_BYTES)]);
            let detail = detail.trim();
            let status = status.as_u16();
            return Some(self.tool_error(if detail.is_empty() {
                format!("Bitview API returned HTTP {status}")
            } else {
                format!("Bitview API returned HTTP {status}: {detail}")
            }));
        }
        let is_json = content_type == "application/json" || content_type.ends_with("+json");
        // Pass JSON through unparsed so every number stays exact.
        let content = if is_json || content_type.starts_with("text/") {
            match String::from_utf8(body) {
                Ok(text) => json!([{ "type": "text", "text": text }]),
                Err(_) => return Some(self.tool_error("Bitview API returned invalid UTF-8 text")),
            }
        } else {
            json!([{
                "type": "resource",
                "resource": {
                    "uri": format!("{}{path}", self.site_url),
                    "mimeType": content_type,
                    "blob": BASE64_STANDARD.encode(body),
                },
            }])
        };
        Some(ToolResult {
            content,
            is_error: false,
            meta: self.server_meta.clone(),
        })
    }

    fn tool_error(&self, message: impl Into<String>) -> ToolResult {
        ToolResult {
            content: json!([{ "type": "text", "text": message.into() }]),
            is_error: true,
            meta: self.server_meta.clone(),
        }
    }
}
