use std::fmt::{self, Debug};

use brk_error::Error;
use brk_types::{Height, ONE_DAY_IN_SEC, Timestamp, TxIndex};
use schemars::JsonSchema;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, Visitor},
};

use crate::{
    Date, Day1, Day3, DifficultyEpoch, EmptyScriptOutputIndex, HOUR1_INTERVAL, HOUR4_INTERVAL,
    HOUR12_INTERVAL, HalvingEpoch, Hour1, Hour4, Hour12, MINUTE10_INTERVAL, MINUTE30_INTERVAL,
    Minute10, Minute30, Month1, Month3, Month6, OpReturnOutputIndex, P2AAddressIndex,
    P2MSOutputIndex, P2PK33AddressIndex, P2PK65AddressIndex, P2PKHAddressIndex, P2SHAddressIndex,
    P2TRAddressIndex, P2WPKHAddressIndex, P2WSHAddressIndex, TxInIndex, TxOutIndex,
    UnknownScriptOutputIndex, Week1, Year1, Year10,
};

/// 2009-01-01 00:00:00 UTC — epoch for fixed-interval time indexes.
pub const INDEX_EPOCH: u32 = 1230768000;

/// How a date-based [`Index`] buckets time, for clients that convert dates themselves (bindgen
/// emits it); agrees with the index types' own conversions. Bucket `i` starts `i * size - phase`
/// units after 2009-01-01 00:00 UTC, except that a month calendar's bucket 0 starts at January
/// 2009.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calendar {
    /// Buckets of this many seconds (sub-daily indexes).
    Seconds(u32),
    /// Buckets of whole UTC days.
    Days { size: u32, phase: u32 },
    /// Buckets of calendar months.
    Months { size: u32, phase: u32 },
}

/// Expands the resolution family in its canonical order, [`Index`]'s too: the date-based
/// periods (time buckets, which may be empty), then the block-count epochs (never empty). The
/// callback receives each field name, index type and container type parameter, then any tokens
/// passed after the callback.
#[macro_export]
macro_rules! with_resolution_fields {
    ($callback:ident $(, $($rest:tt)*)?) => {
        $callback! {
            periods {
                minute10: Minute10 => M10,
                minute30: Minute30 => M30,
                hour1: Hour1 => H1,
                hour4: Hour4 => H4,
                hour12: Hour12 => H12,
                day1: Day1 => D1,
                day3: Day3 => D3,
                week1: Week1 => W1,
                month1: Month1 => Mo1,
                month3: Month3 => Mo3,
                month6: Month6 => Mo6,
                year1: Year1 => Y1,
                year10: Year10 => Y10,
            }
            epochs {
                halving_epoch: HalvingEpoch => HE,
                difficulty_epoch: DifficultyEpoch => DE,
            }
            $($($rest)*)?
        }
    };
}

