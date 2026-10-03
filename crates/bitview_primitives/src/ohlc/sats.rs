use std::fmt::{Display, Formatter, Result as FmtResult};

use brk_types::Sats;
use schemars::JsonSchema;
use serde::{Serialize, Serializer, ser::SerializeTuple};

use super::{Close, High, Low, Open};

#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// OHLC (Open, High, Low, Close) data in satoshis
#[derive(Debug, Default, Clone, Copy, JsonSchema)]
#[repr(C)]
pub struct OHLCSats {
    pub open: Open<Sats>,
    pub high: High<Sats>,
    pub low: Low<Sats>,
    pub close: Close<Sats>,
}

impl Serialize for OHLCSats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&self.open)?;
        tuple.serialize_element(&self.high)?;
        tuple.serialize_element(&self.low)?;
        tuple.serialize_element(&self.close)?;
        tuple.end()
    }
}

impl From<(Open<Sats>, High<Sats>, Low<Sats>, Close<Sats>)> for OHLCSats {
    #[inline]
    fn from(value: (Open<Sats>, High<Sats>, Low<Sats>, Close<Sats>)) -> Self {
        Self {
            open: value.0,
            high: value.1,
            low: value.2,
            close: value.3,
        }
    }
}

impl From<Close<Sats>> for OHLCSats {
    #[inline]
    fn from(value: Close<Sats>) -> Self {
        Self {
            open: Open::from(value),
            high: High::from(value),
            low: Low::from(value),
            close: value,
        }
    }
}

impl Display for OHLCSats {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

#[cfg(feature = "storage")]
impl Formattable for OHLCSats {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.push(b'[');
        self.open.write_to(buf);
        buf.push(b',');
        self.high.write_to(buf);
        buf.push(b',');
        self.low.write_to(buf);
        buf.push(b',');
        self.close.write_to(buf);
        buf.push(b']');
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
}

#[cfg(feature = "storage")]
impl Bytes for OHLCSats {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut arr = [0u8; size_of::<Self>()];
        arr[0..8].copy_from_slice(self.open.to_bytes().as_ref());
        arr[8..16].copy_from_slice(self.high.to_bytes().as_ref());
        arr[16..24].copy_from_slice(self.low.to_bytes().as_ref());
        arr[24..32].copy_from_slice(self.close.to_bytes().as_ref());
        arr
    }

    fn from_bytes(bytes: &[u8]) -> VecdbResult<Self> {
        Ok(Self {
            open: Open::<Sats>::from_bytes(&bytes[0..8])?,
            high: High::<Sats>::from_bytes(&bytes[8..16])?,
            low: Low::<Sats>::from_bytes(&bytes[16..24])?,
            close: Close::<Sats>::from_bytes(&bytes[24..32])?,
        })
    }
}
