use bitview_plugin_coinflow::HorizonId;

use super::super::{MODE_COUNT, ModeId};

const WEIGHTED_MODE_COUNT: usize = MODE_COUNT - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedModeId {
    Cointime,
    Coinflow,
    Coinflow8Y,
    Coinflow4Y,
    Coinflow2Y,
    Coinflow1Y,
    Coinflow6M,
    Coinflow3M,
    Coinflow1M,
}

impl WeightedModeId {
    pub const ALL: [Self; WEIGHTED_MODE_COUNT] = [
        Self::Cointime,
        Self::Coinflow,
        Self::Coinflow8Y,
        Self::Coinflow4Y,
        Self::Coinflow2Y,
        Self::Coinflow1Y,
        Self::Coinflow6M,
        Self::Coinflow3M,
        Self::Coinflow1M,
    ];

    pub const COINFLOW_HORIZONS: [Self; 7] = [
        Self::Coinflow8Y,
        Self::Coinflow4Y,
        Self::Coinflow2Y,
        Self::Coinflow1Y,
        Self::Coinflow6M,
        Self::Coinflow3M,
        Self::Coinflow1M,
    ];

    pub const fn mode(self) -> ModeId {
        match self {
            Self::Cointime => ModeId::Cointime,
            Self::Coinflow => ModeId::Coinflow,
            Self::Coinflow8Y => ModeId::Coinflow8Y,
            Self::Coinflow4Y => ModeId::Coinflow4Y,
            Self::Coinflow2Y => ModeId::Coinflow2Y,
            Self::Coinflow1Y => ModeId::Coinflow1Y,
            Self::Coinflow6M => ModeId::Coinflow6M,
            Self::Coinflow3M => ModeId::Coinflow3M,
            Self::Coinflow1M => ModeId::Coinflow1M,
        }
    }

    pub const fn coinflow_horizon(self) -> Option<HorizonId> {
        match self {
            Self::Cointime | Self::Coinflow => None,
            Self::Coinflow8Y => Some(HorizonId::Y8),
            Self::Coinflow4Y => Some(HorizonId::Y4),
            Self::Coinflow2Y => Some(HorizonId::Y2),
            Self::Coinflow1Y => Some(HorizonId::Y1),
            Self::Coinflow6M => Some(HorizonId::M6),
            Self::Coinflow3M => Some(HorizonId::M3),
            Self::Coinflow1M => Some(HorizonId::M1),
        }
    }
}
