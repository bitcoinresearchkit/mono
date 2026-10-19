use std::{fs, io, path::Path};

pub mod add;
pub mod bytes;
pub mod conversions;
pub mod display;
pub mod sum;
pub mod try_from_path;

use crate::Bytes;

/// Schema or computation version of a stored vector: when it changes, the data is reset and
/// recomputed.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[must_use]
pub struct Version(u32);

impl Version {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);

    pub const fn new(v: u32) -> Self {
        Self(v)
    }

    /// Combines ordered version components without component cancellation from addition.
    pub(crate) const fn combine(self, other: Self) -> Self {
        Self(
            self.0
                ^ other
                    .0
                    .wrapping_add(0x9e37_79b9)
                    .wrapping_add(self.0 << 6)
                    .wrapping_add(self.0 >> 2),
        )
    }

    #[inline]
    pub fn combine_all(versions: impl IntoIterator<Item = Self>) -> Self {
        versions.into_iter().fold(Self::ZERO, Self::combine)
    }

    pub fn write(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, self.to_bytes().as_ref())
    }
}
