//! The two block filters BlindBit's light clients test before fetching anything else: BIP158
//! Golomb-coded sets (its basic filters' parameters, keyed by the block hash), one of the block's
//! new Taproot output keys, one of its spends of Taproot outputs.

use bitcoin::{OutPoint, Transaction, bip158::GcsFilterWriter};
use brk_types::OutputType;
use sha2::{Digest, Sha256};

/// BIP158 basic filters' false positive parameter and modulus, which BlindBit's filters use.
const P: u8 = 19;
const M: u64 = 784_931;

/// A Golomb-coded set of `elements` keyed by `block_hash` (internal byte order), as BIP158
/// serializes it: the element count, then the coded deltas. Repeats count once, as in BIP158.
pub fn filter<E: AsRef<[u8]>>(
    block_hash: &[u8; 32],
    elements: impl IntoIterator<Item = E>,
) -> Vec<u8> {
    let [k0, k1] = [0, 8].map(|at| u64::from_le_bytes(block_hash[at..at + 8].try_into().unwrap()));
    let mut bytes = Vec::new();
    let mut writer = GcsFilterWriter::new(&mut bytes, k0, k1, M, P);
    for element in elements {
        writer.add_element(element.as_ref());
    }
    writer.finish().expect("writing to memory");
    bytes
}

/// The x-only keys of the transactions' Taproot outputs.
pub fn new_outputs(txdata: &[Transaction]) -> impl Iterator<Item = &[u8]> {
    txdata
        .iter()
        .flat_map(|tx| &tx.output)
        .filter(|output| output.script_pubkey.is_p2tr())
        // `OP_1`, a 32-byte push, then the key.
        .map(|output| &output.script_pubkey.as_bytes()[2..])
}

/// The block's spends of Taproot outputs as BlindBit hashes them: the first 8 bytes of
/// `sha256(txid ‖ vout ‖ block hash)`, the txid and block hash in internal byte order and the vout
/// little-endian. `input_types` classifies what each of its inputs spends, in order.
pub fn spent_outputs<'a>(
    block_hash: &'a [u8; 32],
    txdata: &'a [Transaction],
    input_types: &'a [OutputType],
) -> impl Iterator<Item = [u8; 8]> + 'a {
    txdata
        .iter()
        .flat_map(|tx| &tx.input)
        .zip(input_types)
        .filter(|(_, output_type)| **output_type == OutputType::P2TR)
        .map(move |(input, _)| spent_hash(&input.previous_output, block_hash))
}

fn spent_hash(outpoint: &OutPoint, block_hash: &[u8; 32]) -> [u8; 8] {
    let mut hash = Sha256::new();
    hash.update(outpoint.txid.as_ref() as &[u8]);
    hash.update(outpoint.vout.to_le_bytes());
    hash.update(block_hash);
    hash.finalize()[..8].try_into().unwrap()
}
