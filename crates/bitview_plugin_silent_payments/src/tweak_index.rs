use std::ops::Add;

use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

/// Index of an eligible transaction among all of them, in chain order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Pco, Serialize, JsonSchema)]
pub struct TweakIndex(u32);

impl From<usize> for TweakIndex {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(u32::try_from(value).is_ok(), "{value} doesn't fit TweakIndex");
        Self(value as u32)
    }
}

impl From<TweakIndex> for usize {
    #[inline]
    fn from(value: TweakIndex) -> Self {
        value.0 as usize
    }
}

impl Add<usize> for TweakIndex {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self::from(usize::from(self) + rhs)
    }
}

impl Formattable for TweakIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf)
    }
}

impl PrintableIndex for TweakIndex {
    fn to_string() -> &'static str {
        "tweak_index"
    }
}

impl VecIndex for TweakIndex {}
