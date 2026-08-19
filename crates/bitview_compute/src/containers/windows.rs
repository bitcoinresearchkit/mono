use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{ColumnId, VecValue};

const WINDOW_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowId {
    Day1,
    Week1,
    Month1,
    Year1,
}

const WINDOW_IDS: [WindowId; WINDOW_COUNT] = [
    WindowId::Day1,
    WindowId::Week1,
    WindowId::Month1,
    WindowId::Year1,
];

impl WindowId {
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Day1 => "24h",
            Self::Week1 => "1w",
            Self::Month1 => "1m",
            Self::Year1 => "1y",
        }
    }

    pub fn series<T>(mut create: impl FnMut(Self) -> T) -> Windows<T> {
        Windows {
            _24h: create(Self::Day1),
            _1w: create(Self::Week1),
            _1m: create(Self::Month1),
            _1y: create(Self::Year1),
        }
    }

    pub fn select<T>(self, windows: &Windows<T>) -> &T {
        match self {
            Self::Day1 => &windows._24h,
            Self::Week1 => &windows._1w,
            Self::Month1 => &windows._1m,
            Self::Year1 => &windows._1y,
        }
    }
}

impl ColumnId for WindowId {
    type Row<T>
        = [T; WINDOW_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &WINDOW_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| create(WINDOW_IDS[index]))
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

#[derive(Clone, Copy, Traversable)]
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
    pub const SECS: [f64; 4] = [
        Self::DAYS[0] as f64 * 86400.0,
        Self::DAYS[1] as f64 * 86400.0,
        Self::DAYS[2] as f64 * 86400.0,
        Self::DAYS[3] as f64 * 86400.0,
    ];

    pub fn try_from_fn<E>(
        mut f: impl FnMut(&str) -> std::result::Result<A, E>,
    ) -> std::result::Result<Self, E> {
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

    /// Largest window first (1y, 1m, 1w, 24h).
    pub fn as_array_largest_first(&self) -> [&A; 4] {
        [&self._1y, &self._1m, &self._1w, &self._24h]
    }

    pub fn as_mut_array(&mut self) -> [&mut A; 4] {
        [&mut self._24h, &mut self._1w, &mut self._1m, &mut self._1y]
    }

    /// Largest window first (1y, 1m, 1w, 24h).
    pub fn as_mut_array_largest_first(&mut self) -> [&mut A; 4] {
        [&mut self._1y, &mut self._1m, &mut self._1w, &mut self._24h]
    }

    pub fn as_mut_array_from_1w(&mut self) -> [&mut A; 3] {
        [&mut self._1w, &mut self._1m, &mut self._1y]
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

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{WINDOW_IDS, WindowId};

    #[test]
    fn window_columns_match_named_fields() {
        assert_eq!(WindowId::ALL, WINDOW_IDS);

        let windows = WindowId::series(|window| window);
        assert_eq!(windows._24h, WindowId::Day1);
        assert_eq!(windows._1w, WindowId::Week1);
        assert_eq!(windows._1m, WindowId::Month1);
        assert_eq!(windows._1y, WindowId::Year1);
    }
}
