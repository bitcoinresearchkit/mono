use std::net::SocketAddr;

use bitcoin::{Block, Txid as BitcoinTxid, hashes::Hash};
use bitview_query::AsyncQuery;
use brk_types::{Txid, TxidPrefix, Vout};
use serde_json::{Value, from_str, to_value};

use super::server_routes::exchange_with_etag;

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
