use rawdb::{Database, Region, unlikely};

use crate::{Bytes, Error, HEADER_OFFSET, Result};

use super::Page;

/// Manages page metadata for compressed vectors.
///
/// Stores page metadata (offsets, sizes, value counts) separately from the
/// compressed data itself. This allows quick random access to pages without
/// scanning through compressed data.
///
/// Uses incremental flushing to minimize disk writes - only writes changed pages.
#[derive(Debug, Clone)]
pub struct Pages {
    region: Region,
    vec: Vec<Page>,
    /// Index of first changed page, or None if no changes
    change_at: Option<usize>,
}

impl Pages {
    const SIZE_OF_PAGE: usize = size_of::<Page>();

    pub fn import(db: &Database, name: &str) -> Result<Self> {
        let region = db.create_region_if_needed(name)?;

        let vec = region
            .create_reader()
            .read_all()
            .chunks(Self::SIZE_OF_PAGE)
            .map(Page::from_bytes)
            .collect::<Result<_>>()?;

        Ok(Self {
            region,
            vec,
            change_at: None,
        })
    }

    pub fn flush(&mut self) -> Result<()> {
        let Some(change_at) = self.change_at.take() else {
            return Ok(());
        };

        let at = change_at * Self::SIZE_OF_PAGE;
        let pages_to_write = self.vec.len() - change_at;

        let mut bytes = Vec::with_capacity(pages_to_write * Self::SIZE_OF_PAGE);
        for page in &self.vec[change_at..] {
            bytes.extend_from_slice(&page.to_bytes());
        }

        self.region.truncate_write(at, &bytes)?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn get(&self, page_index: usize) -> Option<&Page> {
        self.vec.get(page_index)
    }

    pub fn stored_byte_range(
        &self,
        from: usize,
        to: usize,
        values_per_page: usize,
    ) -> Option<(usize, usize)> {
        if from >= to {
            return None;
        }
        let first = self.get(from / values_per_page)?;
        let last = self.get((to - 1) / values_per_page)?;
        let start = first.start as usize;
        let end = last.end() as usize;
        Some((start, end.checked_sub(start)?))
    }

    pub fn last(&self) -> Option<&Page> {
        self.vec.last()
    }

    /// Pushes a new page, returning an error if the page_index doesn't match the current length.
    pub fn checked_push(&mut self, page_index: usize, page: Page) -> Result<()> {
        if unlikely(page_index != self.vec.len()) {
            return Err(Error::UnexpectedIndex {
                expected: self.vec.len(),
                got: page_index,
                name: self.region.meta().id().to_string(),
            });
        }

        self.set_changed_at(page_index);

        self.vec.push(page);
        Ok(())
    }

    fn set_changed_at(&mut self, page_index: usize) {
        if self.change_at.is_none_or(|pi| pi > page_index) {
            self.change_at.replace(page_index);
        }
    }

    pub fn reset(&mut self) {
        self.truncate(0);
    }

    pub fn truncate(&mut self, page_index: usize) -> Option<Page> {
        let page = self.get(page_index).cloned();
        self.vec.truncate(page_index);
        self.set_changed_at(page_index);
        page
    }

    pub fn next_start(&self) -> u64 {
        self.last().map_or(HEADER_OFFSET as u64, |page| page.end())
    }

    pub fn stored_len(&self, per_page: usize) -> usize {
        if let Some(last) = self.last() {
            (self.len() - 1) * per_page + last.values_count() as usize
        } else {
            0
        }
    }

    pub fn remove(self) -> Result<()> {
        self.region.remove()?;
        Ok(())
    }
}
