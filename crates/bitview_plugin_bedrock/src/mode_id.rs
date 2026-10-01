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
            Self::Raw => "raw",
            Self::Cointime => "cointime",
            Self::Coinflow => "coinflow",
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

        let mut modes = Modes::try_from_fn(|id| Ok::<_, Infallible>((id, false))).unwrap();
        for id in ModeId::ALL {
            let mode = modes.select_mut(id);
            assert_eq!(mode.0, id);
            mode.1 = true;
        }
        assert!(modes.iter().all(|(_, visited)| *visited));
        assert_eq!(
            ModeId::ALL.map(ModeId::name),
            ["raw", "cointime", "coinflow"]
        );
    }
}
