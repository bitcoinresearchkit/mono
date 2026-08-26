/// Metadata for a page in a CompressedVec.
///
/// Compressed pages may share one compression header while remaining
/// independently decodable. Raw pages are used only for the final partial page.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub header_start: u64,
    pub header_end: u64,
    pub start: u64,
    pub bytes: u32,
    pub chunk_start_page: usize,
    pub raw: bool,
}

impl Page {
    #[inline]
    pub fn is_raw(&self) -> bool {
        self.raw
    }

    #[inline]
    pub fn values_count(&self, values_per_page: usize, value_size: usize) -> usize {
        if self.raw {
            self.bytes as usize / value_size
        } else {
            values_per_page
        }
    }

    #[inline]
    pub fn header_len(&self) -> usize {
        (self.header_end - self.header_start) as usize
    }

    #[inline]
    pub fn end(&self) -> u64 {
        self.start + self.bytes as u64
    }
}
