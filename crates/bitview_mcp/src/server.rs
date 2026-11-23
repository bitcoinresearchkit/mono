use std::{future::ready, sync::Arc, time::Duration};

use axum::{
    Router,
    body::Bytes,
    middleware,
    response::Redirect,
    routing::{get, post},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use serde_json::{Map, Value, json};
use tokio::{sync::Semaphore, time::timeout};
use tower_http::{cors::CorsLayer, timeout::RequestBodyTimeoutLayer};
use tracing::debug;

use crate::{
    discovery, logo,
    manifest::Catalog,
    page,
    protocol::{Results, ToolResult, server_meta},
    transport,
    upstream::{MAX_CALLS, Upstream},
    upstream_response::UpstreamResponse,
};

const CACHE_META_KEY: &str = "org.bitcoinresearchkit/upstreamCache";
/// The longest a call waits for its turn among the `MAX_CALLS` in flight.
const TURN_WAIT: Duration = Duration::from_secs(5);
const MAX_UPSTREAM_ERROR_BYTES: usize = 2_048;

pub struct AppState {
    catalog: Catalog,
    upstream: Upstream,
    calls: Semaphore,
    server_meta: Map<String, Value>,
    pub results: Results,
}

pub fn router(
    api_url: String,
    catalog: Catalog,
    display_name: String,
    public_url: String,
    home: Bytes,
) -> Router {
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
    let server_info = json!({
        "name": server_name,
        "title": display_name,
        "version": env!("CARGO_PKG_VERSION"),
        "icons": [{
            "src": format!("{public_url}{}", logo::PATH),
            "mimeType": "image/png",
            "sizes": ["512x512"],
        }],
        "websiteUrl": public_url,
    });
    let server_meta = server_meta(&server_name);
    let results = Results::new(server_info, &server_meta, catalog.instructions(), catalog.tools_json());
    let state = Arc::new(AppState {
        catalog,
        upstream: Upstream::new(api_url),
        calls: Semaphore::new(MAX_CALLS),
        server_meta,
        results,
    });

    Router::new()
        .route("/", home.merge(post(transport::post).with_state(state)))
        .route("/logo.png", get(logo::get))
        .merge(site_pages)
        .merge(discovery_routes)
        .layer(middleware::from_fn(transport::page_guard))
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

impl AppState {
    /// Calls a tool; `None` for a name the catalog doesn't have.
    pub async fn call(&self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult> {
        let operation = self.catalog.operation(name)?;
        let url = match self.upstream.prepare(operation, arguments) {
            Ok(url) => url,
            Err(error) => return Some(self.tool_error(error)),
        };
        let Ok(Ok(_turn)) = timeout(TURN_WAIT, self.calls.acquire()).await else {
            return Some(self.tool_error("Bitview MCP is busy; retry shortly"));
        };
        let response = match self.upstream.fetch(url).await {
            Ok(response) => response,
            Err(error) => return Some(self.tool_error(error)),
        };
        log_cache_status(name, &response);
        Some(self.render(response))
    }

    fn tool_error(&self, message: impl Into<String>) -> ToolResult {
        ToolResult {
            content: json!([{ "type": "text", "text": message.into() }]),
            is_error: true,
            meta: self.server_meta.clone(),
        }
    }

    fn render(&self, response: UpstreamResponse) -> ToolResult {
        if !(200..300).contains(&response.status) {
            let detail = String::from_utf8_lossy(
                &response.body[..response.body.len().min(MAX_UPSTREAM_ERROR_BYTES)],
            );
            let detail = detail.trim();
            return self.tool_error(if detail.is_empty() {
                format!("Bitview API returned HTTP {}", response.status)
            } else {
                format!("Bitview API returned HTTP {}: {detail}", response.status)
            });
        }

        let content_type = response
            .content_type
            .split(';')
            .next()
            .unwrap()
            .trim()
            .to_ascii_lowercase();
        let mut meta = self.server_meta.clone();
        meta.insert(
            CACHE_META_KEY.to_owned(),
            json!({ "status": response.cache_status, "age": response.cache_age }),
        );

        let is_json = content_type == "application/json" || content_type.ends_with("+json");
        // Pass JSON through unparsed so every upstream number stays exact.
        let content = if is_json || content_type.starts_with("text/") {
            match String::from_utf8(response.body) {
                Ok(text) => json!([{ "type": "text", "text": text }]),
                Err(_) => return self.tool_error("Bitview API returned invalid UTF-8 text"),
            }
        } else {
            json!([{
                "type": "resource",
                "resource": {
                    "uri": response.url,
                    "mimeType": content_type,
                    "blob": BASE64_STANDARD.encode(response.body),
                },
            }])
        };
        ToolResult {
            content,
            is_error: false,
            meta,
        }
    }
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
