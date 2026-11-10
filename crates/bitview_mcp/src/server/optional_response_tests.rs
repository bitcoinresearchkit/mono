use super::*;

fn response(status: u16, content_type: &str, body: &[u8], schema: bool) -> CallToolResult {
    let response = UpstreamResponse {
        url: "http://127.0.0.1/fixture".into(),
        status,
        content_type: content_type.into(),
        body: body.into(),
        cache_status: None,
        cache_age: None,
    };
    let CallToolResponse::Complete(result) = render(response, schema, "fixture") else {
        panic!("an upstream HTTP result must complete the call");
    };
    result
}

#[test]
fn no_content_is_success_and_does_not_hide_invalid_json_or_http_errors() {
    for content_type in ["application/json", "application/json; charset=utf-8", "application/octet-stream"] {
        let result = response(204, content_type, &[], false);
        assert!(result.content.is_empty());
        assert!(result.structured_content.is_none());
        assert_ne!(result.is_error, Some(true));
        assert!(result.meta.is_some());
    }
    for body in [b"".as_slice(), b"{"] {
        assert_eq!(response(200, "application/json", body, false).is_error, Some(true));
    }
    let body = br#"{"txid":"fixture"}"#;
    let present = response(200, "application/json", body, false);
    assert_eq!(present.content.len(), 1);
    assert!(present.structured_content.is_none());
    assert_ne!(present.is_error, Some(true));
    assert_eq!(response(200, "application/json", body, true).structured_content,
        Some(json!({"txid":"fixture"})));
    assert_eq!(response(503, "application/json", b"unavailable", false).is_error, Some(true));
}
