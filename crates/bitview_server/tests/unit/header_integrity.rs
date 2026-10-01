use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    net::SocketAddr,
    path::Path,
    str::from_utf8,
};

use bitcoin::{Amount, Block, Txid as BitcoinTxid, consensus::serialize, hashes::Hash};
use bitview_query::AsyncQuery;
use brk_types::{BlockHash, BlockHashPrefix, Txid, TxidPrefix, Vout};
use serde_json::{Value, from_str, to_value};
use vecdb::ReadableVec;

use super::server_routes::{exchange_bytes, exchange_with_etag};
pub(crate) async fn check_header_integrity(
    query: &AsyncQuery,
    address: SocketAddr,
    path: &Path,
    block: &Block,
) {
    let position = query.sync(|q| {
        q.indexer()
            .vecs()
            .blocks
            .position
            .collect_one(1u32.into())
            .unwrap()
    });
    assert_eq!(position.blk_index(), 0);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.seek(SeekFrom::Start(u64::from(position.offset())))
        .unwrap();
    let mut original = [0u8; 80];
    file.read_exact(&mut original).unwrap();
    assert_eq!(original.as_slice(), serialize(&block.header));
    let paths = [
        format!("/api/block/{}", block.block_hash()),
        format!("/api/block/{}/header", block.block_hash()),
        format!("/api/v1/block/{}", block.block_hash()),
        "/api/blocks".to_owned(),
        "/api/v1/blocks".to_owned(),
        "/api/v1/blocks/1".to_owned(),
    ];
    let mut responses = Vec::new();
    for path in &paths {
        let response = exchange_with_etag(address, "GET", path, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let tag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap();
        let legacy = if path.ends_with("/header") {
            format!(
                "W/\"b1-{:x}\"",
                *BlockHashPrefix::from(&BlockHash::from(block.block_hash()))
            )
        } else {
            tag.replace("block-v3-", "block-v2-")
                .replace("v1-4-", "v1-3-")
                .replace("blocks2-", "blocks1-")
        };
        assert_ne!(legacy, tag);
        responses.push((
            legacy,
            response.split_once("\r\n\r\n").unwrap().1.to_owned(),
        ));
    }
    // A validly encoded but different header at the indexed position must not
    // be combined with this block's indexed ID, timestamp or transaction data.
    let mut changed = original;
    changed[76] ^= 1;
    file.seek(SeekFrom::Start(u64::from(position.offset())))
        .unwrap();
    file.write_all(&changed).unwrap();
    file.flush().unwrap();
    let raw_path = format!("/api/block/{}/raw", block.block_hash());
    for method in ["GET", "HEAD"] {
        let response = exchange_bytes(address, method, &raw_path, "\"old\"", 4_100_000).await;
        assert!(response.starts_with(b"HTTP/1.1 500"));
        let headers_end = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
        let headers = from_utf8(&response[..headers_end]).unwrap();
        assert!(headers.contains("\r\ncache-control: no-store\r\n"));
        assert!(!headers.contains("\r\netag:"));
        if method == "HEAD" {
            assert_eq!(response.len(), headers_end);
        }
    }
    for (path, (legacy, _)) in paths.iter().zip(&responses) {
        for method in ["GET", "HEAD"] {
            let response = exchange_with_etag(address, method, path, legacy).await;
            assert!(response.starts_with("HTTP/1.1 500"), "{path}: {response}");
            assert!(
                response.contains("\r\ncache-control: no-store\r\n"),
                "{response}"
            );
            assert!(!response.contains("\r\netag:"));
            if method == "HEAD" {
                assert!(response.ends_with("\r\n\r\n"));
            }
        }
    }
    file.seek(SeekFrom::Start(u64::from(position.offset())))
        .unwrap();
    file.write_all(&original).unwrap();
    file.flush().unwrap();
    for (path, (legacy, expected)) in paths.iter().zip(responses) {
        let response = exchange_with_etag(address, "GET", path, &legacy).await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
    }
    // A body mutation with an unchanged header and record length must also fail.
    let original = serialize(block);
    let mut changed = block.clone();
    changed.txdata[0].output[0].value = Amount::from_sat(1);
    let changed = serialize(&changed);
    assert_eq!(changed.len(), original.len());
    assert_eq!(changed[..80], original[..80]);
    file.seek(SeekFrom::Start(u64::from(position.offset())))
        .unwrap();
    file.write_all(&changed).unwrap();
    file.flush().unwrap();
    let response = exchange_bytes(address, "GET", &raw_path, "\"old\"", 4_100_000).await;
    assert!(response.starts_with(b"HTTP/1.1 500"));
    let headers_end = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    let headers = from_utf8(&response[..headers_end]).unwrap();
    assert!(headers.contains("\r\ncache-control: no-store\r\n"));
    assert!(!headers.contains("\r\netag:"));
    // HEAD validates metadata and header identity, not transaction contents.
    let response = exchange_with_etag(address, "HEAD", &raw_path, "\"old\"").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains(&format!("\r\ncontent-length: {}\r\n", original.len())));
    assert!(response.ends_with("\r\n\r\n"));
    file.seek(SeekFrom::Start(u64::from(position.offset())))
        .unwrap();
    file.write_all(&original).unwrap();
    file.flush().unwrap();
    let response = exchange_bytes(address, "GET", &raw_path, "\"old\"", 4_100_000).await;
    assert!(response.starts_with(b"HTTP/1.1 200"));
    let headers_end = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    assert_eq!(response[headers_end..], original);
}

