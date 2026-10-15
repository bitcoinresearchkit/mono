use bitview_traversable::Traversable;

#[derive(Debug, Clone, Copy, PartialEq, Traversable)]
pub struct Levels<T> {
    /// Returns the 10th percentile of that subset's creation-price distribution.
    pub pct10: T,
    /// Returns the 20th percentile of that subset's creation-price distribution.
    pub pct20: T,
    /// Returns the 30th percentile of that subset's creation-price distribution.
    pub pct30: T,
    /// Returns the 40th percentile of that subset's creation-price distribution.
    pub pct40: T,
    /// Returns the 50th percentile of that subset's creation-price distribution.
    pub median: T,
    /// Returns the 60th percentile of that subset's creation-price distribution.
    pub pct60: T,
    /// Returns the 70th percentile of that subset's creation-price distribution.
    pub pct70: T,
    /// Returns the 80th percentile of that subset's creation-price distribution.
    pub pct80: T,
    /// Returns the 90th percentile of that subset's creation-price distribution.
    pub pct90: T,
}

collection!(LevelId for Levels {
    Pct10 => pct10,
    Pct20 => pct20,
    Pct30 => pct30,
    Pct40 => pct40,
    Median => median,
    Pct60 => pct60,
    Pct70 => pct70,
    Pct80 => pct80,
    Pct90 => pct90,
});
