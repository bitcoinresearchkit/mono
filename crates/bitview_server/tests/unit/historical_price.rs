use std::{net::SocketAddr, time::Duration};

use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use brk_types::{HOUR4_INTERVAL, INDEX_EPOCH, Timestamp};
use serde_json::{Value, from_str, to_value};
use tokio::{
    spawn,
    task::JoinHandle,
    time::{self, timeout},
};
use tower::ServiceExt;

use super::server_routes::exchange_with_etag;
use crate::{AppState, api::ApiRoutes};

pub(crate) struct HistoricalPriceChecks {
    cases: Vec<(Option<Timestamp>, String, String, Value)>,
    pending: Vec<JoinHandle<String>>,
}

impl HistoricalPriceChecks {
    pub(crate) async fn before(state: &AppState, address: SocketAddr, first_time: u32) -> Self {
        let all = state.sync(|q| q.historical_price(None)).unwrap();
        let first_close =
            INDEX_EPOCH + ((1231006505 - INDEX_EPOCH) / HOUR4_INTERVAL + 1) * HOUR4_INTERVAL;
        assert_eq!(all.prices.len(), usize::from(first_time >= first_close));
        if let Some(price) = all.prices.first() {
            assert_eq!(*price.time, first_close);
        }
        let mut cases = Vec::new();
        for timestamp in [
            None,
            Some(0),
            Some(first_close - 1),
            Some(first_close),
            Some(u32::MAX),
        ] {
            let timestamp = timestamp.map(Timestamp::new);
            let path = timestamp.map_or_else(
                || "/api/v1/historical-price".to_owned(),
                |t| format!("/api/v1/historical-price?timestamp={t}"),
            );
            let expected =
                to_value(state.sync(|q| q.historical_price(timestamp)).unwrap()).unwrap();
            let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            let actual: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            assert_eq!(actual, expected);
            assert!(
                response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n")
            );
            let etag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            for method in ["GET", "HEAD"] {
                for tag in [etag.as_str(), "*"] {
                    let conditional = exchange_with_etag(address, method, &path, tag).await;
                    assert!(conditional.starts_with("HTTP/1.1 304"), "{conditional}");
                    assert!(conditional.ends_with("\r\n\r\n"));
                }
            }
            cases.push((timestamp, path, etag, expected));
        }
        let router = ApiRouter::new().add_api_routes().with_state(state.clone());
        let request = |method, tag: &str| {
            Request::builder()
                .method(method)
                .uri(&cases[0].1)
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
            for tag in [cases[0].2.as_str(), "*"] {
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
                let path = cases[0].1.clone();
                let tag = if method == "GET" {
                    cases[0].2.clone()
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
        Self {
            cases,
            pending: Vec::new(),
        }
    }

    pub(crate) async fn during_reorg(&mut self, address: SocketAddr) {
        for (_, path, etag, _) in &self.cases {
            let path = path.clone();
            let etag = etag.clone();
            self.pending.push(spawn(async move {
                exchange_with_etag(address, "GET", &path, &etag).await
            }));
        }
        assert!(
            timeout(Duration::from_millis(50), &mut self.pending[0])
                .await
                .is_err()
        );
        assert!(self.pending.iter().all(|task| !task.is_finished()));
    }

    pub(crate) async fn after(self, state: &AppState, address: SocketAddr) {
        for ((timestamp, path, etag, before), task) in self.cases.into_iter().zip(self.pending) {
            let current = to_value(state.sync(|q| q.historical_price(timestamp)).unwrap()).unwrap();
            let mut response = timeout(Duration::from_secs(5), task)
                .await
                .unwrap()
                .unwrap();
            if response.starts_with("HTTP/1.1 504") {
                assert!(!response.contains("\r\nretry-after:"));
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
                assert!(!response.contains("\r\netag:"));
                response = exchange_with_etag(address, "GET", &path, &etag).await;
            }
            if before == current {
                assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                assert!(response.ends_with("\r\n\r\n"));
            } else {
                assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
                let actual: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(actual, current);
            }
        }
    }
}
