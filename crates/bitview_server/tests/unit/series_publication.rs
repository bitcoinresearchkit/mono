use std::{net::SocketAddr, time::Duration};

use brk_types::{Index, Timestamp};
use serde_json::{Value, from_slice, from_str, from_value, json};
use tokio::{spawn, time::timeout};
use vecdb::ReadableVec;

use super::server_routes::exchange_with_etag;
use crate::AppState;

pub(crate) async fn check(state: &AppState, address: SocketAddr, monotonic_time: u32) {
    let raw_tip = state
        .sync(|q| q.indexer().vecs().blocks.timestamp.collect_one(q.height()))
        .unwrap();
    for index in [Index::Day1, Index::Hour4] {
        let last = index
            .timestamp_to_index(Timestamp::new(monotonic_time))
            .unwrap();
        assert!(
            index.timestamp_to_index(raw_tip).unwrap() < last,
            "fixture must cross a bucket backwards"
        );
        assert_eq!(
            state.sync(|q| q.len(&"price_close".into(), index)).unwrap(),
            last + 1
        );
        // Public readers must install their own bounds, without a caller scope.
        state.sync(|query| {
            let params = from_value(json!({
                "series": "price_close", "index": index.name(),
            }))
            .unwrap();
            let read = query.search(&params).unwrap();
            let column = read.columns().next().unwrap();
            assert_eq!(column.visible_len(), last + 1);
            let mut expected = Vec::new();
            column.write_json(None, None, &mut expected).unwrap();
            let values: Value = from_slice(&expected).unwrap();
            assert_eq!(values.as_array().unwrap().len(), last + 1);
            assert_eq!(values[last].as_f64(), Some(0.0));
            drop(read);

            let resolved = query.resolve(params, usize::MAX).unwrap();
            let column = resolved.columns().next().unwrap();
            let mut actual = Vec::new();
            column
                .write_json(None, Some(usize::MAX), &mut actual)
                .unwrap();
            assert_eq!(actual, expected);
        });
        let path = format!(
            "/api/series/price_close/{}?start={last}&limit=1",
            index.name()
        );
        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["start"], last);
        assert_eq!(body["end"], last + 1);
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
        assert_eq!(body["data"][0].as_f64(), Some(0.0));
        let etag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap()
            .to_owned();
        let mut cases = vec![(path, etag)];
        for suffix in ["len", "latest"] {
            let path = format!("/api/series/price_close/{}/{suffix}", index.name());
            let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            if suffix == "len" {
                assert_eq!(body, last + 1);
            } else {
                assert_eq!(body.as_f64(), Some(0.0));
            }
            let etag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            cases.push((path, etag));
        }
        let gate = state.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        let mut requests = Vec::new();
        for (path, etag) in cases {
            for method in ["GET", "HEAD"] {
                let path = path.clone();
                let tag = if method == "GET" {
                    etag.clone()
                } else {
                    "*".to_owned()
                };
                requests.push(spawn(async move {
                    exchange_with_etag(address, method, &path, &tag).await
                }));
            }
        }
        assert!(
            timeout(Duration::from_millis(50), &mut requests[0])
                .await
                .is_err()
        );
        assert!(requests.iter().all(|request| !request.is_finished()));
        gate.finish_update();
        for request in requests {
            let response = timeout(Duration::from_secs(5), request)
                .await
                .unwrap()
                .unwrap();
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.ends_with("\r\n\r\n"));
        }
    }
}
