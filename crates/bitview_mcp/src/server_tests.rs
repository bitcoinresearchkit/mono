use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, from_slice, json};
use tower::ServiceExt;

use crate::{manifest::Catalog, page, server};

const API_URL: &str = "https://api.example.com";
const PUBLIC_URL: &str = "https://mcp.example.com/";
const DISPLAY_NAME: &str = "Example Node";

fn app() -> Router {
    server::router(
        vec![API_URL.to_owned()],
        Catalog::embedded().expect("embedded MCP catalog should be valid"),
        DISPLAY_NAME.to_owned(),
        PUBLIC_URL.to_owned(),
        page::Pages::render(DISPLAY_NAME, PUBLIC_URL, API_URL),
    )
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
    let body: Value = from_slice(&body).expect("discovery response should be JSON");
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
