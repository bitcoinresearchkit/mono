use std::time::Duration;

use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
};
use tokio::{spawn, time::sleep};
use tower::ServiceExt;

use crate::{AppState, api::ApiRoutes, request_deadline};

pub(crate) async fn check(state: &AppState) {
    check_format_shapes(state);
    let router = ApiRouter::new().add_api_routes().with_state(state.clone());
    let request = |method, path: &str, tag: &str| {
        Request::builder()
            .method(method)
            .uri(path)
            .header("if-none-match", tag)
            .body(Body::empty())
            .unwrap()
    };
    let budget = &state.series_bodies.response_bodies;
    for endpoint in ["timestamp/height", "timestamp/height/data", "bulk"] {
        for format in ["json", "csv"] {
            let selection = if endpoint == "bulk" {
                "series=timestamp&index=height&"
            } else {
                ""
            };
            let path = format!("/api/series/{endpoint}?{selection}limit=1&format={format}");
            let first = router
                .clone()
                .oneshot(request("GET", &path, "\"old\""))
                .await
                .unwrap();
            assert_eq!(first.status(), StatusCode::OK, "{path}");
            if format == "csv" {
                assert!(first.headers().contains_key("content-disposition"));
            }
            let tag = first.headers()["etag"].to_str().unwrap().to_owned();
            let second = router
                .clone()
                .oneshot(request("GET", &path, "\"old\""))
                .await
                .unwrap();
            assert_eq!(second.status(), StatusCode::OK);
            assert_eq!(budget.available_permits(), 0);
            for method in ["GET", "HEAD"] {
                for condition in [tag.as_str(), "*"] {
                    let response = router
                        .clone()
                        .oneshot(request(method, &path, condition))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
                }
            }
            let waiting_router = router.clone().layer(from_fn(request_deadline::apply));
            let pending = spawn(waiting_router.oneshot(request("GET", &path, "\"old\"")));
            sleep(Duration::from_millis(50)).await;
            assert!(
                !pending.is_finished(),
                "capacity must wait inside the server"
            );
            drop(first);
            let admitted = pending.await.unwrap().unwrap();
            assert_eq!(admitted.status(), StatusCode::OK);
            assert_eq!(budget.available_permits(), 0);
            drop(admitted);
            assert_eq!(budget.available_permits(), 1);
            drop(second);
            assert_eq!(budget.available_permits(), 2);
        }
    }
}

fn check_format_shapes(state: &AppState) {
    use bitview_types::{SeriesList, SeriesSelection};
    use serde_json::{Value, from_str, from_value, json};

    state.sync(|query| {
        for columns in [1, 2] {
            for limit in [0, 1, 2] {
                for format in ["json", "csv"] {
                    let outputs = [0, 1, 2].map(|shape| {
                        let mut params: SeriesSelection = from_value(json!({
                            "series": "timestamp", "index": "height", "limit": limit,
                            "format": format,
                        }))
                        .unwrap();
                        params.series = SeriesList::from(vec!["timestamp"; columns]);
                        let resolved = query.resolve(params, usize::MAX).unwrap();
                        match shape {
                            0 => query.format(resolved),
                            1 => query.format_bulk(resolved),
                            _ => query.format_raw(resolved),
                        }
                        .unwrap()
                    });
                    let expected_metadata = (
                        outputs[0].version,
                        outputs[0].total,
                        outputs[0].start,
                        outputs[0].end,
                    );
                    for output in &outputs {
                        assert_eq!(
                            (output.version, output.total, output.start, output.end),
                            expected_metadata
                        );
                    }
                    let [single, bulk, raw] = outputs.map(|output| match output.output {
                        bitview_query::Output::CSV(text) => text,
                        bitview_query::Output::Json(bytes) => String::from_utf8(bytes).unwrap(),
                    });
                    if format == "csv" {
                        assert_eq!(single, bulk);
                        assert_eq!(single, raw);
                        continue;
                    }
                    let mut single: Value = from_str(&single).unwrap();
                    let mut bulk: Value = from_str(&bulk).unwrap();
                    let raw: Value = from_str(&raw).unwrap();
                    let entries = bulk.as_array_mut().unwrap();
                    assert_eq!(entries.len(), columns);
                    let data: Vec<_> = entries
                        .iter_mut()
                        .map(|entry| {
                            assert!(
                                entry
                                    .as_object_mut()
                                    .unwrap()
                                    .remove("stamp")
                                    .unwrap()
                                    .is_string()
                            );
                            entry["data"].clone()
                        })
                        .collect();
                    if columns == 1 {
                        single.as_object_mut().unwrap().remove("stamp");
                        assert_eq!(single, entries[0]);
                        assert_eq!(raw, data[0]);
                    } else {
                        for entry in single.as_array_mut().unwrap() {
                            entry.as_object_mut().unwrap().remove("stamp");
                        }
                        assert_eq!(single, bulk);
                        assert_eq!(raw, Value::Array(data));
                    }
                }
            }
        }
    });
}
