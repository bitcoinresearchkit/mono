//! The MCP Server Card and the AI Catalog that lists it: public, static, and rendered at startup.

use std::net::Ipv6Addr;

use axum::{Router, body::Bytes, routing::MethodRouter};
use serde_json::{Value, json, to_vec};

use super::{PATH, SUPPORTED_PROTOCOL_VERSIONS, page};

const SERVER_CARD_MEDIA_TYPE: &str = "application/mcp-server-card+json";
const AI_CATALOG_MEDIA_TYPE: &str = "application/ai-catalog+json";
const CACHE_CONTROL: &str = "public, max-age=3600";
/// The server's one line, in its card and in the MCP Registry's server.json, which bindgen writes from it.
pub const DESCRIPTION: &str =
    "Read-only Bitcoin chain, mempool, mining, market and on-chain analytics, with a skill; no API key.";

/// The card at `/mcp/server-card` and the catalog naming it at `/mcp/.well-known/ai-catalog.json`.
pub fn router(display_name: &str, site_url: &str, public_url: &str, logo_url: &str) -> Router {
    let card_url = format!("{public_url}/server-card");
    let card = json!({
        "$schema": "https://static.modelcontextprotocol.io/schemas/v1/server-card.schema.json",
        "name": server_name(display_name, site_url),
        "title": display_name,
        "description": DESCRIPTION,
        "version": env!("CARGO_PKG_VERSION"),
        "websiteUrl": public_url,
        "icons": [{ "src": logo_url, "mimeType": "image/png", "sizes": ["512x512"] }],
        "remotes": [{
            "type": "streamable-http",
            "url": public_url,
            "supportedProtocolVersions": SUPPORTED_PROTOCOL_VERSIONS,
        }],
    });
    let catalog = json!({
        "specVersion": "1.0",
        "entries": [{
            "identifier": format!("urn:air:{}:mcp:{}", identity_domain(site_url), slug(display_name)),
            "type": SERVER_CARD_MEDIA_TYPE,
            "url": card_url,
        }],
    });
    Router::new()
        .route(&format!("{PATH}/server-card"), document(&card, SERVER_CARD_MEDIA_TYPE))
        .route(&format!("{PATH}/.well-known/ai-catalog.json"), document(&catalog, AI_CATALOG_MEDIA_TYPE))
}

/// The server's shared identity in the card and at runtime, in reverse-DNS form.
pub fn server_name(display_name: &str, site_url: &str) -> String {
    format!("{}/{}", reverse_dns(&identity_domain(site_url)), slug(display_name))
}

fn document(value: &Value, media_type: &'static str) -> MethodRouter {
    let body = to_vec(value).expect("a JSON value serializes");
    page::document(Bytes::from(body), media_type, CACHE_CONTROL, Vec::new())
}

/// The origin's identity domain; IPv6 uses its reverse-pointer domain (RFC 3596 section 2.5).
fn identity_domain(site_url: &str) -> String {
    let uri = site_url
        .parse::<axum::http::Uri>()
        .expect("the site URL is a validated HTTP(S) origin");
    let host = uri.host().expect("the site URL has a host");
    let Ok(address) = host.trim_matches(['[', ']']).parse::<Ipv6Addr>() else {
        return host.to_owned();
    };
    let mut domain = String::new();
    for nibble in format!("{:032x}", u128::from(address)).chars().rev() {
        domain.push(nibble);
        domain.push('.');
    }
    domain.push_str("ip6.arpa");
    domain
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
