use std::result::Result;

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct Windows<A> {
    /// Uses a trailing 24-hour window.
    pub _24h: A,
    /// Uses a trailing 7-day window.
    pub _1w: A,
    /// Uses a trailing 30-day window.
    pub _1m: A,
    /// Uses a trailing 365-day window.
    pub _1y: A,
}

impl<A> Windows<A> {
    pub const SUFFIXES: [&'static str; 4] = ["24h", "1w", "1m", "1y"];
    pub const DAYS: [usize; 4] = [1, 7, 30, 365];

    pub fn try_from_fn<E>(mut f: impl FnMut(&str) -> Result<A, E>) -> Result<Self, E> {
        Ok(Self {
            _24h: f(Self::SUFFIXES[0])?,
            _1w: f(Self::SUFFIXES[1])?,
            _1m: f(Self::SUFFIXES[2])?,
            _1y: f(Self::SUFFIXES[3])?,
        })
    }

    pub fn as_array(&self) -> [&A; 4] {
        [&self._24h, &self._1w, &self._1m, &self._1y]
    }

    pub fn as_mut_array(&mut self) -> [&mut A; 4] {
        [&mut self._24h, &mut self._1w, &mut self._1m, &mut self._1y]
    }

    pub fn map_with_suffix<B>(&self, mut f: impl FnMut(&str, &A) -> B) -> Windows<B> {
        Windows {
            _24h: f(Self::SUFFIXES[0], &self._24h),
            _1w: f(Self::SUFFIXES[1], &self._1w),
            _1m: f(Self::SUFFIXES[2], &self._1m),
            _1y: f(Self::SUFFIXES[3], &self._1y),
        }
    }
}

impl<A, B> Windows<(A, B)> {
    pub fn unzip(self) -> (Windows<A>, Windows<B>) {
        (
            Windows {
                _24h: self._24h.0,
                _1w: self._1w.0,
                _1m: self._1m.0,
                _1y: self._1y.0,
            },
            Windows {
                _24h: self._24h.1,
                _1w: self._1w.1,
                _1m: self._1m.1,
                _1y: self._1y.1,
            },
        )
    }
}
