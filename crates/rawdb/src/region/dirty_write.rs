use super::inner::RegionInner;

/// Preserves dirty tracking when a batch iterator or write callback panics.
pub(super) struct DirtyWrite<'a> {
    pub(super) region: &'a RegionInner,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl Drop for DirtyWrite<'_> {
    fn drop(&mut self) {
        self.region.mark_dirty(self.start, self.end - self.start);
    }
}
