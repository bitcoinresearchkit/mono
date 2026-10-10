use super::WeightedModeId;

pub const MODE_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ModeId {
    Raw,
    Cointime,
    Coinflow,
}

impl ModeId {
    pub const ALL: [Self; MODE_COUNT] = [Self::Raw, Self::Cointime, Self::Coinflow];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Raw => "unweighted",
            Self::Cointime => "awake",
            Self::Coinflow => "mobile",
        }
    }

    pub const fn weighted(self) -> Option<WeightedModeId> {
        match self {
            Self::Raw => None,
            Self::Cointime => Some(WeightedModeId::Cointime),
            Self::Coinflow => Some(WeightedModeId::Coinflow),
        }
    }
}
