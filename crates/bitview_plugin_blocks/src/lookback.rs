use bitview_collections::Windows;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, Lookback, WindowStarts};
use brk_types::{Height, Timestamp, Version};
use vecdb::ReadableBoxedVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Uses a trailing 1-hour duration.
    _1h: LazyWindowStartVec,
    /// Uses a trailing 24-hour duration.
    pub _24h: LazyWindowStartVec,
    /// Uses a trailing 3-day duration.
    pub _3d: LazyWindowStartVec,
    /// Uses a trailing 7-day duration.
    pub _1w: LazyWindowStartVec,
    /// Uses a trailing 8-day duration.
    _8d: LazyWindowStartVec,
    /// Uses a trailing 9-day duration.
    _9d: LazyWindowStartVec,
    /// Uses a trailing 12-day duration.
    _12d: LazyWindowStartVec,
    /// Uses a trailing 13-day duration.
    _13d: LazyWindowStartVec,
    /// Uses a trailing 14-day duration.
    _2w: LazyWindowStartVec,
    /// Uses a trailing 21-day duration.
    _21d: LazyWindowStartVec,
    /// Uses a trailing 26-day duration.
    _26d: LazyWindowStartVec,
    /// Uses a trailing 30-day duration.
    pub _1m: LazyWindowStartVec,
    /// Uses a trailing 34-day duration.
    _34d: LazyWindowStartVec,
    /// Uses a trailing 50-day duration.
    _50d: LazyWindowStartVec,
    /// Uses a trailing 55-day duration.
    _55d: LazyWindowStartVec,
    /// Uses a trailing 60-day duration.
    _2m: LazyWindowStartVec,
    /// Uses a trailing 63-day duration.
    _9w: LazyWindowStartVec,
    /// Uses a trailing 84-day duration.
    _12w: LazyWindowStartVec,
    /// Uses a trailing 89-day duration.
    _89d: LazyWindowStartVec,
    /// Uses a trailing 90-day duration.
    pub _3m: LazyWindowStartVec,
    /// Uses a trailing 98-day duration.
    _14w: LazyWindowStartVec,
    /// Uses a trailing 111-day duration.
    _111d: LazyWindowStartVec,
    /// Uses a trailing 144-day duration.
    _144d: LazyWindowStartVec,
    /// Uses a trailing 180-day duration.
    pub _6m: LazyWindowStartVec,
    /// Uses a trailing 182-day duration.
    _26w: LazyWindowStartVec,
    /// Uses a trailing 200-day duration.
    _200d: LazyWindowStartVec,
    /// Uses a trailing 270-day duration.
    _9m: LazyWindowStartVec,
    /// Uses a trailing 350-day duration.
    _350d: LazyWindowStartVec,
    /// Uses a trailing 360-day duration.
    _12m: LazyWindowStartVec,
    /// Uses a trailing 365-day duration.
    pub _1y: LazyWindowStartVec,
    /// Uses a trailing 420-day duration.
    _14m: LazyWindowStartVec,
    /// Uses a trailing 730-day duration.
    pub _2y: LazyWindowStartVec,
    /// Uses a trailing 780-day duration.
    _26m: LazyWindowStartVec,
    /// Uses a trailing 1,095-day duration.
    pub _3y: LazyWindowStartVec,
    /// Uses a trailing 1,400-day duration.
    _200w: LazyWindowStartVec,
    /// Uses a trailing 1,460-day duration.
    _4y: LazyWindowStartVec,
    /// Uses a trailing 1,825-day duration.
    _5y: LazyWindowStartVec,
    /// Uses a trailing 2,190-day duration.
    _6y: LazyWindowStartVec,
    /// Uses a trailing 2,920-day duration.
    _8y: LazyWindowStartVec,
    /// Uses a trailing 3,285-day duration.
    _9y: LazyWindowStartVec,
    /// Uses a trailing 3,650-day duration.
    _10y: LazyWindowStartVec,
    /// Uses a trailing 4,380-day duration.
    _12y: LazyWindowStartVec,
    /// Uses a trailing 5,110-day duration.
    _14y: LazyWindowStartVec,
    /// Uses a trailing 9,490-day duration.
    _26y: LazyWindowStartVec,
}

