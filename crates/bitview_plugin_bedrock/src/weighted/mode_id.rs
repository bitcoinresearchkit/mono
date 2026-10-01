use super::super::{MODE_COUNT, ModeId};

const WEIGHTED_MODE_COUNT: usize = MODE_COUNT - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedModeId {
    Cointime,
    Coinflow,
}

impl WeightedModeId {
    pub const ALL: [Self; WEIGHTED_MODE_COUNT] = [Self::Cointime, Self::Coinflow];

    pub const fn mode(self) -> ModeId {
        match self {
            Self::Cointime => ModeId::Cointime,
            Self::Coinflow => ModeId::Coinflow,
        }
    }
}
