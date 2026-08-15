use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{manifest::Catalog, server};

fn app() -> axum::Router {
    server::router(
        vec!["https://bitview.space/api".to_owned()],
        Catalog::embedded().expect("embedded MCP catalog should be valid"),
    )
}

#[tokio::test]
async fn get_root_serves_the_documentation_page() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Origin", "https://bitview.space")
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
    assert!(body.contains("https://mcp.bitview.space/"));
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
                .header("Host", "mcp.bitview.space")
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
