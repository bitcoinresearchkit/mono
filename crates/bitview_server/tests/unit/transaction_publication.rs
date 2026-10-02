use std::{net::SocketAddr, str::from_utf8};

use bitview_query::AsyncQuery;
use brk_types::{Txid, Vout};
use serde_json::to_vec;
use tokio::{join, task::JoinSet};

use super::server_routes::{exchange_bytes, exchange_with_etag};

const SUFFIXES: [&str; 10] = [
    "",
    "/status",
    "/hex",
    "/raw",
    "/outspend/0",
    "/outspends",
    "/cpfp",
    "/rbf",
    "/replacements",
    "/fullrbf/replacements",
];

#[test]
fn batched_outspends_preserve_output_order_and_reorg_visibility() {
    use super::chain_fixture::{raw_fixture_block, run_genesis};
    use brk_types::{Height, Vin};
    use serde_json::{Value, from_str, to_value};

    let mut first = raw_fixture_block();
    first.txdata[1].input.rotate_right(1);
    // The descendant must reference the reordered spending transaction.
    first.txdata[2].input[0].previous_output.txid = first.txdata[1].compute_txid();
    first.header.merkle_root = first.compute_merkle_root().unwrap();
    let txid = Txid::from(first.txdata[0].compute_txid());
    let spending_txid = Txid::from(first.txdata[1].compute_txid());
    run_genesis(first, move |mut fixture| async move {
        fixture.publish(1, 1);
        let expected = fixture.query.sync(|q| {
            let values = q.outspends(&txid).unwrap();
            assert_eq!(values.len(), 5);
            for (index, value) in values.iter().enumerate() {
                assert_eq!(
                    to_value(value).unwrap(),
                    to_value(q.outspend(&txid, Vout::from(index)).unwrap()).unwrap()
                );
                if index < 2 {
                    assert!(!value.spent);
                } else {
                    assert!(value.spent);
                    assert_eq!(value.txid, Some(spending_txid));
                    assert_eq!(value.vin, Some(Vin::from((index - 1) % 3)));
                    assert_eq!(
                        value.status.as_ref().unwrap().block_height,
                        Some(Height::new(1))
                    );
                }
            }
            to_value(values).unwrap()
        });
        let path = format!("/api/tx/{txid}/outspends");
        let response = exchange_with_etag(fixture.address, "GET", &path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body, expected);
        let etag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap()
            .to_owned();
        let unchanged = exchange_with_etag(fixture.address, "GET", &path, &etag).await;
        assert!(unchanged.starts_with("HTTP/1.1 304"), "{unchanged}");
        fixture.publish(2, 1);
        assert!(fixture.query.sync(|q| q.outspends(&txid)).is_err());
        let replaced = exchange_with_etag(fixture.address, "GET", &path, &etag).await;
        assert!(replaced.starts_with("HTTP/1.1 404"), "{replaced}");
    });
}

#[test]
fn confirmed_handoffs_pin_the_prefix_and_revalidate_replaced_blocks() {
    use bitview_plugin_indexer::HasIndexer;
    use brk_error::Error;

    use super::chain_fixture::{default_first, run_genesis};

    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(1, 1);
        let query = fixture.query.clone();
        let txid = Txid::from(fixture.chain[1].txdata[0].compute_txid());
        let confirmed = query.sync(|q| q.resolve_confirmed_tx(&txid).unwrap());
        let raw = query.sync(|q| q.resolve_raw_transaction(&txid).unwrap());
        let json = query.sync(|q| q.resolve_transaction(&txid).unwrap());
        query.sync(|q| {
            q.resolve_confirmed_tx(&txid).unwrap();
            q.merkle_proof_resolved(confirmed).unwrap();
            q.merkleblock_proof_resolved(confirmed).unwrap();
            q.confirmed_cpfp_resolved(confirmed).unwrap();
        });

        // Immutable consumers reacquire rollback protection without waiting for
        // ordinary appends. Derived CPFP data still needs publication exclusion.
        let gate = fixture.plugins.indexer().publication().clone();
        gate.begin_update();
        let (resolve, proof, cpfp) = join!(
            query.run(move |q| q.resolve_confirmed_tx(&txid)),
            query.run(move |q| q.merkle_proof_resolved(confirmed)),
            query.run(move |q| q.confirmed_cpfp_resolved(confirmed)),
        );
        assert!(resolve.is_ok());
        assert!(proof.is_ok());
        assert!(matches!(cpfp, Err(Error::ReadTimeout)));
        gate.finish_update();

        // Multi-plugin views must also wait for the non-indexer participants.
        let gate = query.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        assert!(matches!(
            query
                .run(move |q| q.confirmed_cpfp_resolved(confirmed))
                .await,
            Err(Error::ReadTimeout),
        ));
        gate.finish_update();
        query.sync(|q| q.confirmed_cpfp_resolved(confirmed).unwrap());

        // Replace the block at the same height after the handoff. Its old
        // position is no longer proof that the original transaction exists.
        fixture.publish(2, 1);
        query.sync(|q| {
            assert!(matches!(
                q.resolve_confirmed_tx(&txid),
                Err(Error::UnknownTxid)
            ));
            assert!(matches!(
                q.transaction_raw_resolved(raw),
                Err(Error::UnknownTxid),
            ));
            assert!(matches!(
                q.transaction_json_resolved(json),
                Err(Error::UnknownTxid),
            ));
            assert!(matches!(
                q.merkle_proof_resolved(confirmed),
                Err(Error::UnknownTxid),
            ));
            assert!(matches!(
                q.merkleblock_proof_resolved(confirmed),
                Err(Error::UnknownTxid),
            ));
            assert!(matches!(
                q.confirmed_cpfp_resolved(confirmed),
                Err(Error::UnknownTxid),
            ));
        });
    });
}

