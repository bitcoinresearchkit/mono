use std::{
    fs::OpenOptions,
    net::SocketAddr,
    os::unix::fs::symlink,
    path::Path,
    time::Duration,
};

use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    serve as serve_http,
};
use brk_types::{Cents, CentsCompact, Cohort, Date, Sats, UrpdAggregation, UrpdRaw, UrpdWeight};
use serde_json::{Value, from_str};
use tokio::{
    fs, join,
    net::TcpListener,
    spawn,
    time::{self, timeout},
};
use tower::ServiceExt;
use tower_http::timeout::TimeoutLayer;

use super::{server_routes::exchange_with_etag, urpd_sources};
use crate::{AppState, api::ApiRoutes, urpd_input};

#[cfg(feature = "chain")]
use super::chain_fixture;

#[cfg(feature = "chain")]
#[test]
fn populated_urpd_snapshots() {
    chain_fixture::run(|state, address| async move {
        check_snapshots(&state, address).await;
    });
}

pub async fn check_snapshots(state: &AppState, address: SocketAddr) {
    check_cancelled_admission(state).await;
    urpd_sources::check(state, address).await;
    let cohort = Cohort::new("fixture").unwrap();
    let date = Date::new(2009, 1, 3);
    let path = state.sync(|query| query.plugins().distribution.states_path.clone());
    fs::create_dir_all(UrpdRaw::dir(&path, "emptyfixture"))
        .await
        .unwrap();
    for method in ["GET", "HEAD"] {
        let response = exchange_with_etag(address, method, "/api/urpd/emptyfixture", "*").await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        assert!(!response.contains("\r\netag:"));
        if method == "HEAD" {
            assert!(response.ends_with("\r\n\r\n"));
        }
    }
    let write = |date, sats| {
        UrpdRaw::write(
            &path,
            &cohort,
            date,
            [(CentsCompact::new(100), Sats::from(sats))].into_iter(),
        )
        .unwrap()
    };
    write(date, 100_000_000_u64);
    let route = "/api/urpd/fixture";
    let dated = "/api/urpd/fixture/2009-01-03";
    let response = exchange_with_etag(address, "GET", route, "\"old\"").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let old = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(body["total_supply"], 1.0);
    let dated_response = exchange_with_etag(address, "GET", dated, "\"old\"").await;
    assert!(
        dated_response.starts_with("HTTP/1.1 200"),
        "{dated_response}"
    );
    assert_eq!(
        dated_response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap(),
        old
    );
    assert_eq!(
        dated_response.split_once("\r\n\r\n").unwrap().1,
        response.split_once("\r\n\r\n").unwrap().1
    );
    // Every metadata input participates, independently of the chain tip.
    state.sync(|query| {
        let mut input = query
            .resolve_urpd_latest(&cohort, UrpdAggregation::Raw, UrpdWeight::Raw)
            .unwrap();
        let original = urpd_input::identity(&input);
        input.close = Cents::from(1_u64);
        assert_ne!(urpd_input::identity(&input), original);
        input.close = Cents::ZERO;
        input.scalar = 0.5;
        assert_ne!(urpd_input::identity(&input), original);
        input.scalar = 1.0;
        input.aggregation = UrpdAggregation::Lin200;
        assert_ne!(urpd_input::identity(&input), original);
        input.aggregation = UrpdAggregation::Raw;
        input.weight = UrpdWeight::Coinflow;
        assert_ne!(urpd_input::identity(&input), original);
        input.weight = UrpdWeight::Raw;
        input.date = Date::new(2009, 1, 4);
        assert_ne!(urpd_input::identity(&input), original);
        input.date = date;
        input.cohort = Cohort::new("another").unwrap();
        assert_ne!(urpd_input::identity(&input), original);
    });
    write(date, 200_000_000_u64);
    let response = exchange_with_etag(address, "GET", route, &old).await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let current = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    assert_ne!(old, current);
    let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(body["total_supply"], 2.0);
    check_response_admission(state, dated, &current).await;
    // Discovery must not select a newer directory or impossible calendar date.
    fs::create_dir(UrpdRaw::path(&path, &cohort, Date::new(2009, 1, 5)))
        .await
        .unwrap();
    symlink(
        UrpdRaw::path(&path, &cohort, date),
        UrpdRaw::path(&path, &cohort, Date::new(2009, 1, 6)),
    )
    .unwrap();
    fs::write(UrpdRaw::dir(&path, &cohort).join("2009-02-30"), b"")
        .await
        .unwrap();
    let dates = exchange_with_etag(address, "GET", "/api/urpd/fixture/dates", "\"old\"").await;
    assert!(dates.starts_with("HTTP/1.1 200"), "{dates}");
    assert_eq!(dates.split_once("\r\n\r\n").unwrap().1, "[\"2009-01-03\"]");
    for route in [route, dated] {
        let response = exchange_with_etag(address, "GET", route, &old).await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(
            response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap(),
            current
        );
        let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["total_supply"], 2.0);
        let response = exchange_with_etag(address, "HEAD", route, &old).await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.ends_with("\r\n\r\n"));
        for method in ["GET", "HEAD"] {
            for condition in [&current, "*"] {
                let response = exchange_with_etag(address, method, route, condition).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                assert!(response.ends_with("\r\n\r\n"));
                assert!(
                    response.contains("cdn-cache-control: public, max-age=1, must-revalidate\r\n")
                );
            }
            for query in [
                "?x=1",
                "?agg=raw&agg=raw",
                "?agg=raw&bucket=raw",
                "?weight=raw&weight=raw",
            ] {
                let response =
                    exchange_with_etag(address, method, &format!("{route}{query}"), "*").await;
                assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                assert!(!response.contains("\r\netag:"));
            }
        }
    }
    // Newer valid input with no price must not reuse the older successful body.
    let next_date = Date::new(2009, 1, 4);
    write(next_date, 300_000_000_u64);
    for variant in 0..4 {
        let status = if variant == 0 { 404 } else { 500 };
        let snapshot = UrpdRaw::path(&path, &cohort, next_date);
        if variant == 1 || variant == 2 {
            let mut bytes = b"invalid".to_vec();
            if variant == 2 {
                bytes = vec![0; 24];
                bytes[..8].copy_from_slice(&((UrpdRaw::MAX_ENTRIES + 1) as u64).to_le_bytes());
            }
            fs::write(&snapshot, bytes).await.unwrap();
        }
        if variant == 3 {
            OpenOptions::new()
                .write(true)
                .open(snapshot)
                .unwrap()
                .set_len(UrpdRaw::MAX_ENCODED_BYTES as u64 + 1)
                .unwrap();
        }
        for route in [route, "/api/urpd/fixture/2009-01-04"] {
            for method in ["GET", "HEAD"] {
                for condition in [&current, "*"] {
                    let response = exchange_with_etag(address, method, route, condition).await;
                    assert!(
                        response.starts_with(&format!("HTTP/1.1 {status}")),
                        "{response}"
                    );
                    assert!(!response.contains("\r\netag:"));
                    if method == "HEAD" {
                        assert!(response.ends_with("\r\n\r\n"));
                    }
                }
            }
        }
    }
    for method in ["GET", "HEAD"] {
        // A newer broken snapshot does not change a request for the older date.
        let response = exchange_with_etag(address, method, dated, &current).await;
        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        for route in [
            "/api/urpd/unknown/2009-01-03",
            "/api/urpd/fixture/2009-01-02",
        ] {
            let response = exchange_with_etag(address, method, route, "*").await;
            assert!(response.starts_with("HTTP/1.1 404"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
        for date in ["2009-02-30", "2009_01_03", "123%C3%A901-01"] {
            let response =
                exchange_with_etag(address, method, &format!("/api/urpd/fixture/{date}"), "*")
                    .await;
            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
            assert!(!response.contains("\r\netag:"));
            if method == "HEAD" {
                assert!(response.ends_with("\r\n\r\n"));
            }
        }
    }
    // Symlink loops provide deterministic metadata errors without relying on permissions.
    let loop_cohort = path.join("loop");
    symlink(&loop_cohort, &loop_cohort).unwrap();
    let loop_snapshot = UrpdRaw::path(&path, &cohort, Date::new(2009, 1, 7));
    symlink(&loop_snapshot, &loop_snapshot).unwrap();
    for route in [
        "/api/urpd/loop/dates",
        "/api/urpd/loop",
        "/api/urpd/loop/2009-01-03",
        "/api/urpd/fixture/2009-01-07",
    ] {
        for method in ["GET", "HEAD"] {
            for condition in [&current, "*"] {
                let response = exchange_with_etag(address, method, route, condition).await;
                assert!(response.starts_with("HTTP/1.1 500"), "{route}: {response}");
                assert!(!response.contains("\r\netag:"));
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
                assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                if method == "HEAD" {
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
        }
    }
    // 65,536 days after the populated day: an unchecked u16 cast aliases its price.
    write(Date::new(2188, 6, 9), 100_000_000_u64);
    for route in [route, "/api/urpd/fixture/2188-06-09"] {
        for method in ["GET", "HEAD"] {
            for condition in [&current, "*"] {
                let response = exchange_with_etag(address, method, route, condition).await;
                assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                assert!(!response.contains("\r\netag:"));
                if method == "GET" {
                    assert!(response.contains("unindexable_date"), "{response}");
                }
                if method == "HEAD" {
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
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

pub async fn check_weighted_errors(address: SocketAddr, dir: &Path, weight: UrpdWeight) {
    // Called by the empty-index fixture after publishing the raw date filename.
    assert!(!dir.try_exists().unwrap());
    fs::create_dir_all(dir.parent().unwrap()).await.unwrap();
    for broken in [false, true] {
        if broken {
            symlink(dir, dir).unwrap();
        }
        for suffix in ["/dates", "", "/2026-09-01"] {
            let route = format!("/api/urpd/all{suffix}?weight={weight}");
            check_weighted_error(address, &route, if broken { 500 } else { 404 }).await;
        }
    }
    // Remove only the loop created above; then exercise the dated-file preflight.
    fs::remove_file(dir).await.unwrap();
    fs::create_dir(dir).await.unwrap();
    let snapshot = dir.join("2026-09-01");
    let route = format!("/api/urpd/all/2026-09-01?weight={weight}");
    check_weighted_error(address, &route, 404).await;
    symlink(&snapshot, &snapshot).unwrap();
    check_weighted_error(address, &route, 500).await;
    fs::remove_file(&snapshot).await.unwrap();
    // Only September 1 exists in both sources; the newer weighted-only file
    // must not become latest. Empty payloads deliberately fail decoding.
    fs::write(&snapshot, b"").await.unwrap();
    let newer = dir.join("2026-09-02");
    fs::write(&newer, b"").await.unwrap();
    let dates_route = format!("/api/urpd/all/dates?weight={weight}");
    let response = exchange_with_etag(address, "GET", &dates_route, "\"old\"").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("\r\n\r\n[\"2026-09-01\"]"), "{response}");
    let etag = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap();
    for method in ["GET", "HEAD"] {
        let response = exchange_with_etag(address, method, &dates_route, etag).await;
        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        assert!(response.ends_with("\r\n\r\n"));
    }
    check_weighted_error(address, &format!("/api/urpd/all?weight={weight}"), 500).await;
    fs::remove_file(snapshot).await.unwrap();
    fs::remove_file(newer).await.unwrap();
    fs::remove_dir(dir).await.unwrap();
}

async fn check_weighted_error(address: SocketAddr, route: &str, status: u16) {
    for method in ["GET", "HEAD"] {
        for condition in ["\"old\"", "*"] {
            let response = exchange_with_etag(address, method, route, condition).await;
            assert!(
                response.starts_with(&format!("HTTP/1.1 {status}")),
                "{route}: {response}"
            );
            assert!(!response.contains("\r\netag:"));
            let policy = if status == 500 {
                "no-store"
            } else {
                "public, max-age=1, must-revalidate"
            };
            for header in ["cache-control", "cdn-cache-control"] {
                assert!(
                    response.contains(&format!("\r\n{header}: {policy}\r\n")),
                    "{response}"
                );
            }
            if method == "HEAD" {
                assert!(response.ends_with("\r\n\r\n"));
            }
        }
    }
}
