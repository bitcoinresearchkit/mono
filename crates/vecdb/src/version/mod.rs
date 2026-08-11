use std::{fs, io, path::Path};

mod add;
mod bytes;
mod conversions;
mod display;
mod sum;
mod try_from_path;

use crate::Bytes;

/// Version tracking for data schema and computed values.
///
/// Used to detect when stored data needs to be recomputed due to changes
/// in computation logic or source data versions. Supports validation
/// against persisted versions to ensure compatibility.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[must_use = "Version values should be used for compatibility checks"]
pub struct Version(pub(super) u32);

impl Version {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);

    pub const fn new(v: u32) -> Self {
        Self(v)
    }

    /// Combines ordered version components without component cancellation from addition.
    pub const fn combine(self, other: Self) -> Self {
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

    pub fn swap_bytes(self) -> Self {
        Self(self.0.swap_bytes())
    }
}
