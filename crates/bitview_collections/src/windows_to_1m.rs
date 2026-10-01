use std::result::Result;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct WindowsTo1m<A> {
    /// Uses a 1-day base interval.
    _24h: A,
    /// Uses a 7-day base interval.
    _1w: A,
    /// Uses a 30-day base interval.
    _1m: A,
}

impl<A> WindowsTo1m<A> {
    const SUFFIXES: [&'static str; 3] = ["24h", "1w", "1m"];

    pub fn try_from_fn<E>(mut f: impl FnMut(&str) -> Result<A, E>) -> Result<Self, E> {
        Ok(Self {
            _24h: f(Self::SUFFIXES[0])?,
            _1w: f(Self::SUFFIXES[1])?,
            _1m: f(Self::SUFFIXES[2])?,
        })
    }

    pub fn as_mut_array_with_days(&mut self) -> [(&mut A, usize); 3] {
        [(&mut self._24h, 1), (&mut self._1w, 7), (&mut self._1m, 30)]
    }
}
