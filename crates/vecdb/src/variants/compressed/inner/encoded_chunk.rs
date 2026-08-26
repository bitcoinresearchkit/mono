use crate::{Error, Result};

/// One compression context containing a shared header and independently decodable pages.
#[derive(Debug)]
pub struct EncodedChunk {
    bytes: Vec<u8>,
    header_len: u32,
    page_ends: Vec<u32>,
}

impl EncodedChunk {
    pub fn new(bytes: Vec<u8>, header_len: usize, page_ends: Vec<u32>) -> Result<Self> {
        let header_len = u32::try_from(header_len).map_err(|_| Error::Overflow)?;
        let bytes_len = u32::try_from(bytes.len()).map_err(|_| Error::Overflow)?;
        if page_ends.is_empty()
            || header_len > bytes_len
            || page_ends.last().copied() != Some(bytes_len)
            || page_ends.windows(2).any(|ends| ends[0] >= ends[1])
            || page_ends[0] <= header_len
        {
            return Err(Error::InvalidArgument("invalid encoded chunk layout"));
        }

        Ok(Self {
            bytes,
            header_len,
            page_ends,
        })
    }

    #[inline]
    pub fn single_page(bytes: Vec<u8>) -> Result<Self> {
        let end = u32::try_from(bytes.len()).map_err(|_| Error::Overflow)?;
        Self::new(bytes, 0, vec![end])
    }

    #[inline]
    pub fn page_count(&self) -> usize {
        self.page_ends.len()
    }

    #[inline]
    pub fn into_parts(self) -> (Vec<u8>, u32, Vec<u32>) {
        (self.bytes, self.header_len, self.page_ends)
    }
}
