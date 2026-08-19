use crate::Bytes;

use super::Stamp;

impl Bytes for Stamp {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        self.0.to_bytes()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        Ok(Self(u64::from_bytes(bytes)?))
    }
}
