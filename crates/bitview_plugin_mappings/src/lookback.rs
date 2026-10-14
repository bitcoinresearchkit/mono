use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, WindowStarts, shapes::Windows};
use brk_types::{Height, Timestamp, Version};
use vecdb::ReadableBoxedVec;

/// Declares every trailing duration once: field, id suffix and length in days.
macro_rules! lookbacks {
    ($($(#[$doc:meta])* $vis:vis $field:ident: $suffix:literal = $days:expr,)*) => {
        /// First block height inside each trailing duration, from the monotonic timestamps.
        #[derive(Clone, Traversable)]
        pub struct LookbackVecs {
            $($(#[$doc])* $vis $field: LazyWindowStartVec,)*
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
    /// Uses a trailing 24-hour duration.
    pub _24h: "24h" = 1,
    /// Uses a trailing 3-day duration.
    pub _3d: "3d" = 3,
    /// Uses a trailing 7-day duration.
    pub _1w: "1w" = 7,
    /// Uses a trailing 8-day duration.
    _8d: "8d" = 8,
    /// Uses a trailing 13-day duration.
    _13d: "13d" = 13,
    /// Uses a trailing 21-day duration.
    _21d: "21d" = 21,
    /// Uses a trailing 30-day duration.
    pub _1m: "1m" = 30,
    /// Uses a trailing 34-day duration.
    _34d: "34d" = 34,
    /// Uses a trailing 50-day duration.
    _50d: "50d" = 50,
    /// Uses a trailing 55-day duration.
    _55d: "55d" = 55,
    /// Uses a trailing 60-day duration.
    _2m: "2m" = 60,
    /// Uses a trailing 89-day duration.
    _89d: "89d" = 89,
    /// Uses a trailing 90-day duration.
    pub _3m: "3m" = 90,
    /// Uses a trailing 144-day duration.
    _144d: "144d" = 144,
    /// Uses a trailing 180-day duration.
    pub _6m: "6m" = 180,
    /// Uses a trailing 200-day duration.
    _200d: "200d" = 200,
    /// Uses a trailing 365-day duration.
    pub _1y: "1y" = 365,
    /// Uses a trailing 730-day duration.
    pub _2y: "2y" = 2 * 365,
    /// Uses a trailing 1,095-day duration.
    pub _3y: "3y" = 3 * 365,
    /// Uses a trailing 1,400-day duration.
    _200w: "200w" = 200 * 7,
    /// Uses a trailing 1,460-day duration.
    _4y: "4y" = 4 * 365,
    /// Uses a trailing 1,825-day duration.
    _5y: "5y" = 5 * 365,
    /// Uses a trailing 2,190-day duration.
    _6y: "6y" = 6 * 365,
    /// Uses a trailing 2,920-day duration.
    _8y: "8y" = 8 * 365,
    /// Uses a trailing 3,650-day duration.
    _10y: "10y" = 10 * 365,
}

impl LookbackVecs {
    pub fn window_starts(&self) -> WindowStarts<'_> {
        WindowStarts(Windows {
            _24h: &self._24h,
            _1w: &self._1w,
            _1m: &self._1m,
            _1y: &self._1y,
        })
    }
}
