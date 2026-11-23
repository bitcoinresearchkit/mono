use std::{
    future,
    hash::{DefaultHasher, Hash, Hasher},
};

use axum::{
    body::Bytes,
    http::{
        HeaderMap, HeaderValue,
        header::{CACHE_CONTROL, CONTENT_TYPE, ETAG},
    },
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use base64::{Engine as _, prelude::BASE64_STANDARD};

use super::SUPPORTED_PROTOCOL_VERSIONS;

const HTML: &str = include_str!("../../mcp/index.html");

/// The server's landing page, filled in for this instance: its fonts and links come from the site's origin.
pub fn render(display_name: &str, public_url: &str, logo_url: &str, site_url: &str, tool_count: usize) -> Bytes {
    let cursor_config = BASE64_STANDARD.encode(format!(r#"{{"url":"{public_url}"}}"#));
    let [newest, .., oldest] = &SUPPORTED_PROTOCOL_VERSIONS;
    let title = format!("{display_name} MCP — Bitcoin data for AI");
    let description =
        format!("Free, read-only Bitcoin data for AI through the {display_name} MCP server.");
    Bytes::from(render_template(&[
        ("TITLE", &escape_html(&title)),
        ("DESCRIPTION", &escape_html(&description)),
        ("PUBLIC_URL", &escape_html(public_url)),
        ("API_URL", &escape_html(site_url)),
        ("DISPLAY_NAME", &escape_html(display_name)),
        ("LOGO_URL", &escape_html(logo_url)),
        ("CURSOR_CONFIG", &escape_html(&cursor_config)),
        ("PROTOCOL_OLDEST", oldest),
        ("PROTOCOL_NEWEST", newest),
        ("TOOL_COUNT", &tool_count.to_string()),
    ]))
}

/// Serves a rendered page: a second of reuse, then a cheap `304`. Its fonts and logo come from the site's origin, which
/// the page's older address is not.
pub fn route(html: Bytes, site_url: &str) -> MethodRouter {
    let policy = HeaderValue::try_from(format!(
        "default-src 'none'; style-src 'unsafe-inline'; font-src {site_url}; img-src {site_url}; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
    ))
    .expect("the site URL is a valid header value");
    document(
        html,
        "text/html; charset=utf-8",
        bitview_http::REVALIDATE,
        vec![("content-security-policy", policy)],
    )
}

/// Serves a static document under a content ETag, with the given caching and extra headers.
pub fn document(
    body: Bytes,
    content_type: &'static str,
    cache_control: &'static str,
    extra_headers: Vec<(&'static str, HeaderValue)>,
) -> MethodRouter {
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    let tag = format!("{:x}", hasher.finish());
    let etag =
        HeaderValue::try_from(format!("\"{tag}\"")).expect("a hex ETag is a valid header value");
    get(move |request: HeaderMap| {
        let mut response = if bitview_http::if_none_match(&request, Some(tag.as_bytes())) {
            bitview_http::not_modified()
        } else {
            ([(CONTENT_TYPE, content_type)], body.clone()).into_response()
        };
        let headers = response.headers_mut();
        headers.insert(ETAG, etag.clone());
        headers.insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
        for (name, value) in &extra_headers {
            headers.insert(*name, value.clone());
        }
        future::ready(response)
    })
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('{', "&#123;")
        .replace('}', "&#125;")
}

fn render_template(values: &[(&str, &str)]) -> String {
    let mut output = String::with_capacity(HTML.len());
    let mut remaining = HTML;

    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let value_start = start + 2;
        let value_end = remaining[value_start..]
            .find("}}")
            .map(|end| value_start + end)
            .expect("page template placeholder should be closed");
        let key = &remaining[value_start..value_end];
        let value = values
            .iter()
            .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
            .unwrap_or_else(|| panic!("unknown page template placeholder: {key}"));
        output.push_str(value);
        remaining = &remaining[value_end + 2..];
    }

    output.push_str(remaining);
    output
}
