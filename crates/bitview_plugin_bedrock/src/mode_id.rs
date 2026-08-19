use super::WeightedModeId;

pub const MODE_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ModeId {
    Raw,
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

impl ModeId {
    pub const ALL: [Self; MODE_COUNT] = [
        Self::Raw,
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

    pub const fn name(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Cointime => "cointime",
            Self::Coinflow => "coinflow",
            Self::Coinflow8Y => "coinflow_8y",
            Self::Coinflow4Y => "coinflow_4y",
            Self::Coinflow2Y => "coinflow_2y",
            Self::Coinflow1Y => "coinflow_1y",
            Self::Coinflow6M => "coinflow_6m",
            Self::Coinflow3M => "coinflow_3m",
            Self::Coinflow1M => "coinflow_1m",
        }
    }

    pub const fn weighted(self) -> Option<WeightedModeId> {
        match self {
            Self::Raw => None,
            Self::Cointime => Some(WeightedModeId::Cointime),
            Self::Coinflow => Some(WeightedModeId::Coinflow),
            Self::Coinflow8Y => Some(WeightedModeId::Coinflow8Y),
            Self::Coinflow4Y => Some(WeightedModeId::Coinflow4Y),
            Self::Coinflow2Y => Some(WeightedModeId::Coinflow2Y),
            Self::Coinflow1Y => Some(WeightedModeId::Coinflow1Y),
            Self::Coinflow6M => Some(WeightedModeId::Coinflow6M),
            Self::Coinflow3M => Some(WeightedModeId::Coinflow3M),
            Self::Coinflow1M => Some(WeightedModeId::Coinflow1M),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::ModeId;
    use crate::{Modes, WeightedModeId};

    #[test]
    fn ids_match_named_fields_and_storage_names() {
        assert_eq!(
            WeightedModeId::ALL.map(WeightedModeId::mode).as_slice(),
            &ModeId::ALL[1..]
        );
        assert_eq!(
            WeightedModeId::COINFLOW_HORIZONS
                .map(WeightedModeId::mode)
                .as_slice(),
            &ModeId::ALL[3..]
        );

        let mut modes = Modes::try_from_fn(|id| Ok::<_, Infallible>((id, false))).unwrap();
        for id in ModeId::ALL {
            let mode = modes.select_mut(id);
            assert_eq!(mode.0, id);
            mode.1 = true;
        }
        assert!(modes.iter().all(|(_, visited)| *visited));
        assert_eq!(
            ModeId::ALL.map(ModeId::name),
            [
                "raw",
                "cointime",
                "coinflow",
                "coinflow_8y",
                "coinflow_4y",
                "coinflow_2y",
                "coinflow_1y",
                "coinflow_6m",
                "coinflow_3m",
                "coinflow_1m",
            ]
        );
    }
}
