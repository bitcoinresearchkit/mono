use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

const MAX: u64 = (1 << 40) - 1;

/// A `u64` index stored in 5 little-endian bytes: raw vecs of global
/// transaction input/output indexes, 3/8 smaller than `u64`.
///
/// Holds values below `2^40 - 1`; all-ones bytes encode `u64::MAX`
/// (sentinels such as `TxInIndex::UNSPENT`). Serializes, formats and
/// describes itself as `T`.
#[repr(transparent)]
pub struct Index40<T>([u8; 5], PhantomData<T>);

// Manual impls: derives would bound `T`, which only tags the index domain.
impl<T> Clone for Index40<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Index40<T> {}

impl<T> PartialEq for Index40<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Index40<T> {}

impl<T> Index40<T>
where
    T: From<u64>,
    u64: From<T>,
{
    #[inline(always)]
    pub fn new(value: T) -> Self {
        let value = u64::from(value);
        debug_assert!(value < MAX || value == u64::MAX, "{value} exceeds 40 bits");
        let [a, b, c, d, e, ..] = value.to_le_bytes();
        Self([a, b, c, d, e], PhantomData)
    }

    #[inline(always)]
    pub fn get(self) -> T {
        let [a, b, c, d, e] = self.0;
        let value = u64::from_le_bytes([a, b, c, d, e, 0, 0, 0]);
        T::from(if value == MAX { u64::MAX } else { value })
    }
}

impl<T> From<Index40<T>> for u64
where
    T: From<u64>,
    u64: From<T>,
{
    #[inline(always)]
    fn from(value: Index40<T>) -> Self {
        u64::from(value.get())
    }
}

impl<T> Debug for Index40<T>
where
    T: From<u64> + Debug,
    u64: From<T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl<T> Serialize for Index40<T>
where
    T: From<u64> + Serialize,
    u64: From<T>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Index40<T>
where
    T: From<u64> + Deserialize<'de>,
    u64: From<T>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u64::from(T::deserialize(deserializer)?);
        if value >= MAX && value != u64::MAX {
            return Err(D::Error::custom(format!("{value} exceeds 40 bits")));
        }
        Ok(Self::new(T::from(value)))
    }
}

impl<T: JsonSchema> JsonSchema for Index40<T> {
    fn inline_schema() -> bool {
        T::inline_schema()
    }

    fn schema_name() -> Cow<'static, str> {
        T::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        T::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        T::json_schema(generator)
    }
}

#[cfg(feature = "storage")]
impl<T> Bytes for Index40<T> {
    type Array = [u8; 5];
    // `repr(transparent)` over the stored bytes.
    const IS_NATIVE_LAYOUT: bool = true;

    #[inline(always)]
    fn to_bytes(&self) -> Self::Array {
        self.0
    }

    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        <[u8; 5]>::from_bytes(bytes).map(|bytes| Self(bytes, PhantomData))
    }
}

#[cfg(feature = "storage")]
impl<T> Formattable for Index40<T>
where
    T: From<u64> + Formattable,
    u64: From<T>,
{
    const UNDEFINED: bool = T::UNDEFINED;
    const MISSING: bool = T::MISSING;

    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.get().write_to(buf)
    }

    #[inline(always)]
    fn fmt_csv(&self, f: &mut String) -> fmt::Result {
        self.get().fmt_csv(f)
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        self.get().fmt_json(buf)
    }
}
