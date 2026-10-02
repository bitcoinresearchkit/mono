use std::io::Cursor;

use bitcoin::{
    Amount, Block, Network, ScriptBuf, TxOut, Witness, WitnessMerkleNode,
    blockdata::constants::genesis_block, consensus::serialize, hashes::Hash,
};

use super::*;

fn frame(block: &Block) -> Vec<u8> {
    let bytes = serialize(block);
    let mut frame = Magic::BITCOIN.to_bytes().to_vec();
    frame.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    frame.extend_from_slice(&bytes);
    frame
}

#[test]
fn raw_records_bound_allocation_and_verify_framing_and_identity() {
    let block = genesis_block(Network::Bitcoin);
    let bytes = serialize(&block);
    let hash = block.block_hash().into();
    let framed = frame(&block);
    let mut prefix_only = Cursor::new(&framed[..88]);
    assert_eq!(
        Query::read_raw_prefix(&mut prefix_only, bytes.len() as u64, &hash).unwrap(),
        bytes[..80]
    );
    assert_eq!(prefix_only.position(), 88);
    assert_eq!(
        Query::read_raw_record(&framed[..], bytes.len() as u64, &hash).unwrap(),
        bytes
    );
    for size in [0, 80, MAX_BLOCK_BYTES + 1, u64::MAX] {
        assert!(matches!(
            Query::read_raw_record(&[][..], size, &hash),
            Err(Error::Internal(_))
        ));
    }
    for offset in [0, 4, 8 + 76] {
        let mut changed = framed.clone();
        changed[offset] ^= 1;
        assert!(Query::read_raw_record(&changed[..], bytes.len() as u64, &hash).is_err());
    }
    assert!(
        Query::read_raw_record(&framed[..framed.len() - 1], bytes.len() as u64, &hash).is_err()
    );
}

#[test]
fn raw_payload_checks_transactions_weight_trailing_bytes_and_witness() {
    let mut block = genesis_block(Network::Bitcoin);
    let bytes = serialize(&block);
    let weight = block.weight().to_wu();
    Query::verify_raw_payload(&bytes, weight, 1).unwrap();
    assert!(Query::verify_raw_payload(&bytes, weight + 1, 1).is_err());
    assert!(Query::verify_raw_payload(&bytes, weight, 2).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Query::verify_raw_payload(&trailing, weight, 1).is_err());
    block.txdata[0].output[0].value = Amount::from_sat(1);
    assert!(Query::verify_raw_payload(&serialize(&block), weight, 1).is_err());

    let mut witness_block = genesis_block(Network::Bitcoin);
    witness_block.txdata[0].input[0].witness = Witness::from_slice(&[[0u8; 32]]);
    let commitment = Block::compute_witness_commitment(&WitnessMerkleNode::all_zeros(), &[0; 32]);
    let mut script = vec![0x6a, 0x24, 0xaa, 0x21, 0xa9, 0xed];
    script.extend_from_slice(commitment.as_byte_array());
    witness_block.txdata[0].output.push(TxOut {
        value: Amount::ZERO,
        script_pubkey: ScriptBuf::from_bytes(script),
    });
    witness_block.header.merkle_root = witness_block.compute_merkle_root().unwrap();
    let weight = witness_block.weight().to_wu();
    Query::verify_raw_payload(&serialize(&witness_block), weight, 1).unwrap();
    witness_block.txdata[0].input[0].witness = Witness::from_slice(&[[1u8; 32]]);
    assert!(Query::verify_raw_payload(&serialize(&witness_block), weight, 1).is_err());
}
