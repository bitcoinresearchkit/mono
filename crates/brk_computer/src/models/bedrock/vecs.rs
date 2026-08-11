use std::path::PathBuf;

use brk_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, Formattable, Rw, StorageMode, VecValue};

pub(super) use super::{Levels, ModeVecs, Modes, Percentiles, PriceBands, WeightedModes};

macro_rules! impl_named_row_formattable {
    ($row:ident { $($field:ident),+ $(,)? }) => {
        impl<T: Formattable> Formattable for $row<T> {
            fn write_to(&self, output: &mut Vec<u8>) {
                output.push(b'{');
                let mut first = true;
                $(
                    if !first {
                        output.push(b',');
                    }
                    first = false;
                    output.extend_from_slice(concat!("\"", stringify!($field), "\":").as_bytes());
                    self.$field.fmt_json(output);
                )+
                let _ = first;
                output.push(b'}');
            }

            fn fmt_csv(&self, output: &mut String) -> std::fmt::Result {
                let mut json = Vec::new();
                self.write_to(&mut json);
                let json = std::str::from_utf8(&json).map_err(|_| std::fmt::Error)?;

                output.push('"');
                for character in json.chars() {
                    if character == '"' {
                        output.push('"');
                    }
                    output.push(character);
                }
                output.push('"');
                Ok(())
            }
        }
    };
}

