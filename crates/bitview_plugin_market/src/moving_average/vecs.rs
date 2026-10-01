use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPriceWithRatioPerBlock};
use brk_types::{Cents, Height, StoredU64};
use vecdb::{Rw, StorageMode};

use super::{ema_vecs::EmaVecs, sma::SmaVecs};

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
    pub const ALL: &'static [Self] = &EMA_PERIOD_IDS;
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

    pub fn try_series<T, E>(mut create: impl FnMut(Self) -> Result<T, E>) -> Result<EmaVecs<T>, E> {
        Ok(EmaVecs {
            _1w: create(Self::OneWeek)?,
            _8d: create(Self::EightDays)?,
            _12d: create(Self::TwelveDays)?,
            _13d: create(Self::ThirteenDays)?,
            _21d: create(Self::TwentyOneDays)?,
            _26d: create(Self::TwentySixDays)?,
            _1m: create(Self::OneMonth)?,
            _34d: create(Self::ThirtyFourDays)?,
            _55d: create(Self::FiftyFiveDays)?,
            _89d: create(Self::EightyNineDays)?,
            _144d: create(Self::OneHundredFortyFourDays)?,
            _200d: create(Self::TwoHundredDays)?,
            _1y: create(Self::OneYear)?,
            _2y: create(Self::TwoYears)?,
            _200w: create(Self::TwoHundredWeeks)?,
            _4y: create(Self::FourYears)?,
        })
    }
    pub fn select<T>(self, values: &EmaVecs<T>) -> &T {
        match self {
            Self::OneWeek => &values._1w,
            Self::EightDays => &values._8d,
            Self::TwelveDays => &values._12d,
            Self::ThirteenDays => &values._13d,
            Self::TwentyOneDays => &values._21d,
            Self::TwentySixDays => &values._26d,
            Self::OneMonth => &values._1m,
            Self::ThirtyFourDays => &values._34d,
            Self::FiftyFiveDays => &values._55d,
            Self::EightyNineDays => &values._89d,
            Self::OneHundredFortyFourDays => &values._144d,
            Self::TwoHundredDays => &values._200d,
            Self::OneYear => &values._1y,
            Self::TwoYears => &values._2y,
            Self::TwoHundredWeeks => &values._200w,
            Self::FourYears => &values._4y,
        }
    }

    pub fn select_mut<T>(self, values: &mut EmaVecs<T>) -> &mut T {
        match self {
            Self::OneWeek => &mut values._1w,
            Self::EightDays => &mut values._8d,
            Self::TwelveDays => &mut values._12d,
            Self::ThirteenDays => &mut values._13d,
            Self::TwentyOneDays => &mut values._21d,
            Self::TwentySixDays => &mut values._26d,
            Self::OneMonth => &mut values._1m,
            Self::ThirtyFourDays => &mut values._34d,
            Self::FiftyFiveDays => &mut values._55d,
            Self::EightyNineDays => &mut values._89d,
            Self::OneHundredFortyFourDays => &mut values._144d,
            Self::TwoHundredDays => &mut values._200d,
            Self::OneYear => &mut values._1y,
            Self::TwoYears => &mut values._2y,
            Self::TwoHundredWeeks => &mut values._200w,
            Self::FourYears => &mut values._4y,
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

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Simple moving averages of block-level Bitcoin spot prices over trailing
    /// monotonic-time windows, including the represented block.
    pub sma: SmaVecs,
    /// Exponential moving average of block-level Bitcoin spot price. At each
    /// block it recursively applies `alpha = 2 / (span + 1)`, where `span` is
    /// the number of blocks from the trailing period's monotonic-time start
    /// through the represented block.
    pub(crate) ema: EmaVecs<LazyPriceWithRatioPerBlock>,
    #[traversable(hidden)]
    pub(crate) ema_stored: EmaVecs<CachedSeries<Height, Cents, M>>,
    /// Cumulative integer-cent prices shared by all SMA windows.
    #[traversable(hidden)]
    pub sma_prefix_sum: CachedSeries<Height, StoredU64, M>,
}

#[cfg(test)]
mod tests {
    use super::{EMA_PERIOD_IDS, EmaPeriodId};

    #[test]
    fn ema_windows_match_public_fields() {
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
