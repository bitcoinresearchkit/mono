use super::inner::RegionInner;

/// Marks the batch's validated span dirty, including after an iterator/callback panic.
pub(super) struct DirtyWrite<'a> {
    pub(super) region: &'a RegionInner,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl Drop for DirtyWrite<'_> {
    fn drop(&mut self) {
        // SAFETY: both batch writers drop this before their access/barrier guards.
        unsafe { self.region.mark_dirty(self.start, self.end - self.start) };
    }
}
