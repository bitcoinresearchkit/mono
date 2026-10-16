use std::fmt::{self, Debug};

use brk_error::Error;
use brk_types::{Height, Timestamp, TxIndex};
use schemars::JsonSchema;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

use crate::{
    Date, Day1, Day3, EmptyOutputIndex, Epoch, HOUR1_INTERVAL, HOUR4_INTERVAL, HOUR12_INTERVAL,
    Halving, Hour1, Hour4, Hour12, MINUTE10_INTERVAL, MINUTE30_INTERVAL, Minute10, Minute30,
    Month1, Month3, Month6, OpReturnIndex, P2AAddressIndex, P2MSOutputIndex, P2PK33AddressIndex,
    P2PK65AddressIndex, P2PKHAddressIndex, P2SHAddressIndex, P2TRAddressIndex, P2WPKHAddressIndex,
    P2WSHAddressIndex, TxInIndex, TxOutIndex, UnknownOutputIndex, Week1, Year1, Year10,
};

/// 2009-01-01 00:00:00 UTC — epoch for fixed-interval time indexes.
pub const INDEX_EPOCH: u32 = 1230768000;

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
                halving: Halving => HE,
                epoch: Epoch => DE,
            }
            $($($rest)*)?
        }
    };
}

/// Declares [`Index`] from the resolution family followed by the `other` index types, each
/// variant named after its type. Generates the enum, `all`, `possible_values`, `name` and
/// `is_date_based`.
macro_rules! indexes {
    (
        periods { $($period_field:ident: $period:ident => $period_param:ident,)* }
        epochs { $($epoch_field:ident: $epoch:ident => $epoch_param:ident,)* }
        other: [$($(#[$attr:meta])* $other:ident),+ $(,)?]
    ) => {
        /// Aggregation dimension for querying series. Includes time-based (date, week, month, year),
        /// block-based (height, tx_index), and address/output type indexes.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
        #[serde(rename_all = "lowercase")]
        #[schemars(example = Index::Day1)]
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

            pub fn possible_values(&self) -> &'static [&'static str] {
                match self {
                    $(Self::$period => $period::index_aliases(),)*
                    $(Self::$epoch => $epoch::index_aliases(),)*
                    $(Self::$other => $other::index_aliases(),)+
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$period => $period::index_name(),)*
                    $(Self::$epoch => $epoch::index_name(),)*
                    $(Self::$other => $other::index_name(),)+
                }
            }

            /// Returns true if this index type is date-based.
            pub const fn is_date_based(&self) -> bool {
                matches!(self, $(Self::$period)|*)
            }
        }
    };
}

with_resolution_fields!(indexes, other: [
    Height,
    #[serde(rename = "tx_index")]
    TxIndex,
    #[serde(rename = "txin_index")]
    TxInIndex,
    #[serde(rename = "txout_index")]
    TxOutIndex,
    #[serde(rename = "empty_output_index")]
    EmptyOutputIndex,
    #[serde(rename = "op_return_index")]
    OpReturnIndex,
    #[serde(rename = "p2a_address_index")]
    P2AAddressIndex,
    #[serde(rename = "p2ms_output_index")]
    P2MSOutputIndex,
    #[serde(rename = "p2pk33_address_index")]
    P2PK33AddressIndex,
    #[serde(rename = "p2pk65_address_index")]
    P2PK65AddressIndex,
    #[serde(rename = "p2pkh_address_index")]
    P2PKHAddressIndex,
    #[serde(rename = "p2sh_address_index")]
    P2SHAddressIndex,
    #[serde(rename = "p2tr_address_index")]
    P2TRAddressIndex,
    #[serde(rename = "p2wpkh_address_index")]
    P2WPKHAddressIndex,
    #[serde(rename = "p2wsh_address_index")]
    P2WSHAddressIndex,
    #[serde(rename = "unknown_output_index")]
    UnknownOutputIndex,
]);

impl Index {
    /// Convert an index value to a timestamp for time-based indexes.
    /// Returns None for non-time-based indexes.
    pub fn index_to_timestamp(&self, i: usize) -> Option<Timestamp> {
        let interval = match self {
            Self::Minute10 => MINUTE10_INTERVAL,
            Self::Minute30 => MINUTE30_INTERVAL,
            Self::Hour1 => HOUR1_INTERVAL,
            Self::Hour4 => HOUR4_INTERVAL,
            Self::Hour12 => HOUR12_INTERVAL,
            Self::Day3 => return Some(Day3::from(i).to_timestamp()),
            _ => return self.index_to_date(i).map(|d| d.into()),
        };
        Some(Timestamp::new(INDEX_EPOCH + i as u32 * interval))
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
    /// Works for both sub-daily (minute, hour) and daily+ indexes.
    /// Returns None for non-date-based indexes or times before the index's first one.
    pub fn timestamp_to_index(&self, ts: Timestamp) -> Option<usize> {
        if ts < self.index_to_timestamp(0)? {
            return None;
        }
        let interval = match self {
            Self::Minute10 => MINUTE10_INTERVAL,
            Self::Minute30 => MINUTE30_INTERVAL,
            Self::Hour1 => HOUR1_INTERVAL,
            Self::Hour4 => HOUR4_INTERVAL,
            Self::Hour12 => HOUR12_INTERVAL,
            Self::Day3 => return Some(usize::from(Day3::from_timestamp(ts))),
            _ => return self.date_to_index(Date::from(ts)),
        };
        Some((*ts).saturating_sub(INDEX_EPOCH) as usize / interval as usize)
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
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::all()
            .into_iter()
            .find(|index| {
                index
                    .possible_values()
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(value))
            })
            .ok_or_else(|| Error::Parse(format!("Invalid index: {value}")))
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
