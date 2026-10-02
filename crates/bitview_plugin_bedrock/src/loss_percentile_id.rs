use super::Percentiles;

const PERCENTILE_COUNT: usize = 5;
const LOSS_PERCENTILE_IDS: [LossPercentileId; PERCENTILE_COUNT] = [
    LossPercentileId::Pct95,
    LossPercentileId::Pct98,
    LossPercentileId::Pct99,
    LossPercentileId::Pct99_5,
    LossPercentileId::Pct99_9,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossPercentileId {
    Pct95,
    Pct98,
    Pct99,
    Pct99_5,
    Pct99_9,
}

impl LossPercentileId {
    pub const ALL: [Self; PERCENTILE_COUNT] = LOSS_PERCENTILE_IDS;

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

    pub fn select_mut<T>(self, values: &mut Percentiles<T>) -> &mut T {
        match self {
            Self::Pct95 => &mut values.pct95,
            Self::Pct98 => &mut values.pct98,
            Self::Pct99 => &mut values.pct99,
            Self::Pct99_5 => &mut values.pct99_5,
            Self::Pct99_9 => &mut values.pct99_9,
        }
    }
}
