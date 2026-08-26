use rawdb::{Database, Region, unlikely};

use crate::{Bytes, Error, HEADER_OFFSET};

use super::Page;

pub const PAGES_PER_BLOCK: usize = 16;
const BLOCK_BASE_BYTES: usize = size_of::<u64>();
const RECORD_BYTES: usize = size_of::<u32>() * 2;
const BLOCK_BYTES: usize = BLOCK_BASE_BYTES + PAGES_PER_BLOCK * RECORD_BYTES;

#[derive(Debug, Clone, Copy, Default)]
struct PageRecord {
    span: u32,
    body_and_flags: u32,
}

impl PageRecord {
    const RAW_FLAG: u32 = 1 << 31;
    const CHUNK_START_FLAG: u32 = 1 << 30;
    const BODY_MASK: u32 = Self::CHUNK_START_FLAG - 1;

    fn new(span: u32, body: u32, chunk_start: bool, raw: bool) -> crate::Result<Self> {
        if body == 0 || body > Self::BODY_MASK || span < body {
            return Err(Error::InvalidArgument("invalid compressed page size"));
        }
        let mut body_and_flags = body;
        if chunk_start {
            body_and_flags |= Self::CHUNK_START_FLAG;
        }
        if raw {
            body_and_flags |= Self::RAW_FLAG;
        }
        Ok(Self {
            span,
            body_and_flags,
        })
    }

    #[inline]
    fn body(self) -> u32 {
        self.body_and_flags & Self::BODY_MASK
    }

    #[inline]
    fn is_chunk_start(self) -> bool {
        self.body_and_flags & Self::CHUNK_START_FLAG != 0
    }

    #[inline]
    fn is_raw(self) -> bool {
        self.body_and_flags & Self::RAW_FLAG != 0
    }
}

#[derive(Debug, Clone)]
struct PageBlock {
    base: u64,
    records: [PageRecord; PAGES_PER_BLOCK],
}

impl PageBlock {
    fn new(base: u64) -> Self {
        Self {
            base,
            records: [PageRecord::default(); PAGES_PER_BLOCK],
        }
    }
}

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
    blocks: Vec<PageBlock>,
    len: usize,
    /// Index of the first changed metadata block.
    change_at: Option<usize>,
}

