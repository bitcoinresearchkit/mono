use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{manifest::Catalog, page, server};

const API_URL: &str = "https://api.example.com";
const PUBLIC_URL: &str = "https://mcp.example.com/";
const DISPLAY_NAME: &str = "Example Node";

fn app() -> axum::Router {
    server::router(
        vec![API_URL.to_owned()],
        Catalog::embedded().expect("embedded MCP catalog should be valid"),
        DISPLAY_NAME.to_owned(),
        PUBLIC_URL.to_owned(),
        page::Pages::render(DISPLAY_NAME, PUBLIC_URL, API_URL),
    )
}

#[tokio::test]
async fn get_support_serves_project_and_operator_guidance() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/support")
                .body(Body::empty())
                .expect("support request should build"),
        )
        .await
        .expect("support request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("support body should be readable");
    let body = str::from_utf8(&body).expect("support page should be UTF-8");
    assert!(body.contains("<body data-page=\"support\">"));
    assert!(body.contains("Support — Example Node MCP"));
    assert!(body.contains("support@bitcoinresearchkit.org"));
    assert!(body.contains("Never send secrets"));
}

#[tokio::test]
async fn get_terms_serves_the_instance_terms() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/terms")
                .body(Body::empty())
                .expect("terms request should build"),
        )
        .await
        .expect("terms request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("terms body should be readable");
    let body = str::from_utf8(&body).expect("terms page should be UTF-8");
    assert!(body.contains("<body data-page=\"terms\">"));
    assert!(body.contains("Terms — Example Node MCP"));
    assert!(body.contains("No financial advice"));
    assert!(body.contains("support@bitcoinresearchkit.org"));
}

#[tokio::test]
async fn get_privacy_serves_the_instance_policy() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/privacy")
                .body(Body::empty())
                .expect("privacy request should build"),
        )
        .await
        .expect("privacy request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("privacy body should be readable");
    let body = str::from_utf8(&body).expect("privacy page should be UTF-8");
    assert!(body.contains("<body data-page=\"privacy\">"));
    assert!(body.contains("Privacy — Example Node MCP"));
    assert!(body.contains("https://api.example.com"));
    assert!(body.contains("support@bitcoinresearchkit.org"));
}

#[tokio::test]
async fn get_root_serves_the_documentation_page() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Origin", "https://example.com")
                .body(Body::empty())
                .expect("GET request should build"),
        )
        .await
        .expect("GET request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("page body should be readable");
    let body = str::from_utf8(&body).expect("page should be UTF-8");
    assert!(body.contains("Bitcoin data for AI"));
    assert!(body.contains(DISPLAY_NAME));
    assert!(body.contains(PUBLIC_URL));
    assert!(body.contains("https://api.example.com/api.json"));
    assert!(body.contains("https://api.example.com/health"));
    assert!(body.contains("https://api.example.com/api/server/sync"));
    assert!(body.contains("https://api.example.com/api/mempool/hash"));
    assert!(body.contains("Know how current the data is"));
    assert!(body.contains("https://cdn.jsdelivr.net/"));
    assert!(body.contains("href=\"https://mcp.example.com/logo.png\""));
    assert!(!body.contains("{{"));
}

#[tokio::test]
async fn get_logo_serves_the_embedded_production_asset() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/logo.png")
                .body(Body::empty())
                .expect("logo request should build"),
        )
        .await
        .expect("logo request should complete");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("logo body should be readable");
    assert_eq!(
        body.as_ref(),
        include_bytes!("../../../website/assets/favicon/web-app-manifest-512x512.png")
    );
}

#[tokio::test]
async fn post_root_still_reaches_mcp_discovery() {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "server/discover",
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientInfo": {
                    "name": "route-test",
                    "version": "1.0.0"
                },
                "io.modelcontextprotocol/clientCapabilities": {}
            }
        }
    });
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Host", "mcp.example.com")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json, text/event-stream")
                .header("MCP-Protocol-Version", "2026-07-28")
                .header("Mcp-Method", "server/discover")
                .body(Body::from(body.to_string()))
                .expect("discovery request should build"),
        )
        .await
        .expect("discovery request should complete");

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("discovery body should be readable");
    let raw_body = str::from_utf8(&body).expect("discovery response should be UTF-8");
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected discovery response: {raw_body}"
    );
    let body: Value = serde_json::from_slice(&body).expect("discovery response should be JSON");
    assert_eq!(body["result"]["supportedVersions"], json!(["2026-07-28"]));
    let server_info = &body["result"]["_meta"]["io.modelcontextprotocol/serverInfo"];
    assert_eq!(server_info["name"], "bitview_mcp");
    assert_eq!(server_info["title"], DISPLAY_NAME);
    assert_eq!(server_info["websiteUrl"], PUBLIC_URL);
    assert_eq!(
        server_info["icons"],
        json!([{
            "src": "https://mcp.example.com/logo.png",
            "mimeType": "image/png",
            "sizes": ["512x512"]
        }])
    );
}

#[tokio::test]
async fn root_supports_head_and_rejects_unrelated_methods() {
    let head = app()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/")
                .body(Body::empty())
                .expect("HEAD request should build"),
        )
        .await
        .expect("HEAD request should complete");
    assert_eq!(head.status(), StatusCode::OK);

    let put = app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/")
                .body(Body::empty())
                .expect("PUT request should build"),
        )
        .await
        .expect("PUT request should complete");
    assert_eq!(put.status(), StatusCode::METHOD_NOT_ALLOWED);
}
