use crate::Error;

use super::Bytes;

macro_rules! impl_bytes_for_numeric {
    ($($t:ty),*) => {
        $(
            impl Bytes for $t {
                type Array = [u8; std::mem::size_of::<$t>()];
                const IS_NATIVE_LAYOUT: bool = cfg!(target_endian = "little");

                #[inline]
                fn to_bytes(&self) -> Self::Array {
                    self.to_le_bytes()
                }

                #[inline]
                fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
                    let arr: [u8; std::mem::size_of::<$t>()] = bytes
                        .try_into()
                        .map_err(|_| Error::WrongLength {
                            expected: std::mem::size_of::<$t>(),
                            received: bytes.len(),
                        })?;
                    Ok(<$t>::from_le_bytes(arr))
                }
            }
        )*
    };
}

impl_bytes_for_numeric!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);
