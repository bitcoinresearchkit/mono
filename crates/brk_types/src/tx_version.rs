use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{Formattable, Pco};

use super::StoredU8;

/// Compact indexed transaction-version category. Values 1, 2, and 3 preserve
/// those exact signed 32-bit Bitcoin transaction versions; 255 represents every
/// other version.
#[derive(
    Debug,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Pco,
    JsonSchema,
)]
pub struct TxVersion(u8);

impl TxVersion {
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);
    pub const THREE: Self = Self(3);
    pub const NON_STANDARD: Self = Self(u8::MAX);
}

impl From<bitcoin::transaction::Version> for TxVersion {
    #[inline]
    fn from(value: bitcoin::transaction::Version) -> Self {
        match value.0 {
            1 => Self::ONE,
            2 => Self::TWO,
            3 => Self::THREE,
            _ => Self::NON_STANDARD,
        }
    }
}

impl From<TxVersion> for bitcoin::transaction::Version {
    #[inline]
    fn from(value: TxVersion) -> Self {
        Self(value.0 as i32)
    }
}

impl From<TxVersion> for StoredU8 {
    #[inline]
    fn from(value: TxVersion) -> Self {
        Self::from(value.0)
    }
}

impl std::fmt::Display for TxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

impl Formattable for TxVersion {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::transaction::Version;

    use super::TxVersion;

    #[test]
    fn groups_signed_raw_versions_without_misclassifying_boundaries() {
        for (raw, expected) in [
            (i32::MIN, TxVersion::NON_STANDARD),
            (-1, TxVersion::NON_STANDARD),
            (0, TxVersion::NON_STANDARD),
            (1, TxVersion::ONE),
            (2, TxVersion::TWO),
            (3, TxVersion::THREE),
            (4, TxVersion::NON_STANDARD),
            (i32::MAX, TxVersion::NON_STANDARD),
        ] {
            assert_eq!(TxVersion::from(Version(raw)), expected, "raw version {raw}");
        }
    }
}
