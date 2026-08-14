mod cached_window_start;
mod window_start;

use brk_traversable::Traversable;
use brk_types::{Height, Timestamp, Version};
use vecdb::CachedBoxedVec;

use crate::internal::{WindowStarts, Windows};

pub use cached_window_start::CachedWindowStartVec;
pub use window_start::LazyWindowStartVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    pub _1h: LazyWindowStartVec,
    pub _24h: CachedWindowStartVec,
    pub _3d: LazyWindowStartVec,
    pub _1w: CachedWindowStartVec,
    pub _8d: LazyWindowStartVec,
    pub _9d: LazyWindowStartVec,
    pub _12d: LazyWindowStartVec,
    pub _13d: LazyWindowStartVec,
    pub _2w: LazyWindowStartVec,
    pub _21d: LazyWindowStartVec,
    pub _26d: LazyWindowStartVec,
    pub _1m: CachedWindowStartVec,
    pub _34d: LazyWindowStartVec,
    pub _50d: LazyWindowStartVec,
    pub _55d: LazyWindowStartVec,
    pub _2m: LazyWindowStartVec,
    pub _9w: LazyWindowStartVec,
    pub _12w: LazyWindowStartVec,
    pub _89d: LazyWindowStartVec,
    pub _3m: CachedWindowStartVec,
    pub _14w: LazyWindowStartVec,
    pub _111d: LazyWindowStartVec,
    pub _144d: LazyWindowStartVec,
    pub _6m: CachedWindowStartVec,
    pub _26w: LazyWindowStartVec,
    pub _200d: LazyWindowStartVec,
    pub _9m: LazyWindowStartVec,
    pub _350d: LazyWindowStartVec,
    pub _12m: LazyWindowStartVec,
    pub _1y: CachedWindowStartVec,
    pub _14m: LazyWindowStartVec,
    pub _2y: CachedWindowStartVec,
    pub _26m: LazyWindowStartVec,
    pub _3y: CachedWindowStartVec,
    pub _200w: LazyWindowStartVec,
    pub _4y: CachedWindowStartVec,
    pub _5y: CachedWindowStartVec,
    pub _6y: CachedWindowStartVec,
    pub _8y: CachedWindowStartVec,
    pub _9y: LazyWindowStartVec,
    pub _10y: CachedWindowStartVec,
    pub _12y: LazyWindowStartVec,
    pub _14y: LazyWindowStartVec,
    pub _26y: LazyWindowStartVec,
}

impl Vecs {
    pub(crate) fn new(version: Version, timestamps: CachedBoxedVec<Height, Timestamp>) -> Self {
        macro_rules! hours {
            ($suffix:literal, $hours:literal) => {
                LazyWindowStartVec::hours(
                    concat!("height_", $suffix, "_ago"),
                    version,
                    $hours,
                    timestamps.clone(),
                )
            };
        }
        macro_rules! days {
            ($suffix:literal, $days:expr) => {
                LazyWindowStartVec::days(
                    concat!("height_", $suffix, "_ago"),
                    version,
                    $days,
                    timestamps.clone(),
                )
            };
        }
        macro_rules! cached_days {
            ($suffix:literal, $days:expr) => {
                CachedWindowStartVec::new(days!($suffix, $days))
            };
        }

        Self {
            _1h: hours!("1h", 1),
            _24h: cached_days!("24h", 1),
            _3d: days!("3d", 3),
            _1w: cached_days!("1w", 7),
            _8d: days!("8d", 8),
            _9d: days!("9d", 9),
            _12d: days!("12d", 12),
            _13d: days!("13d", 13),
            _2w: days!("2w", 14),
            _21d: days!("21d", 21),
            _26d: days!("26d", 26),
            _1m: cached_days!("1m", 30),
            _34d: days!("34d", 34),
            _50d: days!("50d", 50),
            _55d: days!("55d", 55),
            _2m: days!("2m", 60),
            _9w: days!("9w", 9 * 7),
            _12w: days!("12w", 12 * 7),
            _89d: days!("89d", 89),
            _3m: cached_days!("3m", 90),
            _14w: days!("14w", 14 * 7),
            _111d: days!("111d", 111),
            _144d: days!("144d", 144),
            _6m: cached_days!("6m", 180),
            _26w: days!("26w", 26 * 7),
            _200d: days!("200d", 200),
            _9m: days!("9m", 270),
            _350d: days!("350d", 350),
            _12m: days!("12m", 360),
            _1y: cached_days!("1y", 365),
            _14m: days!("14m", 420),
            _2y: cached_days!("2y", 2 * 365),
            _26m: days!("26m", 780),
            _3y: cached_days!("3y", 3 * 365),
            _200w: days!("200w", 200 * 7),
            _4y: cached_days!("4y", 4 * 365),
            _5y: cached_days!("5y", 5 * 365),
            _6y: cached_days!("6y", 6 * 365),
            _8y: cached_days!("8y", 8 * 365),
            _9y: days!("9y", 9 * 365),
            _10y: cached_days!("10y", 10 * 365),
            _12y: days!("12y", 12 * 365),
            _14y: days!("14y", 14 * 365),
            _26y: days!("26y", 26 * 365),
        }
    }

