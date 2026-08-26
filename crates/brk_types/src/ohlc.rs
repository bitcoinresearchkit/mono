use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div},
};

use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
};
use vecdb::{Bytes, Formattable, Pco};

use crate::StoredF64;

use super::{Cents, Dollars, Sats};

/// OHLC (Open, High, Low, Close) data in cents
#[derive(Debug, Default, Clone, JsonSchema)]
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

                        Ok(Self::Value {
                            open: Open::new(open),
                            high: High::new(high),
                            low: Low::new(low),
                            close: Close::new(close),
                        })
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

impl Display for OHLCCents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

impl Formattable for OHLCCents {
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

    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }
}

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

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
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

/// OHLC (Open, High, Low, Close) data in dollars
#[derive(Debug, Default, Clone, Copy, JsonSchema)]
#[repr(C)]
pub struct OHLCDollars {
    pub open: Open<Dollars>,
    pub high: High<Dollars>,
    pub low: Low<Dollars>,
    pub close: Close<Dollars>,
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

impl From<(Open<Dollars>, High<Dollars>, Low<Dollars>, Close<Dollars>)> for OHLCDollars {
    #[inline]
    fn from(value: (Open<Dollars>, High<Dollars>, Low<Dollars>, Close<Dollars>)) -> Self {
        Self {
            open: value.0,
            high: value.1,
            low: value.2,
            close: value.3,
        }
    }
}

impl From<Close<Dollars>> for OHLCDollars {
    #[inline]
    fn from(value: Close<Dollars>) -> Self {
        Self {
            open: Open::from(value),
            high: High::from(value),
            low: Low::from(value),
            close: value,
        }
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
            open: value.open.into(),
            high: value.high.into(),
            low: value.low.into(),
            close: value.close.into(),
        }
    }
}

impl Display for OHLCDollars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

impl Formattable for OHLCDollars {
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

    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }
}

impl Bytes for OHLCDollars {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut arr = [0u8; size_of::<Self>()];
        arr[0..8].copy_from_slice(self.open.to_bytes().as_ref());
        arr[8..16].copy_from_slice(self.high.to_bytes().as_ref());
        arr[16..24].copy_from_slice(self.low.to_bytes().as_ref());
        arr[24..32].copy_from_slice(self.close.to_bytes().as_ref());
        arr
    }

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        Ok(Self {
            open: Open::<Dollars>::from_bytes(&bytes[0..8])?,
            high: High::<Dollars>::from_bytes(&bytes[8..16])?,
            low: Low::<Dollars>::from_bytes(&bytes[16..24])?,
            close: Close::<Dollars>::from_bytes(&bytes[24..32])?,
        })
    }
}

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.open, self.high, self.low, self.close
        )
    }
}

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

    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }
}

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

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        Ok(Self {
            open: Open::<Sats>::from_bytes(&bytes[0..8])?,
            high: High::<Sats>::from_bytes(&bytes[8..16])?,
            low: Low::<Sats>::from_bytes(&bytes[16..24])?,
            close: Close::<Sats>::from_bytes(&bytes[24..32])?,
        })
    }
}

/// Opening price value for a time period
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    DerefMut,
    Serialize,
    Pco,
    JsonSchema,
)]
#[schemars(transparent)]
#[repr(transparent)]
pub struct Open<T>(T);

impl<T> Open<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for Open<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for Open<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<Open<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: Open<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Open<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: Open<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for Open<T>
where
    T: Copy,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self(*value)
    }
}

impl From<Open<Cents>> for Open<Dollars> {
    #[inline]
    fn from(value: Open<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for Open<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Open<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for Open<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T> Display for Open<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Formattable for Open<T>
where
    T: Display,
{
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

/// Highest price value for a time period
#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Serialize,
    Pco,
    JsonSchema,
)]
#[schemars(transparent)]
#[repr(transparent)]
pub struct High<T>(T);

impl<T> High<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for High<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for High<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<High<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: High<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<High<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: High<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for High<T>
where
    T: Copy,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self(*value)
    }
}

impl From<High<Cents>> for High<Dollars> {
    #[inline]
    fn from(value: High<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for High<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for High<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for High<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T> Display for High<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Formattable for High<T>
where
    T: Display,
{
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

/// Lowest price value for a time period
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    DerefMut,
    Serialize,
    Pco,
    JsonSchema,
)]
#[schemars(transparent)]
#[repr(transparent)]
pub struct Low<T>(T);

impl<T> Low<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for Low<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for Low<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<Low<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: Low<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Low<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: Low<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for Low<T>
where
    T: Copy,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self(*value)
    }
}

impl From<Low<Cents>> for Low<Dollars> {
    #[inline]
    fn from(value: Low<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for Low<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Low<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for Low<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T> Display for Low<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Formattable for Low<T>
where
    T: Display,
{
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

/// Closing price value for a time period
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    DerefMut,
    Serialize,
    Pco,
    JsonSchema,
)]
#[schemars(transparent)]
#[repr(transparent)]
pub struct Close<T>(T);

impl<T> Close<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<usize> for Close<T>
where
    T: From<usize>,
{
    #[inline]
    fn from(value: usize) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f32> for Close<T>
where
    T: From<f32>,
{
    #[inline]
    fn from(value: f32) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<f64> for Close<T>
where
    T: From<f64>,
{
    #[inline]
    fn from(value: f64) -> Self {
        Self(T::from(value))
    }
}

impl<T> From<Close<T>> for f32
where
    f32: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for f64
where
    f64: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

impl<T> From<Close<T>> for StoredF64
where
    StoredF64: From<T>,
{
    #[inline]
    fn from(value: Close<T>) -> Self {
        Self::from(value.0)
    }
}

// impl<A, B> From<Close<A>> for Close<B>
// where
//     B: From<A>,
// {
// #[inline]
// fn from(value: Close<A>) -> Self {
//         Self(B::from(*value))
impl From<Close<Cents>> for Close<Dollars> {
    #[inline]
    fn from(value: Close<Cents>) -> Self {
        Self(Dollars::from(*value))
    }
}

impl<T> Add for Close<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Close<T>
where
    T: Add<Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        **self = self.0.clone() + rhs.0
    }
}

impl<T> Div<usize> for Close<T>
where
    T: Div<usize, Output = T>,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Sum for Close<Dollars> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(Dollars::from(iter.map(|v| f64::from(v.0)).sum::<f64>()))
    }
}

impl<T> Display for Close<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Formattable for Close<T>
where
    T: Display,
{
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
