use std::ops::Range;

/// Sorted, nonempty byte ranges with overlapping and adjacent ranges merged.
#[derive(Debug, Default)]
pub(crate) struct DirtyRanges(Vec<Range<usize>>);

impl DirtyRanges {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Range<usize>> {
        self.0.iter()
    }

    pub(crate) fn insert(&mut self, mut range: Range<usize>) {
        if range.is_empty() {
            return;
        }

        // Appends and address-ordered flush collection only touch the last range.
        if let Some(last) = self.0.last_mut()
            && range.start >= last.start
        {
            if range.start <= last.end {
                last.end = last.end.max(range.end);
            } else {
                self.0.push(range);
            }
            return;
        }

        let first = self
            .0
            .partition_point(|existing| existing.end < range.start);
        let mut end = first;
        while end < self.0.len() && self.0[end].start <= range.end {
            range.start = range.start.min(self.0[end].start);
            range.end = range.end.max(self.0[end].end);
            end += 1;
        }
        if first == end {
            self.0.insert(first, range);
        } else {
            self.0[first] = range;
            // Shift the untouched suffix once, rather than once per merged range.
            self.0.drain(first + 1..end);
        }
    }
}

impl Extend<Range<usize>> for DirtyRanges {
    fn extend<T: IntoIterator<Item = Range<usize>>>(&mut self, ranges: T) {
        for range in ranges {
            self.insert(range);
        }
    }
}
