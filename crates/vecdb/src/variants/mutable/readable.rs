use crate::{ReadableVec, unlikely};

use super::{MutableRawVec, MutableVec};

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn has_mutations(&self) -> bool {
        !self.holes().is_empty() || !self.updated.current.is_empty()
    }

    fn try_fold_mutable<B, E, F>(&self, from: usize, to: usize, init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, V::T) -> Result<B, E>,
    {
        let stored_len = self.vec.stored_len();
        let stored_to = to.min(stored_len);
        let reader = self.vec.reader();
        let mut holes = self.holes().range(from..to).peekable();
        let mut updated = self
            .updated
            .current
            .range(from.min(stored_to)..stored_to)
            .peekable();
        let mut acc = init;

        for index in from..stored_to {
            if unlikely(holes.peek() == Some(&&index)) {
                holes.next();
                // A rollback restores a hole's backing value as an update: skip it too.
                if updated.peek().is_some_and(|&(&key, _)| key == index) {
                    updated.next();
                }
                continue;
            }
            let value = if unlikely(updated.peek().is_some_and(|&(&key, _)| key == index)) {
                updated.next().unwrap().1.clone()
            } else {
                V::read_stored(&reader, index)
            };
            acc = f(acc, value)?;
        }

        let pushed = self.vec.pushed();
        for index in from.max(stored_len)..to {
            if unlikely(holes.peek() == Some(&&index)) {
                holes.next();
                continue;
            }
            acc = f(acc, pushed[index - stored_len].clone())?;
        }

        Ok(acc)
    }
}

impl<V> ReadableVec<V::I, V::T> for MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        self.vec.cursor_chunk_size()
    }

    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<V::T> {
        if index >= self.vec.len() || self.holes().contains(&index) {
            return None;
        }
        self.updated
            .current
            .get(&index)
            .cloned()
            .or_else(|| self.vec.collect_one_at(index))
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<V::T>) {
        if !self.has_mutations() {
            self.vec.read_into_at(from, to, buf);
            return;
        }
        let to = to.min(self.vec.len());
        if from >= to {
            return;
        }
        buf.reserve(to - from);
        self.fold_range_at(from, to, (), |(), value| buf.push(value));
    }

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<V::T>) {
        if !self.has_mutations() {
            self.vec.read_sorted_into_at(indices, out);
            return;
        }
        let indices = &indices[..indices.partition_point(|&i| i < self.vec.len())];
        let (Some(&first), Some(&last)) = (indices.first(), indices.last()) else {
            return;
        };
        out.reserve(indices.len());
        if last - first < indices.len() && indices.len() > 1 {
            // Merge dense requests with mutations. Sparse requests retain
            // tree lookups rather than walking mutations across large gaps.
            let mut holes = self.holes().range(first..=last).peekable();
            let mut updates = self.updated.current.range(first..=last).peekable();
            let reader = self.vec.reader();
            let stored_len = self.vec.stored_len();
            let pushed = self.vec.pushed();
            for &index in indices {
                while holes.peek().is_some_and(|&&hole| hole < index) {
                    holes.next();
                }
                if holes.peek() == Some(&&index) {
                    continue;
                }
                while updates.peek().is_some_and(|&(&key, _)| key < index) {
                    updates.next();
                }
                let value = if let Some(&(&key, value)) = updates.peek()
                    && key == index
                {
                    value.clone()
                } else if index < stored_len {
                    V::read_stored(&reader, index)
                } else {
                    pushed[index - stored_len].clone()
                };
                out.push(value);
            }
            return;
        }
        for &index in indices {
            if let Some(value) = self.collect_one_at(index) {
                out.push(value);
            }
        }
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(V::T)) {
        self.fold_range_at(from, to, (), |(), value| f(value));
    }

    #[inline]
    fn try_fold_range_at<B, E, F>(&self, from: usize, to: usize, init: B, f: F) -> Result<B, E>
    where
        Self: Sized,
        F: FnMut(B, V::T) -> Result<B, E>,
    {
        let len = self.vec.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }
        if self.has_mutations() {
            self.try_fold_mutable(from, to, init, f)
        } else {
            self.vec.try_fold_range_at(from, to, init, f)
        }
    }
}
