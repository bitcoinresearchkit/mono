use bitview_traversable::Traversable;
use brk_types::Cents;

use super::Percentiles;

/// The price ranges a mode's floors cut below its 95th-percentile floor, highest first.
#[derive(Clone, Copy, Default, Traversable)]
pub struct FloorBands<T> {
    /// Below the 95th-percentile floor, at or above the 98th.
    #[traversable(lead = "Between the 95th- and 98th-percentile floors.")]
    pub between_p95_and_p98_floors: T,
    /// Below the 98th-percentile floor, at or above the 99th.
    #[traversable(lead = "Between the 98th- and 99th-percentile floors.")]
    pub between_p98_and_p99_floors: T,
    /// Below the 99th-percentile floor, at or above the 99.5th.
    #[traversable(lead = "Between the 99th- and 99.5th-percentile floors.")]
    pub between_p99_and_p99_5_floors: T,
    /// Below the 99.5th-percentile floor, at or above the 99.9th.
    #[traversable(lead = "Between the 99.5th- and 99.9th-percentile floors.")]
    pub between_p99_5_and_p99_9_floors: T,
    /// Below the 99.9th-percentile floor.
    #[traversable(lead = "Below the 99.9th-percentile floor.")]
    pub below_p99_9_floor: T,
}

collection!(FloorBandId for FloorBands {
    BetweenP95AndP98Floors => between_p95_and_p98_floors,
    BetweenP98AndP99Floors => between_p98_and_p99_floors,
    BetweenP99AndP99_5Floors => between_p99_and_p99_5_floors,
    BetweenP99_5AndP99_9Floors => between_p99_5_and_p99_9_floors,
    BelowP99_9Floor => below_p99_9_floor,
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
