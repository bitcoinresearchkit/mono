use bitview_traversable::Traversable;
use brk_types::Cents;

use super::Percentiles;

/// The price ranges a mode's floors cut below its 95th-percentile floor, highest first.
#[derive(Clone, Copy, Default, Traversable)]
pub struct FloorBands<T> {
    /// Below the 95th-percentile floor, at or above the 98th.
    #[traversable(lead = "Between the 95th- and 98th-percentile floors.")]
    pub p95_to_p98: T,
    /// Below the 98th-percentile floor, at or above the 99th.
    #[traversable(lead = "Between the 98th- and 99th-percentile floors.")]
    pub p98_to_p99: T,
    /// Below the 99th-percentile floor, at or above the 99.5th.
    #[traversable(lead = "Between the 99th- and 99.5th-percentile floors.")]
    pub p99_to_p99_5: T,
    /// Below the 99.5th-percentile floor, at or above the 99.9th.
    #[traversable(lead = "Between the 99.5th- and 99.9th-percentile floors.")]
    pub p99_5_to_p99_9: T,
    /// Below the 99.9th-percentile floor.
    #[traversable(lead = "Below the 99.9th-percentile floor.")]
    pub below_p99_9: T,
}

collection!(FloorBandId for FloorBands {
    P95ToP98 => p95_to_p98,
    P98ToP99 => p98_to_p99,
    P99ToP99_5 => p99_to_p99_5,
    P99_5ToP99_9 => p99_5_to_p99_9,
    BelowP99_9 => below_p99_9,
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
