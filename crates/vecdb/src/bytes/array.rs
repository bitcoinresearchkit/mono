use crate::Error;

use super::Bytes;

macro_rules! impl_bytes_for_array {
    ($($n:expr),*) => {
        $(
            impl Bytes for [u8; $n] {
                type Array = [u8; $n];
                const IS_NATIVE_LAYOUT: bool = true;

                #[inline]
                fn to_bytes(&self) -> Self::Array {
                    *self
                }

                #[inline]
                fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
                    bytes.try_into().map_err(|_| Error::WrongLength {
                        expected: $n,
                        received: bytes.len(),
                    })
                }
            }
        )*
    };
}

impl_bytes_for_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 64, 65
);
