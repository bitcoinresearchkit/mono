use std::{future, sync::Arc};

use axum::{
    http::{HeaderValue, header::CACHE_CONTROL},
    response::{Html, IntoResponse, Response},
    routing::{MethodRouter, get},
};

const HTML: &str = include_str!("../assets/index.html");

pub struct Pages {
    home: Arc<str>,
    privacy: Arc<str>,
    terms: Arc<str>,
    support: Arc<str>,
}

impl Pages {
    pub fn render(display_name: &str, public_url: &str, api_url: &str) -> Self {
        Self {
            home: render_home(display_name, public_url, api_url).into(),
            privacy: render_privacy(display_name, public_url, api_url).into(),
            terms: render_terms(display_name, public_url, api_url).into(),
            support: render_support(display_name, public_url, api_url).into(),
        }
    }

    pub fn home(&self) -> Arc<str> {
        self.home.clone()
    }

    pub fn privacy(&self) -> Arc<str> {
        self.privacy.clone()
    }

    pub fn terms(&self) -> Arc<str> {
        self.terms.clone()
    }

    pub fn support(&self) -> Arc<str> {
        self.support.clone()
    }
}

fn render_home(display_name: &str, public_url: &str, api_url: &str) -> String {
    render(
        "home",
        &format!("{display_name} MCP — Bitcoin data for AI"),
        &format!("Free, read-only Bitcoin data for AI through the {display_name} MCP server."),
        public_url,
        display_name,
        public_url,
        api_url,
    )
}

fn render_privacy(display_name: &str, public_url: &str, api_url: &str) -> String {
    render(
        "privacy",
        &format!("Privacy — {display_name} MCP"),
        &format!("Privacy policy for the {display_name} MCP server."),
        &format!("{public_url}privacy"),
        display_name,
        public_url,
        api_url,
    )
}

fn render_terms(display_name: &str, public_url: &str, api_url: &str) -> String {
    render(
        "terms",
        &format!("Terms — {display_name} MCP"),
        &format!("Terms of use for the {display_name} MCP server."),
        &format!("{public_url}terms"),
        display_name,
        public_url,
        api_url,
    )
}

fn render_support(display_name: &str, public_url: &str, api_url: &str) -> String {
    render(
        "support",
        &format!("Support — {display_name} MCP"),
        &format!("Support information for the {display_name} MCP server."),
        &format!("{public_url}support"),
        display_name,
        public_url,
        api_url,
    )
}

fn render(
    page: &str,
    title: &str,
    description: &str,
    canonical_url: &str,
    display_name: &str,
    public_url: &str,
    api_url: &str,
) -> String {
    let page = escape_html(page);
    let title = escape_html(title);
    let description = escape_html(description);
    let canonical_url = escape_html(canonical_url);
    let display_name = escape_html(display_name);
    let public_url = escape_html(public_url);
    let api_url = escape_html(api_url);

    render_template(&[
        ("PAGE", &page),
        ("PAGE_TITLE", &title),
        ("PAGE_DESCRIPTION", &description),
        ("CANONICAL_URL", &canonical_url),
        ("PUBLIC_URL", &public_url),
        ("API_URL", &api_url),
        ("DISPLAY_NAME", &display_name),
    ])
}

pub fn route(html: Arc<str>) -> MethodRouter {
    get(move || future::ready(response(html.clone())))
}

fn response(html: Arc<str>) -> Response {
    let mut response = Html(html.to_string()).into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
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