impl Pages {
    pub fn import(db: &Database, name: &str, page_capacity: usize) -> crate::Result<Self> {
        let region = db.create_region_if_needed(name)?;
        if page_capacity > 0 {
            let full_blocks = page_capacity / PAGES_PER_BLOCK;
            let remaining = page_capacity % PAGES_PER_BLOCK;
            let capacity = full_blocks
                .checked_mul(BLOCK_BYTES)
                .and_then(|bytes| {
                    bytes.checked_add(if remaining == 0 {
                        0
                    } else {
                        BLOCK_BASE_BYTES + remaining * RECORD_BYTES
                    })
                })
                .ok_or(Error::Overflow)?;
            region.reserve_capacity(capacity)?;
        }
        let reader = region.create_reader();
        let bytes = reader.read_all();
        let mut blocks = Vec::with_capacity(bytes.len().div_ceil(BLOCK_BYTES));
        let mut len = 0;
        let mut offset = 0;
        let mut raw_page = None;

        while offset < bytes.len() {
            let remaining = bytes.len() - offset;
            if remaining < BLOCK_BASE_BYTES + RECORD_BYTES {
                return Err(Error::WrongLength {
                    expected: BLOCK_BASE_BYTES + RECORD_BYTES,
                    received: remaining,
                });
            }
            let record_count = if remaining >= BLOCK_BYTES {
                PAGES_PER_BLOCK
            } else {
                let records_bytes = remaining - BLOCK_BASE_BYTES;
                if !records_bytes.is_multiple_of(RECORD_BYTES) {
                    return Err(Error::WrongLength {
                        expected: RECORD_BYTES,
                        received: records_bytes % RECORD_BYTES,
                    });
                }
                records_bytes / RECORD_BYTES
            };

            let base = u64::from_bytes(&bytes[offset..offset + BLOCK_BASE_BYTES])?;
            let mut block = PageBlock::new(base);
            offset += BLOCK_BASE_BYTES;
            for local in 0..record_count {
                let span = u32::from_bytes(&bytes[offset..offset + 4])?;
                let body_and_flags = u32::from_bytes(&bytes[offset + 4..offset + 8])?;
                let record = PageRecord {
                    span,
                    body_and_flags,
                };
                if record.body() == 0
                    || record.span < record.body()
                    || (local == 0 && !record.is_chunk_start())
                    || (record.is_raw() && !record.is_chunk_start())
                    || (record.is_raw() && (raw_page.is_some() || record.span != record.body()))
                    || (!record.is_chunk_start() && record.span != record.body())
                {
                    return Err(Error::CorruptedRegion {
                        name: name.to_string(),
                        region_len: bytes.len(),
                    });
                }
                if record.is_raw() {
                    raw_page = Some(len + local);
                }
                block.records[local] = record;
                offset += RECORD_BYTES;
            }
            len += record_count;
            blocks.push(block);
        }

        let this = Self {
            region,
            blocks,
            len,
            change_at: None,
        };
        if raw_page.is_some_and(|page| page + 1 != len)
            || this
                .blocks
                .first()
                .is_some_and(|block| block.base != HEADER_OFFSET as u64)
            || this
                .blocks
                .windows(2)
                .any(|blocks| blocks[1].base != Self::block_end(&blocks[0], PAGES_PER_BLOCK))
        {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len: bytes.len(),
            });
        }
        Ok(this)
    }

    pub fn flush(&mut self) -> crate::Result<()> {
        let Some(change_at) = self.change_at else {
            return Ok(());
        };

        let at = change_at * BLOCK_BYTES;
        let mut bytes =
            Vec::with_capacity(self.blocks.len().saturating_sub(change_at) * BLOCK_BYTES);
        for (block_index, block) in self.blocks.iter().enumerate().skip(change_at) {
            bytes.extend_from_slice(&block.base.to_bytes());
            let record_count = self
                .len
                .saturating_sub(block_index * PAGES_PER_BLOCK)
                .min(PAGES_PER_BLOCK);
            for record in &block.records[..record_count] {
                bytes.extend_from_slice(&record.span.to_bytes());
                bytes.extend_from_slice(&record.body_and_flags.to_bytes());
            }
        }

        self.region.truncate_write(at, &bytes)?;
        self.change_at = None;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, page_index: usize) -> Option<Page> {
        if page_index >= self.len {
            return None;
        }
        let block_index = page_index / PAGES_PER_BLOCK;
        let local_index = page_index % PAGES_PER_BLOCK;
        let block = self.blocks.get(block_index)?;
        let mut end = block.base;
        let mut header_start = block.base;
        let mut header_end = block.base;
        let mut chunk_start_page = block_index * PAGES_PER_BLOCK;

        for local in 0..=local_index {
            let record = block.records[local];
            let previous_end = end;
            end = end.checked_add(record.span as u64)?;
            let start = end.checked_sub(record.body() as u64)?;
            if record.is_chunk_start() {
                header_start = previous_end;
                header_end = start;
                chunk_start_page = block_index * PAGES_PER_BLOCK + local;
            }
            if local == local_index {
                return Some(Page {
                    header_start,
                    header_end,
                    start,
                    bytes: record.body(),
                    chunk_start_page,
                    raw: record.is_raw(),
                });
            }
        }
        None
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
        let start = first.header_start as usize;
        let end = last.end() as usize;
        Some((start, end.checked_sub(start)?))
    }

    pub fn last(&self) -> Option<Page> {
        self.len.checked_sub(1).and_then(|index| self.get(index))
    }

    pub fn push_compressed(
        &mut self,
        page_index: usize,
        header_bytes: Option<u32>,
        body_bytes: u32,
    ) -> crate::Result<()> {
        self.checked_push(page_index, header_bytes, body_bytes, false)
    }

    pub fn push_raw(&mut self, page_index: usize, body_bytes: u32) -> crate::Result<()> {
        self.checked_push(page_index, Some(0), body_bytes, true)
    }

    fn checked_push(
        &mut self,
        page_index: usize,
        header_bytes: Option<u32>,
        body_bytes: u32,
        raw: bool,
    ) -> crate::Result<()> {
        if unlikely(page_index != self.len) {
            return Err(Error::UnexpectedIndex {
                expected: self.len,
                got: page_index,
                name: self.region.meta().id().to_string(),
            });
        }
        let block_index = page_index / PAGES_PER_BLOCK;
        let local_index = page_index % PAGES_PER_BLOCK;
        if local_index == 0 && header_bytes.is_none() {
            return Err(Error::InvalidArgument(
                "compression chunks cannot cross page-table blocks",
            ));
        }
        if local_index == 0 {
            self.blocks.push(PageBlock::new(self.next_start()));
        }
        let chunk_start = header_bytes.is_some();
        let header_bytes = header_bytes.unwrap_or_default();
        let span = header_bytes
            .checked_add(body_bytes)
            .ok_or(Error::Overflow)?;
        let record = PageRecord::new(span, body_bytes, chunk_start, raw)?;
        self.blocks[block_index].records[local_index] = record;
        self.len += 1;
        self.set_changed_at(block_index);
        Ok(())
    }

    fn set_changed_at(&mut self, block_index: usize) {
        if self.change_at.is_none_or(|index| index > block_index) {
            self.change_at.replace(block_index);
        }
    }

    pub fn reset(&mut self) {
        self.truncate(0);
    }

    pub fn truncate(&mut self, page_index: usize) -> Option<Page> {
        let page = self.get(page_index);
        if page_index >= self.len {
            return page;
        }
        self.len = page_index;
        self.blocks.truncate(page_index.div_ceil(PAGES_PER_BLOCK));
        self.set_changed_at(page_index / PAGES_PER_BLOCK);
        page
    }

    pub fn next_start(&self) -> u64 {
        self.last().map_or(HEADER_OFFSET as u64, |page| page.end())
    }

    pub fn stored_len(&self, per_page: usize, value_size: usize) -> usize {
        if let Some(last) = self.last() {
            let last_values = last.values_count(per_page, value_size);
            (self.len() - 1) * per_page + last_values
        } else {
            0
        }
    }

    #[inline]
    fn block_end(block: &PageBlock, record_count: usize) -> u64 {
        block.records[..record_count]
            .iter()
            .fold(block.base, |end, record| end + record.span as u64)
    }

    pub fn remove(self) -> crate::Result<()> {
        self.region.remove()?;
        Ok(())
    }
}