/// Declares [`Index`] from the resolution family followed by the `other` index types, each
/// variant named after its type, with every period's [`Calendar`].
macro_rules! indexes {
    (
        periods { $($period_field:ident: $period:ident => $period_param:ident,)* }
        epochs { $($epoch_field:ident: $epoch:ident => $epoch_param:ident,)* }
        calendars { $($calendar:ident => $kind:ident $value:tt,)* }
        other: [$($(#[$attr:meta])* $other:ident),+ $(,)?]
    ) => {
        /// Aggregation dimension for querying series. Includes time-based (date, week, month, year),
        /// block-based (height, tx_index), and address/output type indexes. Written by its name
        /// (`Index::name`) everywhere: JSON, the OpenAPI enum and `Display`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Index {
            $($period,)*
            $($epoch,)*
            $($(#[$attr])* $other,)+
        }

        impl Index {
            const COUNT: usize = [$(Self::$period,)* $(Self::$epoch,)* $(Self::$other,)+].len();

            pub const fn all() -> [Self; Self::COUNT] {
                [$(Self::$period,)* $(Self::$epoch,)* $(Self::$other,)+]
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$period => $period::index_name(),)*
                    $(Self::$epoch => $epoch::index_name(),)*
                    $(Self::$other => $other::index_name(),)+
                }
            }

            pub const fn is_date_based(&self) -> bool {
                matches!(self, $(Self::$period)|*)
            }

            /// How this index buckets time; `None` unless date-based.
            pub const fn calendar(self) -> Option<Calendar> {
                match self {
                    $(Self::$calendar => Some(Calendar::$kind $value),)*
                    $(Self::$epoch => None,)*
                    $(Self::$other => None,)+
                }
            }
        }
    };
}

with_resolution_fields!(indexes, calendars {
    Minute10 => Seconds(MINUTE10_INTERVAL),
    Minute30 => Seconds(MINUTE30_INTERVAL),
    Hour1 => Seconds(HOUR1_INTERVAL),
    Hour4 => Seconds(HOUR4_INTERVAL),
    Hour12 => Seconds(HOUR12_INTERVAL),
    Day1 => Days { size: 1, phase: 0 },
    // The first bucket starts on 2008-12-31.
    Day3 => Days { size: 3, phase: 1 },
    // ISO weeks: 2009-W01 starts on 2008-12-29.
    Week1 => Days { size: 7, phase: 3 },
    Month1 => Months { size: 1, phase: 0 },
    Month3 => Months { size: 3, phase: 0 },
    Month6 => Months { size: 6, phase: 0 },
    Year1 => Months { size: 12, phase: 0 },
    // Calendar decades, 2009 alone in the first.
    Year10 => Months { size: 120, phase: 108 },
} other: [
    Height,
    TxIndex,
    TxInIndex,
    TxOutIndex,
    EmptyScriptOutputIndex,
    OpReturnOutputIndex,
    P2AAddressIndex,
    P2MSOutputIndex,
    P2PK33AddressIndex,
    P2PK65AddressIndex,
    P2PKHAddressIndex,
    P2SHAddressIndex,
    P2TRAddressIndex,
    P2WPKHAddressIndex,
    P2WSHAddressIndex,
    UnknownScriptOutputIndex,
]);

impl Index {
    /// Convert an index value to a timestamp for time-based indexes.
    /// Returns None for non-time-based indexes.
    pub fn index_to_timestamp(&self, i: usize) -> Option<Timestamp> {
        match self.calendar()? {
            Calendar::Seconds(size) => Some(Timestamp::new(INDEX_EPOCH + i as u32 * size)),
            Calendar::Days { size, phase } => Some(Timestamp::new(
                INDEX_EPOCH - phase * ONE_DAY_IN_SEC + i as u32 * size * ONE_DAY_IN_SEC,
            )),
            Calendar::Months { .. } => self.index_to_date(i).map(Timestamp::from),
        }
    }

    /// Convert a date to an index value for day-precision or coarser indexes.
    /// Returns None for sub-daily indexes (use `timestamp_to_index` instead),
    /// non-date-based indexes, or dates before the index's first one (2009-01-01;
    /// day3's is 2008-12-31, week1's 2008-12-29).
    pub fn date_to_index(&self, date: Date) -> Option<usize> {
        if date < Date::from(self.index_to_timestamp(0)?) {
            return None;
        }
        match self {
            Self::Day1 => Day1::try_from(date).ok().map(usize::from),
            Self::Day3 => Day3::try_from(date).ok().map(usize::from),
            Self::Week1 => Week1::try_from(date).ok().map(usize::from),
            Self::Month1 => Month1::try_from(date).ok().map(usize::from),
            Self::Month3 => Month3::try_from(date).ok().map(usize::from),
            Self::Month6 => Month6::try_from(date).ok().map(usize::from),
            Self::Year1 => Year1::try_from(date).ok().map(usize::from),
            Self::Year10 => Year10::try_from(date).ok().map(usize::from),
            _ => None,
        }
    }

    /// Convert a timestamp to an index value for any date-based index.
    /// Returns None for non-date-based indexes or times before the index's first one.
    pub fn timestamp_to_index(&self, ts: Timestamp) -> Option<usize> {
        if ts < self.index_to_timestamp(0)? {
            return None;
        }
        match self.calendar()? {
            Calendar::Seconds(size) => Some((*ts - INDEX_EPOCH) as usize / size as usize),
            Calendar::Days { size, phase } => Some(
                (*ts - (INDEX_EPOCH - phase * ONE_DAY_IN_SEC)) as usize
                    / (size * ONE_DAY_IN_SEC) as usize,
            ),
            Calendar::Months { .. } => self.date_to_index(Date::from(ts)),
        }
    }

    /// Convert an index value to a date for day-precision or coarser indexes.
    /// Returns None for sub-daily indexes (use `index_to_timestamp` instead)
    /// and non-date-based indexes.
    pub fn index_to_date(&self, i: usize) -> Option<Date> {
        match self {
            Self::Day1 => Some(Date::from(Day1::from(i))),
            Self::Day3 => Some(Date::from(Day3::from(i).to_timestamp())),
            Self::Week1 => Some(Date::from(Week1::from(i))),
            Self::Month1 => Some(Date::from(Month1::from(i))),
            Self::Year1 => Some(Date::from(Year1::from(i))),
            Self::Month3 => Some(Date::from(Month3::from(i))),
            Self::Month6 => Some(Date::from(Month6::from(i))),
            Self::Year10 => Some(Date::from(Year10::from(i))),
            _ => None,
        }
    }
}

impl TryFrom<&str> for Index {
    type Error = Error;
    /// The index named exactly `value`; an error naming every index otherwise.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::all()
            .into_iter()
            .find(|index| index.name() == value)
            .ok_or_else(|| {
                let names: Vec<&str> = Self::all().iter().map(Self::name).collect();
                Error::Parse(format!(
                    "unknown index `{value}`, expected one of {}",
                    names.join(", ")
                ))
            })
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Serialize for Index {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl JsonSchema for Index {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Index".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let names: Vec<&str> = Self::all().iter().map(Self::name).collect();
        schemars::json_schema!({
            "description": "Aggregation dimension for querying series. Includes time-based (date, week, month, year), block-based (height, tx_index), and address/output type indexes.",
            "type": "string",
            "enum": names,
            "examples": ["day1"],
        })
    }
}

impl<'de> Deserialize<'de> for Index {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IndexVisitor;

        impl Visitor<'_> for IndexVisitor {
            type Value = Index;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an index name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Index::try_from(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(IndexVisitor)
    }
}
