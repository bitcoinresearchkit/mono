use super::Percentiles;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossPercentileId {
    Pct95,
    Pct98,
    Pct99,
    Pct99_5,
    Pct99_9,
}

impl LossPercentileId {
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Pct95 => "pct95",
            Self::Pct98 => "pct98",
            Self::Pct99 => "pct99",
            Self::Pct99_5 => "pct99_5",
            Self::Pct99_9 => "pct99_9",
        }
    }

    pub fn select<T>(self, values: &Percentiles<T>) -> &T {
        match self {
            Self::Pct95 => &values.pct95,
            Self::Pct98 => &values.pct98,
            Self::Pct99 => &values.pct99,
            Self::Pct99_5 => &values.pct99_5,
            Self::Pct99_9 => &values.pct99_9,
        }
    }
}
