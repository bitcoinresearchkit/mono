//! Native address pagination and same-height response identity contracts.
use super::{
    chain_fixture::{default_first, run_genesis},
    server_routes::exchange_with_etag,
};
use brk_error::Error as QueryError;
use brk_types::Addr;
use serde_json::{Value, from_str};

#[test]
fn address_history_preserves_exclusive_cursors_and_published_bounds() {
    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(4, 2);
        let addr = Addr::try_from(&fixture.chain[1].txdata[0].output[0].script_pubkey).unwrap();
        fixture.query.sync(|q| {
            let (output_type, type_index) = q.resolve_addr(&addr).unwrap();
            let stores = q.indexer().stores();
            let indices: Vec<_> = stores
                .addr_tx_indexes_before(
                    output_type,
                    type_index,
                    q.indexer().safe_lengths().tx_index,
                )
                .unwrap()
                .rev()
                .collect();
            assert!(indices.len() >= 2);
            // Compare the old filter contract with the bounded range, including
            // an empty publication and a cursor above the published boundary.
            for published in 0..=4u32 {
                for cursor in 0..=4u32 {
                    let expected: Vec<_> = indices
                        .iter()
                        .copied()
                        .filter(|index| *index < published.into() && *index < cursor.into())
                        .collect();
                    let actual: Vec<_> = stores
                        .addr_tx_indexes_before(
                            output_type,
                            type_index,
                            published.min(cursor).into(),
                        )
                        .unwrap()
                        .rev()
                        .collect();
                    assert_eq!(actual, expected);
                }
            }
            let txids = q.addr_txids(addr.clone(), None, usize::MAX).unwrap();
            assert_eq!(txids.len(), indices.len());
            for limit in 0..=txids.len() + 1 {
                assert_eq!(
                    q.addr_txids(addr.clone(), None, limit).unwrap(),
                    txids.iter().copied().take(limit).collect::<Vec<_>>()
                );
                for (position, cursor) in txids.iter().enumerate() {
                    assert_eq!(
                        q.addr_txids(addr.clone(), Some(*cursor), limit).unwrap(),
                        txids
                            .iter()
                            .copied()
                            .skip(position + 1)
                            .take(limit)
                            .collect::<Vec<_>>()
                    );
                }
            }
            for (position, cursor) in txids.iter().enumerate() {
                let page = q
                    .resolve_addr_chain_txs(&addr, Some(*cursor), usize::MAX)
                    .unwrap();
                if let Some(index) = indices.get(position + 1) {
                    let (_, height) = q.txid_and_height_by_index(*index).unwrap();
                    assert_eq!(
                        page.activity_anchor(),
                        q.resolve_block_hash(height).unwrap()
                    );
                } else {
                    assert_eq!(page.activity_anchor(), q.tip_blockhash());
                }
            }
            let unknown = "00".repeat(32).parse().unwrap();
            assert!(matches!(
                q.addr_txids(addr.clone(), Some(unknown), 0),
                Err(QueryError::UnknownTxid)
            ));
            assert!(matches!(
                q.resolve_addr_chain_txs(&addr, Some(unknown), usize::MAX),
                Err(QueryError::UnknownTxid)
            ));
        });
        let txids = fixture
            .query
            .sync(|q| q.addr_txids(addr.clone(), None, usize::MAX).unwrap());
        for (position, cursor) in txids.iter().enumerate() {
            let path = format!("/api/address/{addr}/txs/chain/{cursor}");
            let response = exchange_with_etag(fixture.address, "GET", &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            let body: Vec<Value> = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            let actual: Vec<_> = body.iter().map(|tx| tx["txid"].as_str().unwrap()).collect();
            let expected: Vec<_> = txids[position + 1..]
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(actual, expected);
            let response = exchange_with_etag(fixture.address, "GET", &path, "*").await;
            assert!(response.starts_with("HTTP/1.1 304"), "{response}");
        }
        let path = format!("/api/address/{addr}/txs/chain/{}", "00".repeat(32));
        let response = exchange_with_etag(fixture.address, "GET", &path, "*").await;
        assert!(response.starts_with("HTTP/1.1 404"), "{response}");
    });
}

