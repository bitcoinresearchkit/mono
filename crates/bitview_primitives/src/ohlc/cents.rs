use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Add,
};

use brk_types::{Cents, Dollars};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeTuple};

use super::OHLCDollars;

#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// OHLC (Open, High, Low, Close) data in cents
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct OHLCCents {
    pub open: Cents,
    pub high: Cents,
    pub low: Cents,
    pub close: Cents,
}

/// A single price is a flat candle: every role takes it.
impl From<Cents> for OHLCCents {
    #[inline]
    fn from(price: Cents) -> Self {
        Self {
            open: price,
            high: price,
            low: price,
            close: price,
        }
    }
}

impl Serialize for OHLCCents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.open)?;
        tup.serialize_element(&self.high)?;
        tup.serialize_element(&self.low)?;
        tup.serialize_element(&self.close)?;
        tup.end()
    }
}

/// From `[open, high, low, close]`.
macro_rules! impl_ohlc_deserialize {
    ($ohlc_type:ident, $inner_type:ty) => {
        impl<'de> Deserialize<'de> for $ohlc_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let [open, high, low, close] = <[$inner_type; 4]>::deserialize(deserializer)?;
                Ok($ohlc_type {
                    open,
                    high,
                    low,
                    close,
                })
            }
        }
    };
}

impl_ohlc_deserialize!(OHLCCents, Cents);
impl_ohlc_deserialize!(OHLCDollars, Dollars);

/// The schema of the serialized form: `[open, high, low, close]`.
macro_rules! impl_ohlc_schema {
    ($ohlc_type:ident, $inner_type:ty, $description:literal) => {
        impl JsonSchema for $ohlc_type {
            fn schema_name() -> Cow<'static, str> {
                stringify!($ohlc_type).into()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                let mut schema = <[$inner_type; 4]>::json_schema(generator);
                schema.insert("description".into(), $description.into());
                schema
            }
        }
    };
}

impl_ohlc_schema!(
    OHLCCents,
    Cents,
    "Open, high, low and close prices in cents: [open, high, low, close]."
);
impl_ohlc_schema!(
    OHLCDollars,
    Dollars,
    "Open, high, low and close prices in US dollars: [open, high, low, close]."
);

impl Display for OHLCCents {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

#[cfg(feature = "storage")]
impl Formattable for OHLCCents {
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
// SAFETY: not native layout.
unsafe impl Bytes for OHLCCents {
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
            open: Cents::from_bytes(&bytes[0..8])?,
            high: Cents::from_bytes(&bytes[8..16])?,
            low: Cents::from_bytes(&bytes[16..24])?,
            close: Cents::from_bytes(&bytes[24..32])?,
        })
    }
}

impl Add for OHLCCents {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            open: self.open + rhs.open,
            high: self.high + rhs.high,
            low: self.low + rhs.low,
            close: self.close + rhs.close,
        }
    }
}
