use std::sync::Arc;

use parking_lot::RwLock;
use rawdb::{Reader, Region};

use super::{
    super::inner::{CompressionStrategy, Page},
    CompressedSource, PageBytes,
};
use crate::{Pages, VecValue};

/// Page bytes read in place from the mapping.
pub struct Mapped(Reader);

impl PageBytes for Mapped {
    #[inline(always)]
    fn page(&mut self, _: &Pages, _: usize, page: Page, _: usize) -> Option<(&[u8], &[u8])> {
        Some((
            self.0.read(page.header_start as usize, page.header_len()),
            self.0.read(page.start as usize, page.bytes as usize),
        ))
    }
}

/// Mmap-backed [`CompressedSource`].
pub type CompressedMmapSource<'a, I, T, S> = CompressedSource<'a, I, T, S, Mapped>;

impl<'a, I, T, S> CompressedMmapSource<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    pub fn new_from_parts(
        region: &Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        Self::with_bytes(
            pages,
            || Mapped(region.create_reader()),
            stored_len,
            from,
            to,
        )
    }
}
