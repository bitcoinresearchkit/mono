use vecdb::Version;

/// Window state a rolling kernel keeps between updates, so an update continues from the
/// previous one instead of rebuilding its window from history.
///
/// Held by the writable family as `M::WriteOnly<Retained<S>>`: readers never see it and it is
/// never persisted. A kernel takes it back only to continue at exactly the row it was kept for,
/// under the same dependency version; anything else (restart, rewind, version change) takes
/// the rebuild path, whose values the continued state matches bit for bit.
pub struct Retained<S>(Option<(usize, Version, S)>);

impl<S> Default for Retained<S> {
    fn default() -> Self {
        Self(None)
    }
}

impl<S> Retained<S> {
    /// The kept state if it continues at row `next` under `version`; it is consumed either way.
    pub(crate) fn take(&mut self, next: usize, version: Version) -> Option<S> {
        self.0
            .take()
            .and_then(|(at, kept, state)| (at == next && kept == version).then_some(state))
    }

    pub(crate) fn keep(&mut self, next: usize, version: Version, state: S) {
        self.0 = Some((next, version, state));
    }
}
