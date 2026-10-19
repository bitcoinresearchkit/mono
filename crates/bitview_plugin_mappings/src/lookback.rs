use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, WindowStarts, shapes::Windows};
use brk_types::{Height, Timestamp, Version};
use vecdb::ReadableBoxedVec;

/// Declares every trailing duration once: field, id suffix and length in days.
macro_rules! lookbacks {
    ($($vis:vis $field:ident: $suffix:literal = $days:expr,)*) => {
        /// First block height inside each trailing duration, from the monotonic timestamps.
        /// Durations are fixed day counts, not calendar periods: a week is 7 days, a month 30
        /// (`_1m`, `_3m`, `_6m`: 30, 90, 180) and a year 365 (`_1y`, `_2y`, `_3y`: 365, 730 and
        /// 1,095).
        #[derive(Clone, Traversable)]
        pub struct LookbackVecs {
            $($vis $field: LazyWindowStartVec,)*
        }

        impl LookbackVecs {
            pub(crate) fn new(
                version: Version,
                timestamps: &ReadableBoxedVec<Height, Timestamp>,
            ) -> Self {
                Self {
                    $($field: LazyWindowStartVec::days(
                        concat!("height_", $suffix, "_ago"),
                        version,
                        $days,
                        timestamps,
                    ),)*
                }
            }

            /// Carries every duration's latest window starts past the timestamps
            /// written from `starting_height`.
            pub(crate) fn update(&self, starting_height: Height) {
                $(self.$field.update(starting_height);)*
            }

            /// The window start of a trailing duration in days.
            pub fn start_vec(&self, days: usize) -> &LazyWindowStartVec {
                $(if days == $days {
                    return &self.$field;
                })*
                panic!("No start vec for {days} days")
            }
        }
    };
}

lookbacks! {
    pub _24h: "24h" = 1,
    pub _3d: "3d" = 3,
    pub _1w: "1w" = 7,
    _8d: "8d" = 8,
    _13d: "13d" = 13,
    _21d: "21d" = 21,
    pub _1m: "1m" = 30,
    _34d: "34d" = 34,
    _50d: "50d" = 50,
    _55d: "55d" = 55,
    _2m: "2m" = 60,
    _89d: "89d" = 89,
    pub _3m: "3m" = 90,
    _144d: "144d" = 144,
    pub _6m: "6m" = 180,
    _200d: "200d" = 200,
    pub _1y: "1y" = 365,
    pub _2y: "2y" = 2 * 365,
    pub _3y: "3y" = 3 * 365,
    _200w: "200w" = 200 * 7,
    _4y: "4y" = 4 * 365,
    _5y: "5y" = 5 * 365,
    _6y: "6y" = 6 * 365,
    _8y: "8y" = 8 * 365,
    _10y: "10y" = 10 * 365,
}

impl LookbackVecs {
    pub fn window_starts(&self) -> WindowStarts<'_> {
        Windows {
            _24h: &self._24h,
            _1w: &self._1w,
            _1m: &self._1m,
            _1y: &self._1y,
        }
    }
}
