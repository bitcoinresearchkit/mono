use std::result::Result;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct WindowsFrom1w<A> {
    /// Uses a trailing 7-day window.
    _1w: A,
    /// Uses a trailing 30-day window.
    _1m: A,
    /// Uses a trailing 365-day window.
    _1y: A,
}

impl<A> WindowsFrom1w<A> {
    const SUFFIXES: [&'static str; 3] = ["1w", "1m", "1y"];

    pub fn try_from_fn<E>(mut f: impl FnMut(&str) -> Result<A, E>) -> Result<Self, E> {
        Ok(Self {
            _1w: f(Self::SUFFIXES[0])?,
            _1m: f(Self::SUFFIXES[1])?,
            _1y: f(Self::SUFFIXES[2])?,
        })
    }

    pub fn as_mut_array(&mut self) -> [&mut A; 3] {
        [&mut self._1w, &mut self._1m, &mut self._1y]
    }
}