pub(crate) struct TransactionPublication {
    txid: Txid,
    tags: Vec<String>,
}

impl TransactionPublication {
    pub(crate) fn new(txid: Txid) -> Self {
        Self {
            txid,
            tags: Vec::new(),
        }
    }

    fn path(&self, suffix: &str) -> String {
        if suffix == "/cpfp" {
            format!("/api/v1/cpfp/{}", self.txid)
        } else if suffix == "/rbf" {
            format!("/api/v1/tx/{}/rbf", self.txid)
        } else if suffix.ends_with("/replacements") {
            format!("/api/v1{suffix}")
        } else {
            format!("/api/tx/{}{suffix}", self.txid)
        }
    }

    pub(crate) async fn check_unavailable(&self, address: SocketAddr) {
        let mut requests = JoinSet::new();
        for (index, suffix) in SUFFIXES.iter().enumerate() {
            let path = self.path(suffix);
            for method in ["GET", "HEAD"] {
                for tag in [
                    "*",
                    self.tags
                        .get(index)
                        .map(String::as_str)
                        .unwrap_or("\"old\""),
                ] {
                    let path = path.clone();
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

    pub(crate) async fn check_available(&mut self, query: &AsyncQuery, address: SocketAddr) {
        query.sync(|q| {
            for (index, outspend) in q.outspends(&self.txid).unwrap().iter().enumerate() {
                assert_eq!(
                    to_vec(&q.outspend(&self.txid, Vout::from(index)).unwrap()).unwrap(),
                    to_vec(outspend).unwrap(),
                );
            }
        });
        self.tags.clear();
        for suffix in SUFFIXES {
            let expected = query.sync(|q| match suffix {
                "" => q
                    .transaction_json_resolved(q.resolve_transaction(&self.txid).unwrap())
                    .unwrap(),
                "/status" => to_vec(&q.transaction_status(&self.txid).unwrap()).unwrap(),
                "/hex" => q
                    .resolve_raw_transaction(&self.txid)
                    .and_then(|resolved| q.transaction_hex_resolved(resolved))
                    .unwrap()
                    .into_bytes(),
                "/raw" => q
                    .resolve_raw_transaction(&self.txid)
                    .and_then(|resolved| q.transaction_raw_resolved(resolved))
                    .unwrap(),
                "/outspend/0" => to_vec(&q.outspend(&self.txid, Vout::ZERO).unwrap()).unwrap(),
                "/outspends" => to_vec(&q.outspends(&self.txid).unwrap()).unwrap(),
                "/cpfp" => q
                    .cpfp_json_resolved(q.resolve_cpfp(&self.txid).unwrap())
                    .unwrap(),
                "/rbf" => to_vec(
                    &q.resolve_rbf(&self.txid)
                        .and_then(|resolved| q.tx_rbf_resolved(resolved))
                        .unwrap(),
                )
                .unwrap(),
                "/replacements" | "/fullrbf/replacements" => to_vec(
                    &q.recent_replacements(suffix == "/fullrbf/replacements")
                        .unwrap(),
                )
                .unwrap(),
                _ => unreachable!(),
            });
            let path = self.path(suffix);
            let response = exchange_bytes(address, "GET", &path, "\"old\"", 4_100_000).await;
            let header_end = response
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .unwrap()
                + 4;
            let headers = from_utf8(&response[..header_end]).unwrap();
            assert!(headers.starts_with("HTTP/1.1 200"), "{path}: {headers}");
            assert_eq!(&response[header_end..], expected);
            let tag = headers
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            for method in ["GET", "HEAD"] {
                for validator in [tag.as_str(), "*"] {
                    let response = exchange_with_etag(address, method, &path, validator).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
            self.tags.push(tag);
        }
    }
}
