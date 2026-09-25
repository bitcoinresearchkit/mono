use crate::{InternalValue, Result, Slice};

/// Type-erased double-ended iterator over internal table values.
pub struct BoxedIterator<'a, K = Slice, V = Slice>(
    Box<dyn DoubleEndedIterator<Item = Result<InternalValue<K, V>>> + Send + 'a>,
);

impl<'a, K, V> BoxedIterator<'a, K, V> {
    pub fn new(
        iterator: impl DoubleEndedIterator<Item = Result<InternalValue<K, V>>> + Send + 'a,
    ) -> Self {
        Self(Box::new(iterator))
    }
}

impl<K, V> Iterator for BoxedIterator<'_, K, V> {
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K, V> DoubleEndedIterator for BoxedIterator<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}
