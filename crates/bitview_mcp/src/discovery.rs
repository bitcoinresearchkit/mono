//! The MCP Server Card and the AI Catalog that lists it: public, static, and rendered at startup.

use axum::{
    Router,
    body::Bytes,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::MethodRouter,
};
use serde_json::{Value, json, to_vec};

use crate::{logo, page, server::SUPPORTED_PROTOCOL_VERSIONS};

const SERVER_CARD_MEDIA_TYPE: &str = "application/mcp-server-card+json";
const AI_CATALOG_MEDIA_TYPE: &str = "application/ai-catalog+json";
const CACHE_CONTROL: &str = "public, max-age=3600";
const CORS: &[(&str, &str)] = &[
    ("access-control-allow-origin", "*"),
    ("access-control-allow-methods", "GET"),
    ("access-control-allow-headers", "Content-Type, If-None-Match"),
    ("access-control-expose-headers", "ETag"),
];
const SERVER_CARD_PATH: &str = "server-card";
const AI_CATALOG_PATH: &str = "/.well-known/ai-catalog.json";
const DESCRIPTION: &str =
    "Read-only Bitcoin blockchain, mempool, mining, market, and on-chain analytics; no API key.";

pub fn router(display_name: &str, public_url: &str) -> Router {
    let card = server_card(display_name, public_url);
    let catalog = ai_catalog(display_name, public_url);
    Router::new()
        .route(
            &format!("/{SERVER_CARD_PATH}"),
            document(&card, SERVER_CARD_MEDIA_TYPE),
        )
        .route(AI_CATALOG_PATH, document(&catalog, AI_CATALOG_MEDIA_TYPE))
}

/// The server's name, the card's and the one it gives at runtime: its host in reverse-DNS form.
pub fn server_name(display_name: &str, public_url: &str) -> String {
    format!("{}/{}", reverse_dns(&host(public_url)), slug(display_name))
}

fn document(value: &Value, media_type: &'static str) -> MethodRouter {
    let body = to_vec(value).expect("a JSON value serializes");
    page::document(Bytes::from(body), media_type, CACHE_CONTROL, CORS).options(preflight)
}

/// Conditional requests carry `If-None-Match`, so a browser asks first.
async fn preflight() -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    for &(name, value) in CORS {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}

fn server_card(display_name: &str, public_url: &str) -> Value {
    let versions = SUPPORTED_PROTOCOL_VERSIONS
        .iter()
        .map(|version| version.as_str())
        .collect::<Vec<_>>();
    json!({
        "$schema": "https://static.modelcontextprotocol.io/schemas/v1/server-card.schema.json",
        "name": server_name(display_name, public_url),
        "title": display_name,
        "description": DESCRIPTION,
        "version": env!("CARGO_PKG_VERSION"),
        "websiteUrl": public_url,
        "icons": [{
            "src": format!("{public_url}{}", logo::PATH),
            "mimeType": "image/png",
            "sizes": ["512x512"],
        }],
        "remotes": [{
            "type": "streamable-http",
            "url": public_url,
            "supportedProtocolVersions": versions,
        }],
    })
}

fn ai_catalog(display_name: &str, public_url: &str) -> Value {
    json!({
        "specVersion": "1.0",
        "entries": [{
            "identifier": format!("urn:air:{}:mcp:{}", host(public_url), slug(display_name)),
            "type": SERVER_CARD_MEDIA_TYPE,
            "url": format!("{public_url}{SERVER_CARD_PATH}"),
        }],
    })
}

/// The host of a public URL, which is an absolute origin with a trailing slash.
fn host(public_url: &str) -> String {
    let authority = public_url
        .split_once("://")
        .map_or(public_url, |(_, rest)| rest)
        .trim_end_matches('/');
    authority
        .split_once(':')
        .map_or(authority, |(host, _)| host)
        .to_owned()
}

fn reverse_dns(host: &str) -> String {
    host.rsplit('.').collect::<Vec<_>>().join(".")
}

fn slug(display_name: &str) -> String {
    let slug = display_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    slug.trim_matches('-').to_owned()
}
