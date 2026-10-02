use axum::{
    body as AxumBody,
    http::{
        HeaderName,
        header::{CACHE_CONTROL, ETAG},
    },
};
use brk_error::SeriesNotFound;
use serde_json::{Value, from_slice};

use super::*;

#[tokio::test]
async fn shared_error_mapping_preserves_status_code_body_and_cache_policy() {
    let cases = [
        (BrkError::InvalidAddr, 400, "invalid_addr"),
        (BrkError::InvalidTxid, 400, "invalid_txid"),
        (BrkError::InvalidNetwork, 400, "invalid_network"),
        (
            BrkError::UnsupportedType("test".into()),
            400,
            "unsupported_type",
        ),
        (BrkError::Parse("test".into()), 400, "parse_error"),
        (BrkError::NoSeries, 400, "no_series"),
        (
            BrkError::SeriesUnsupportedIndex {
                series: "test".into(),
                supported: "height".into(),
            },
            400,
            "series_unsupported_index",
        ),
        (
            BrkError::WeightExceeded {
                requested: 2,
                max: 1,
            },
            400,
            "weight_exceeded",
        ),
        (BrkError::TooManyUtxos, 400, "too_many_utxos"),
        (BrkError::UnknownAddr, 404, "unknown_addr"),
        (BrkError::UnknownTxid, 404, "unknown_txid"),
        (BrkError::NotFound("test".into()), 404, "not_found"),
        (BrkError::OutOfRange("test".into()), 404, "out_of_range"),
        (BrkError::UnindexableDate, 404, "unindexable_date"),
        (BrkError::NoData, 404, "no_data"),
        (
            BrkError::SeriesNotFound(SeriesNotFound::new("test".into(), vec![], 0)),
            404,
            "series_not_found",
        ),
        (BrkError::MempoolNotAvailable, 503, "mempool_not_available"),
        (BrkError::StateUpdating, 503, "state_updating"),
        (BrkError::ReadTimeout, 504, "timeout"),
        (BrkError::AuthFailed, 403, "auth_failed"),
        (BrkError::Internal("test"), 500, "internal_error"),
    ];
    for (error, status, code) in cases {
        let message = error.to_string();
        let response = Error::from(error).into_response();
        assert_eq!(response.status().as_u16(), status);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        let policy = if matches!(code, "invalid_addr" | "invalid_network" | "invalid_txid") {
            "public, max-age=31536000, immutable"
        } else if matches!(status, 400 | 404) {
            "public, max-age=1, must-revalidate"
        } else {
            "no-store"
        };
        assert_cache_control(&response, policy);
        assert_eq!(
            response.headers().contains_key(header::RETRY_AFTER),
            code == "state_updating"
        );
        let bytes = AxumBody::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: Value = from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["code"], code);
        assert_eq!(body["error"]["message"], message);
        assert_eq!(body["error"]["doc_url"], DOC_URL);
    }
}

fn assert_cache_control(response: &Response, expected: &'static str) {
    let expected = HeaderValue::from_static(expected);
    assert_eq!(response.headers().get(CACHE_CONTROL), Some(&expected));
    assert_eq!(
        response
            .headers()
            .get(HeaderName::from_static("cdn-cache-control")),
        Some(&expected)
    );
    assert!(!response.headers().contains_key(ETAG));
}

#[test]
fn state_updating_is_a_retryable_service_unavailable_response() {
    let error = Error::from(BrkError::StateUpdating);
    assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.code.as_str(), "state_updating");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response.headers().get(header::RETRY_AFTER),
        Some(&HeaderValue::from_static("1"))
    );
    assert_cache_control(&response, "no-store");
}
