use bitview_traversable::Traversable;
use brk_types::Cents;

use super::Percentiles;

/// The price ranges a mode's floors cut below its 95th-percentile floor, highest first.
#[derive(Clone, Copy, Default, Traversable)]
pub struct FloorBands<T> {
    /// Below the 95th-percentile floor, at or above the 98th.
    pub pct95_to_pct98: T,
    /// Below the 98th-percentile floor, at or above the 99th.
    pub pct98_to_pct99: T,
    /// Below the 99th-percentile floor, at or above the 99.5th.
    pub pct99_to_pct99_5: T,
    /// Below the 99.5th-percentile floor, at or above the 99.9th.
    pub pct99_5_to_pct99_9: T,
    /// Below the 99.9th-percentile floor.
    pub below_pct99_9: T,
}

collection!(FloorBandId for FloorBands {
    Pct95ToPct98 => pct95_to_pct98,
    Pct98ToPct99 => pct98_to_pct99,
    Pct99ToPct99_5 => pct99_to_pct99_5,
    Pct99_5ToPct99_9 => pct99_5_to_pct99_9,
    BelowPct99_9 => below_pct99_9,
});

impl FloorBandId {
    /// The band `spot` sits in: under as many floors as the band's index plus one. None at or
    /// above every floor, or without floors (all are missing together).
    pub fn locate(floors: &Percentiles<Cents>, spot: Cents) -> Option<Self> {
        if spot.is_nan() || floors.iter().any(|floor| floor.is_nan()) {
            return None;
        }
        debug_assert!(floors.iter().is_sorted_by(|higher, lower| higher >= lower));
        let under = floors.iter().filter(|&&floor| spot < floor).count();
        under.checked_sub(1).map(|band| Self::ALL[band])
    }
}
