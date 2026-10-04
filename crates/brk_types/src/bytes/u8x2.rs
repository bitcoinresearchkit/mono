use derive_more::{Deref, DerefMut};
#[cfg(feature = "storage")]
use vecdb::Bytes;

#[derive(Debug, Clone, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct U8x2([u8; 2]);
impl From<&[u8]> for U8x2 {
    #[inline]
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0; 2];
        arr.copy_from_slice(slice);
        Self(arr)
    }
}
