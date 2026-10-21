use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Levels<T> {
    pub p10: T,
    pub p20: T,
    pub p30: T,
    pub p40: T,
    pub median: T,
    pub p60: T,
    pub p70: T,
    pub p80: T,
    pub p90: T,
}

collection!(LevelId for Levels {
    P10 => p10,
    P20 => p20,
    P30 => p30,
    P40 => p40,
    Median => median,
    P60 => p60,
    P70 => p70,
    P80 => p80,
    P90 => p90,
});
