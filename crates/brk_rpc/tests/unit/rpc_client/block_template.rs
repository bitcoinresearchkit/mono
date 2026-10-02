use bitcoin::{
    ScriptBuf, Transaction, TxIn, TxOut, absolute::LockTime, consensus::encode,
    transaction::Version,
};

use super::*;

fn tx(seed: u32) -> Transaction {
    let mut input = TxIn::default();
    input.previous_output.vout = seed;
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![input],
        output: vec![TxOut {
            value: Amount::from_sat(1_000),
            script_pubkey: ScriptBuf::new(),
        }],
    }
}

fn row(tx: &Transaction, depends: Vec<i64>) -> BlockTemplateTransaction {
    BlockTemplateTransaction {
        data: encode::serialize_hex(tx),
        txid: tx.compute_txid().to_string(),
        hash: tx.compute_wtxid().to_string(),
        depends,
        fee: 100,
        sigops: 0,
        weight: tx.weight().to_wu(),
    }
}

#[test]
fn repeated_parent_inputs_normalize_cores_repeated_dependency_indexes() {
    let mut parent = tx(0);
    parent.output.push(parent.output[0].clone());
    let mut child = tx(1);
    child.input[0].previous_output.txid = parent.compute_txid();
    child.input[0].previous_output.vout = 0;
    let mut second = child.input[0].clone();
    second.previous_output.vout = 1;
    child.input.push(second);
    let rows = ClientInner::build_gbt(vec![row(&parent, vec![]), row(&child, vec![1, 1])]).unwrap();
    assert_eq!(rows[1].depends, vec![Txid::from(parent.compute_txid())]);
}
