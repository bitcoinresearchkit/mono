use super::Format;
use crate::{Bytes, Error, Result};

// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for Format {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let len = bytes.len();
        if len != size_of::<Self>() {
            return Err(Error::WrongLength {
                expected: size_of::<Self>(),
                received: len,
            });
        }

        match bytes[0] {
            0 => Ok(Self::Bytes),
            1 => Ok(Self::ZeroCopy),
            64 => Ok(Self::Pco),
            65 => Ok(Self::LZ4),
            66 => Ok(Self::Zstd),
            b => Err(Error::InvalidFormat(b)),
        }
    }
}
