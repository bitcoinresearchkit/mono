use std::{fmt, mem, str::FromStr};

use bitcoin::{
    Txid as BitcoinTxid,
    hashes::{Hash, hex::HexToArrayError},
};
use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Transaction ID (hash)
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

impl Txid {
    /// Coinbase transaction "txid" - all zeros (used for coinbase inputs)
    pub const COINBASE: Self = Self([0u8; 32]);

    /// Reinterpret a slice of `Txid`s as a slice of `bitcoin::Txid`s.
    /// Both are `#[repr(C)]` newtypes over `[u8; 32]` with identical
    /// layout, so this is a zero-cost view (no allocation, no copy).
    #[inline]
    pub fn as_bitcoin_slice(slice: &[Txid]) -> &[BitcoinTxid] {
        unsafe { &*(slice as *const [Txid] as *const [BitcoinTxid]) }
    }
}

impl From<BitcoinTxid> for Txid {
    #[inline]
    fn from(value: BitcoinTxid) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&BitcoinTxid> for &Txid {
    #[inline]
    fn from(value: &BitcoinTxid) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<Txid> for BitcoinTxid {
    #[inline]
    fn from(value: Txid) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<&Txid> for BitcoinTxid {
    #[inline]
    fn from(value: &Txid) -> Self {
        BitcoinTxid::from_slice(&value.0).unwrap()
    }
}

impl From<&Txid> for &BitcoinTxid {
    #[inline]
    fn from(value: &Txid) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl fmt::Display for Txid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BitcoinTxid::from(self))
    }
}

impl FromStr for Txid {
    type Err = HexToArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BitcoinTxid::from_str(s).map(Self::from)
    }
}

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TxidVisitor)
    }
}

struct TxidVisitor;

impl de::Visitor<'_> for TxidVisitor {
    type Value = Txid;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a transaction ID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Txid::from_str(value).map_err(E::custom)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Txid {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
