use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Add,
};

use brk_types::{Cents, Dollars, Sats};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
};

use super::{Close, High, Low, OHLCDollars, OHLCSats, Open};

#[cfg(feature = "storage")]
use vecdb::Result as VecdbResult;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// OHLC (Open, High, Low, Close) data in cents
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct OHLCCents {
    pub open: Open<Cents>,
    pub high: High<Cents>,
    pub low: Low<Cents>,
    pub close: Close<Cents>,
}

impl From<(Open<Cents>, High<Cents>, Low<Cents>, Close<Cents>)> for OHLCCents {
    #[inline]
    fn from(value: (Open<Cents>, High<Cents>, Low<Cents>, Close<Cents>)) -> Self {
        Self {
            open: value.0,
            high: value.1,
            low: value.2,
            close: value.3,
        }
    }
}

impl From<Close<Cents>> for OHLCCents {
    #[inline]
    fn from(value: Close<Cents>) -> Self {
        Self {
            open: Open::from(value),
            high: High::from(value),
            low: Low::from(value),
            close: value,
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

macro_rules! impl_ohlc_deserialize {
    ($ohlc_type:ty, $inner_type:ty) => {
        impl<'de> Deserialize<'de> for $ohlc_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct OHLCVisitor;

                impl<'de> Visitor<'de> for OHLCVisitor {
                    type Value = $ohlc_type;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a tuple of 4 elements (open, high, low, close)")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        let open = seq
                            .next_element::<$inner_type>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                        let high = seq
                            .next_element::<$inner_type>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        let low = seq
                            .next_element::<$inner_type>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                        let close = seq
                            .next_element::<$inner_type>()?
                            .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;

                        Ok(Self::Value::from((
                            Open::new(open),
                            High::new(high),
                            Low::new(low),
                            Close::new(close),
                        )))
                    }
                }

                deserializer.deserialize_tuple(4, OHLCVisitor)
            }
        }
    };
}

impl_ohlc_deserialize!(OHLCCents, Cents);
impl_ohlc_deserialize!(OHLCDollars, Dollars);
impl_ohlc_deserialize!(OHLCSats, Sats);

/// The schema of the serialized form: `[open, high, low, close]`.
macro_rules! impl_ohlc_schema {
    ($ohlc_type:ident, $inner_type:ty) => {
        impl JsonSchema for $ohlc_type {
            fn schema_name() -> Cow<'static, str> {
                stringify!($ohlc_type).into()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                let mut schema = <[$inner_type; 4]>::json_schema(generator);
                schema.insert("description".into(), "[open, high, low, close]".into());
                schema
            }
        }
    };
}

impl_ohlc_schema!(OHLCCents, Cents);
impl_ohlc_schema!(OHLCDollars, Dollars);
impl_ohlc_schema!(OHLCSats, Sats);

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
        (*self.open).fmt_json(buf);
        buf.push(b',');
        (*self.high).fmt_json(buf);
        buf.push(b',');
        (*self.low).fmt_json(buf);
        buf.push(b',');
        (*self.close).fmt_json(buf);
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
impl Bytes for OHLCCents {
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
            open: Open::<Cents>::from_bytes(&bytes[0..8])?,
            high: High::<Cents>::from_bytes(&bytes[8..16])?,
            low: Low::<Cents>::from_bytes(&bytes[16..24])?,
            close: Close::<Cents>::from_bytes(&bytes[24..32])?,
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
