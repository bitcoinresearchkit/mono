use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Levels<T> {
    pub pct10: T,
    pub pct20: T,
    pub pct30: T,
    pub pct40: T,
    pub median: T,
    pub pct60: T,
    pub pct70: T,
    pub pct80: T,
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
