use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult},
};

use brk_types::{Cents, Dollars};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable, Result as VecdbResult};

/// Declares an OHLC (open, high, low, close) record over an 8-byte price type: serialized,
/// formatted and documented as `[open, high, low, close]`, stored as the four prices in order.
macro_rules! ohlc {
    ($(#[$meta:meta])* $name:ident($price:ty), schema: $description:literal) => {
        $(#[$meta])*
        #[derive(Debug, Default, Clone, Copy)]
        #[repr(C)]
        pub struct $name {
            pub open: $price,
            pub high: $price,
            pub low: $price,
            pub close: $price,
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                [self.open, self.high, self.low, self.close].serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let [open, high, low, close] = <[$price; 4]>::deserialize(deserializer)?;
                Ok(Self {
                    open,
                    high,
                    low,
                    close,
                })
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                let mut schema = <[$price; 4]>::json_schema(generator);
                schema.insert("description".into(), $description.into());
                schema
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(
                    f,
                    "{}, {}, {}, {}",
                    self.open, self.high, self.low, self.close
                )
            }
        }

        #[cfg(feature = "storage")]
        impl Formattable for $name {
            /// `[open, high, low, close]`, each price in its JSON form.
            fn write_to(&self, buf: &mut Vec<u8>) {
                // Prices are always defined: a period without blocks carries the previous close.
                debug_assert!(
                    !(self.open.is_nan()
                        || self.high.is_nan()
                        || self.low.is_nan()
                        || self.close.is_nan()),
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
        unsafe impl Bytes for $name {
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
                    open: <$price>::from_bytes(&bytes[0..8])?,
                    high: <$price>::from_bytes(&bytes[8..16])?,
                    low: <$price>::from_bytes(&bytes[16..24])?,
                    close: <$price>::from_bytes(&bytes[24..32])?,
                })
            }
        }
    };
}

ohlc!(
    /// OHLC (Open, High, Low, Close) data in cents
    OHLCCents(Cents),
    schema: "Open, high, low and close prices in cents: [open, high, low, close]."
);

ohlc!(
    /// OHLC (Open, High, Low, Close) data in dollars
    OHLCDollars(Dollars),
    schema: "Open, high, low and close prices in US dollars: [open, high, low, close]."
);

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

impl From<OHLCCents> for OHLCDollars {
    #[inline]
    fn from(value: OHLCCents) -> Self {
        Self {
            open: Dollars::from(value.open),
            high: Dollars::from(value.high),
            low: Dollars::from(value.low),
            close: Dollars::from(value.close),
        }
    }
}
