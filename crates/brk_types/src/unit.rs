use crate::StoreValue;

#[derive(Debug, Clone, Copy)]
pub struct Unit;

impl StoreValue for Unit {
    type Bytes = [u8; 0];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        []
    }

    #[inline]
    fn from_store_bytes(_bytes: Self::Bytes) -> Self {
        Self
    }
}
