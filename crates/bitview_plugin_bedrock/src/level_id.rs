use super::Levels;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelId {
    Pct10,
    Pct20,
    Pct30,
    Pct40,
    Median,
    Pct60,
    Pct70,
    Pct80,
    Pct90,
}

impl LevelId {
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Pct10 => "pct10",
            Self::Pct20 => "pct20",
            Self::Pct30 => "pct30",
            Self::Pct40 => "pct40",
            Self::Median => "median",
            Self::Pct60 => "pct60",
            Self::Pct70 => "pct70",
            Self::Pct80 => "pct80",
            Self::Pct90 => "pct90",
        }
    }

    pub fn select<T>(self, values: &Levels<T>) -> &T {
        match self {
            Self::Pct10 => &values.pct10,
            Self::Pct20 => &values.pct20,
            Self::Pct30 => &values.pct30,
            Self::Pct40 => &values.pct40,
            Self::Median => &values.median,
            Self::Pct60 => &values.pct60,
            Self::Pct70 => &values.pct70,
            Self::Pct80 => &values.pct80,
            Self::Pct90 => &values.pct90,
        }
    }
}
