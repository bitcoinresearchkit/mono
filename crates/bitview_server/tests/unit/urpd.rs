#[cfg(feature = "chain")]
use super::chain_fixture;
use super::{server_routes::exchange_with_etag, urpd_sources};
use crate::{AppState, api::ApiRoutes, urpd_input};
use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    serve as serve_http,
};
use bitcoin::Amount;
use brk_types::{Cents, Cohort, Height, UrpdAggregation, UrpdWeight};
use std::{fs, net::SocketAddr, time::Duration};
use tokio::{
    join,
    net::TcpListener,
    spawn,
    time::{self, timeout},
};
use tower::ServiceExt;
use tower_http::timeout::TimeoutLayer;
#[cfg(feature = "chain")]
#[test]
fn populated_urpd_history() {
    let mut first = chain_fixture::default_first();
    first.txdata[0].output[0].value = Amount::from_sat(4_000_000_000);
    first.header.merkle_root = first.compute_merkle_root().unwrap();
    chain_fixture::run_genesis(first, |mut fixture| async move {
        fixture.publish(1, 1);
        check_reconstruction(&fixture.state, fixture.address).await;
        let route = "/api/urpd/all/1";
        let before = exchange_with_etag(fixture.address, "GET", route, "\"old\"").await;
        let tag = before
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap();
        fixture.publish(2, 1);
        let after = exchange_with_etag(fixture.address, "GET", route, tag).await;
        assert!(after.starts_with("HTTP/1.1 200"), "{after}");
        assert_ne!(
            before.split_once("\r\n\r\n").unwrap().1,
            after.split_once("\r\n\r\n").unwrap().1
        );

        let path = fixture.directory.path().join("origins/snapshots/latest");
        let saved = fs::read(&path).unwrap();
        let mut corrupt = saved.clone();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        fs::write(&path, corrupt).unwrap();
        for method in ["GET", "HEAD"] {
            let response = exchange_with_etag(fixture.address, method, route, "*").await;
            assert!(response.starts_with("HTTP/1.1 500"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
        fs::write(path, saved).unwrap();
    });
}
pub async fn check_reconstruction(state: &AppState, address: SocketAddr) {
    check_cancelled_admission(state).await;
    urpd_sources::check(state, address).await;
    let route = "/api/urpd/all";
    let response = exchange_with_etag(address, "GET", route, "\"old\"").await;
    let current = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap();
    check_response_admission(state, route, current).await;
    state.sync(|query| {
        let mut input = query
            .resolve_urpd_latest(
                &Cohort::new("all").unwrap(),
                UrpdAggregation::Raw,
                UrpdWeight::Raw,
            )
            .unwrap();
        let original = urpd_input::identity(&input).unwrap();
        input.height = Height::new(u32::from(input.height) + 1);
        assert_ne!(urpd_input::identity(&input).unwrap(), original);
        input.height = Height::new(u32::from(input.height) - 1);
        input.close = Cents::new(u64::from(input.close) + 1);
        assert_ne!(urpd_input::identity(&input).unwrap(), original);
        input.close = Cents::NAN;
        assert!(urpd_input::identity(&input).is_err());
    });
    for method in ["GET", "HEAD"] {
        for (route, status) in [
            ("/api/urpd/unknown", 404),
            ("/api/urpd/unknown/dates", 404),
            ("/api/urpd/all/99999999", 404),
            ("/api/urpd/all/2009-01-04", 404),
            ("/api/urpd/all/2009-02-30", 400),
            ("/api/urpd/all/4294967296", 400),
            ("/api/urpd/all?weight=raw&weight=raw", 400),
            ("/api/urpd/all?x=1", 400),
        ] {
            let response = exchange_with_etag(address, method, route, "*").await;
            assert!(
                response.starts_with(&format!("HTTP/1.1 {status}")),
                "{route}: {response}"
            );
            assert!(!response.contains("\r\netag:"));
        }
    }
}

async fn check_response_admission(state: &AppState, path: &str, current: &str) {
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
    assert_eq!(state.urpd_bodies.available_permits(), 0);
    assert_eq!(state.urpd_query.available_permits(), 2);
    let pending = spawn(router.clone().oneshot(request("GET", "\"old\"")));
    time::sleep(Duration::from_millis(50)).await;
    assert!(!pending.is_finished());
    assert_eq!(state.urpd_query.available_permits(), 2);
    for method in ["GET", "HEAD"] {
        for tag in [current, "*"] {
            let response = router.clone().oneshot(request(method, tag)).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        }
    }
    assert_eq!(state.urpd_bodies.available_permits(), 0);
    drop(first);
    let admitted = pending.await.unwrap().unwrap();
    assert_eq!(admitted.status(), StatusCode::OK);
    drop(admitted);
    assert_eq!(state.urpd_bodies.available_permits(), 1);
    drop(second);
    assert_eq!(state.urpd_bodies.available_permits(), 2);
}

async fn check_cancelled_admission(state: &AppState) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = ApiRouter::new()
        .add_api_routes()
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_millis(100),
        ))
        .with_state(state.clone());
    let serving = spawn(async move {
        serve_http(listener, router.into_make_service())
            .await
            .unwrap();
    });
    let gate = state.sync(|query| query.indexer().publication().clone());
    let admission = state.urpd_query.clone();
    assert_eq!(admission.available_permits(), 2);
    gate.begin_update();
    let (first, second) = join!(
        exchange_with_etag(address, "GET", "/api/urpd/all", "*"),
        exchange_with_etag(address, "HEAD", "/api/urpd/all", "*"),
    );
    let after_cancel = admission.available_permits();
    let queued = exchange_with_etag(address, "GET", "/api/urpd/all", "*").await;
    let after_queued = admission.available_permits();
    gate.finish_update();
    // Cancellation cannot release a still-running blocking worker's admission.
    // Publishing wakes the workers, which then release their slots.
    let recovered = timeout(Duration::from_secs(2), admission.acquire_many_owned(2)).await;
    serving.abort();
    assert!(first.starts_with("HTTP/1.1 504"), "{first}");
    assert!(second.starts_with("HTTP/1.1 504"), "{second}");
    assert!(second.ends_with("\r\n\r\n"));
    assert_eq!(
        after_cancel, 0,
        "cancelled publication waits must retain worker slots"
    );
    assert!(queued.starts_with("HTTP/1.1 504"), "{queued}");
    assert_eq!(after_queued, 0);
    drop(recovered.unwrap().unwrap());
    assert_eq!(state.urpd_query.available_permits(), 2);
}
