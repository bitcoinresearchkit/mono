use std::{net::SocketAddr, time::Duration};

use aide::axum::ApiRouter;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, from_str, to_value};
use tokio::{spawn as TokioSpawn, task::JoinSet, time};
use tower::ServiceExt;

use super::server_routes::exchange_with_etag;
use crate::{AppState, api::ApiRoutes};

const PATHS: [&str; 3] = ["/api/mempool", "/api/mempool/recent", "/api/mempool/txids"];

pub(crate) struct MempoolPublication {
    state: AppState,
    tags: Vec<String>,
}

impl MempoolPublication {
    pub(crate) fn new(state: AppState) -> Self {
        Self {
            state,
            tags: Vec::new(),
        }
    }

    pub(crate) async fn check_unavailable(&self, address: SocketAddr) {
        let mut requests = JoinSet::new();
        for (index, path) in PATHS.iter().enumerate() {
            for method in ["GET", "HEAD"] {
                for tag in [
                    "*",
                    self.tags
                        .get(index)
                        .map(String::as_str)
                        .unwrap_or("\"old\""),
                ] {
                    let path = path.to_string();
                    let tag = tag.to_owned();
                    requests.spawn(async move {
                        let response = exchange_with_etag(address, method, &path, &tag).await;
                        assert!(response.starts_with("HTTP/1.1 503"), "{path}: {response}");
                        assert!(!response.contains("\r\netag:"));
                        assert!(response.contains("\r\ncache-control: no-store\r\n"));
                    });
                }
            }
        }
        while let Some(result) = requests.join_next().await {
            result.unwrap();
        }
    }

    pub(crate) async fn check_available(&mut self, address: SocketAddr) {
        let first = self.tags.is_empty();
        self.tags.clear();
        for (index, path) in PATHS.iter().enumerate() {
            let expected = self.state.sync(|q| match index {
                0 => to_value(q.mempool_info().unwrap()).unwrap(),
                1 => to_value(q.mempool_recent().unwrap()).unwrap(),
                _ => {
                    to_value(q.mempool_txids_with_hash().map(|(txids, _)| txids).unwrap()).unwrap()
                }
            });
            let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            assert_eq!(body, expected);
            let tag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            for method in ["GET", "HEAD"] {
                for validator in [tag.as_str(), "*"] {
                    let response = exchange_with_etag(address, method, path, validator).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
            self.tags.push(tag);
        }
        if first {
            self.check_txid_ownership().await;
        }
    }

    #[cfg(feature = "price")]
    pub(crate) async fn check_live_outputs(&self, address: SocketAddr) {
        time::timeout(Duration::from_secs(5), async {
            let mut prices = Vec::new();
            for path in LIVE_OUTPUT_PATHS {
                let expected = self.state.sync(|q| match path {
                    "/api/oracle/price" | "/api/mempool/price" => {
                        to_value(q.live_price().unwrap()).unwrap()
                    }
                    "/api/oracle/histogram/payments/live" => {
                        to_value(q.live_payment_histogram().unwrap()).unwrap()
                    }
                    _ => to_value(q.live_output_histogram().unwrap()).unwrap(),
                });
                let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
                assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
                let actual: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(actual, expected);
                let tag = response
                    .lines()
                    .find_map(|line| line.strip_prefix("etag: "))
                    .unwrap();
                if path.ends_with("/price") {
                    prices.push(tag.to_owned());
                }
                for method in ["GET", "HEAD"] {
                    for condition in [tag, "*"] {
                        let response = exchange_with_etag(address, method, path, condition).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                        assert!(response.ends_with("\r\n\r\n"));
                        assert!(response.contains(&format!("\r\netag: {tag}\r\n")));
                    }
                }
            }
            assert_eq!(prices[0], prices[1]);
        })
        .await
        .expect("published price and histograms must remain available during private updates");
    }

    #[cfg(feature = "price")]
    pub(crate) async fn check_live_outputs_unavailable(&self, address: SocketAddr) {
        for path in LIVE_OUTPUT_PATHS {
            for method in ["GET", "HEAD"] {
                let response = exchange_with_etag(address, method, path, "*").await;
                assert!(response.starts_with("HTTP/1.1 503"), "{path}: {response}");
                assert!(!response.contains("\r\netag:"));
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
            }
        }
    }

    async fn check_txid_ownership(&self) {
        let router = ApiRouter::new()
            .add_api_routes()
            .with_state(self.state.clone());
        let request = |method, tag| {
            Request::builder()
                .method(method)
                .uri(PATHS[2])
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
        assert_eq!(self.state.mempool_txid_bodies.available_permits(), 0);
        let pending = TokioSpawn(router.clone().oneshot(request("GET", "\"old\"")));
        time::sleep(Duration::from_millis(50)).await;
        assert!(!pending.is_finished());
        for method in ["GET", "HEAD"] {
            for tag in [self.tags[2].as_str(), "*"] {
                let response = router.clone().oneshot(request(method, tag)).await.unwrap();
                assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
            }
        }
        drop(first);
        let admitted = pending.await.unwrap().unwrap();
        assert_eq!(admitted.status(), StatusCode::OK);
        drop(admitted);
        assert_eq!(self.state.mempool_txid_bodies.available_permits(), 1);
        drop(second);
        assert_eq!(self.state.mempool_txid_bodies.available_permits(), 2);
    }
}

#[cfg(feature = "price")]
const LIVE_OUTPUT_PATHS: [&str; 4] = [
    "/api/oracle/price",
    "/api/mempool/price",
    "/api/oracle/histogram/payments/live",
    "/api/oracle/histogram/outputs/live",
];
