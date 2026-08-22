use bitview_traversable::Traversable;
use brk_types::{Cents, Version};
use vecdb::{ColumnId, Rw, StorageMode, VecValue};

use super::ema_vecs::EmaVecs;
use bitview_compute::{ColumnarPerBlock, LazyColumnPriceWithRatioPerBlock};

const EMA_PERIOD_COUNT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmaPeriodId {
    OneWeek,
    EightDays,
    TwelveDays,
    ThirteenDays,
    TwentyOneDays,
    TwentySixDays,
    OneMonth,
    ThirtyFourDays,
    FiftyFiveDays,
    EightyNineDays,
    OneHundredFortyFourDays,
    TwoHundredDays,
    OneYear,
    TwoYears,
    TwoHundredWeeks,
    FourYears,
}

const EMA_PERIOD_IDS: [EmaPeriodId; EMA_PERIOD_COUNT] = [
    EmaPeriodId::OneWeek,
    EmaPeriodId::EightDays,
    EmaPeriodId::TwelveDays,
    EmaPeriodId::ThirteenDays,
    EmaPeriodId::TwentyOneDays,
    EmaPeriodId::TwentySixDays,
    EmaPeriodId::OneMonth,
    EmaPeriodId::ThirtyFourDays,
    EmaPeriodId::FiftyFiveDays,
    EmaPeriodId::EightyNineDays,
    EmaPeriodId::OneHundredFortyFourDays,
    EmaPeriodId::TwoHundredDays,
    EmaPeriodId::OneYear,
    EmaPeriodId::TwoYears,
    EmaPeriodId::TwoHundredWeeks,
    EmaPeriodId::FourYears,
];

impl EmaPeriodId {
    pub const fn days(self) -> usize {
        match self {
            Self::OneWeek => 7,
            Self::EightDays => 8,
            Self::TwelveDays => 12,
            Self::ThirteenDays => 13,
            Self::TwentyOneDays => 21,
            Self::TwentySixDays => 26,
            Self::OneMonth => 30,
            Self::ThirtyFourDays => 34,
            Self::FiftyFiveDays => 55,
            Self::EightyNineDays => 89,
            Self::OneHundredFortyFourDays => 144,
            Self::TwoHundredDays => 200,
            Self::OneYear => 365,
            Self::TwoYears => 2 * 365,
            Self::TwoHundredWeeks => 200 * 7,
            Self::FourYears => 4 * 365,
        }
    }

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::OneWeek => "1w",
            Self::EightDays => "8d",
            Self::TwelveDays => "12d",
            Self::ThirteenDays => "13d",
            Self::TwentyOneDays => "21d",
            Self::TwentySixDays => "26d",
            Self::OneMonth => "1m",
            Self::ThirtyFourDays => "34d",
            Self::FiftyFiveDays => "55d",
            Self::EightyNineDays => "89d",
            Self::OneHundredFortyFourDays => "144d",
            Self::TwoHundredDays => "200d",
            Self::OneYear => "1y",
            Self::TwoYears => "2y",
            Self::TwoHundredWeeks => "200w",
            Self::FourYears => "4y",
        }
    }

    pub fn series<T>(mut create: impl FnMut(Self) -> T) -> EmaVecs<T> {
        EmaVecs {
            _1w: create(Self::OneWeek),
            _8d: create(Self::EightDays),
            _12d: create(Self::TwelveDays),
            _13d: create(Self::ThirteenDays),
            _21d: create(Self::TwentyOneDays),
            _26d: create(Self::TwentySixDays),
            _1m: create(Self::OneMonth),
            _34d: create(Self::ThirtyFourDays),
            _55d: create(Self::FiftyFiveDays),
            _89d: create(Self::EightyNineDays),
            _144d: create(Self::OneHundredFortyFourDays),
            _200d: create(Self::TwoHundredDays),
            _1y: create(Self::OneYear),
            _2y: create(Self::TwoYears),
            _200w: create(Self::TwoHundredWeeks),
            _4y: create(Self::FourYears),
        }
    }
}

impl ColumnId for EmaPeriodId {
    type Row<T>
        = [T; EMA_PERIOD_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &EMA_PERIOD_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| create(EMA_PERIOD_IDS[index]))
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

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Simple moving averages of block-level Bitcoin spot prices over trailing
    /// monotonic-time windows, including the represented block.
    pub sma: super::sma::SmaVecs,
    /// Exponential moving average of block-level Bitcoin spot price. At each
    /// block it recursively applies `alpha = 2 / (span + 1)`, where `span` is
    /// the number of blocks from the trailing period's monotonic-time start
    /// through the represented block.
    pub ema: ColumnarPerBlock<
        Cents,
        EmaPeriodId,
        EmaVecs<LazyColumnPriceWithRatioPerBlock<EmaPeriodId>>,
        M,
    >,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{EMA_PERIOD_IDS, EmaPeriodId};

    #[test]
    fn ema_columns_match_public_fields() {
        assert_eq!(EmaPeriodId::ALL, EMA_PERIOD_IDS);

        let series = EmaPeriodId::series(|period| period);
        assert_eq!(series._1w, EmaPeriodId::OneWeek);
        assert_eq!(series._8d, EmaPeriodId::EightDays);
        assert_eq!(series._12d, EmaPeriodId::TwelveDays);
        assert_eq!(series._13d, EmaPeriodId::ThirteenDays);
        assert_eq!(series._21d, EmaPeriodId::TwentyOneDays);
        assert_eq!(series._26d, EmaPeriodId::TwentySixDays);
        assert_eq!(series._1m, EmaPeriodId::OneMonth);
        assert_eq!(series._34d, EmaPeriodId::ThirtyFourDays);
        assert_eq!(series._55d, EmaPeriodId::FiftyFiveDays);
        assert_eq!(series._89d, EmaPeriodId::EightyNineDays);
        assert_eq!(series._144d, EmaPeriodId::OneHundredFortyFourDays);
        assert_eq!(series._200d, EmaPeriodId::TwoHundredDays);
        assert_eq!(series._1y, EmaPeriodId::OneYear);
        assert_eq!(series._2y, EmaPeriodId::TwoYears);
        assert_eq!(series._200w, EmaPeriodId::TwoHundredWeeks);
        assert_eq!(series._4y, EmaPeriodId::FourYears);
    }
}
