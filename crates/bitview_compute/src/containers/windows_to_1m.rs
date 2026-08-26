use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct WindowsTo1m<A> {
    /// Uses a 1-day base interval.
    pub _24h: A,
    /// Uses a 7-day base interval.
    pub _1w: A,
    /// Uses a 30-day base interval.
    pub _1m: A,
}

impl<A> WindowsTo1m<A> {
    pub const SUFFIXES: [&'static str; 3] = ["24h", "1w", "1m"];

    pub fn try_from_fn<E>(
        mut f: impl FnMut(&str) -> std::result::Result<A, E>,
    ) -> std::result::Result<Self, E> {
        Ok(Self {
            _24h: f(Self::SUFFIXES[0])?,
            _1w: f(Self::SUFFIXES[1])?,
            _1m: f(Self::SUFFIXES[2])?,
        })
    }

    pub fn as_array(&self) -> [&A; 3] {
        [&self._24h, &self._1w, &self._1m]
    }

    pub fn as_mut_array(&mut self) -> [&mut A; 3] {
        [&mut self._24h, &mut self._1w, &mut self._1m]
    }

    pub fn as_mut_array_with_days(&mut self) -> [(&mut A, usize); 3] {
        [(&mut self._24h, 1), (&mut self._1w, 7), (&mut self._1m, 30)]
    }
}
