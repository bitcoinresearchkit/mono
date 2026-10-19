use std::{ops::Deref, sync::Arc};

use brk_types::{Height, TxIndex};
use rangeindex::RangeMap;
use vecdb::VecIndex;

const BUCKET_SHIFT: usize = 16;
const DIRECTORY_BITS: u32 = 20;

/// Block index boundaries with a small directory narrowing floor lookups.
/// The directory indexes the same boundary pages; it stores no separate mapping.
#[derive(Clone)]
pub struct HeightMap<I> {
    ranges: RangeMap<I, Height>,
    // First boundary at or above each bucket; the final entry is ranges.len(). At most 2^20 + 1
    // entries, so an update copies it whole: contiguous, it keeps the hot lookups a load shorter.
    bounds: Arc<[u32]>,
    // The last boundary, inline for the hot tip check; meaningless while empty.
    last: I,
    shift: usize,
}

impl<I: VecIndex> Default for HeightMap<I> {
    fn default() -> Self {
        Self {
            ranges: RangeMap::default(),
            bounds: Arc::new([]),
            last: I::from(0),
            shift: BUCKET_SHIFT,
        }
    }
}

impl HeightMap<TxIndex> {
    pub(crate) fn resume_height(
        &self,
        tx_len: usize,
        target_tx: usize,
        target_height: usize,
    ) -> usize {
        if tx_len >= target_tx {
            target_height
        } else {
            usize::from(self.get(TxIndex::from(tx_len)).unwrap())
        }
    }
}

impl<I: VecIndex> HeightMap<I> {
    #[inline]
    pub fn get(&self, index: I) -> Option<Height> {
        let starts = self.ranges.starts();
        // The first block range starts at index zero, so every index is at or past it.
        if starts.is_empty() {
            return None;
        }
        if index >= self.last {
            return Some(Height::from(starts.len() - 1));
        }
        // Narrow indexes always fit the fixed directory; keep their shift constant.
        let shift = if size_of::<I>() <= size_of::<u32>() {
            BUCKET_SHIFT
        } else {
            self.shift
        };
        let bucket = index.to_usize() >> shift;
        let from = (self.bounds[bucket] as usize).saturating_sub(1);
        let to = self.bounds[bucket + 1] as usize;
        let position = starts.partition_point_in(from..to, |&start| start <= index);
        Some(Height::from(position - 1))
    }

    pub(super) fn update_at(&mut self, from: usize, starts: impl IntoIterator<Item = I>) {
        debug_assert!(from <= self.ranges.len());
        self.ranges.truncate(from);
        let mut bounds = self.bounds.to_vec();
        if let Some(&last) = self.ranges.starts().last() {
            bounds.truncate((last.to_usize() >> self.shift) + 2);
            *bounds.last_mut().unwrap() = from as u32;
        } else {
            bounds.clear();
        }
        self.ranges.extend(starts);
        let len = self.ranges.len();
        let shift = directory_shift(self.ranges.starts().last().copied());
        let from = if shift == self.shift {
            from
        } else {
            self.shift = shift;
            bounds.clear();
            0
        };
        for (start, chunk) in self.ranges.starts().chunks(from..len) {
            for (offset, &index) in chunk.iter().enumerate() {
                extend_bounds(&mut bounds, start + offset, index, shift);
            }
        }
        self.bounds = bounds.into();
        let starts = self.ranges.starts();
        debug_assert!(
            starts.first().is_none_or(|first| first.to_usize() == 0),
            "block ranges start at index zero"
        );
        self.last = starts.last().copied().unwrap_or(I::from(0));
    }
}

impl<I> Deref for HeightMap<I> {
    type Target = RangeMap<I, Height>;

    fn deref(&self) -> &Self::Target {
        &self.ranges
    }
}

fn directory_shift<I: VecIndex>(last: Option<I>) -> usize {
    let bits = last.map_or(0, |index| usize::BITS - index.to_usize().leading_zeros());
    BUCKET_SHIFT.max(bits.saturating_sub(DIRECTORY_BITS) as usize)
}

fn extend_bounds<I: VecIndex>(bounds: &mut Vec<u32>, position: usize, index: I, shift: usize) {
    let bucket = index.to_usize() >> shift;
    let position = u32::try_from(position).expect("height fits u32");
    let end = position.checked_add(1).expect("boundary count fits u32");
    while bounds.len() <= bucket {
        bounds.push(position);
    }
    if bounds.len() == bucket + 1 {
        bounds.push(end);
    } else {
        *bounds.last_mut().unwrap() = end;
    }
}
