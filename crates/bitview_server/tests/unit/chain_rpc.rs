//! Shared synthetic-chain RPC replies for indexer-backed fixtures.

use bitcoin::{Block, consensus::serialize};
use serde_json::{Value, json};

pub(crate) fn reply(request: &Value, tip: u32, blocks: &[Block], active: usize) -> Value {
    if let Some(batch) = request.as_array() {
        return batch
            .iter()
            .map(|request| reply(request, tip, blocks, active))
            .collect();
    }
    let position = request["params"][0]
        .as_str()
        .map(|hash| {
            blocks
                .iter()
                .position(|block| block.block_hash().to_string() == hash)
                .unwrap()
        })
        .unwrap_or(0);
    let block = &blocks[position];
    let hash = block.block_hash().to_string();
    let ancestry = |mut position: usize| {
        let mut path = vec![position];
        while position != 0 {
            position = blocks
                .iter()
                .position(|candidate| {
                    candidate.block_hash() == blocks[position].header.prev_blockhash
                })
                .unwrap();
            path.push(position);
        }
        path.reverse();
        path
    };
    let canonical = ancestry(active);
    let result = match request["method"].as_str().unwrap() {
        "getblockcount" => json!(tip),
        "getblockhash" => {
            let height = request["params"][0].as_u64().unwrap();
            json!(blocks[canonical[height as usize]].block_hash().to_string())
        }
        "getblockheader" | "getblock" => {
            assert_eq!(request["params"][0], hash);
            json!({"hash": hash, "confirmations": if canonical.contains(&position) { 1 } else { -1 },
                "height": ancestry(position).len() - 1,
                "previousblockhash": (position != 0).then(|| block.header.prev_blockhash.to_string()),
                "version": 1, "versionHex": "00000001", "merkleroot": block.header.merkle_root.to_string(),
                "time": block.header.time, "mediantime": block.header.time,
                "nonce": block.header.nonce, "bits": "1d00ffff", "difficulty": 1,
                "chainwork": "0000000000000000000000000000000000000000000000000000000100010001", "nTx": block.txdata.len(),
                "size": serialize(block).len(), "weight": block.weight().to_wu(),
                "tx": block.txdata.iter().map(|tx| tx.compute_txid().to_string()).collect::<Vec<_>>()})
        }
        "getblockchaininfo" => json!({"chain": "main", "blocks": 0, "headers": 0,
            "bestblockhash": hash, "difficulty": 1, "time": block.header.time,
            "mediantime": block.header.time, "verificationprogress": 1,
            "initialblockdownload": false, "chainwork": "01", "size_on_disk": 293,
            "pruned": false, "warnings": []}),
        method => panic!("unexpected fixture RPC: {method}"),
    };
    json!({"id": request["id"], "result": result, "error": null})
}
