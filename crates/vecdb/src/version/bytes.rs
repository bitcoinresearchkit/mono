use super::Version;
use crate::{Bytes, Result};

// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for Version {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        self.0.to_bytes()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(u32::from_bytes(bytes)?))
    }
}