pub(crate) const MODE_COUNT: usize = 10;
pub(crate) const PERCENTILE_COUNT: usize = 5;
pub(crate) const LEVEL_COUNT: usize = 9;
const WEIGHTED_MODE_COUNT: usize = MODE_COUNT - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum ModeId {
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
    pub(super) const ALL: [Self; MODE_COUNT] = [
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

    pub(super) const fn name(self) -> &'static str {
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

    pub(super) const fn weighted(self) -> Option<WeightedModeId> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WeightedModeId {
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
    pub(super) const ALL: [Self; WEIGHTED_MODE_COUNT] = [
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

    pub(super) const COINFLOW_HORIZONS: [Self; 7] = [
        Self::Coinflow8Y,
        Self::Coinflow4Y,
        Self::Coinflow2Y,
        Self::Coinflow1Y,
        Self::Coinflow6M,
        Self::Coinflow3M,
        Self::Coinflow1M,
    ];

    pub(super) const fn mode(self) -> ModeId {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossPercentileId {
    Pct95,
    Pct98,
    Pct99,
    Pct99_5,
    Pct99_9,
}

impl LossPercentileId {
    pub(super) const fn suffix(self) -> &'static str {
        match self {
            Self::Pct95 => "pct95",
            Self::Pct98 => "pct98",
            Self::Pct99 => "pct99",
            Self::Pct99_5 => "pct99_5",
            Self::Pct99_9 => "pct99_9",
        }
    }

    pub(super) fn select<T>(self, values: &Percentiles<T>) -> &T {
        match self {
            Self::Pct95 => &values.pct95,
            Self::Pct98 => &values.pct98,
            Self::Pct99 => &values.pct99,
            Self::Pct99_5 => &values.pct99_5,
            Self::Pct99_9 => &values.pct99_9,
        }
    }

    pub(super) fn select_mut<T>(self, values: &mut Percentiles<T>) -> &mut T {
        match self {
            Self::Pct95 => &mut values.pct95,
            Self::Pct98 => &mut values.pct98,
            Self::Pct99 => &mut values.pct99,
            Self::Pct99_5 => &mut values.pct99_5,
            Self::Pct99_9 => &mut values.pct99_9,
        }
    }
}

const LOSS_PERCENTILE_IDS: [LossPercentileId; PERCENTILE_COUNT] = [
    LossPercentileId::Pct95,
    LossPercentileId::Pct98,
    LossPercentileId::Pct99,
    LossPercentileId::Pct99_5,
    LossPercentileId::Pct99_9,
];

impl ColumnId for LossPercentileId {
    type Row<T>
        = Percentiles<T>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &LOSS_PERCENTILE_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        self.select(row)
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        self.select_mut(row)
    }

    #[inline]
    fn from_fn<T, F>(create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        Percentiles::from_fn(create)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceBandId {
    FloorPct95,
    FloorPct98,
    FloorPct99,
    FloorPct99_5,
    FloorPct99_9,
    LevelPct10,
    LevelPct20,
    LevelPct30,
    LevelPct40,
    LevelPct50,
    LevelPct60,
    LevelPct70,
    LevelPct80,
    LevelPct90,
}

impl PriceBandId {
    pub(super) const fn suffix(self) -> &'static str {
        match self {
            Self::FloorPct95 => "floor_pct95",
            Self::FloorPct98 => "floor_pct98",
            Self::FloorPct99 => "floor_pct99",
            Self::FloorPct99_5 => "floor_pct99_5",
            Self::FloorPct99_9 => "floor_pct99_9",
            Self::LevelPct10 => "level_pct10",
            Self::LevelPct20 => "level_pct20",
            Self::LevelPct30 => "level_pct30",
            Self::LevelPct40 => "level_pct40",
            Self::LevelPct50 => "level_pct50",
            Self::LevelPct60 => "level_pct60",
            Self::LevelPct70 => "level_pct70",
            Self::LevelPct80 => "level_pct80",
            Self::LevelPct90 => "level_pct90",
        }
    }

    pub(super) fn select<T>(self, values: &PriceBands<T>) -> &T {
        match self {
            Self::FloorPct95 => &values.floor.pct95,
            Self::FloorPct98 => &values.floor.pct98,
            Self::FloorPct99 => &values.floor.pct99,
            Self::FloorPct99_5 => &values.floor.pct99_5,
            Self::FloorPct99_9 => &values.floor.pct99_9,
            Self::LevelPct10 => &values.level.pct10,
            Self::LevelPct20 => &values.level.pct20,
            Self::LevelPct30 => &values.level.pct30,
            Self::LevelPct40 => &values.level.pct40,
            Self::LevelPct50 => &values.level.pct50,
            Self::LevelPct60 => &values.level.pct60,
            Self::LevelPct70 => &values.level.pct70,
            Self::LevelPct80 => &values.level.pct80,
            Self::LevelPct90 => &values.level.pct90,
        }
    }

    pub(super) fn select_mut<T>(self, values: &mut PriceBands<T>) -> &mut T {
        match self {
            Self::FloorPct95 => &mut values.floor.pct95,
            Self::FloorPct98 => &mut values.floor.pct98,
            Self::FloorPct99 => &mut values.floor.pct99,
            Self::FloorPct99_5 => &mut values.floor.pct99_5,
            Self::FloorPct99_9 => &mut values.floor.pct99_9,
            Self::LevelPct10 => &mut values.level.pct10,
            Self::LevelPct20 => &mut values.level.pct20,
            Self::LevelPct30 => &mut values.level.pct30,
            Self::LevelPct40 => &mut values.level.pct40,
            Self::LevelPct50 => &mut values.level.pct50,
            Self::LevelPct60 => &mut values.level.pct60,
            Self::LevelPct70 => &mut values.level.pct70,
            Self::LevelPct80 => &mut values.level.pct80,
            Self::LevelPct90 => &mut values.level.pct90,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LevelId {
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

    const fn price_band(self) -> PriceBandId {
        match self {
            Self::Pct10 => PriceBandId::LevelPct10,
            Self::Pct20 => PriceBandId::LevelPct20,
            Self::Pct30 => PriceBandId::LevelPct30,
            Self::Pct40 => PriceBandId::LevelPct40,
            Self::Pct50 => PriceBandId::LevelPct50,
            Self::Pct60 => PriceBandId::LevelPct60,
            Self::Pct70 => PriceBandId::LevelPct70,
            Self::Pct80 => PriceBandId::LevelPct80,
            Self::Pct90 => PriceBandId::LevelPct90,
        }
    }
}

const PRICE_BAND_COUNT: usize = PERCENTILE_COUNT + LEVEL_COUNT;
const PRICE_BAND_IDS: [PriceBandId; PRICE_BAND_COUNT] = [
    PriceBandId::FloorPct95,
    PriceBandId::FloorPct98,
    PriceBandId::FloorPct99,
    PriceBandId::FloorPct99_5,
    PriceBandId::FloorPct99_9,
    PriceBandId::LevelPct10,
    PriceBandId::LevelPct20,
    PriceBandId::LevelPct30,
    PriceBandId::LevelPct40,
    PriceBandId::LevelPct50,
    PriceBandId::LevelPct60,
    PriceBandId::LevelPct70,
    PriceBandId::LevelPct80,
    PriceBandId::LevelPct90,
];

pub(super) const LEVEL_IDS: [LevelId; LEVEL_COUNT] = [
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

impl ColumnId for PriceBandId {
    type Row<T>
        = PriceBands<T>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &PRICE_BAND_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        self.select(row)
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        self.select_mut(row)
    }

    #[inline]
    fn from_fn<T, F>(create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        PriceBands::from_fn(create)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

impl_named_row_formattable!(Percentiles {
    pct95,
    pct98,
    pct99,
    pct99_5,
    pct99_9,
});

impl<T> Percentiles<T> {
    pub(super) fn from_fn(mut create: impl FnMut(LossPercentileId) -> T) -> Self {
        Self {
            pct95: create(LossPercentileId::Pct95),
            pct98: create(LossPercentileId::Pct98),
            pct99: create(LossPercentileId::Pct99),
            pct99_5: create(LossPercentileId::Pct99_5),
            pct99_9: create(LossPercentileId::Pct99_9),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.pct95,
            &self.pct98,
            &self.pct99,
            &self.pct99_5,
            &self.pct99_9,
        ]
        .into_iter()
    }

    fn map<U>(self, mut map: impl FnMut(T) -> U) -> Percentiles<U> {
        Percentiles {
            pct95: map(self.pct95),
            pct98: map(self.pct98),
            pct99: map(self.pct99),
            pct99_5: map(self.pct99_5),
            pct99_9: map(self.pct99_9),
        }
    }
}

impl_named_row_formattable!(Levels {
    pct10,
    pct20,
    pct30,
    pct40,
    pct50,
    pct60,
    pct70,
    pct80,
    pct90,
});

impl<T> Levels<T> {
    pub(super) fn from_fn(mut create: impl FnMut(LevelId) -> T) -> Self {
        Self {
            pct10: create(LevelId::Pct10),
            pct20: create(LevelId::Pct20),
            pct30: create(LevelId::Pct30),
            pct40: create(LevelId::Pct40),
            pct50: create(LevelId::Pct50),
            pct60: create(LevelId::Pct60),
            pct70: create(LevelId::Pct70),
            pct80: create(LevelId::Pct80),
            pct90: create(LevelId::Pct90),
        }
    }

    fn map<U>(self, mut map: impl FnMut(T) -> U) -> Levels<U> {
        Levels {
            pct10: map(self.pct10),
            pct20: map(self.pct20),
            pct30: map(self.pct30),
            pct40: map(self.pct40),
            pct50: map(self.pct50),
            pct60: map(self.pct60),
            pct70: map(self.pct70),
            pct80: map(self.pct80),
            pct90: map(self.pct90),
        }
    }
}

impl_named_row_formattable!(PriceBands { floor, level });

impl<T> PriceBands<T> {
    pub(super) fn from_fn(mut create: impl FnMut(PriceBandId) -> T) -> Self {
        Self {
            floor: Percentiles {
                pct95: create(PriceBandId::FloorPct95),
                pct98: create(PriceBandId::FloorPct98),
                pct99: create(PriceBandId::FloorPct99),
                pct99_5: create(PriceBandId::FloorPct99_5),
                pct99_9: create(PriceBandId::FloorPct99_9),
            },
            level: Levels::from_fn(|id| create(id.price_band())),
        }
    }

    fn map<U>(self, mut map: impl FnMut(T) -> U) -> PriceBands<U> {
        PriceBands {
            floor: self.floor.map(&mut map),
            level: self.level.map(map),
        }
    }
}

impl LossPercentileId {
    pub(super) fn series<T>(mut create: impl FnMut(Self) -> T) -> Percentiles<T> {
        Percentiles::from_fn(&mut create)
    }
}

impl PriceBandId {
    pub(super) fn series<T>(mut create: impl FnMut(Self) -> T) -> PriceBands<T> {
        PriceBands::from_fn(&mut create)
    }
}

impl<T> WeightedModes<T> {
    pub(super) fn from_fn(mut create: impl FnMut(WeightedModeId) -> T) -> Self {
        Self {
            cointime: create(WeightedModeId::Cointime),
            coinflow: create(WeightedModeId::Coinflow),
            coinflow_8y: create(WeightedModeId::Coinflow8Y),
            coinflow_4y: create(WeightedModeId::Coinflow4Y),
            coinflow_2y: create(WeightedModeId::Coinflow2Y),
            coinflow_1y: create(WeightedModeId::Coinflow1Y),
            coinflow_6m: create(WeightedModeId::Coinflow6M),
            coinflow_3m: create(WeightedModeId::Coinflow3M),
            coinflow_1m: create(WeightedModeId::Coinflow1M),
        }
    }

    pub(super) fn try_from_fn<E>(
        mut create: impl FnMut(WeightedModeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            cointime: create(WeightedModeId::Cointime)?,
            coinflow: create(WeightedModeId::Coinflow)?,
            coinflow_8y: create(WeightedModeId::Coinflow8Y)?,
            coinflow_4y: create(WeightedModeId::Coinflow4Y)?,
            coinflow_2y: create(WeightedModeId::Coinflow2Y)?,
            coinflow_1y: create(WeightedModeId::Coinflow1Y)?,
            coinflow_6m: create(WeightedModeId::Coinflow6M)?,
            coinflow_3m: create(WeightedModeId::Coinflow3M)?,
            coinflow_1m: create(WeightedModeId::Coinflow1M)?,
        })
    }

    pub(super) fn select_mut(&mut self, id: WeightedModeId) -> &mut T {
        match id {
            WeightedModeId::Cointime => &mut self.cointime,
            WeightedModeId::Coinflow => &mut self.coinflow,
            WeightedModeId::Coinflow8Y => &mut self.coinflow_8y,
            WeightedModeId::Coinflow4Y => &mut self.coinflow_4y,
            WeightedModeId::Coinflow2Y => &mut self.coinflow_2y,
            WeightedModeId::Coinflow1Y => &mut self.coinflow_1y,
            WeightedModeId::Coinflow6M => &mut self.coinflow_6m,
            WeightedModeId::Coinflow3M => &mut self.coinflow_3m,
            WeightedModeId::Coinflow1M => &mut self.coinflow_1m,
        }
    }

    pub(super) fn select(&self, id: WeightedModeId) -> &T {
        match id {
            WeightedModeId::Cointime => &self.cointime,
            WeightedModeId::Coinflow => &self.coinflow,
            WeightedModeId::Coinflow8Y => &self.coinflow_8y,
            WeightedModeId::Coinflow4Y => &self.coinflow_4y,
            WeightedModeId::Coinflow2Y => &self.coinflow_2y,
            WeightedModeId::Coinflow1Y => &self.coinflow_1y,
            WeightedModeId::Coinflow6M => &self.coinflow_6m,
            WeightedModeId::Coinflow3M => &self.coinflow_3m,
            WeightedModeId::Coinflow1M => &self.coinflow_1m,
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.cointime,
            &self.coinflow,
            &self.coinflow_8y,
            &self.coinflow_4y,
            &self.coinflow_2y,
            &self.coinflow_1y,
            &self.coinflow_6m,
            &self.coinflow_3m,
            &self.coinflow_1m,
        ]
        .into_iter()
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.cointime,
            &mut self.coinflow,
            &mut self.coinflow_8y,
            &mut self.coinflow_4y,
            &mut self.coinflow_2y,
            &mut self.coinflow_1y,
            &mut self.coinflow_6m,
            &mut self.coinflow_3m,
            &mut self.coinflow_1m,
        ]
        .into_iter()
    }
}

impl<T> Modes<T> {
    pub(super) fn from_fn(mut create: impl FnMut(ModeId) -> T) -> Self {
        Self {
            raw: create(ModeId::Raw),
            weighted: WeightedModes::from_fn(|id| create(id.mode())),
        }
    }

    pub(super) fn try_from_fn<E>(
        mut create: impl FnMut(ModeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            raw: create(ModeId::Raw)?,
            weighted: WeightedModes::try_from_fn(|id| create(id.mode()))?,
        })
    }

    pub(super) fn select_mut(&mut self, id: ModeId) -> &mut T {
        match id {
            ModeId::Raw => &mut self.raw,
            _ => self
                .weighted
                .select_mut(id.weighted().expect("weighted mode")),
        }
    }

    pub(super) fn select(&self, id: ModeId) -> &T {
        match id {
            ModeId::Raw => &self.raw,
            _ => self.weighted.select(id.weighted().expect("weighted mode")),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.raw).chain(self.weighted.iter())
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        std::iter::once(&mut self.raw).chain(self.weighted.iter_mut())
    }
}

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) states_path: PathBuf,

    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub modes: Modes<ModeVecs<M>>,
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use vecdb::ColumnId;

    use super::{
        LOSS_PERCENTILE_IDS, Levels, LossPercentileId, ModeId, Modes, PRICE_BAND_IDS, Percentiles,
        PriceBandId, PriceBands, WeightedModeId,
    };

    #[test]
    fn bedrock_columns_match_public_band_order() {
        assert_eq!(LossPercentileId::ALL, LOSS_PERCENTILE_IDS);
        let percentiles = LossPercentileId::from_fn(|percentile| percentile);
        assert_eq!(
            percentiles,
            Percentiles {
                pct95: LossPercentileId::Pct95,
                pct98: LossPercentileId::Pct98,
                pct99: LossPercentileId::Pct99,
                pct99_5: LossPercentileId::Pct99_5,
                pct99_9: LossPercentileId::Pct99_9,
            },
        );
        assert_eq!(PriceBandId::ALL, PRICE_BAND_IDS);
        let bands = PriceBandId::from_fn(|band| band);
        assert_eq!(
            bands,
            PriceBands {
                floor: Percentiles {
                    pct95: PriceBandId::FloorPct95,
                    pct98: PriceBandId::FloorPct98,
                    pct99: PriceBandId::FloorPct99,
                    pct99_5: PriceBandId::FloorPct99_5,
                    pct99_9: PriceBandId::FloorPct99_9,
                },
                level: Levels {
                    pct10: PriceBandId::LevelPct10,
                    pct20: PriceBandId::LevelPct20,
                    pct30: PriceBandId::LevelPct30,
                    pct40: PriceBandId::LevelPct40,
                    pct50: PriceBandId::LevelPct50,
                    pct60: PriceBandId::LevelPct60,
                    pct70: PriceBandId::LevelPct70,
                    pct80: PriceBandId::LevelPct80,
                    pct90: PriceBandId::LevelPct90,
                },
            },
        );
        let suffixes = LossPercentileId::from_fn(LossPercentileId::suffix);
        assert_eq!(
            suffixes,
            Percentiles {
                pct95: "pct95",
                pct98: "pct98",
                pct99: "pct99",
                pct99_5: "pct99_5",
                pct99_9: "pct99_9",
            },
        );
        let suffixes = PriceBandId::from_fn(PriceBandId::suffix);
        assert_eq!(
            suffixes,
            PriceBands {
                floor: Percentiles {
                    pct95: "floor_pct95",
                    pct98: "floor_pct98",
                    pct99: "floor_pct99",
                    pct99_5: "floor_pct99_5",
                    pct99_9: "floor_pct99_9",
                },
                level: Levels {
                    pct10: "level_pct10",
                    pct20: "level_pct20",
                    pct30: "level_pct30",
                    pct40: "level_pct40",
                    pct50: "level_pct50",
                    pct60: "level_pct60",
                    pct70: "level_pct70",
                    pct80: "level_pct80",
                    pct90: "level_pct90",
                },
            },
        );
    }

    #[test]
    fn mode_ids_match_named_fields_and_storage_names() {
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
