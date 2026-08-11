use super::Levels;

pub(crate) const LEVEL_COUNT: usize = 9;
pub(crate) const LEVEL_IDS: [LevelId; LEVEL_COUNT] = [
    LevelId::Pct10,
    LevelId::Pct20,
    LevelId::Pct30,
    LevelId::Pct40,
    LevelId::Pct50,
    LevelId::Pct60,
    LevelId::Pct70,
    LevelId::Pct80,
    LevelId::Pct90,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelId {
    Pct10,
    Pct20,
    Pct30,
    Pct40,
    Pct50,
    Pct60,
    Pct70,
    Pct80,
    Pct90,
}

impl LevelId {
    pub(super) fn select<T>(self, values: &Levels<T>) -> &T {
        match self {
            Self::Pct10 => &values.pct10,
            Self::Pct20 => &values.pct20,
            Self::Pct30 => &values.pct30,
            Self::Pct40 => &values.pct40,
            Self::Pct50 => &values.pct50,
            Self::Pct60 => &values.pct60,
            Self::Pct70 => &values.pct70,
            Self::Pct80 => &values.pct80,
            Self::Pct90 => &values.pct90,
        }
    }

    pub(super) fn select_mut<T>(self, values: &mut Levels<T>) -> &mut T {
        match self {
            Self::Pct10 => &mut values.pct10,
            Self::Pct20 => &mut values.pct20,
            Self::Pct30 => &mut values.pct30,
            Self::Pct40 => &mut values.pct40,
            Self::Pct50 => &mut values.pct50,
            Self::Pct60 => &mut values.pct60,
            Self::Pct70 => &mut values.pct70,
            Self::Pct80 => &mut values.pct80,
            Self::Pct90 => &mut values.pct90,
        }
    }
}
