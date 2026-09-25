/// Stable fixed-width encoding of a store key or value.
///
/// Numeric keys use big-endian bytes so byte order matches key order.
pub trait StoreValue: Copy {
    type Bytes: Copy + AsRef<[u8]>;

    fn to_store_bytes(&self) -> Self::Bytes;
    fn from_store_bytes(bytes: Self::Bytes) -> Self;
}
