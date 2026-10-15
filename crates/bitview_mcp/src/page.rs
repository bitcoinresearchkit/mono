use std::{
    future,
    hash::{DefaultHasher, Hash, Hasher},
};

use axum::{
    body::Bytes,
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH},
    },
    response::{Html, IntoResponse, Response},
    routing::{MethodRouter, get},
};

const HTML: &str = include_str!("../assets/index.html");

pub struct Pages {
    pub(crate) home: Bytes,
    pub(crate) privacy: Bytes,
    pub(crate) terms: Bytes,
    pub(crate) support: Bytes,
}

impl Pages {
    pub fn render(display_name: &str, public_url: &str, api_url: &str) -> Self {
        let render = |page: &str, title: &str, description: &str| {
            let canonical_url = match page {
                "home" => public_url.to_owned(),
                page => format!("{public_url}{page}"),
            };
            Bytes::from(render_template(&[
                ("PAGE", &escape_html(page)),
                ("PAGE_TITLE", &escape_html(title)),
                ("PAGE_DESCRIPTION", &escape_html(description)),
                ("CANONICAL_URL", &escape_html(&canonical_url)),
                ("PUBLIC_URL", &escape_html(public_url)),
                ("API_URL", &escape_html(api_url)),
                ("DISPLAY_NAME", &escape_html(display_name)),
                ("LOGO_PATH", &escape_html(crate::logo::PATH)),
            ]))
        };
        Self {
            home: render(
                "home",
                &format!("{display_name} MCP — Bitcoin data for AI"),
                &format!(
                    "Free, read-only Bitcoin data for AI through the {display_name} MCP server."
                ),
            ),
            privacy: render(
                "privacy",
                &format!("Privacy — {display_name} MCP"),
                &format!("Privacy policy for the {display_name} MCP server."),
            ),
            terms: render(
                "terms",
                &format!("Terms — {display_name} MCP"),
                &format!("Terms of use for the {display_name} MCP server."),
            ),
            support: render(
                "support",
                &format!("Support — {display_name} MCP"),
                &format!("Support information for the {display_name} MCP server."),
            ),
        }
    }
}

/// Serves a rendered page under a content ETag: a second of reuse, then a cheap `304`.
pub fn route(html: Bytes) -> MethodRouter {
    let mut hasher = DefaultHasher::new();
    html.hash(&mut hasher);
    let etag = HeaderValue::try_from(format!("\"{:x}\"", hasher.finish()))
        .expect("a hex ETag is a valid header value");
    get(move |headers: HeaderMap| future::ready(response(html.clone(), &etag, &headers)))
}

fn response(html: Bytes, etag: &HeaderValue, request: &HeaderMap) -> Response {
    let not_modified = request
        .get_all(IF_NONE_MATCH)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|value| {
            value.trim() == "*"
                || value.split(',').any(|tag| {
                    let tag = tag.trim();
                    tag.strip_prefix("W/").unwrap_or(tag).as_bytes() == etag.as_bytes()
                })
        });
    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        Html(html).into_response()
    };
    response.headers_mut().insert(ETAG, etag.clone());
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=1, must-revalidate"),
    );
    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'none'; style-src 'unsafe-inline'; font-src https://cdn.jsdelivr.net; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        ),
    );
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
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
