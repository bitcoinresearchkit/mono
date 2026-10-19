use crate::RangeMap;

/// Floor lookups that reuse the last matching interval without shared state.
pub struct RangeMapCursor<'a, I, V> {
    source: &'a RangeMap<I, V>,
    position: Option<usize>,
}

impl<'a, I, V> RangeMapCursor<'a, I, V> {
    pub(super) fn new(source: &'a RangeMap<I, V>) -> Self {
        Self {
            source,
            position: None,
        }
    }
}

impl<I: Ord + Copy, V: From<usize>> RangeMapCursor<'_, I, V> {
    #[inline]
    pub fn get(&mut self, index: I) -> Option<V> {
        let starts = self.source.starts();
        if let Some(position) = self.position
            && starts[position] <= index
            && starts.get(position + 1).is_none_or(|&end| index < end)
        {
            return Some(V::from(position));
        }
        self.position = starts
            .partition_point(|&start| start <= index)
            .checked_sub(1);
        self.position.map(V::from)
    }
}
