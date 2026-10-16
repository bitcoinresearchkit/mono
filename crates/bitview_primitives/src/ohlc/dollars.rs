use std::fmt::{Display, Formatter, Result as FmtResult};

use brk_types::Dollars;
use serde::{Serialize, Serializer, ser::SerializeTuple};

use super::OHLCCents;

#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// OHLC (Open, High, Low, Close) data in dollars
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct OHLCDollars {
    pub open: Dollars,
    pub high: Dollars,
    pub low: Dollars,
    pub close: Dollars,
}

impl Serialize for OHLCDollars {
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

impl From<OHLCCents> for OHLCDollars {
    #[inline]
    fn from(value: OHLCCents) -> Self {
        Self::from(&value)
    }
}

impl From<&OHLCCents> for OHLCDollars {
    #[inline]
    fn from(value: &OHLCCents) -> Self {
        Self {
            open: Dollars::from(value.open),
            high: Dollars::from(value.high),
            low: Dollars::from(value.low),
            close: Dollars::from(value.close),
        }
    }
}

impl Display for OHLCDollars {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

#[cfg(feature = "storage")]
impl Formattable for OHLCDollars {
    /// `[open, high, low, close]`, each price in its JSON form.
    fn write_to(&self, buf: &mut Vec<u8>) {
        // Prices are always defined: a period without blocks carries the previous close.
        debug_assert!(
            !(self.open.is_nan() || self.high.is_nan() || self.low.is_nan() || self.close.is_nan()),
            "undefined OHLC price"
        );
        buf.push(b'[');
        self.open.fmt_json(buf);
        buf.push(b',');
        self.high.fmt_json(buf);
        buf.push(b',');
        self.low.fmt_json(buf);
        buf.push(b',');
        self.close.fmt_json(buf);
        buf.push(b']');
    }

    fn fmt_csv(&self, buf: &mut Vec<u8>) {
        let start = buf.len();
        self.write_to(buf);
        vecdb::escape_csv_cell(buf, start);
    }
}

#[cfg(feature = "storage")]
// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for OHLCDollars {
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
            open: Dollars::from_bytes(&bytes[0..8])?,
            high: Dollars::from_bytes(&bytes[8..16])?,
            low: Dollars::from_bytes(&bytes[16..24])?,
            close: Dollars::from_bytes(&bytes[24..32])?,
        })
    }
}