    pub fn cached_window_starts(&self) -> Windows<&CachedWindowStartVec> {
        Windows {
            _24h: &self._24h,
            _1w: &self._1w,
            _1m: &self._1m,
            _1y: &self._1y,
        }
    }

    pub fn window_starts(&self) -> WindowStarts<'_> {
        WindowStarts(Windows {
            _24h: self._24h.lazy(),
            _1w: self._1w.lazy(),
            _1m: self._1m.lazy(),
            _1y: self._1y.lazy(),
        })
    }

    pub fn start_vec(&self, days: usize) -> &LazyWindowStartVec {
        match days {
            1 => self._24h.lazy(),
            3 => &self._3d,
            7 => self._1w.lazy(),
            8 => &self._8d,
            9 => &self._9d,
            12 => &self._12d,
            13 => &self._13d,
            14 => &self._2w,
            21 => &self._21d,
            26 => &self._26d,
            30 => self._1m.lazy(),
            34 => &self._34d,
            50 => &self._50d,
            55 => &self._55d,
            60 => &self._2m,
            63 => &self._9w,
            84 => &self._12w,
            89 => &self._89d,
            90 => self._3m.lazy(),
            98 => &self._14w,
            111 => &self._111d,
            144 => &self._144d,
            180 => self._6m.lazy(),
            182 => &self._26w,
            200 => &self._200d,
            270 => &self._9m,
            350 => &self._350d,
            360 => &self._12m,
            365 => self._1y.lazy(),
            420 => &self._14m,
            730 => self._2y.lazy(),
            780 => &self._26m,
            1095 => self._3y.lazy(),
            1400 => &self._200w,
            1460 => self._4y.lazy(),
            1825 => self._5y.lazy(),
            2190 => self._6y.lazy(),
            2920 => self._8y.lazy(),
            3285 => &self._9y,
            3650 => self._10y.lazy(),
            4380 => &self._12y,
            5110 => &self._14y,
            9490 => &self._26y,
            _ => panic!("No start vec for {days} days"),
        }
    }

    pub fn cached_start_vec(&self, days: usize) -> &CachedWindowStartVec {
        match days {
            1 => &self._24h,
            7 => &self._1w,
            30 => &self._1m,
            90 => &self._3m,
            180 => &self._6m,
            365 => &self._1y,
            730 => &self._2y,
            1095 => &self._3y,
            1460 => &self._4y,
            1825 => &self._5y,
            2190 => &self._6y,
            2920 => &self._8y,
            3650 => &self._10y,
            _ => panic!("No cached start vec for {days} days"),
        }
    }

    pub(crate) fn invalidate_caches(&self) {
        self._24h.invalidate();
        self._1w.invalidate();
        self._1m.invalidate();
        self._3m.invalidate();
        self._6m.invalidate();
        self._1y.invalidate();
        self._2y.invalidate();
        self._3y.invalidate();
        self._4y.invalidate();
        self._5y.invalidate();
        self._6y.invalidate();
        self._8y.invalidate();
        self._10y.invalidate();
    }
}