#[test]
fn reorganization_preserves_publication_and_validator_contracts() {
    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(1, 1);
        #[cfg(feature = "price")]
        super::historical_price::check(&fixture.state, fixture.address).await;
        let mut live = super::addr_publication::AddrPublication::start(
            &fixture.plugins,
            fixture.directory.path(),
            &fixture.chain[1],
        )
        .await;
        let old_hash = fixture.chain[1].block_hash();
        let txid = fixture.chain[1].txdata[0].compute_txid();
        let address = fixture.address;
        let mut paths = vec![
            "/api/server/sync".to_owned(),
            "/api/blocks".to_owned(),
            "/api/blocks/1".to_owned(),
            "/api/blocks/tip/height".to_owned(),
            "/api/blocks/tip/hash".to_owned(),
            "/api/block-height/1".to_owned(),
            "/api/v1/blocks".to_owned(),
            "/api/v1/blocks/0".to_owned(),
            "/api/v1/blocks/1".to_owned(),
            "/api/v1/historical-price".to_owned(),
            "/api/v1/mining/blocks/timestamp/4294967295".to_owned(),
            "/api/v1/mining/pool/unknown/blocks".to_owned(),
            format!("/api/block/{old_hash}"),
            format!("/api/block/{old_hash}/header"),
            format!("/api/block/{old_hash}/txids"),
            format!("/api/v1/block/{old_hash}"),
            format!("/api/block/{}/status", fixture.chain[0].block_hash()),
        ];
        for suffix in ["", "/hex", "/status", "/merkle-proof", "/merkleblock-proof"] {
            paths.push(format!("/api/tx/{txid}{suffix}"));
        }
        #[cfg(feature = "series")]
        paths.extend([
            "/api/series/timestamp/height?start=1&end=2".to_owned(),
            "/api/series/timestamp/height/latest".to_owned(),
            "/api/series/timestamp/height/len".to_owned(),
            "/api/series/timestamp/height/version".to_owned(),
            "/api/series/bulk?series=timestamp,timestamp_monotonic&index=height&start=1&end=2"
                .to_owned(),
        ]);
        let mut captured = Vec::new();
        for path in paths {
            let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
            let tag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            let body = response.split_once("\r\n\r\n").unwrap().1.to_owned();
            captured.push((path, tag, body));
        }
        live.consume_snapshot().await;
        fixture.publish(2, 1);
        live.after_reorg(&fixture.chain[2]).await;
        assert_ne!(fixture.chain[2].block_hash(), old_hash);
        for (path, old_tag, old_body) in captured {
            let fresh = exchange_with_etag(address, "GET", &path, "\"old\"").await;
            for method in ["GET", "HEAD"] {
                let response = exchange_with_etag(address, method, &path, &old_tag).await;
                if fresh.starts_with("HTTP/1.1 404") {
                    assert!(response.starts_with("HTTP/1.1 404"), "{path}: {response}");
                    assert!(!response.contains("\r\netag:"));
                    continue;
                }
                assert!(fresh.starts_with("HTTP/1.1 200"), "{path}: {fresh}");
                let fresh_body = fresh.split_once("\r\n\r\n").unwrap().1;
                let new_tag = fresh
                    .lines()
                    .find_map(|line| line.strip_prefix("etag: "))
                    .unwrap();
                if old_body != fresh_body {
                    assert_ne!(
                        old_tag, new_tag,
                        "changed representation kept its validator: {path}"
                    );
                }
                let unchanged = old_tag == new_tag;
                assert!(
                    response.starts_with(if unchanged {
                        "HTTP/1.1 304"
                    } else {
                        "HTTP/1.1 200"
                    }),
                    "{path}: {response}"
                );
                assert_eq!(
                    response.split_once("\r\n\r\n").unwrap().1,
                    if unchanged || method == "HEAD" {
                        ""
                    } else {
                        fresh_body
                    },
                    "{path}"
                );
                let repeated = exchange_with_etag(address, method, &path, new_tag).await;
                assert!(repeated.starts_with("HTTP/1.1 304"), "{path}: {repeated}");
                assert!(repeated.ends_with("\r\n\r\n"));
            }
        }
    });
}
