use std::str::from_utf8;

use axum::http::{Method, StatusCode};
use bitcoin::{Network, blockdata::constants::genesis_block, consensus::serialize};

use super::{
    chain_fixture::{raw_fixture_block, run_populated as run_fixture_with_first},
    header_integrity::check_prevouts,
    server_routes::{exchange_bytes, exchange_headers_bytes, exchange_with_etag},
};
use crate::raw_body::RawBodyPermit;
#[test]
fn raw_head_preserves_get_metadata_with_negotiated_compression() {
    let first = raw_fixture_block();
    let path = format!("/api/block/{}/raw", first.block_hash());
    let expected = serialize(&first);
    let verified_block = first.clone();
    run_fixture_with_first(first, move |state, address| async move {
        for (accept, encoding) in [
            ("identity", None),
            ("gzip", Some("gzip")),
            ("br", Some("br")),
            ("zstd", Some("zstd")),
            ("gzip;q=0, identity", None),
            ("gzip;q=0.5, zstd;q=1", Some("zstd")),
        ] {
            let request_headers = format!("Accept-Encoding: {accept}\r\n");
            let get =
                exchange_headers_bytes(address, "GET", &path, &request_headers, 4_100_000).await;
            let head =
                exchange_headers_bytes(address, "HEAD", &path, &request_headers, 4_100_000).await;
            let get_end = get.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
            let get_headers = from_utf8(&get[..get_end]).unwrap();
            let head_headers = from_utf8(&head).unwrap();
            assert!(get_headers.starts_with("HTTP/1.1 200"), "{get_headers}");
            assert!(head_headers.starts_with("HTTP/1.1 200"), "{head_headers}");
            assert!(head_headers.ends_with("\r\n\r\n"));
            for field in [
                "etag: ",
                "content-type: ",
                "cache-control: ",
                "cdn-cache-control: ",
                "content-encoding: ",
            ] {
                assert_eq!(
                    get_headers.lines().find(|line| line.starts_with(field)),
                    head_headers.lines().find(|line| line.starts_with(field)),
                    "{accept}: {field}"
                );
            }
            let selected = head_headers
                .lines()
                .find_map(|line| line.strip_prefix("content-encoding: "));
            assert_eq!(selected, encoding, "{accept}");
            if encoding.is_some() {
                assert!(
                    !head_headers.contains("\r\ncontent-length:"),
                    "{head_headers}"
                );
                assert!(
                    head_headers.contains("\r\nvary: accept-encoding\r\n"),
                    "{head_headers}"
                );
            } else {
                assert!(
                    head_headers.contains(&format!("\r\ncontent-length: {}\r\n", expected.len())),
                    "{head_headers}"
                );
                assert_eq!(get[get_end..], expected);
            }
            let tag = get_headers
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap();
            let request_headers = format!("Accept-Encoding: {accept}\r\nIf-None-Match: {tag}\r\n");
            for method in ["GET", "HEAD"] {
                let response =
                    exchange_headers_bytes(address, method, &path, &request_headers, 4096).await;
                let response = from_utf8(&response).unwrap();
                assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                assert!(response.ends_with("\r\n\r\n"));
                assert!(
                    !response.contains("\r\ncontent-length:"),
                    "{accept} {method}: {response}"
                );
            }
        }
        // Hold complete responses instead of only worker jobs: slow clients
        // must not allow successive full payloads to escape admission.
        let hash = state.sync(|q| q.resolve_blocks(None, 1).unwrap().anchor().unwrap());
        let mut retained = Vec::new();
        for _ in 0..RawBodyPermit::CAPACITY {
            let response = state
                .respond_block_raw(Default::default(), hash, Method::GET)
                .await
                .unwrap_or_else(|_| panic!("raw response failed"));
            assert_eq!(response.status(), StatusCode::OK);
            retained.push(response);
        }
        assert_eq!(state.raw_block_bodies.available_permits(), 0);
        let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 504"), "{response}");
        assert!(response.contains("\r\ncache-control: no-store\r\n"));
        assert!(!response.contains("\r\nretry-after:"));
        assert!(!response.contains("\r\netag:"));
        assert!(response.contains("\"code\":\"timeout\""));
        for (method, etag, status) in [
            ("GET", "*", 304),
            ("HEAD", "\"old\"", 200),
            ("HEAD", "*", 304),
        ] {
            let response = exchange_with_etag(address, method, &path, etag).await;
            assert!(
                response.starts_with(&format!("HTTP/1.1 {status}")),
                "{response}"
            );
            assert!(response.ends_with("\r\n\r\n"));
        }
        let missing = format!("/api/block/{}/raw", "0".repeat(64));
        let response = exchange_with_etag(address, "GET", &missing, "*").await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
        drop(retained.pop());
        assert_eq!(state.raw_block_bodies.available_permits(), 1);
        let response = exchange_bytes(address, "GET", &path, "\"old\"", 4_100_000).await;
        assert!(response.starts_with(b"HTTP/1.1 200"));
        drop(retained);
        check_prevouts(
            &state.query,
            address,
            &verified_block,
            &genesis_block(Network::Bitcoin),
        )
        .await;
    });
}
