//! BIP352 from the receiver's side: the public key each input contributes, then the tweak of the
//! transaction, `input_hash·A`, which a wallet multiplies by its scan key. Keys are found as bytes
//! and parsed once per block ([`Points`]): parsing takes a square root, and a block's transactions
//! spend the same keys over and over (a third as many distinct Taproot keys as spends at the tip).

use bitcoin::{
    TxIn,
    hashes::{Hash, hash160},
    secp256k1::{PublicKey, Scalar, Secp256k1, VerifyOnly, XOnlyPublicKey},
};
use brk_types::Tweak;
use rayon::prelude::*;
use sha2::{Digest, Sha256};

/// What an input spends, as far as BIP352 tells input keys apart; every other output type
/// contributes no key.
pub enum Prevout {
    /// The public key hash the spent output locks to.
    P2PKH([u8; 20]),
    P2SH,
    P2WPKH,
    /// The spent output's x-only key.
    P2TR([u8; 32]),
}

/// BIP341's NUMS point H: a script path spend from this internal key has no key path, so BIP352
/// counts no key for it.
const NUMS_H: [u8; 32] = [
    0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
    0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
];

/// BIP341: the last of at least two witness items is the annex when it starts with this byte.
const ANNEX_TAG: u8 = 0x50;

/// The scriptSig of a P2SH-P2WPKH spend starts with a single 22-byte push of the redeem script,
/// which is `OP_0` and a 20-byte push.
const P2SH_P2WPKH_SCRIPT_SIG: [u8; 3] = [0x16, 0x00, 0x14];

/// A compressed public key's bytes, parsed later by [`Points`].
pub type Key = [u8; 33];

/// The input's public key, if BIP352 counts one: compressed keys only, a P2TR output key lifted to
/// its even point. Bytes that aren't a point count nothing once parsed.
pub fn input_key(prevout: &Prevout, input: &TxIn) -> Option<Key> {
    match prevout {
        Prevout::P2PKH(hash) => p2pkh_key(input.script_sig.as_bytes(), hash),
        Prevout::P2SH => {
            let script_sig = input.script_sig.as_bytes();
            if script_sig.len() != 23 || script_sig[..3] != P2SH_P2WPKH_SCRIPT_SIG {
                return None;
            }
            compressed_key(input.witness.last()?)
        }
        Prevout::P2WPKH => compressed_key(input.witness.last()?),
        Prevout::P2TR(output_key) => {
            let witness = &input.witness;
            let mut items = witness.len();
            if items > 1 && witness.last()?.first() == Some(&ANNEX_TAG) {
                items -= 1;
            } else if items == 0 {
                return None;
            }
            // A script path spend: its control block is the last item left, the internal key
            // after its first byte.
            if items > 1 && witness.nth(items - 1)?.get(1..33) == Some(&NUMS_H[..]) {
                return None;
            }
            Some(even(output_key))
        }
    }
}

/// The key of a P2PKH spend, malleated scriptSigs included: the last 33-byte window of the
/// scriptSig that hashes to the spent output's key hash (only a hash160 collision could make a
/// second window match, so the one that does is the key, valid or not).
fn p2pkh_key(script_sig: &[u8], hash: &[u8; 20]) -> Option<Key> {
    script_sig.windows(33).rev().find_map(|window| {
        (hash160::Hash::hash(window).as_byte_array() == hash)
            .then(|| compressed_key(window))
            .flatten()
    })
}

fn compressed_key(bytes: &[u8]) -> Option<Key> {
    bytes.try_into().ok()
}

/// An x-only key lifted to its even point, `0x02 ‖ x`: on the curve exactly when it parses.
pub fn even(x: &[u8]) -> Key {
    let mut key = [0x02; 33];
    key[1..].copy_from_slice(x);
    key
}

/// A block's keys, each distinct one parsed once.
pub struct Points {
    keys: Vec<Key>,
    points: Vec<Option<PublicKey>>,
}

impl Points {
    pub fn new(mut keys: Vec<Key>) -> Self {
        keys.sort_unstable();
        keys.dedup();
        let points = keys.par_iter().map(|key| PublicKey::from_slice(key).ok()).collect();
        Self { keys, points }
    }

    /// The point of a key given to [`Points::new`], `None` when it isn't one.
    pub fn get(&self, key: &Key) -> Option<PublicKey> {
        self.points[self.keys.binary_search(key).expect("a key given to Points::new")]
    }
}

/// Whether a Taproot output's x-only key is a point on the curve. One that isn't can't be spent or
/// be a silent payment, and spdk refuses a whole block's outputs over it.
pub fn is_on_curve(key: &[u8]) -> bool {
    XOnlyPublicKey::from_slice(key).is_ok()
}

/// The smallest outpoint the inputs spend, by its 36-byte serialization: txid, then vout
/// little-endian.
pub fn smallest_outpoint<'a>(inputs: impl IntoIterator<Item = &'a TxIn>) -> Option<[u8; 36]> {
    inputs
        .into_iter()
        .map(|input| {
            let mut outpoint = [0; 36];
            outpoint[..32].copy_from_slice(input.previous_output.txid.as_ref());
            outpoint[32..].copy_from_slice(&input.previous_output.vout.to_le_bytes());
            outpoint
        })
        .min()
}

/// Computes tweaks, from any thread.
pub struct Tweaker {
    secp: Secp256k1<VerifyOnly>,
    /// `BIP0352/Inputs`' tagged hash, its tag already hashed in.
    inputs_hash: Sha256,
}

impl Default for Tweaker {
    fn default() -> Self {
        let tag = Sha256::digest(b"BIP0352/Inputs");
        let mut inputs_hash = Sha256::new();
        inputs_hash.update(tag);
        inputs_hash.update(tag);
        Self {
            secp: Secp256k1::verification_only(),
            inputs_hash,
        }
    }
}

impl Tweaker {
    /// `input_hash·A`, where `A` sums `keys` and `input_hash` hashes the [`smallest_outpoint`]
    /// with `A`. `None` when `keys` is empty or sums to infinity: the transaction isn't eligible.
    pub fn tweak(&self, smallest_outpoint: &[u8; 36], keys: &[PublicKey]) -> Option<Tweak> {
        // One key is its own sum: combining it would still pay an inversion.
        let sum = match keys {
            [key] => *key,
            keys => PublicKey::combine_keys(&keys.iter().collect::<Vec<_>>()).ok()?,
        };
        let mut hash = self.inputs_hash.clone();
        hash.update(smallest_outpoint);
        hash.update(sum.serialize());
        let input_hash = Scalar::from_be_bytes(hash.finalize().into()).ok()?;
        Some(Tweak::from(sum.mul_tweak(&self.secp, &input_hash).ok()?.serialize()))
    }
}
