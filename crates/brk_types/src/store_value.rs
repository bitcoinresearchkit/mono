/// Stable fixed-width encoding of a store key or value.
///
/// Numeric keys use big-endian bytes so byte order matches key order.
pub trait StoreValue: Copy {
    type Bytes: Copy + AsRef<[u8]>;

    fn to_store_bytes(&self) -> Self::Bytes;
    fn from_store_bytes(bytes: Self::Bytes) -> Self;
}

/// Implements [`StoreValue`] for a newtype `$t(...)`: a byte array is stored as is, an integer as
/// its big-endian bytes.
#[macro_export]
macro_rules! impl_store_value {
    ($t:ty, [u8; $n:literal]) => {
        impl $crate::StoreValue for $t {
            type Bytes = [u8; $n];

            #[inline]
            fn to_store_bytes(&self) -> Self::Bytes {
                self.0
            }

            #[inline]
            fn from_store_bytes(bytes: Self::Bytes) -> Self {
                Self(bytes)
            }
        }
    };
    ($t:ty, $int:ty) => {
        impl $crate::StoreValue for $t {
            type Bytes = [u8; ::std::mem::size_of::<$int>()];

            #[inline]
            fn to_store_bytes(&self) -> Self::Bytes {
                self.0.to_be_bytes()
            }

            #[inline]
            fn from_store_bytes(bytes: Self::Bytes) -> Self {
                Self(<$int>::from_be_bytes(bytes))
            }
        }
    };
}
