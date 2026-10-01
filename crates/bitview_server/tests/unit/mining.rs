use std::net::SocketAddr;

use brk_types::PoolSlug;
use serde_json::{Value, from_str, to_string, to_value};
use vecdb::ReadableVec;

use super::server_routes::exchange_with_etag;
use crate::AppState;

const STAT_ROUTES: &[&str] = &[
    "/api/v1/mining/pools/24h",
    "/api/v1/mining/pool/unknown",
    "/api/v1/mining/hashrate/pools",
    "/api/v1/mining/hashrate/pools/all",
    "/api/v1/mining/hashrate/pools/1m",
    "/api/v1/mining/pool/unknown/hashrate",
    "/api/v1/mining/hashrate",
    "/api/v1/mining/hashrate/all",
    "/api/v1/mining/hashrate/24h",
    "/api/v1/mining/difficulty-adjustments",
    "/api/v1/mining/difficulty-adjustments/all",
    "/api/v1/mining/difficulty-adjustments/24h",
    "/api/v1/mining/reward-stats/2",
    "/api/v1/mining/blocks/fees/24h",
    "/api/v1/mining/blocks/rewards/24h",
    "/api/v1/mining/blocks/fee-rates/24h",
    "/api/v1/mining/blocks/sizes-weights/24h",
];

pub(crate) async fn check_statistics(address: SocketAddr) -> Vec<(&'static str, String)> {
    let mut validators = Vec::new();
    for &path in STAT_ROUTES {
        let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
        let tag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap();
        let expected = response.split_once("\r\n\r\n").unwrap().1;
        for method in ["GET", "HEAD"] {
            for condition in [tag, tag.strip_prefix("W/").unwrap(), "*"] {
                let response = exchange_with_etag(address, method, path, condition).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                assert!(response.ends_with("\r\n\r\n"));
            }
            let response = exchange_with_etag(address, method, path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
            assert_eq!(
                response.split_once("\r\n\r\n").unwrap().1,
                if method == "HEAD" { "" } else { expected }
            );
            let response = exchange_with_etag(address, method, &format!("{path}?x=1"), tag).await;
            assert!(response.starts_with("HTTP/1.1 400"), "{path}: {response}");
            assert!(!response.contains("\r\netag:"));
        }
        validators.push((path, tag.to_owned()));
    }
    validators
}

pub(crate) async fn check_pool_blocks(state: &AppState, address: SocketAddr) {
    state.sync(|q| {
        let detail = q.pool_detail(PoolSlug::Unknown).unwrap();
        assert_eq!(detail.block_count.all, 2);
        assert!((1..=2).contains(&detail.block_count.day));
        assert!((1..=2).contains(&detail.block_count.week));
        assert_eq!(detail.block_share.all, 1.0);
        assert_eq!(detail.block_share.day, 1.0);
        assert_eq!(detail.block_share.week, 1.0);
    });
    for (suffix, before, heights) in [
        ("", None, vec![1, 0]),
        ("/0", Some(0u32.into()), vec![0]),
        ("/4294967295", Some(u32::MAX.into()), vec![1, 0]),
    ] {
        let path = format!("/api/v1/mining/pool/unknown/blocks{suffix}");
        let expected = state.sync(|q| {
            let prices = &q.plugins().price.spot.cents.height;
            let snapshot = q
                .resolve_pool_blocks(PoolSlug::Unknown, before, 100)
                .unwrap();
            assert_eq!(
                snapshot
                    .heights()
                    .iter()
                    .copied()
                    .map(u32::from)
                    .collect::<Vec<_>>(),
                heights
            );
            let captured = snapshot.prices().to_vec();
            let retained = prices.collect_range_at(0, 2);
            let rows = q.pool_blocks_resolved(snapshot).unwrap();
            assert_eq!(
                rows.iter().map(|row| row.extras.price).collect::<Vec<_>>(),
                captured
            );
            let mut cached = Vec::new();
            assert!(prices.read_cached_into_at(0, 2, &mut cached));
            assert_eq!(cached, retained);
            let body = to_string(&rows).unwrap();
            assert_eq!(
                body,
                to_string(
                    &q.resolve_pool_blocks(PoolSlug::Unknown, before, 100)
                        .and_then(|resolved| q.pool_blocks_resolved(resolved))
                        .unwrap()
                )
                .unwrap()
            );
            body
        });
        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
        let tag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap();
        for method in ["GET", "HEAD"] {
            for condition in [tag, tag.strip_prefix("W/").unwrap(), "*"] {
                let response = exchange_with_etag(address, method, &path, condition).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                assert!(response.ends_with("\r\n\r\n"));
            }
            let response = exchange_with_etag(address, method, &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            assert_eq!(
                response.split_once("\r\n\r\n").unwrap().1,
                if method == "HEAD" { "" } else { &expected }
            );
            let response = exchange_with_etag(address, method, &format!("{path}?x=1"), tag).await;
            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
    }
    for method in ["GET", "HEAD"] {
        for path in [
            "/api/v1/mining/pool/not-a-pool/blocks",
            "/api/v1/mining/pool/unknown/blocks/4294967296",
        ] {
            let response = exchange_with_etag(address, method, path, "*").await;
            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
    }
    let path = "/api/v1/mining/pools";
    let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let tag = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap();
    assert!(
        tag.starts_with("W/\"c"),
        "catalog must use content identity: {tag}"
    );
    let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(body, state.sync(|q| to_value(q.all_pools()).unwrap()));
    for method in ["GET", "HEAD"] {
        let response = exchange_with_etag(address, method, path, tag).await;
        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
    }
}
