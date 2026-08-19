use crate::{BoxedIterator, Error, InternalValue, Result, Slice, Table, version::Run};
use std::{ops::RangeBounds, sync::Arc};

/// Reads through a disjoint run.
pub struct RunReader {
    run: Arc<Run<Table>>,
    lo: usize,
    hi: usize,
    lo_reader: Option<BoxedIterator<'static>>,
    hi_reader: Option<BoxedIterator<'static>>,
}

impl RunReader {
    #[must_use]
    pub fn new<R: RangeBounds<Slice> + Clone + Send + 'static>(
        run: Arc<Run<Table>>,
        range: R,
    ) -> Option<Self> {
        let (lo, hi) = run.range_overlap_indexes(&range)?;
        let lo_reader = run.get(lo)?.range(range.clone());
        let hi_reader = if hi > lo {
            Some(run.get(hi)?.range(range))
        } else {
            None
        };

        Some(Self {
            run,
            lo,
            hi,
            lo_reader: Some(BoxedIterator::new(lo_reader)),
            hi_reader: hi_reader.map(BoxedIterator::new),
        })
    }
}

impl Iterator for RunReader {
    type Item = Result<InternalValue>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(reader) = &mut self.lo_reader {
                if let Some(item) = reader.next() {
                    return Some(item);
                }

                self.lo_reader = None;
                self.lo += 1;
                if self.lo < self.hi {
                    let Some(table) = self.run.get(self.lo) else {
                        return Some(Err(Error::Unrecoverable));
                    };
                    self.lo_reader = Some(BoxedIterator::new(table.iter()));
                }
            } else {
                return self.hi_reader.as_mut()?.next();
            }
        }
    }
}

impl DoubleEndedIterator for RunReader {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(reader) = &mut self.hi_reader {
                if let Some(item) = reader.next_back() {
                    return Some(item);
                }

                self.hi_reader = None;
                self.hi -= 1;
                if self.lo < self.hi {
                    let Some(table) = self.run.get(self.hi) else {
                        return Some(Err(Error::Unrecoverable));
                    };
                    self.hi_reader = Some(BoxedIterator::new(table.iter()));
                }
            } else {
                return self.lo_reader.as_mut()?.next_back();
            }
        }
    }
}
