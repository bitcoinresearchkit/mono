use std::ops::Deref;

use brk_types::{Height, TxIndex};
use rangeindex::RangeMap;
use vecdb::VecIndex;

const BUCKET_SHIFT: usize = 16;
const DIRECTORY_BITS: u32 = 20;

/// Block index boundaries with a small directory narrowing floor lookups.
/// The directory indexes the same boundary array; it stores no separate mapping.
pub struct HeightMap<I> {
    ranges: RangeMap<I, Height>,
    // First boundary at or above each bucket; the final entry is ranges.len().
    bounds: Vec<u32>,
    shift: usize,
}

impl<I: VecIndex> From<Vec<I>> for HeightMap<I> {
    fn from(starts: Vec<I>) -> Self {
        let shift = directory_shift(starts.last().copied());
        let capacity = starts
            .last()
            .map_or(0, |&index| (index.to_usize() >> shift) + 2);
        let mut bounds = Vec::with_capacity(capacity);
        for (position, &index) in starts.iter().enumerate() {
            extend_bounds(&mut bounds, position, index, shift);
        }
        Self {
            ranges: RangeMap::from(starts),
            bounds,
            shift,
        }
    }
}

impl HeightMap<TxIndex> {
    pub fn resume_height(&self, tx_len: usize, target_tx: usize, target_height: usize) -> usize {
        if tx_len >= target_tx {
            target_height
        } else {
            usize::from(self.get_shared(TxIndex::from(tx_len)).unwrap())
        }
    }
}

impl<I: VecIndex> HeightMap<I> {
    #[inline]
    pub fn get_shared(&self, index: I) -> Option<Height> {
        let starts = self.ranges.as_slice();
        if index < *starts.first()? {
            return None;
        }
        if index >= *starts.last().unwrap() {
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
        let position = starts[from..to].partition_point(|&start| start <= index);
        Some(Height::from(from + position - 1))
    }

    pub(super) fn update_at(&mut self, from: usize, starts: impl IntoIterator<Item = I>) {
        assert!(from <= self.ranges.len());
        self.ranges.truncate(from);
        if let Some(&last) = self.ranges.as_slice().last() {
            self.bounds.truncate((last.to_usize() >> self.shift) + 2);
            *self.bounds.last_mut().unwrap() = from as u32;
        } else {
            self.bounds.clear();
        }
        self.ranges.extend(starts);
        let shift = directory_shift(self.ranges.as_slice().last().copied());
        if shift != self.shift {
            self.shift = shift;
            self.bounds.clear();
            for (position, &index) in self.ranges.as_slice().iter().enumerate() {
                extend_bounds(&mut self.bounds, position, index, shift);
            }
            return;
        }
        for (offset, &index) in self.ranges.as_slice()[from..].iter().enumerate() {
            extend_bounds(&mut self.bounds, from + offset, index, self.shift);
        }
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

#[cfg(test)]
mod tests;
