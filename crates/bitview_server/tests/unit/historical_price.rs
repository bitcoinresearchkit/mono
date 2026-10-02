use std::{net::SocketAddr, time::Duration};

use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tokio::{
    spawn,
    time::{self, timeout},
};
use tower::ServiceExt;

use super::server_routes::exchange_with_etag;
use crate::{AppState, api::ApiRoutes};

pub(crate) async fn check(state: &AppState, address: SocketAddr) {
    let path = "/api/v1/historical-price";
    let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
    let current = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let router = ApiRouter::new().add_api_routes().with_state(state.clone());
    let request = |method, tag: &str| {
        Request::builder()
            .method(method)
            .uri(path)
            .header("if-none-match", tag)
            .body(Body::empty())
            .unwrap()
    };
    let first = router
        .clone()
        .oneshot(request("GET", "\"old\""))
        .await
        .unwrap();
    let second = router
        .clone()
        .oneshot(request("GET", "\"old\""))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(state.historical_price_bodies.available_permits(), 0);
    let pending = spawn(router.clone().oneshot(request("GET", "\"old\"")));
    time::sleep(Duration::from_millis(50)).await;
    assert!(!pending.is_finished());
    for method in ["GET", "HEAD"] {
        for tag in [current.as_str(), "*"] {
            assert_eq!(
                router
                    .clone()
                    .oneshot(request(method, tag))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::NOT_MODIFIED
            );
        }
    }
    drop(first);
    let admitted = pending.await.unwrap().unwrap();
    assert_eq!(admitted.status(), StatusCode::OK);
    drop(admitted);
    assert_eq!(state.historical_price_bodies.available_permits(), 1);
    drop(second);
    assert_eq!(state.historical_price_bodies.available_permits(), 2);

    // Neither a current full-history validator nor wildcard HEAD may skip
    // mappings/price publication merely because the indexer tip is unchanged.
    {
        let gate = state.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        let mut tasks = Vec::new();
        for method in ["GET", "HEAD"] {
            let path = path.to_owned();
            let tag = if method == "GET" {
                current.clone()
            } else {
                "*".into()
            };
            tasks.push(spawn(async move {
                exchange_with_etag(address, method, &path, &tag).await
            }));
        }
        assert!(
            timeout(Duration::from_millis(50), &mut tasks[0])
                .await
                .is_err()
        );
        assert!(!tasks[1].is_finished());
        gate.finish_update();
        for task in tasks {
            let response = timeout(Duration::from_secs(5), task)
                .await
                .unwrap()
                .unwrap();
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        }
    }
}
