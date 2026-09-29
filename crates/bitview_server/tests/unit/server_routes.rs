use super::broadcast;
#[cfg(feature = "chain")]
use crate::test_cache::init_cache;
#[cfg(feature = "chain")]
use bitcoin::consensus::encode;
#[cfg(feature = "chain")]
use brk_types::BlockHash;
#[cfg(any(feature = "chain", all(feature = "chain", feature = "series")))]
use serde_json::from_str;
#[cfg(feature = "chain")]
use serde_json::to_string as SerdeJsonToString;
#[cfg(feature = "chain")]
use std::str as StdStr;
#[cfg(any(feature = "chain", all(feature = "chain", feature = "series")))]
use tempfile::tempdir;
#[cfg(feature = "chain")]
use tokio::fs;
#[cfg(any(feature = "chain", all(feature = "chain", feature = "series")))]
use tokio::spawn as TokioSpawn;
#[cfg(feature = "chain")]
use tokio::time;

use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[cfg(feature = "chain")]
use std::{
    net::{Ipv4Addr, TcpListener as StdListener},
    sync::mpsc as blocking_channel,
    thread,
    time::Duration,
};

#[cfg(feature = "chain")]
use axum::{body::to_bytes, http::header::ETAG};
#[cfg(feature = "chain")]
use bitview_default::DefaultPlugins;
#[cfg(feature = "chain")]
use bitview_plugin::ImportContext;
#[cfg(feature = "chain")]
use bitview_query::AsyncQuery;
#[cfg(all(feature = "chain", feature = "series"))]
use bitview_types::{Limit, Pagination, SearchQuery};
#[cfg(feature = "chain")]
use brk_reader::Reader;
#[cfg(feature = "chain")]
use brk_rpc::{Auth, Client};
#[cfg(feature = "chain")]
use serde_json::Value;
#[cfg(feature = "chain")]
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    runtime::Builder,
    sync::{mpsc, oneshot},
    task::{spawn_blocking, yield_now},
    time::timeout,
};

#[cfg(feature = "chain")]
use vecdb::ReadableVec;

#[cfg(feature = "chain")]
use crate::{AppState, Server, ServerConfig};

