use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{TxIndex, Vout};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct OutPoint(u64);

impl OutPoint {
    pub const COINBASE: Self = Self(u64::MAX);

    pub fn new(tx_index: TxIndex, vout: Vout) -> Self {
        let tx_index_bits = u64::from(tx_index) << 32;
        let vout_bits = u64::from(vout);
        Self(tx_index_bits | vout_bits)
    }

    #[inline(always)]
    pub fn tx_index(self) -> TxIndex {
        TxIndex::from((self.0 >> 32) as u32)
    }

    #[inline(always)]
    pub fn vout(self) -> Vout {
        if self.is_coinbase() {
            return Vout::MAX;
        }
        Vout::from(self.0 as u32)
    }

    #[inline(always)]
    pub fn is_coinbase(self) -> bool {
        self == Self::COINBASE
    }

    #[inline(always)]
    pub fn is_not_coinbase(self) -> bool {
        self != Self::COINBASE
    }
}

impl Display for OutPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "tx_index: {}, vout: {}", self.tx_index(), self.vout())
    }
}

#[cfg(feature = "storage")]
impl Formattable for OutPoint {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_csv(&self, f: &mut String) -> FmtResult {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl Serialize for OutPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OutPoint", 2)?;
        state.serialize_field("tx_index", &self.tx_index())?;
        state.serialize_field("vout", &self.vout())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for OutPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            tx_index: TxIndex,
            vout: Vout,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Self::new(h.tx_index, h.vout))
    }
}
