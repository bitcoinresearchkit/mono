use std::{net::SocketAddr, time::Duration};

use bitview_plugin_price::feed_blocks_for_warmup;
use brk_oracle::Oracle;
use brk_types::{Date, Day1};
use serde_json::{Value, from_str, to_value};
use tokio::{spawn, time::timeout};

use super::server_routes::exchange_with_etag;
use crate::AppState;

pub async fn check(state: &AppState, address: SocketAddr) {
    state.sync(|q| {
        let safe = q.indexer().safe_lengths();
        let run = |range, cap| {
            let mut oracle = Oracle::from_seed();
            feed_blocks_for_warmup(&mut oracle, q.indexer(), range, Some(&cap))
        };
        assert!(run(0..1, safe).is_ok());
        assert!(run(usize::MAX..usize::MAX, safe).is_err());
        let mut short_txs = safe;
        short_txs.tx_index = 0_usize.into();
        assert!(run(0..1, short_txs).is_err());
        let mut short_outputs = safe;
        short_outputs.txout_index = 0_usize.into();
        assert!(run(0..1, short_outputs).is_err());
    });
    let day = Day1::try_from(Date::new(2009, 1, 3)).unwrap();
    let last_day = Day1::from(usize::from(u16::MAX));
    let mut valid = Vec::new();
    for payments in [false, true] {
        let kind = if payments { "payments" } else { "outputs" };
        for daily in [false, true] {
            let point = if daily {
                Date::from(day).to_string()
            } else {
                "0".to_owned()
            };
            let path = format!("/api/oracle/histogram/{kind}/{point}");
            let expected = state.sync(|q| {
                if payments {
                    to_value(
                        if daily {
                            q.confirmed_payment_histogram_day(day)
                        } else {
                            q.confirmed_payment_histogram(0)
                        }
                        .unwrap(),
                    )
                    .unwrap()
                } else {
                    to_value(
                        if daily {
                            q.confirmed_output_histogram_day(day)
                        } else {
                            q.confirmed_output_histogram(0)
                        }
                        .unwrap(),
                    )
                    .unwrap()
                }
            });
            let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            assert_eq!(body, expected);
            let etag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            for method in ["GET", "HEAD"] {
                for tag in [etag.as_str(), "*"] {
                    let response = exchange_with_etag(address, method, &path, tag).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
            valid.push((path, etag));
        }
        for point in [u32::MAX.to_string(), Date::from(last_day).to_string()] {
            let path = format!("/api/oracle/histogram/{kind}/{point}");
            for method in ["GET", "HEAD"] {
                let response = exchange_with_etag(address, method, &path, "*").await;
                assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                assert!(!response.contains("\r\netag:"));
                if method == "HEAD" {
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
        }
    }
    {
        let gate = state.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        let mut requests = Vec::new();
        for (path, etag) in &valid {
            let path = path.clone();
            let tag = etag.clone();
            requests.push(spawn(async move {
                exchange_with_etag(address, "GET", &path, &tag).await
            }));
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
        }
    }
    check_live(state, address).await;
}

async fn check_live(state: &AppState, address: SocketAddr) {
    let mut cases = Vec::new();
    for path in [
        "/api/oracle/price",
        "/api/mempool/price",
        "/api/oracle/histogram/payments/live",
        "/api/oracle/histogram/outputs/live",
    ] {
        let expected = state.sync(|q| match path {
            "/api/oracle/price" | "/api/mempool/price" => {
                to_value(q.live_price().unwrap()).unwrap()
            }
            "/api/oracle/histogram/payments/live" => {
                to_value(q.live_payment_histogram().unwrap()).unwrap()
            }
            _ => to_value(q.live_output_histogram().unwrap()).unwrap(),
        });
        let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let actual: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(actual, expected);
        let etag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap()
            .to_owned();
        for method in ["GET", "HEAD"] {
            for tag in [etag.as_str(), "*"] {
                let response = exchange_with_etag(address, method, path, tag).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                assert!(response.ends_with("\r\n\r\n"));
            }
        }
        cases.push((path, etag));
    }
    assert_eq!(
        cases[0].1, cases[1].1,
        "price aliases share representation identity"
    );
    {
        let gate = state.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        // Raw mempool outputs only pin the safe prefix; unlike the price and
        // payment oracle, they do not consume mutable confirmed oracle state.
        // Check this before waiting requests occupy worker admission.
        let (path, tag) = cases
            .iter()
            .find(|(path, _)| *path == "/api/oracle/histogram/outputs/live")
            .unwrap();
        let response = timeout(
            Duration::from_secs(5),
            exchange_with_etag(address, "GET", path, tag),
        )
        .await
        .unwrap();
        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        let mut requests = cases
            .iter()
            .filter(|(path, _)| *path != "/api/oracle/histogram/outputs/live")
            .map(|(path, tag)| {
                let path = *path;
                let tag = tag.clone();
                spawn(async move { exchange_with_etag(address, "GET", path, &tag).await })
            })
            .collect::<Vec<_>>();
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
        }
    }
}
