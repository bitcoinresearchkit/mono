use std::{
    fmt::Debug,
    fs,
    ops::{Add, AddAssign, Rem},
};

use byteview::ByteView;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{Bytes, CheckedSub, Formattable, Pco, PrintableIndex, Stamp, VecIndex};

use crate::{BLOCKS_PER_DIFF_EPOCHS, BLOCKS_PER_HALVING, FromCoarserIndex};

use super::{Epoch, Halving, StoredU64};

/// Block height
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Deref,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Pco,
    JsonSchema,
    Hash,
)]
#[allow(clippy::duplicated_attributes)]
#[schemars(example = 0, example = 210_000, example = 420_000, example = 840_000)]
pub struct Height(u32);

impl Height {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u32::MAX);

    pub const fn new(height: u32) -> Self {
        Self(height)
    }

    pub fn write(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        fs::write(path, self.to_bytes())
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn decrement(&mut self) {
        *self = self.decremented().unwrap();
    }

    pub fn decremented(self) -> Option<Self> {
        self.checked_sub(1_u32)
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn is_not_zero(self) -> bool {
        self != Self::ZERO
    }

    pub fn is_deeply_confirmed(self, current_height: Self) -> bool {
        (*current_height).saturating_sub(*self) > 6
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

impl CheckedSub<u32> for Height {
    fn checked_sub(self, rhs: u32) -> Option<Self> {
        self.0.checked_sub(rhs).map(Height::from)
    }
}

impl CheckedSub<usize> for Height {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs as u32).map(Height::from)
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

impl From<StoredU64> for Height {
    #[inline]
    fn from(value: StoredU64) -> Self {
        Self(*value as u32)
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

impl From<bitcoin::locktime::absolute::Height> for Height {
    #[inline]
    fn from(value: bitcoin::locktime::absolute::Height) -> Self {
        Self(value.to_consensus_u32())
    }
}

impl TryFrom<&std::path::Path> for Height {
    type Error = brk_error::Error;
    fn try_from(value: &std::path::Path) -> Result<Self, Self::Error> {
        Ok(Self::from_bytes(std::fs::read(value)?.as_slice())?.to_owned())
    }
}

impl From<ByteView> for Height {
    #[inline(always)]
    fn from(value: ByteView) -> Self {
        Self(u32::from_be_bytes((&*value).try_into().unwrap()))
    }
}

impl From<Height> for ByteView {
    #[inline(always)]
    fn from(value: Height) -> Self {
        ByteView::from(&value)
    }
}

impl From<&Height> for ByteView {
    #[inline(always)]
    fn from(value: &Height) -> Self {
        Self::new(&value.0.to_be_bytes())
    }
}

impl From<Stamp> for Height {
    #[inline]
    fn from(value: Stamp) -> Self {
        let u = u64::from(value);
        assert!(u <= u32::MAX as u64);
        Self(u as u32)
    }
}

impl From<Height> for Stamp {
    #[inline]
    fn from(value: Height) -> Self {
        Self::from(value.0 as u64)
    }
}

impl PrintableIndex for Height {
    fn to_string() -> &'static str {
        "height"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["h", "height", "blk", "block"]
    }
}

impl VecIndex for Height {
    const INITIAL_CAPACITY: usize = 1_200_000;
}

impl std::fmt::Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

impl Formattable for Height {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

impl FromCoarserIndex<Epoch> for Height {
    fn min_from(coarser: Epoch) -> usize {
        usize::from(coarser) * BLOCKS_PER_DIFF_EPOCHS as usize
    }

    fn max_from_(coarser: Epoch) -> usize {
        (usize::from(coarser) + 1) * BLOCKS_PER_DIFF_EPOCHS as usize - 1
    }
}

impl FromCoarserIndex<Halving> for Height {
    fn min_from(coarser: Halving) -> usize {
        usize::from(coarser) * BLOCKS_PER_HALVING as usize
    }

    fn max_from_(coarser: Halving) -> usize {
        (usize::from(coarser) + 1) * BLOCKS_PER_HALVING as usize - 1
    }
}
