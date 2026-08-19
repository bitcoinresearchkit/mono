use crate::{InternalValue, Result};

/// Type-erased double-ended iterator over internal table values.
pub struct BoxedIterator<'a>(
    Box<dyn DoubleEndedIterator<Item = Result<InternalValue>> + Send + 'a>,
);

impl<'a> BoxedIterator<'a> {
    pub fn new(
        iterator: impl DoubleEndedIterator<Item = Result<InternalValue>> + Send + 'a,
    ) -> Self {
        Self(Box::new(iterator))
    }
}

impl Iterator for BoxedIterator<'_> {
    type Item = Result<InternalValue>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl DoubleEndedIterator for BoxedIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}
