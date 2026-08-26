use rawdb::unlikely;

use crate::ReadableVec;

use super::{MutableRawVec, MutableVec};

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn has_mutations(&self) -> bool {
        !self.current_holes().is_empty() || !self.current_updated().is_empty()
    }

    fn fold_mutable<B, F>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        F: FnMut(B, V::T) -> B,
    {
        let stored_len = self.vec.stored_len();
        let reader = self.vec.reader();
        let mut holes = self.current_holes().range(from..to).peekable();
        let mut updated = self
            .current_updated()
            .range(from..to.min(stored_len))
            .peekable();
        let mut acc = init;

        for index in from..to.min(stored_len) {
            if unlikely(holes.peek() == Some(&&index)) {
                holes.next();
                continue;
            }
            let value = if unlikely(updated.peek().is_some_and(|&(&key, _)| key == index)) {
                updated.next().unwrap().1.clone()
            } else {
                V::read_stored(&reader, index)
            };
            acc = f(acc, value);
        }

        let pushed = self.vec.pushed();
        for index in from.max(stored_len)..to {
            if unlikely(holes.peek() == Some(&&index)) {
                holes.next();
                continue;
            }
            if let Some(value) = pushed.get(index - stored_len) {
                acc = f(acc, value.clone());
            }
        }

        acc
    }

    fn try_fold_mutable<B, E, F>(&self, from: usize, to: usize, init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, V::T) -> Result<B, E>,
    {
        let stored_len = self.vec.stored_len();
        let reader = self.vec.reader();
        let mut holes = self.current_holes().range(from..to).peekable();
        let mut updated = self
            .current_updated()
            .range(from..to.min(stored_len))
            .peekable();
        let mut acc = init;

        for index in from..to.min(stored_len) {
            if unlikely(holes.peek() == Some(&&index)) {
                holes.next();
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
            if let Some(value) = pushed.get(index - stored_len) {
                acc = f(acc, value.clone())?;
            }
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
        if index >= self.vec.len() || self.current_holes().contains(&index) {
            return None;
        }
        self.current_updated()
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
        let len = self.vec.len();
        let from = from.min(len);
        let to = to.min(len);
        buf.reserve(to.saturating_sub(from));
        self.fold_mutable(from, to, (), |(), value| buf.push(value));
    }

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<V::T>) {
        if !self.has_mutations() {
            self.vec.read_sorted_into_at(indices, out);
            return;
        }

        out.reserve(indices.len());
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
    fn fold_range_at<B, F>(&self, from: usize, to: usize, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, V::T) -> B,
    {
        let len = self.vec.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }
        if self.has_mutations() {
            self.fold_mutable(from, to, init, f)
        } else {
            self.vec.fold_range_at(from, to, init, f)
        }
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