pub(crate) async fn check_prevouts(
    query: &AsyncQuery,
    address: SocketAddr,
    block: &Block,
    genesis: &Block,
) {
    let parent = &block.txdata[0];
    for transaction in block.txdata.iter().skip(1) {
        let txid = transaction.compute_txid().into();
        let bytes = query
            .sync(|q| q.transaction_json_resolved(q.resolve_transaction(&txid)?))
            .unwrap();
        let actual: Value = serde_json::from_slice(&bytes).unwrap();
        let inputs = actual["vin"].as_array().unwrap();
        assert_eq!(inputs.len(), transaction.input.len());
        for (input, expected) in inputs.iter().zip(&transaction.input) {
            let parent = block
                .txdata
                .iter()
                .find(|parent| parent.compute_txid() == expected.previous_output.txid)
                .unwrap();
            let output = &parent.output[expected.previous_output.vout as usize];
            assert_eq!(
                input["prevout"]["scriptpubkey"],
                to_value(&output.script_pubkey).unwrap()
            );
            assert_eq!(
                input["prevout"]["value"].as_u64().unwrap(),
                output.value.to_sat()
            );
        }
        let response =
            exchange_with_etag(address, "GET", &format!("/api/tx/{txid}"), "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body, actual);
    }
    let parent_txid = Txid::from(parent.compute_txid());
    let mut collision = parent.compute_txid().to_byte_array();
    collision[8] ^= 1;
    let collision = Txid::from(BitcoinTxid::from_byte_array(collision));
    assert_eq!(TxidPrefix::from(parent_txid), TxidPrefix::from(collision));
    let mut holes = (0..parent.output.len())
        .map(|vout| (parent_txid, Vout::from(vout)))
        .collect::<Vec<_>>();
    holes.extend([
        (parent_txid, Vout::from(parent.output.len())),
        (parent_txid, Vout::from(u16::MAX)),
        (collision, Vout::from(0u16)),
        (genesis.txdata[0].compute_txid().into(), Vout::from(1u16)),
    ]);
    let resolved = query
        .run(move |q| Ok(q.indexer_prevout_resolver()(&holes)))
        .await
        .unwrap();
    assert_eq!(
        resolved.len(),
        parent.output.len(),
        "prevout resolution must reject collision and cross-transaction indices"
    );
    for (vout, output) in parent.output.iter().enumerate() {
        let actual = &resolved[&(parent_txid, Vout::from(vout))];
        assert_eq!(actual.script_pubkey, output.script_pubkey);
        assert_eq!(u64::from(actual.value), output.value.to_sat());
    }
}