impl Vecs {
    pub(crate) fn new(version: Version, timestamps: ReadableBoxedVec<Height, Timestamp>) -> Self {
        let days = |suffix, days| {
            LazyWindowStartVec::days(&format!("height_{suffix}_ago"), version, days, &timestamps)
        };

        Self {
            _1h: LazyWindowStartVec::hours("height_1h_ago", version, 1, &timestamps),
            _24h: days("24h", 1),
            _3d: days("3d", 3),
            _1w: days("1w", 7),
            _8d: days("8d", 8),
            _9d: days("9d", 9),
            _12d: days("12d", 12),
            _13d: days("13d", 13),
            _2w: days("2w", 14),
            _21d: days("21d", 21),
            _26d: days("26d", 26),
            _1m: days("1m", 30),
            _34d: days("34d", 34),
            _50d: days("50d", 50),
            _55d: days("55d", 55),
            _2m: days("2m", 60),
            _9w: days("9w", 9 * 7),
            _12w: days("12w", 12 * 7),
            _89d: days("89d", 89),
            _3m: days("3m", 90),
            _14w: days("14w", 14 * 7),
            _111d: days("111d", 111),
            _144d: days("144d", 144),
            _6m: days("6m", 180),
            _26w: days("26w", 26 * 7),
            _200d: days("200d", 200),
            _9m: days("9m", 270),
            _350d: days("350d", 350),
            _12m: days("12m", 360),
            _1y: days("1y", 365),
            _14m: days("14m", 420),
            _2y: days("2y", 2 * 365),
            _26m: days("26m", 780),
            _3y: days("3y", 3 * 365),
            _200w: days("200w", 200 * 7),
            _4y: days("4y", 4 * 365),
            _5y: days("5y", 5 * 365),
            _6y: days("6y", 6 * 365),
            _8y: days("8y", 8 * 365),
            _9y: days("9y", 9 * 365),
            _10y: days("10y", 10 * 365),
            _12y: days("12y", 12 * 365),
            _14y: days("14y", 14 * 365),
            _26y: days("26y", 26 * 365),
        }
    }

    pub fn window_starts(&self) -> WindowStarts<'_> {
        WindowStarts(Windows {
            _24h: self.start_vec(1),
            _1w: self.start_vec(7),
            _1m: self.start_vec(30),
            _1y: self.start_vec(365),
        })
    }

    pub fn start_vec(&self, days: usize) -> &LazyWindowStartVec {
        match days {
            1 => &self._24h,
            3 => &self._3d,
            7 => &self._1w,
            8 => &self._8d,
            9 => &self._9d,
            12 => &self._12d,
            13 => &self._13d,
            14 => &self._2w,
            21 => &self._21d,
            26 => &self._26d,
            30 => &self._1m,
            34 => &self._34d,
            50 => &self._50d,
            55 => &self._55d,
            60 => &self._2m,
            63 => &self._9w,
            84 => &self._12w,
            89 => &self._89d,
            90 => &self._3m,
            98 => &self._14w,
            111 => &self._111d,
            144 => &self._144d,
            180 => &self._6m,
            182 => &self._26w,
            200 => &self._200d,
            270 => &self._9m,
            350 => &self._350d,
            360 => &self._12m,
            365 => &self._1y,
            420 => &self._14m,
            730 => &self._2y,
            780 => &self._26m,
            1095 => &self._3y,
            1400 => &self._200w,
            1460 => &self._4y,
            1825 => &self._5y,
            2190 => &self._6y,
            2920 => &self._8y,
            3285 => &self._9y,
            3650 => &self._10y,
            4380 => &self._12y,
            5110 => &self._14y,
            9490 => &self._26y,
            _ => panic!("No start vec for {days} days"),
        }
    }
}

impl Lookback for Vecs {
    fn start_vec(&self, days: usize) -> &LazyWindowStartVec {
        self.start_vec(days)
    }
}
