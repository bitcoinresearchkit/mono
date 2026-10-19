use std::{
    iter,
    ops::{Index, Range},
    sync::Arc,
};

const PAGE_BITS: u32 = 12;
const PAGE_LEN: usize = 1 << PAGE_BITS;
const PAGE_MASK: usize = PAGE_LEN - 1;

/// A sequence stored in shared pages of 4,096 elements. Clones share every page and a write
/// copies only the pages it changes, so a published clone stays immutable for its readers
/// while the writer extends or truncates its own.
pub struct Pages<T> {
    // Never holds an empty page; every page but the last is full.
    pages: Vec<Arc<Vec<T>>>,
    len: usize,
}

impl<T> Clone for Pages<T> {
    fn clone(&self) -> Self {
        Self {
            pages: self.pages.clone(),
            len: self.len,
        }
    }
}

impl<T> Default for Pages<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Pages<T> {
    pub const fn new() -> Self {
        Self {
            pages: Vec::new(),
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.pages
            .get(index >> PAGE_BITS)
            .and_then(|page| page.get(index & PAGE_MASK))
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.pages.first().and_then(|page| page.first())
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.pages.last().and_then(|page| page.last())
    }

    /// Contiguous slices covering `range` (clamped to the length), each with its first index.
    pub fn chunks(&self, range: Range<usize>) -> impl Iterator<Item = (usize, &[T])> {
        let end = range.end.min(self.len);
        let mut at = range.start.min(end);
        iter::from_fn(move || {
            (at < end).then(|| {
                let offset = at & PAGE_MASK;
                let count = (PAGE_LEN - offset).min(end - at);
                let chunk = (at, &self.pages[at >> PAGE_BITS][offset..offset + count]);
                at += count;
                chunk
            })
        })
    }

    /// The elements in `range` (clamped to the length).
    pub fn iter(&self, range: Range<usize>) -> impl Iterator<Item = &T> {
        self.chunks(range).flat_map(|(_, chunk)| chunk)
    }

    /// The first index whose element fails `pred`, which must hold for a prefix only.
    pub fn partition_point(&self, mut pred: impl FnMut(&T) -> bool) -> usize {
        let page = self
            .pages
            .partition_point(|page| pred(page.last().unwrap()));
        match self.pages.get(page) {
            Some(values) => (page << PAGE_BITS) + values.partition_point(pred),
            None => self.len,
        }
    }

    /// [`Self::partition_point`] within `range`, which must lie within the length: the
    /// absolute index, `range.end` when every element in it holds.
    pub fn partition_point_in(
        &self,
        range: Range<usize>,
        mut pred: impl FnMut(&T) -> bool,
    ) -> usize {
        debug_assert!(range.start <= range.end && range.end <= self.len);
        let Range { mut start, end } = range;
        while start < end {
            let page = start >> PAGE_BITS;
            let page_start = page << PAGE_BITS;
            let chunk_end = (page_start + PAGE_LEN).min(end);
            let chunk = &self.pages[page][start - page_start..chunk_end - page_start];
            if !pred(chunk.last().unwrap()) {
                return start + chunk.partition_point(pred);
            }
            start = chunk_end;
        }
        end
    }
}

impl<T: Clone> Pages<T> {
    pub fn push(&mut self, value: T) {
        if self.len & PAGE_MASK == 0 {
            self.pages.push(Arc::default());
        }
        unshare(self.pages.last_mut().unwrap()).push(value);
        self.len += 1;
    }

    pub fn truncate(&mut self, len: usize) {
        if len >= self.len {
            return;
        }
        self.pages.truncate(len.div_ceil(PAGE_LEN));
        let kept = len & PAGE_MASK;
        if kept != 0 {
            let page = self.pages.last_mut().unwrap();
            match Arc::get_mut(page) {
                Some(page) => page.truncate(kept),
                None => *page = copy(&page[..kept]),
            }
        }
        self.len = len;
    }
}

impl<T> Index<usize> for Pages<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.pages[index >> PAGE_BITS][index & PAGE_MASK]
    }
}

/// The page itself, copied first while a clone still shares it.
fn unshare<T: Clone>(page: &mut Arc<Vec<T>>) -> &mut Vec<T> {
    if Arc::get_mut(page).is_none() {
        *page = copy(page);
    }
    Arc::get_mut(page).unwrap()
}

/// Room for the next push; power-of-two capacities keep doubling growth within one page.
fn copy<T: Clone>(values: &[T]) -> Arc<Vec<T>> {
    let mut page = Vec::with_capacity((values.len() + 1).next_power_of_two().min(PAGE_LEN));
    page.extend_from_slice(values);
    Arc::new(page)
}
