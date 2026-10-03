use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Rem},
};

use bitcoin::locktime::absolute::Height as AbsoluteHeight;
use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, StoreValue};

#[cfg(feature = "storage")]
use brk_error::Error as ErrorError;

#[cfg(feature = "storage")]
use std::io::Error as IoError;

#[cfg(feature = "storage")]
use std::path::Path;
#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use std::fs;
#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable, Pco, PrintableIndex, Stamp, VecIndex};

pub const BLOCKS_PER_DIFF_EPOCHS: u32 = 2016;
pub const BLOCKS_PER_HALVING: u32 = 210_000;

/// Block height
#[derive(
    Debug, Clone, Copy, PartialEq, Deref, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Hash,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[allow(clippy::duplicated_attributes)]
#[cfg_attr(
    feature = "schemars",
    schemars(example = 0, example = 210_000, example = 420_000, example = 840_000)
)]
pub struct Height(u32);

impl Height {
    pub const ZERO: Self = Self(0);
    pub const fn new(height: u32) -> Self {
        Self(height)
    }

    #[cfg(feature = "storage")]
    pub fn write(&self, path: &Path) -> Result<(), IoError> {
        fs::write(path, self.to_bytes())
    }

    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn decremented(self) -> Option<Self> {
        CheckedSub::checked_sub(self, 1_u32)
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn left_before_next_diff_adj(self) -> u32 {
        BLOCKS_PER_DIFF_EPOCHS - (*self % BLOCKS_PER_DIFF_EPOCHS)
    }

    pub fn left_before_next_halving(self) -> u32 {
        BLOCKS_PER_HALVING - (*self % BLOCKS_PER_HALVING)
    }
}

impl PartialEq<u64> for Height {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other as u32
    }
}

impl Add<Height> for Height {
    type Output = Self;

    fn add(self, rhs: Height) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl Add<u32> for Height {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self::from(self.0 + rhs)
    }
}

impl Add<u64> for Height {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::from(self.0 + rhs as u32)
    }
}

impl Add<usize> for Height {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u32)
    }
}

impl CheckedSub<Height> for Height {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<Height> for Height {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<u32> for Height {
    fn checked_sub(self, rhs: u32) -> Option<Self> {
        self.0.checked_sub(rhs).map(Height::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<u32> for Height {
    fn checked_sub(self, rhs: u32) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<usize> for Height {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs as u32).map(Height::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for Height {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl AddAssign<usize> for Height {
    fn add_assign(&mut self, rhs: usize) {
        *self = self.add(rhs);
    }
}

impl CheckedSub<u64> for Height {
    fn checked_sub(self, rhs: u64) -> Option<Self> {
        self.0.checked_sub(rhs as u32).map(Height::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<u64> for Height {
    fn checked_sub(self, rhs: u64) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl AddAssign<u64> for Height {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.add(rhs);
    }
}

impl Rem<Height> for Height {
    type Output = Self;
    fn rem(self, rhs: Height) -> Self::Output {
        Self(self.0.rem(rhs.0))
    }
}

impl Rem<usize> for Height {
    type Output = Self;
    fn rem(self, rhs: usize) -> Self::Output {
        Self(self.0.rem(Height::from(rhs).0))
    }
}

impl From<u32> for Height {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<u64> for Height {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}

impl From<usize> for Height {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<Height> for usize {
    #[inline]
    fn from(value: Height) -> Self {
        value.0 as usize
    }
}

impl From<Height> for u32 {
    #[inline]
    fn from(value: Height) -> Self {
        value.0
    }
}
impl From<Height> for u64 {
    #[inline]
    fn from(value: Height) -> Self {
        value.0 as u64
    }
}

impl From<AbsoluteHeight> for Height {
    #[inline]
    fn from(value: AbsoluteHeight) -> Self {
        Self(value.to_consensus_u32())
    }
}

#[cfg(feature = "storage")]
impl TryFrom<&Path> for Height {
    type Error = ErrorError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Ok(Self::from_bytes(fs::read(value)?.as_slice())?.to_owned())
    }
}

impl StoreValue for Height {
    type Bytes = [u8; 4];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

#[cfg(feature = "storage")]
impl From<Stamp> for Height {
    #[inline]
    fn from(value: Stamp) -> Self {
        let u = u64::from(value);
        assert!(u <= u32::MAX as u64);
        Self(u as u32)
    }
}

#[cfg(feature = "storage")]
impl From<Height> for Stamp {
    #[inline]
    fn from(value: Height) -> Self {
        Self::from(value.0 as u64)
    }
}

impl Height {
    pub fn index_name() -> &'static str {
        "height"
    }
    pub fn index_aliases() -> &'static [&'static str] {
        &["h", "height", "blk", "block"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Height {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for Height {
    const INITIAL_CAPACITY: usize = 1_200_000;
}

impl Display for Height {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Height {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
