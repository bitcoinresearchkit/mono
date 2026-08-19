use crate::{Bytes, Error};

use super::Page;

impl Bytes for Page {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.start.to_bytes());
        bytes[8..12].copy_from_slice(&self.bytes.to_bytes());
        bytes[12..16].copy_from_slice(&self.values.to_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() < size_of::<Page>() {
            return Err(Error::WrongLength {
                expected: size_of::<Page>(),
                received: bytes.len(),
            });
        }

        let start = u64::from_bytes(&bytes[0..8])?;
        let bytes_val = u32::from_bytes(&bytes[8..12])?;
        let values = u32::from_bytes(&bytes[12..16])?;

        Ok(Self {
            start,
            bytes: bytes_val,
            values,
        })
    }
}
