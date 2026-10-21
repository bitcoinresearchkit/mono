use std::str::FromStr;

use bitcoin::{Txid as BitcoinTxid, hashes::hex::HexToArrayError};
use derive_more::Deref;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[cfg(feature = "storage")]
use vecdb::Bytes;

/// A transaction ID: its hash.
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Bytes))]
#[cfg_attr(
    feature = "schemars",
    schemars(
        example = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        example = "2bb85f4b004be6da54f766c17c1e855187327112c231ef2ff35ebad0ea67c69e",
        example = "9a0b3b8305bb30cacf9e8443a90d53a76379fb3305047fdeaa4e4b0934a2a1ba"
    )
)]
#[repr(C)]
#[cfg_attr(feature = "schemars", schemars(transparent, with = "String"))]
pub struct Txid([u8; 32]);

crate::bytes::impl_hash32!(Txid, BitcoinTxid, expecting: "a transaction ID");

impl Txid {
    /// All zeros: the previous-output txid of coinbase inputs.
    pub const COINBASE: Self = Self([0u8; 32]);

    /// Reinterpret a slice of `Txid`s as a slice of `bitcoin::Txid`s: a zero-cost view (no
    /// allocation, no copy).
    #[inline]
    pub fn as_bitcoin_slice(slice: &[Txid]) -> &[BitcoinTxid] {
        // SAFETY: same layout and stride (see the layout assertion).
        unsafe { &*(slice as *const [Txid] as *const [BitcoinTxid]) }
    }
}

impl FromStr for Txid {
    type Err = HexToArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BitcoinTxid::from_str(s).map(Self::from)
    }
}