#[cfg(feature = "chain")]
pub async fn check_recent_blocks(state: &AppState, address: SocketAddr) {
    check_height_block_lists(state, address).await;
    let timestamp_path = "/api/v1/mining/blocks/timestamp/4294967295";
    let timestamp_response = exchange_with_etag(address, "GET", timestamp_path, "\"old\"").await;
    assert!(
        timestamp_response.starts_with("HTTP/1.1 200"),
        "{timestamp_response}"
    );
    let timestamp_tag = timestamp_response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let timestamp_list = format!("\"ignored,tag\", {timestamp_tag}");
    let expected = state.sync(|q| {
        SerdeJsonToString(
            &q.resolve_block_by_timestamp(u32::MAX.into())
                .map(|resolved| resolved.into_value())
                .unwrap(),
        )
        .unwrap()
    });
    assert_eq!(
        timestamp_response.split_once("\r\n\r\n").unwrap().1,
        expected
    );
    for method in ["GET", "HEAD"] {
        for condition in [
            timestamp_tag.as_str(),
            "*",
            timestamp_tag.strip_prefix("W/").unwrap(),
            timestamp_list.as_str(),
        ] {
            let response = exchange_with_etag(address, method, timestamp_path, condition).await;
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"));
            assert!(
                response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n")
            );
            assert!(response.ends_with("\r\n\r\n"));
            let response = exchange_with_etag(
                address,
                method,
                "/api/v1/mining/blocks/timestamp/0",
                condition,
            )
            .await;
            assert!(response.starts_with("HTTP/1.1 404"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
        let response = exchange_with_etag(
            address,
            method,
            &format!("{timestamp_path}?x=1"),
            &timestamp_tag,
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
        for value in ["4294967296", "-1", "nope"] {
            let response = exchange_with_etag(
                address,
                method,
                &format!("/api/v1/mining/blocks/timestamp/{value}"),
                "*",
            )
            .await;
            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
        let response =
            exchange_with_etag(address, method, &format!("{timestamp_path}?"), "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(
            response.split_once("\r\n\r\n").unwrap().1,
            if method == "HEAD" { "" } else { &expected }
        );
    }
    let response = exchange_with_etag(address, "POST", timestamp_path, &timestamp_tag).await;
    assert!(response.starts_with("HTTP/1.1 405"), "{response}");
    state.sync(|q| {
        assert!(
            q.indexer()
                .vecs()
                .blocks
                .timestamp
                .read_cached_into_at(1, 2, &mut Vec::new())
        );
        assert!(
            q.plugins()
                .mappings
                .timestamp
                .monotonic
                .read_cached_into_at(1, 2, &mut Vec::new())
        );
    });
    check_block_height(state, address).await;
    let historical = exchange_with_etag(address, "GET", "/api/v1/blocks/0", "\"old\"").await;
    assert!(historical.starts_with("HTTP/1.1 200"), "{historical}");
    let historical_tag = historical
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let historical_expected = state.sync(|q| {
        SerdeJsonToString(
            &q.resolve_blocks_v1(Some(0u32.into()), 15)
                .and_then(|resolved| resolved.build(q))
                .unwrap(),
        )
        .unwrap()
    });
    assert_eq!(
        historical.split_once("\r\n\r\n").unwrap().1,
        historical_expected
    );
    let hash = state.sync(|q| q.resolve_blocks(None, 1).unwrap().anchor().unwrap());
    let base_path = format!("/api/block/{hash}");
    let raw_path = format!("{base_path}/raw");
    let raw_response = exchange_bytes(address, "GET", &raw_path, "\"old\"", 4_100_000).await;
    let boundary = raw_response
        .windows(4)
        .position(|v| v == b"\r\n\r\n")
        .unwrap()
        + 4;
    let raw_headers = StdStr::from_utf8(&raw_response[..boundary]).unwrap();
    assert!(raw_headers.starts_with("HTTP/1.1 200"), "{raw_headers}");
    assert!(raw_headers.contains("\r\ncontent-type: application/octet-stream\r\n"));
    assert!(raw_headers.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"));
    assert!(raw_headers.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"));
    let raw_tag = raw_headers
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let expected_raw = state.sync(|q| {
        let height = q
            .resolve_block_snapshot(&hash)
            .map(|resolved| resolved.last_height().unwrap())
            .unwrap();
        let blocks = &q.indexer().vecs().blocks;
        q.reader()
            .read_raw_bytes(
                blocks.position.collect_one(height).unwrap(),
                *blocks.total.collect_one(height).unwrap() as usize,
            )
            .unwrap()
    });
    assert_eq!(&raw_response[boundary..], expected_raw);
    let base = exchange_with_etag(address, "GET", &base_path, "\"old\"").await;
    assert!(base.starts_with("HTTP/1.1 200"), "{base}");
    assert!(
        base.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"),
        "{base}"
    );
    assert!(
        base.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"),
        "{base}"
    );
    let base_tag = base
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let base_expected = state.sync(|q| {
        SerdeJsonToString(
            &q.resolve_block_snapshot(&hash)
                .and_then(|resolved| resolved.build(q))
                .map(|mut rows| rows.pop().unwrap())
                .unwrap(),
        )
        .unwrap()
    });
    assert_eq!(base.split_once("\r\n\r\n").unwrap().1, base_expected);
    let header_path = format!("{base_path}/header");
    let header = exchange_with_etag(address, "GET", &header_path, "\"old\"").await;
    assert!(header.starts_with("HTTP/1.1 200"), "{header}");
    assert!(
        header.contains("\r\ncontent-type: text/plain\r\n"),
        "{header}"
    );
    assert!(
        header.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"),
        "{header}"
    );
    assert!(
        header.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"),
        "{header}"
    );
    let header_tag = header
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let header_expected = state.sync(|q| {
        let bytes = q
            .resolve_block_snapshot(&hash)
            .unwrap()
            .anchor_header_hex(q)
            .unwrap();
        let header: bitcoin::block::Header = encode::deserialize_hex(&bytes).unwrap();
        assert_eq!(BlockHash::from(header.block_hash()), hash);
        bytes
    });
    assert_eq!(header_expected.len(), 160);
    assert_eq!(header.split_once("\r\n\r\n").unwrap().1, header_expected);
    let single_path = format!("/api/v1/block/{hash}");
    let single = exchange_with_etag(address, "GET", &single_path, "\"old\"").await;
    assert!(single.starts_with("HTTP/1.1 200"), "{single}");
    let single_tag = single
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap()
        .to_owned();
    let single_expected = state.sync(|q| {
        SerdeJsonToString(
            &q.resolve_block_v1(&hash)
                .unwrap()
                .build(q)
                .unwrap()
                .pop()
                .unwrap(),
        )
        .unwrap()
    });
    assert_eq!(single.split_once("\r\n\r\n").unwrap().1, single_expected);
    let v1 = state
        .respond_blocks_v1(Default::default(), None)
        .await
        .unwrap_or_else(|_| panic!("V1 response failed"));
    let v1_tag = v1.headers()[ETAG].to_str().unwrap().to_owned();
    let v1_expected = state.sync(|q| {
        SerdeJsonToString(
            &q.resolve_blocks_v1(None, 15)
                .and_then(|resolved| resolved.build(q))
                .unwrap(),
        )
        .unwrap()
    });
    assert_eq!(
        to_bytes(v1.into_body(), usize::MAX).await.unwrap().as_ref(),
        v1_expected.as_bytes()
    );
    let (etag, expected) = state.sync(|q| {
        let snapshot = q.resolve_blocks(None, 10).unwrap();
        let etag = format!("W/\"blocks2-{}\"", snapshot.anchor().unwrap());
        let body = SerdeJsonToString(&snapshot.build(q).unwrap()).unwrap();
        assert_eq!(
            body,
            SerdeJsonToString(&q.resolve_blocks(None, 10).unwrap().build(q).unwrap()).unwrap()
        );
        (etag, body)
    });
    let permits = state
        .sync_query
        .clone()
        .acquire_many_owned(state.sync_query.available_permits() as u32)
        .await
        .unwrap();
    // Exact and wildcard validators work before any origin body was requested,
    // even with no blocking-work admission available.
    for method in ["GET", "HEAD"] {
        for condition in [header_tag.as_str(), header_tag.strip_prefix("W/").unwrap()] {
            let header = timeout(
                Duration::from_secs(2),
                exchange_with_etag(address, method, &header_path, condition),
            )
            .await
            .unwrap();
            assert!(header.starts_with("HTTP/1.1 304"), "{header}");
            assert!(header.ends_with("\r\n\r\n"));
        }
        for condition in [raw_tag.as_str(), raw_tag.strip_prefix("W/").unwrap()] {
            let raw = timeout(
                Duration::from_secs(2),
                exchange_with_etag(address, method, &raw_path, condition),
            )
            .await
            .unwrap();
            assert!(raw.starts_with("HTTP/1.1 304"), "{raw}");
            assert!(raw.ends_with("\r\n\r\n"));
        }
        let base = timeout(
            Duration::from_secs(2),
            exchange_with_etag(address, method, &base_path, &base_tag),
        )
        .await
        .unwrap();
        assert!(base.starts_with("HTTP/1.1 304"), "{base}");
        assert!(base.ends_with("\r\n\r\n"));
        let single = timeout(
            Duration::from_secs(2),
            exchange_with_etag(address, method, &single_path, &single_tag),
        )
        .await
        .unwrap();
        assert!(single.starts_with("HTTP/1.1 304"), "{single}");
        assert!(single.ends_with("\r\n\r\n"));
        for condition in [historical_tag.as_str(), "*"] {
            let response = timeout(
                Duration::from_secs(2),
                exchange_with_etag(address, method, "/api/v1/blocks/0", condition),
            )
            .await
            .unwrap();
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.ends_with("\r\n\r\n"));
        }
        for condition in [v1_tag.as_str(), "*"] {
            let response = timeout(
                Duration::from_secs(2),
                exchange_with_etag(address, method, "/api/v1/blocks", condition),
            )
            .await
            .unwrap();
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.ends_with("\r\n\r\n"));
        }
        for condition in [etag.as_str(), "*"] {
            let response = timeout(
                Duration::from_secs(2),
                exchange_with_etag(address, method, "/api/blocks", condition),
            )
            .await
            .unwrap();
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.ends_with("\r\n\r\n"));
        }
    }
    drop(permits);
    assert!(
        exchange_with_etag(address, "GET", &raw_path, "*")
            .await
            .starts_with("HTTP/1.1 304")
    );
    for condition in ["*", raw_tag.as_str()] {
        let unknown = format!("/api/block/{}/raw", "0".repeat(64));
        let response = exchange_with_etag(address, "GET", &unknown, condition).await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        assert!(!response.contains("\r\netag:"));
    }
    assert!(
        exchange_with_etag(address, "GET", &format!("{raw_path}?x=1"), &raw_tag)
            .await
            .starts_with("HTTP/1.1 400")
    );
    for (condition, status) in [
        ("*".to_owned(), "304"),
        ("W/\"block-header-v2-4294967295-forged\"".to_owned(), "200"),
        (
            format!("W/\"block-header-v2-0-forged\", {header_tag}"),
            "304",
        ),
    ] {
        let response = exchange_with_etag(address, "GET", &header_path, &condition).await;
        assert!(
            response.starts_with(&format!("HTTP/1.1 {status}")),
            "{response}"
        );
    }
    for condition in ["*", header_tag.as_str()] {
        let unknown = format!("/api/block/{}/header", "0".repeat(64));
        let response = exchange_with_etag(address, "GET", &unknown, condition).await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        assert!(!response.contains("\r\netag:"));
    }
    let invalid =
        exchange_with_etag(address, "GET", &format!("{header_path}?x=1"), &header_tag).await;
    assert!(invalid.starts_with("HTTP/1.1 400"), "{invalid}");
    for (condition, status) in [
        ("*".to_owned(), "304"),
        ("W/\"block-v3-4294967295-forged\"".to_owned(), "200"),
        (format!("W/\"block-v3-0-forged\", {base_tag}"), "304"),
    ] {
        let response = exchange_with_etag(address, "GET", &base_path, &condition).await;
        assert!(
            response.starts_with(&format!("HTTP/1.1 {status}")),
            "{response}"
        );
    }
    for condition in ["*", base_tag.as_str()] {
        let unknown = format!("/api/block/{}", "0".repeat(64));
        let response = exchange_with_etag(address, "GET", &unknown, condition).await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        assert!(!response.contains("\r\netag:"));
    }
    let invalid = exchange_with_etag(address, "GET", &format!("{base_path}?x=1"), &base_tag).await;
    assert!(invalid.starts_with("HTTP/1.1 400"), "{invalid}");
    for (condition, status) in [
        ("*".to_owned(), "304"),
        ("W/\"block-v1-4-4294967295-forged\"".to_owned(), "200"),
        (format!("W/\"block-v1-4-0-forged\", {single_tag}"), "304"),
    ] {
        let response = exchange_with_etag(address, "GET", &single_path, &condition).await;
        assert!(
            response.starts_with(&format!("HTTP/1.1 {status}")),
            "{response}"
        );
    }
    for condition in ["*", single_tag.as_str()] {
        let unknown = format!("/api/v1/block/{}", "0".repeat(64));
        let response = exchange_with_etag(address, "GET", &unknown, condition).await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        assert!(!response.contains("\r\netag:"));
    }
    let invalid =
        exchange_with_etag(address, "GET", &format!("{single_path}?x=1"), &single_tag).await;
    assert!(invalid.starts_with("HTTP/1.1 400"), "{invalid}");
    let retained = state.sync(|q| {
        let mut values = Vec::new();
        let hit = q
            .plugins()
            .price
            .spot
            .cents
            .height
            .read_cached_into_at(0, 2, &mut values);
        (hit, values)
    });
    let cold = exchange_with_etag(address, "GET", "/api/v1/blocks", &v1_tag).await;
    assert!(cold.starts_with("HTTP/1.1 304"), "{cold}");
    state.sync(|q| {
        let mut values = Vec::new();
        let hit = q
            .plugins()
            .price
            .spot
            .cents
            .height
            .read_cached_into_at(0, 2, &mut values);
        assert_eq!(
            (hit, values),
            retained,
            "V1 validation must leave price retention unchanged"
        );
    });
    let response = exchange_with_etag(address, "GET", "/api/blocks", "\"old\"").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
    assert_eq!(
        response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap(),
        etag
    );

    // Mutable-source reads retain admission while awaiting publication.
    // Immutable prefix routes no longer wait on this gate.
    #[derive(Clone, Copy)]
    enum MutableBlockRead {
        List,
        Single,
    }
    for mode in [MutableBlockRead::List, MutableBlockRead::Single] {
        assert_eq!(state.sync_query.available_permits(), 1);
        let gate = state.sync(|q| q.indexer().publication().clone());
        let closing = gate.clone();
        spawn_blocking(move || closing.begin_update())
            .await
            .unwrap();
        let spawn_request = |owned: AppState| {
            TokioSpawn(async move {
                match mode {
                    MutableBlockRead::List => {
                        owned.respond_blocks_v1(Default::default(), None).await
                    }
                    MutableBlockRead::Single => {
                        owned.respond_block_v1(Default::default(), hash).await
                    }
                }
            })
        };
        let request = spawn_request(state.clone());
        time::sleep(Duration::from_millis(50)).await;
        assert!(!request.is_finished());
        assert_eq!(state.sync_query.available_permits(), 0);
        request.abort();
        assert!(matches!(request.await, Err(error) if error.is_cancelled()));
        let retained = state.sync_query.available_permits() == 0;
        let mut following = spawn_request(state.clone());
        let queued = timeout(Duration::from_millis(50), &mut following)
            .await
            .is_err();
        gate.finish_update();
        assert!(
            retained,
            "cancelled publication wait released worker admission"
        );
        assert!(queued, "following block request bypassed publication");
        let response = timeout(Duration::from_secs(2), following)
            .await
            .unwrap()
            .unwrap()
            .unwrap_or_else(|_| panic!("following block response failed"));
        assert_eq!(response.status(), 200);
        assert_eq!(state.sync_query.available_permits(), 1);
    }
}

#[cfg(feature = "chain")]
async fn check_height_block_lists(state: &AppState, address: SocketAddr) {
    let mut cases = Vec::new();
    for height in [0u32, 1, u32::MAX] {
        let path = format!("/api/blocks/{height}");
        let (expected, anchor) = state.sync(|q| {
            let snapshot = q.resolve_blocks(Some(height.into()), 10).unwrap();
            let anchor = snapshot.anchor().unwrap();
            let expected = SerdeJsonToString(&snapshot.build(q).unwrap()).unwrap();
            assert_eq!(
                expected,
                SerdeJsonToString(
                    &q.resolve_blocks(Some(height.into()), 10)
                        .unwrap()
                        .build(q)
                        .unwrap()
                )
                .unwrap()
            );
            (expected, anchor)
        });
        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
        let etag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap()
            .to_owned();
        assert_eq!(etag, format!("W/\"blocks2-{anchor}\""));
        cases.push((path, etag));
    }
    let gate = state.sync(|q| q.indexer().publication().clone());
    gate.begin_update();
    let mut pending = Vec::new();
    for (path, etag) in cases {
        for method in ["GET", "HEAD"] {
            let path = path.clone();
            let tag = if method == "GET" {
                etag.clone()
            } else {
                "*".to_owned()
            };
            pending.push(TokioSpawn(async move {
                exchange_with_etag(address, method, &path, &tag).await
            }));
        }
    }
    for request in pending {
        let response = timeout(Duration::from_secs(1), request)
            .await
            .unwrap()
            .unwrap();
        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        assert!(response.ends_with("\r\n\r\n"));
    }
    gate.finish_update();
}

#[cfg(feature = "chain")]
async fn check_block_height(state: &AppState, address: SocketAddr) {
    let expected = state.sync(|q| {
        q.resolve_blocks(Some(0u32.into()), 1)
            .unwrap()
            .anchor()
            .unwrap()
            .to_string()
    });
    // No worker admission is required for the bounded, uncontended read,
    // including a full 200 response.
    let permits = state
        .sync_query
        .clone()
        .acquire_many_owned(state.sync_query.available_permits() as u32)
        .await
        .unwrap();
    let response = timeout(
        Duration::from_secs(2),
        exchange_with_etag(address, "GET", "/api/block-height/0", "\"old\""),
    )
    .await
    .unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
    assert!(response.contains("\r\ncontent-type: text/plain\r\n"));
    let tag = response
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .unwrap();
    for method in ["GET", "HEAD"] {
        for condition in [
            tag.to_owned(),
            tag.strip_prefix("W/").unwrap().to_owned(),
            "*".to_owned(),
            format!("\"irrelevant,tag\", {tag}"),
        ] {
            let response =
                exchange_with_etag(address, method, "/api/block-height/0", &condition).await;
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
            assert!(response.ends_with("\r\n\r\n"));
            assert!(response.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"));
            assert!(
                response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n")
            );
        }
        for path in [
            "/api/block-height/4294967295",
            "/api/block-height/0?x=1",
            "/api/block-height/4294967296",
            "/api/block-height/-1",
            "/api/block-height/nope",
        ] {
            for condition in [tag, "*"] {
                let response = exchange_with_etag(address, method, path, condition).await;
                let code = if path.ends_with("4294967295") {
                    "404"
                } else {
                    "400"
                };
                assert!(
                    response.starts_with(&format!("HTTP/1.1 {code}")),
                    "{response}"
                );
                assert!(!response.contains("\r\netag:"));
            }
        }
        let response = exchange_with_etag(address, method, "/api/block-height/0?", "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(
            response.split_once("\r\n\r\n").unwrap().1,
            if method == "HEAD" { "" } else { &expected }
        );
    }
    let response = exchange_with_etag(address, "POST", "/api/block-height/0", tag).await;
    assert!(response.starts_with("HTTP/1.1 405"), "{response}");
    drop(permits);
}

#[cfg(feature = "chain")]
async fn exchange(address: SocketAddr, method: &str, path: &str) -> String {
    exchange_with_etag(address, method, path, "*").await
}

#[cfg(feature = "chain")]
pub async fn check_tip_cached_routes(address: SocketAddr) {
    for route in [
        "/api/blocks",
        "/api/v1/blocks",
        "/api/v1/blocks/0",
        "/api/v1/mining/difficulty-adjustments",
    ] {
        let response = exchange_with_etag(address, "GET", route, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{route}: {response}");
        let etag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap();
        for method in ["GET", "HEAD"] {
            for condition in [etag, "*"] {
                let response = exchange_with_etag(address, method, route, condition).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{route}: {response}");
                assert!(response.ends_with("\r\n\r\n"));
            }
            let response = exchange_with_etag(address, method, &format!("{route}?x=1"), etag).await;
            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
            assert!(!response.contains("\r\netag:"));
        }
    }
}

#[cfg(all(feature = "chain", feature = "series"))]
fn series_name_paths(encoded: &str) -> Vec<String> {
    let mut paths = vec![format!(
        "/api/series/bulk?series={encoded}&index=height&limit=0"
    )];
    for prefix in ["series"] {
        paths.push(format!("/api/{prefix}/{encoded}"));
        for suffix in ["", "/data", "/latest", "/len", "/version"] {
            paths.push(format!("/api/{prefix}/{encoded}/height{suffix}"));
        }
    }
    paths
}

pub async fn exchange_with_etag(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
) -> String {
    exchange_with_limit(address, method, path, etag, 65536).await
}

async fn exchange_with_limit(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
    limit: u64,
) -> String {
    String::from_utf8(exchange_bytes(address, method, path, etag, limit).await).unwrap()
}

pub async fn exchange_bytes(
    address: SocketAddr,
    method: &str,
    path: &str,
    etag: &str,
    limit: u64,
) -> Vec<u8> {
    exchange_headers_bytes(
        address,
        method,
        path,
        &format!("If-None-Match: {etag}\r\n"),
        limit,
    )
    .await
}

pub async fn exchange_headers_bytes(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &str,
    limit: u64,
) -> Vec<u8> {
    let mut socket = TcpStream::connect(address).await.unwrap();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\n{headers}Connection: close\r\n\r\n"
    );
    socket.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    socket
        .take(limit + 1)
        .read_to_end(&mut response)
        .await
        .unwrap();
    assert!(
        response.len() as u64 <= limit,
        "fixture response exceeds {limit} bytes: {path}"
    );
    response
}

#[test]
#[cfg(feature = "chain")]
fn server_routes_preserve_validation_and_errors_before_conditionals() {
    init_cache();
    thread::Builder::new().stack_size(8 * 1024 * 1024).spawn(|| {
        let directory = tempdir().unwrap();
        let node = StdListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        node.set_nonblocking(true).unwrap();
        let client = Client::new(&format!("http://{}", node.local_addr().unwrap()), Auth::None).unwrap();
        let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
        let plugins = DefaultPlugins::import(ImportContext::new(directory.path()), &reader).unwrap();
        let query = AsyncQuery::build(&plugins, None);
        query.sync(|q| {
            let prices = &q.plugins().price.spot.cents.height;
            let snapshot = q.try_resolve_blocks_v1(None, 15).unwrap().unwrap();
            assert!(snapshot.anchor().is_none());
            assert!(snapshot.prices().is_empty());
            assert!(snapshot.build(q).unwrap().is_empty());
            assert!(!prices.read_cached_into_at(0, 1, &mut Vec::new()));
        });
        Builder::new_current_thread().max_blocking_threads(1).enable_all().build().unwrap().block_on(async {
            let node = TcpListener::from_std(node).unwrap();
            let (calls, mut observed) = mpsc::unbounded_channel();
            let mock = TokioSpawn(async move {
                let valid = r#"{"id":1,"result":100}"#;
                for body in [valid, "invalid json", valid, valid, valid, valid, "", valid] {
                    let mut socket = BufReader::new(node.accept().await.unwrap().0);
                    let mut line = String::new();
                    let mut length = 0;
                    loop {
                        line.clear();
                        assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
                        if line == "\r\n" { break; }
                        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
                            length = value.trim().parse::<usize>().unwrap();
                        }
                    }
                    assert!(length < 1024);
                    socket.read_exact(&mut vec![0; length]).await.unwrap();
                    calls.send(()).unwrap();
                    if body.is_empty() {
                        // Stall until the real HTTP deadline cancels the RPC exchange.
                        assert_eq!(socket.read(&mut [0]).await.unwrap(), 0);
                        continue;
                    }
                    let response = format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len());
                    socket.get_mut().write_all(response.as_bytes()).await.unwrap();
                }
            });
            let disk_path = directory.path().join("disk");
            fs::create_dir(&disk_path).await.unwrap();
            fs::write(disk_path.join("data"), [0; 8192]).await.unwrap();
            let blocks_path = directory.path().join("blocks");
            fs::create_dir(&blocks_path).await.unwrap();
            let server = Server::bind(&query, ServerConfig {
                bind: Ipv4Addr::LOCALHOST.into(), port: 0.into(), data_path: disk_path,
                ..ServerConfig::default()
            }).await.unwrap();
            let address = server.listener.local_addr().unwrap();
            let admission = server.state.sync_query.clone();
            let disk_admission = server.state.disk_query.clone();
            #[cfg(feature = "series")]
            let search_admission = server.state.series_bodies.search_query.clone();
            #[cfg(feature = "series")]
            let data_admission = server.state.series_bodies.data_query.clone();
            let disk_file = server.state.data_path.join("data");
            let serving = TokioSpawn(server.serve());
            // Unavailable snapshot joins fail immediately; lock waits and
            // queued work still share the bounded publication budget.
            timeout(Duration::from_secs(120), async {
                for method in ["GET", "HEAD"] {
                    for prefix in ["%2Bf", "-1", "0x1", "fffffffffffffffff", "nope"] {
                        let response = exchange_with_etag(address, method, &format!("/api/address/hash-prefix/p2wpkh/{prefix}"), "*").await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                }
                for method in ["GET", "HEAD"] {
                    for condition in ["*", "W/\"block-timestamp-v2-forged\""] {
                        let response = exchange_with_etag(address, method, "/api/v1/mining/blocks/timestamp/4294967295", condition).await;
                        assert!(response.starts_with("HTTP/1.1 503"), "{response}");
                        assert!(response.contains("\r\ncache-control: no-store\r\n"));
                        assert!(!response.contains("\r\netag:"));
                    }
                }
                for method in ["GET", "HEAD"] {
                    for condition in ["W/\"blocks2-empty\"", "*"] {
                        let response = exchange_with_etag(address, method, "/api/blocks", condition).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        assert!(response.ends_with("\r\n\r\n"));
                    }
                }
                let response = exchange_with_etag(address, "GET", "/api/blocks", "\"old\"").await;
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                assert!(response.ends_with("\r\n\r\n[]"), "{response}");
                #[cfg(feature = "series")]
                {
                    let response = exchange_with_limit(address, "GET", "/openapi.json", "\"old\"", 4 * 1024 * 1024).await;
                    assert!(response.starts_with("HTTP/1.1 200"));
                    let spec: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                    let broadcast = &spec["paths"]["/api/tx"]["post"]["responses"];
                    assert_eq!(broadcast["200"]["content"]["text/plain; charset=utf-8"]["schema"]["$ref"], "#/components/schemas/Txid");
                    assert!(broadcast["200"]["content"].get("application/json").is_none());
                    for status in ["400", "413", "500", "503", "504"] {
                        assert!(broadcast[status].is_object(), "missing broadcast status {status}");
                    }
                    assert!(spec["paths"].as_object().unwrap().keys().all(|path| {
                        !path.starts_with("/api/metric") && !path.starts_with("/api/vecs")
                            && !path.starts_with("/api/series/cost-basis")
                    }));
                    for name in ["LegacySeriesParam", "LegacySeriesWithIndex", "SeriesSelectionLegacy", "CostBasisParams", "CostBasisCohortParam", "CostBasisQuery", "CostBasisValue"] {
                        assert!(spec["components"]["schemas"].get(name).is_none(), "obsolete schema {name}");
                    }
                    for (path, publication) in [
                        ("/api/blocks", true),
                        ("/api/v1/blocks", true),
                        ("/api/v1/blocks/{height}", true),
                        ("/api/v1/block/{hash}", true),
                        ("/api/block/{hash}", true),
                        ("/api/block/{hash}/header", true),
                        ("/api/block/{hash}/raw", true),
                        ("/api/block-height/{height}", true),
                        ("/api/v1/mining/blocks/timestamp/{timestamp}", true),
                        ("/api/series/list", false),
                        ("/api/series/search", false),
                        ("/api/series/{series}", false),
                        ("/api/series/{series}/{index}/version", false),
                        ("/api/series/{series}/{index}", true),
                        ("/api/series/{series}/{index}/data", true),
                        ("/api/series/bulk", true),
                    ] {
                        let responses = &spec["paths"][path]["get"]["responses"];
                        for status in ["500", "504"].into_iter().chain(publication.then_some("503")) {
                            assert!(responses[status]["content"]["application/problem+json"].is_object(), "missing {status} problem response for {path}");
                        }
                        if path == "/api/series/bulk" {
                            assert!(responses["404"]["content"]["application/problem+json"].is_object(), "missing 404 problem response for {path}");
                        }
                    }
                }
                for path in ["/api/metrics", "/api/metrics/count", "/api/metrics/indexes", "/api/metrics/list", "/api/metrics/search", "/api/metrics/bulk", "/api/metric/timestamp", "/api/metric/timestamp/height", "/api/metric/timestamp/height/data", "/api/metric/timestamp/height/latest", "/api/metric/timestamp/height/len", "/api/metric/timestamp/height/version", "/api/vecs/query", "/api/vecs/height_to_timestamp", "/api/series/cost-basis", "/api/series/cost-basis/all/dates", "/api/series/cost-basis/all/2026-01-01"] {
                    for method in ["GET", "HEAD"] {
                        let response = exchange(address, method, path).await;
                        assert!(response.starts_with("HTTP/1.1 404"), "{path}: {response}");
                        assert!(!response.contains("\r\netag:"));
                        if method == "HEAD" { assert!(response.ends_with("\r\n\r\n")); }
                    }
                }
                for path in ["/health?x=1", "/version?x=1&x=2", "/api/server/sync?x=1", "/api/server/disk?x=1&x=2"] {
                    let response = exchange(address, "GET", path).await;
                    assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                    assert!(response.contains("content-type: application/problem+json\r\n"));
                    assert!(!response.contains("\r\netag:"));
                }
                assert!(observed.try_recv().is_err());
                #[cfg(feature = "series")]
                for path in ["/api/series", "/api/series/count", "/api/series/indexes"] {
                    let response = exchange_with_etag(address, "HEAD", path, "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert!(response.ends_with("\r\n\r\n"));
                    let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                    assert!(etag.starts_with("W/\"c"), "{etag}");
                    for method in ["GET", "HEAD"] {
                        let response = exchange_with_etag(address, method, path, etag).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        assert!(response.ends_with("\r\n\r\n"));
                        assert!(response.contains("cdn-cache-control: public, max-age=1, must-revalidate\r\n"));
                        let response = exchange(address, method, &format!("{path}?x=1&x=2")).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                }
                #[cfg(feature = "urpd")]
                {
                    let response = exchange_with_etag(address, "GET", "/api/urpd", "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert!(response.ends_with("\r\n\r\n[]"), "{response}");
                    let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                    for method in ["GET", "HEAD"] {
                        for path in ["/api/urpd/all", "/api/urpd/all/0", "/api/urpd/unknown", "/api/urpd/unknown/dates"] {
                            let response = exchange_with_etag(address, method, path, "*").await;
                            assert!(response.starts_with("HTTP/1.1 404"), "{path}: {response}");
                            assert!(!response.contains("\r\netag:"));
                        }
                    }
                    let response = exchange_with_etag(address, "GET", "/api/urpd/all/dates", "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert!(response.ends_with("\r\n\r\n[]"), "{response}");
                    let response = exchange_with_etag(address, "GET", "/api/urpd", etag).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                }
                #[cfg(feature = "series")]
                {
                    let names = vec!["timestamp"; 32].join(",");
                    for (route, key) in [("/api/series/bulk", "series")] {
                        for (suffix, status) in [
                            (format!("{key}=no_such_series_zzzz&index=height&limit=0"), 404),
                            (format!("{key}=timestamp&index=height&index=height"), 400),
                            (format!("{key}=timestamp&index=height&unexpected=1"), 400),
                        ] {
                            for method in ["GET", "HEAD"] {
                                let response = exchange(address, method, &format!("{route}?{suffix}")).await;
                                assert!(response.starts_with(&format!("HTTP/1.1 {status}")), "{response}");
                                assert!(response.contains("content-type: application/problem+json\r\n"));
                                assert!(!response.contains("\r\netag:"));
                                if method == "HEAD" { assert!(response.ends_with("\r\n\r\n")); }
                            }
                        }
                        for method in ["GET", "HEAD"] {
                            let response = exchange(address, method, &format!("{route}?{key}={names}&index=height&limit=0")).await;
                            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        }
                    }
                    let index = query.sync(|query| query.series_info(&"price_close".into()).unwrap().indexes[0]);
                    for format in ["json", "csv"] {
                        let path = format!("/api/series/price_close/{index}?limit=0&format={format}");
                        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                        if format == "csv" {
                            assert!(response.contains(&format!("content-disposition: attachment; filename=\"price_close-{index}.csv\"\r\n")), "{response}");
                            assert!(response.contains("content-type: text/csv\r\n"));
                        } else {
                            assert!(!response.contains("content-disposition:"));
                            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                            assert!(body["data"].as_array().unwrap().is_empty());
                        }
                        let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                        for method in ["GET", "HEAD"] {
                            let response = exchange_with_etag(address, method, &path, etag).await;
                            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                            assert!(response.ends_with("\r\n\r\n"));
                            assert!(!response.contains("content-disposition:"));
                            assert!(!response.contains("content-type:"));
                        }
                    }
                    for parameters in ["x=1", "limit=0&limit=1", "start=0&from=1", "format=xml"] {
                        let response = exchange(address, "GET", &format!("/api/series/price_close/{index}?{parameters}")).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                    assert!(query.sync(|query| query.vecs().series_names().iter().all(|name| name.len() <= 1024)));
                    for path in ["a".repeat(1024), "%C3%A9".repeat(512), "%E6%BC%A2".repeat(341)].iter().flat_map(|encoded| series_name_paths(encoded)) {
                        let response = exchange(address, "GET", &path).await;
                        assert!(response.starts_with("HTTP/1.1 404"), "{path}: {response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                    let response = exchange(address, "GET", "/api/series/no_such_series_zzzz").await;
                    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                    assert!(!response.contains("\r\netag:"));
                    for (encoded, text) in [("price", "price"), ("short+term+holder+balance", "short term holder balance")] {
                        let path = format!("/api/series/search?q={encoded}&limit=2");
                        let expected = query.sync(|query| SerdeJsonToString(&query.search_series(&SearchQuery { q: text.into(), limit: Limit::from(2) })).unwrap());
                        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
                        let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                        assert!(etag.starts_with("W/\"search2-"));
                        for method in ["GET", "HEAD"] {
                            let response = exchange_with_etag(address, method, &path, etag).await;
                            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                            assert!(response.ends_with("\r\n\r\n"));
                        }
                    }
                    for (parameters, pagination) in [
                        ("page=0&per_page=2".to_owned(), Pagination { page: Some(0), per_page: Some(2) }),
                        ("p=1&per_page=2".to_owned(), Pagination { page: Some(1), per_page: Some(2) }),
                        (format!("page={}&per_page=1", usize::MAX), Pagination { page: Some(usize::MAX), per_page: Some(1) }),
                    ] {
                        let path = format!("/api/series/list?{parameters}");
                        let expected = query.sync(|query| SerdeJsonToString(&query.series_list(pagination)).unwrap());
                        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
                        let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                        assert!(etag.starts_with("W/\"list1-"));
                        for method in ["GET", "HEAD"] {
                            let response = exchange_with_etag(address, method, &path, etag).await;
                            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                            assert!(response.ends_with("\r\n\r\n"));
                        }
                    }
                    for parameters in ["x=1", "page=0&page=1", "page=0&p=1", "per_page=1&per_page=2", "page=-1", "per_page=bad", "page=999999999999999999999999"] {
                        let response = exchange(address, "GET", &format!("/api/series/list?{parameters}")).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(response.contains("content-type: application/problem+json\r\n"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                }
                for method in ["GET", "HEAD"] {
                    let response = exchange(address, method, "/health").await;
                    assert!(response.starts_with("HTTP/1.1 503"), "{response}");
                    assert!(response.contains("\r\ncache-control: no-store\r\n"));
                    assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                    if method == "HEAD" { assert!(response.ends_with("\r\n\r\n")); }
                }
                let response = exchange(address, "GET", "/version").await;
                assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                assert!(response.ends_with("\r\n\r\n"));
                for status in [503, 500] {
                    let response = exchange(address, "GET", "/api/server/sync").await;
                    assert!(response.starts_with(&format!("HTTP/1.1 {status}")), "{response}");
                    assert!(response.contains("content-type: application/problem+json\r\n"));
                    assert!(response.contains("\r\ncache-control: no-store\r\n"));
                    assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                    assert!(!response.contains("\r\netag:"));
                }
                observed.recv().await.unwrap();
                observed.recv().await.unwrap();
                assert!(observed.try_recv().is_err());

                // An empty index returns unavailable without replaying the RPC.
                for _ in 0..4 {
                    let response = exchange(address, "GET", "/api/server/sync").await;
                    assert!(response.starts_with("HTTP/1.1 503"), "{response}");
                    observed.recv().await.unwrap();
                    assert!(observed.try_recv().is_err());
                    assert_eq!(admission.available_permits(), 1);
                }

                let response = exchange(address, "GET", "/api/server/sync").await;
                assert!(response.starts_with("HTTP/1.1 504"), "{response}");
                assert!(response.contains("content-type: application/problem+json\r\n"));
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
                assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                assert!(!response.contains("\r\netag:"));
                observed.recv().await.unwrap();
                let response = exchange(address, "GET", "/api/server/sync").await;
                assert!(response.starts_with("HTTP/1.1 503"), "{response}");
                observed.recv().await.unwrap();
                assert!(observed.try_recv().is_err());
                mock.await.unwrap();

                // Occupy the only blocking worker so the real disk handler's
                // admitted job remains queued across its HTTP deadline.
                let (started, ready) = oneshot::channel();
                let (release, held) = blocking_channel::channel();
                let blocker = spawn_blocking(move || {
                    let _ = started.send(());
                    let _ = held.recv();
                });
                ready.await.unwrap();
                #[cfg(feature = "series")]
                {
                    let names = vec!["timestamp"; 33].join(",");
                    for (route, key) in [("/api/series/bulk", "series")] {
                        for method in ["GET", "HEAD"] {
                            let response = exchange(address, method, &format!("{route}?{key}={names}&index=height&limit=0")).await;
                            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                            assert!(!response.contains("\r\netag:"));
                            if method == "GET" { assert!(response.contains("At most 32 series may be requested")); }
                        }
                    }
                }
                #[cfg(feature = "series")]
                let data_capacity = data_admission.available_permits();
                #[cfg(feature = "series")]
                let remaining_data = data_admission.clone().acquire_many_owned((data_capacity - 1).try_into().unwrap()).await.unwrap();
                #[cfg(feature = "series")]
                let search_capacity = search_admission.available_permits();
                #[cfg(feature = "series")]
                let search_hold = search_admission.clone().acquire_many_owned(search_capacity.try_into().unwrap()).await.unwrap();
                #[cfg(feature = "series")]
                for path in ["a".repeat(1025), format!("{}a", "%C3%A9".repeat(512))].iter().flat_map(|encoded| series_name_paths(encoded)) {
                    for method in ["GET", "HEAD"] {
                        let response = exchange(address, method, &path).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(response.contains("\r\ncache-control: public, max-age=1, must-revalidate\r\n"), "{response}");
                        assert!(response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                        if method == "GET" {
                            assert!(response.contains("series name exceeds 1024 UTF-8 bytes"));
                            assert!(response.len() < 2048);
                        }
                    }
                }
                #[cfg(feature = "series")]
                for name in ["price_close", "PRICE-CLOSE"] {
                    let path = format!("/api/series/{name}");
                    let expected = query.sync(|query| SerdeJsonToString(&query.series_info(&name.into()).unwrap()).unwrap());
                    let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
                    let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                    assert!(etag.starts_with("W/\"info1-"));
                    for method in ["GET", "HEAD"] {
                        let response = exchange_with_etag(address, method, &path, etag).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        assert!(response.ends_with("\r\n\r\n"));
                        assert!(response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"));
                        let response = exchange_with_etag(address, method, &format!("{path}?x=1"), etag).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(!response.contains("\r\netag:"));
                    }
                }
                #[cfg(feature = "series")]
                assert!(exchange(address, "GET", "/api/series/search?q=price&limit=2").await.starts_with("HTTP/1.1 304"));
                #[cfg(feature = "series")]
                {
                    let path = "/api/series/search?q=anything&limit=0";
                    let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert!(response.ends_with("\r\n\r\n[]"));
                    let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                    for method in ["GET", "HEAD"] {
                        let response = exchange_with_etag(address, method, path, etag).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        assert!(response.ends_with("\r\n\r\n"));
                    }
                    for parameters in ["limit=0", "q=x&limit=0&x=1", "q=x&q=y&limit=0", "q=x&limit=0&limit=0"] {
                        let response = exchange(address, "GET", &format!("/api/series/search?{parameters}")).await;
                        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                        assert!(response.contains("content-type: application/problem+json\r\n"));
                        assert!(!response.contains("\r\netag:"));
                    }
                    for encoded in ["a".repeat(1024), "%C3%A9".repeat(512)] {
                        let path = format!("/api/series/search?q={encoded}&limit=0");
                        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                        assert!(response.ends_with("\r\n\r\n[]"));
                        for limit in [0, 1] {
                            let path = format!("/api/series/search?q={encoded}a&limit={limit}");
                            let response = exchange(address, "GET", &path).await;
                            assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                            assert!(response.contains("content-type: application/problem+json\r\n"));
                            assert!(response.contains("search query exceeds 1024 UTF-8 bytes"));
                            assert!(!response.contains("\r\netag:"));
                        }
                    }
                }
                #[cfg(feature = "series")]
                assert!(exchange(address, "GET", "/api/series/list?page=0&per_page=2").await.starts_with("HTTP/1.1 304"));
                #[cfg(feature = "series")]
                for prefix in ["series"] {
                    let path = format!("/api/{prefix}/timestamp/height/version");
                    let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                    for method in ["GET", "HEAD"] {
                        let response = exchange_with_etag(address, method, &path, etag).await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                    }
                }
                let timed_out = TokioSpawn(exchange(address, "GET", "/api/server/disk"));
                #[cfg(feature = "series")]
                let timed_out_latest = TokioSpawn(exchange(address, "GET", "/api/series/timestamp/height/latest"));
                #[cfg(feature = "series")]
                let waiting_data = {
                    timeout(Duration::from_secs(1), async {
                        while data_admission.available_permits() != 0 { yield_now().await; }
                    }).await.unwrap();
                    let response = exchange(address, "GET", "/api/series/timestamp/height?limit=0&x=1").await;
                    assert!(response.starts_with("HTTP/1.1 400"), "{response}");
                    TokioSpawn(exchange(address, "HEAD", "/api/series/timestamp/height?limit=0"))
                };
                #[cfg(feature = "series")]
                let waiting_length = TokioSpawn(exchange(address, "HEAD", "/api/series/timestamp/height/len"));
                #[cfg(feature = "series")]
                let mut timed_out_search = TokioSpawn(exchange_with_etag(address, "GET", "/api/series/search?q=price&limit=2", "\"old\""));
                #[cfg(feature = "series")]
                let remaining_search = {
                    assert!(timeout(Duration::from_millis(50), &mut timed_out_search).await.is_err());
                    drop(search_hold);
                    let remaining = search_admission.clone().acquire_many_owned((search_capacity - 1).try_into().unwrap()).await.unwrap();
                    timeout(Duration::from_secs(1), async {
                        while search_admission.available_permits() != 0 { yield_now().await; }
                    }).await.unwrap();
                    remaining
                };
                #[cfg(feature = "series")]
                let waiting_search = TokioSpawn(exchange_with_etag(address, "GET", "/api/series/search?q=price&limit=2", "\"old\""));
                #[cfg(feature = "series")]
                let waiting_suggestion = TokioSpawn(exchange(address, "GET", "/api/series/no_such_series_zzzz"));
                #[cfg(feature = "series")]
                let waiting_version = TokioSpawn(exchange(address, "GET", "/api/series/no_such_series_zzzz/height/version"));
                timeout(Duration::from_secs(1), async {
                    while disk_admission.available_permits() != 0 { yield_now().await; }
                }).await.unwrap();
                let response = timed_out.await.unwrap();
                assert!(response.starts_with("HTTP/1.1 504"), "{response}");
                assert!(response.contains("content-type: application/problem+json\r\n"));
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
                assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                assert!(!response.contains("\r\netag:"));
                assert_eq!(disk_admission.available_permits(), 0);
                #[cfg(feature = "series")]
                {
                    for response in [timed_out_latest.await.unwrap(), waiting_data.await.unwrap(), waiting_length.await.unwrap()] {
                        assert!(response.starts_with("HTTP/1.1 504"), "{response}");
                        assert!(response.contains("\r\ncache-control: no-store\r\n"));
                        assert!(!response.contains("\r\netag:"));
                    }
                    // The queued job still owns its slot after the HTTP owner exits.
                    assert_eq!(data_admission.available_permits(), 0);
                }
                #[cfg(feature = "series")]
                let mut following_search = {
                    for response in [timed_out_search.await.unwrap(), waiting_search.await.unwrap(), waiting_suggestion.await.unwrap(), waiting_version.await.unwrap()] {
                        assert!(response.starts_with("HTTP/1.1 504"), "{response}");
                        assert!(response.contains("\r\ncache-control: no-store\r\n"));
                        assert!(response.contains("\r\ncdn-cache-control: no-store\r\n"));
                        assert!(!response.contains("\r\netag:"));
                    }
                    assert_eq!(search_admission.available_permits(), 0);
                    TokioSpawn(exchange_with_etag(address, "GET", "/api/series/search?q=price&limit=2", "\"old\""))
                };
                #[cfg(feature = "series")]
                assert!(timeout(Duration::from_millis(50), &mut following_search).await.is_err());
                #[cfg(feature = "series")]
                let mut following_suggestion = TokioSpawn(exchange(address, "GET", "/api/series/no_such_series_zzzz"));
                #[cfg(feature = "series")]
                assert!(timeout(Duration::from_millis(50), &mut following_suggestion).await.is_err());
                let mut next = TokioSpawn(exchange_with_etag(address, "GET", "/api/server/disk", "\"old\""));
                assert!(timeout(Duration::from_millis(50), &mut next).await.is_err());
                release.send(()).unwrap();
                blocker.await.unwrap();
                #[cfg(feature = "series")]
                {
                    let response = following_search.await.unwrap();
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    let response = following_suggestion.await.unwrap();
                    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                    assert!(!response.contains("\r\netag:"));
                    drop(remaining_search);
                    assert_eq!(search_admission.available_permits(), search_capacity);
                    for method in ["GET", "HEAD"] {
                        let response = exchange(address, method, "/api/series/timestamp/height?limit=0").await;
                        assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                        assert!(response.ends_with("\r\n\r\n"));
                    }
                    drop(remaining_data);
                    assert_eq!(data_admission.available_permits(), data_capacity);
                    let response = exchange(address, "GET", "/api/series/timestamp/height/latest").await;
                    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                    assert!(!response.contains("\r\netag:"));
                    let response = exchange_with_etag(address, "GET", "/api/series/timestamp/height/len", "\"old\"").await;
                    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, "0");
                }
                assert!(next.await.unwrap().starts_with("HTTP/1.1 200"));
                assert_eq!(disk_admission.available_permits(), 1);

                let permit = disk_admission.clone().acquire_owned().await.unwrap();
                let mut waiting = TokioSpawn(exchange_with_etag(address, "GET", "/api/server/disk", "\"old\""));
                assert!(timeout(Duration::from_millis(50), &mut waiting).await.is_err());
                drop(permit);
                let response = waiting.await.unwrap();
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(body["bitcoin_bytes"], 0);
                assert_eq!(body["ratio"], 0.0);
                let etag = response.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                assert_eq!(etag, format!("W/\"disk1-{}-0\"", body["brk_bytes"].as_u64().unwrap()));
                for method in ["GET", "HEAD"] {
                    let response = exchange_with_etag(address, method, "/api/server/disk", etag).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                    assert!(response.ends_with("\r\n\r\n"));
                    assert!(response.contains("\r\ncdn-cache-control: public, max-age=1, must-revalidate\r\n"));
                }
                fs::write(disk_file, [1; 16384]).await.unwrap();
                let changed = exchange_with_etag(address, "GET", "/api/server/disk", etag).await;
                assert!(changed.starts_with("HTTP/1.1 200"), "{changed}");
                let changed_body: Value = from_str(changed.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_ne!(changed_body["brk_bytes"], body["brk_bytes"]);
                let etag = changed.lines().find_map(|line| line.strip_prefix("etag: ")).unwrap();
                assert!(exchange_with_etag(address, "GET", "/api/server/disk", etag).await.starts_with("HTTP/1.1 304"));
                // A previously valid tag must not conceal an unreadable tree.
                fs::remove_dir(blocks_path).await.unwrap();
                let response = exchange_with_etag(address, "GET", "/api/server/disk", etag).await;
                assert!(response.starts_with("HTTP/1.1 500"), "{response}");
                assert!(response.contains("\r\ncache-control: no-store\r\n"));
                assert!(!response.contains("\r\netag:"));
                assert_eq!(disk_admission.available_permits(), 1);
            }).await.unwrap();
            serving.abort();
            let _ = serving.await;
            broadcast::check(&query).await;
        });
    }).unwrap().join().unwrap();
}

#[test]
#[cfg(all(feature = "chain", feature = "series"))]
fn search_endpoint_ranks_live_catalog_and_revalidates_new_revision() {
    init_cache();
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let directory = tempdir().unwrap();
            let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
            let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
            let plugins =
                DefaultPlugins::import(ImportContext::new(directory.path()), &reader).unwrap();
            let query = AsyncQuery::build(&plugins, None);
            Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let server = Server::bind(
                        &query,
                        ServerConfig {
                            bind: Ipv4Addr::LOCALHOST.into(),
                            port: 0.into(),
                            data_path: directory.path().to_path_buf(),
                            ..ServerConfig::default()
                        },
                    )
                    .await
                    .unwrap();
                    let address = server.listener.local_addr().unwrap();
                    let serving = TokioSpawn(server.serve());
                    for (query, expected) in [
                        ("realized+price+sth", "sth_realized_price"),
                        ("short+term+holder+realized+price", "sth_realized_price"),
                        ("realized+price+short+term", "sth_realized_price"),
                        ("realized+prcie+sth", "sth_realized_price"),
                        ("short+term+supply+in+profit", "sth_supply_in_profit"),
                        ("long+term+holder+supply", "lth_supply"),
                        ("bitcoin+price", "price"),
                        ("prcie", "price"),
                        ("hashrate", "hash_rate"),
                        ("hashraet", "hash_rate"),
                        ("cap+market", "market_cap"),
                        ("active+addresses", "active_addrs"),
                        ("coin+days+destroyed", "coindays_destroyed"),
                        ("cdd", "coindays_destroyed"),
                    ] {
                        let path = format!("/api/series/search?q={query}&limit=5");
                        let response =
                            exchange_with_etag(address, "GET", &path, "W/\"search1-old\"").await;
                        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                        let values: Vec<String> =
                            from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                        assert_eq!(
                            values.first().map(String::as_str),
                            Some(expected),
                            "{query}: {values:?}"
                        );
                        let etag = response
                            .lines()
                            .find_map(|line| line.strip_prefix("etag: "))
                            .unwrap();
                        assert!(etag.starts_with("W/\"search2-"));
                        let cached = exchange_with_etag(address, "GET", &path, etag).await;
                        assert!(cached.starts_with("HTTP/1.1 304"), "{cached}");
                    }
                    serving.abort();
                    let _ = serving.await;
                });
        })
        .unwrap()
        .join()
        .unwrap();
}
