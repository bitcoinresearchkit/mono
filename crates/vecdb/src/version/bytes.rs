use crate::Bytes;

use super::Version;

impl Bytes for Version {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        self.0.to_bytes()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        Ok(Self(u32::from_bytes(bytes)?))
    }
}
