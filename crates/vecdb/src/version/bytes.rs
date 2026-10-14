use super::Version;
use crate::{Bytes, Result};

// SAFETY: not native layout.
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
