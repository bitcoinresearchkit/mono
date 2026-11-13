//! What bytes are as a signature, the one rule the feature flags and the counters share: what a
//! node reads to verify one, an ECDSA signature in DER, strict or lax, or a Schnorr signature by
//! BIP-341's lengths, each with its sighash type.

use std::cmp::Ordering;

use bitcoin::{
    secp256k1::ecdsa::Signature,
    sighash::{EcdsaSighashType, TapSighashType},
};

/// The half of secp256k1's order: an S at or below it is low.
const HALF_ORDER: [u8; 32] = [
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D, 0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
];
/// The fewest bytes that can hold an ECDSA signature and its sighash byte. Lax DER has no most:
/// its lengths and integers may carry any padding.
const MIN_ECDSA: usize = 9;

/// An ECDSA signature and its sighash byte.
pub struct Ecdsa {
    pub strict_der: bool,
    pub low_r: bool,
    pub low_s: bool,
    /// Its sighash byte's standard type, none for another byte, which consensus also signs with.
    pub sighash: Option<EcdsaSighashType>,
}

impl Ecdsa {
    /// `bytes` as a node parses them to verify a signature: DER, strict or else lax, then a
    /// sighash byte. None if they can't be one.
    pub fn of(bytes: &[u8]) -> Option<Self> {
        // Both encodings open with a sequence's tag: a key or a script's push rarely gets past it.
        if bytes.len() < MIN_ECDSA || bytes[0] != 0x30 {
            return None;
        }
        let sighash = EcdsaSighashType::from_standard(u32::from(bytes[bytes.len() - 1])).ok();
        if is_strict_der(bytes) {
            let r_len = usize::from(bytes[3]);
            let s_len = usize::from(bytes[5 + r_len]);
            let (r, s) = (&bytes[4..4 + r_len], &bytes[6 + r_len..6 + r_len + s_len]);
            return Some(Self {
                strict_der: true,
                low_r: r.len() < 33 && (r.len() < 32 || r[0] < 0x80),
                low_s: is_low_s(s),
                sighash,
            });
        }
        let compact = Signature::from_der_lax(&bytes[..bytes.len() - 1])
            .ok()?
            .serialize_compact();
        // An R or S past the curve's order, which the lax parser reads as an all-zero signature
        // that verifies nothing.
        if compact == [0; 64] {
            return None;
        }
        Some(Self {
            strict_der: false,
            low_r: compact[0] < 0x80,
            low_s: compact[32..] <= HALF_ORDER[..],
            sighash,
        })
    }
}

/// A Schnorr signature's sighash type: 64 bytes sign with the default, 65 name theirs in the last
/// byte, which may not be 0x00. None if `bytes` can't be one.
pub fn schnorr_sighash(bytes: &[u8]) -> Option<TapSighashType> {
    match bytes.len() {
        64 => Some(TapSighashType::Default),
        65 if bytes[64] != 0 => TapSighashType::from_consensus_u8(bytes[64]).ok(),
        _ => None,
    }
}

/// A DER integer against the half order, compared as 32 big-endian bytes.
fn is_low_s(s: &[u8]) -> bool {
    let s = s.trim_prefix(&[0]);
    match s.len().cmp(&32) {
        Ordering::Less => true,
        Ordering::Equal => s <= &HALF_ORDER[..],
        Ordering::Greater => false,
    }
}

/// Bitcoin Core's `IsValidSignatureEncoding`: strict DER, with its sighash byte.
fn is_strict_der(signature: &[u8]) -> bool {
    let len = signature.len();
    if !(9..=73).contains(&len) || signature[0] != 0x30 || usize::from(signature[1]) != len - 3 {
        return false;
    }
    let r_len = usize::from(signature[3]);
    if 5 + r_len >= len {
        return false;
    }
    let s_len = usize::from(signature[5 + r_len]);
    if r_len + s_len + 7 != len || signature[2] != 2 || r_len == 0 || signature[4] & 0x80 != 0 {
        return false;
    }
    if r_len > 1 && signature[4] == 0 && signature[5] & 0x80 == 0 {
        return false;
    }
    if signature[r_len + 4] != 2 || s_len == 0 || signature[r_len + 6] & 0x80 != 0 {
        return false;
    }
    !(s_len > 1 && signature[r_len + 6] == 0 && signature[r_len + 7] & 0x80 == 0)
}
