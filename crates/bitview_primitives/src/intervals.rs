use std::ops::Add;

use brk_types::{CheckedSub, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::INDEX_EPOCH;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex, VecIndex};

/// A fixed-length period index counted from [`INDEX_EPOCH`], with its `Index` name and aliases.
macro_rules! fixed_interval {
    (
        $(#[$meta:meta])* $name:ident,
        $interval_vis:vis $interval:ident = $seconds:literal,
        $index_name:literal, [$($alias:literal),+]
    ) => {
        $interval_vis const $interval: u32 = $seconds;

        $(#[$meta])*
        #[derive(
            Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
        )]
        #[cfg_attr(feature = "storage", derive(Pco))]
        pub struct $name(u32);

        impl $name {
            pub fn from_timestamp(ts: Timestamp) -> Self {
                Self((*ts - INDEX_EPOCH) / $interval)
            }

            pub fn to_timestamp(&self) -> Timestamp {
                Timestamp::new(INDEX_EPOCH + self.0 * $interval)
            }

            pub(crate) fn index_name() -> &'static str {
                $index_name
            }

            pub(crate) fn index_aliases() -> &'static [&'static str] {
                &[$($alias),+]
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(value: $name) -> Self {
                value.0 as usize
            }
        }

        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                Self(value as u32)
            }
        }

        impl Add<usize> for $name {
            type Output = Self;
            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs as u32)
            }
        }

        impl CheckedSub for $name {
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }
        }

        #[cfg(feature = "storage")]
        impl PrintableIndex for $name {
            fn to_string() -> &'static str {
                Self::index_name()
            }
        }

        #[cfg(feature = "storage")]
        impl VecIndex for $name {}

        brk_types::impl_int_format!($name);
    };
}

fixed_interval!(
    /// Index of a 10-minute period.
    Minute10, pub(crate) MINUTE10_INTERVAL = 600, "minute10", ["10mn", "minute10"]
);
fixed_interval!(
    /// Index of a 30-minute period.
    Minute30, pub(crate) MINUTE30_INTERVAL = 1800, "minute30", ["30mn", "minute30"]
);
fixed_interval!(
    /// Index of an hour.
    Hour1, pub(crate) HOUR1_INTERVAL = 3600, "hour1", ["1h", "hour", "hourly", "hour1"]
);
fixed_interval!(
    /// Index of a 4-hour period.
    Hour4, pub HOUR4_INTERVAL = 14400, "hour4", ["4h", "hour4"]
);
fixed_interval!(
    /// Index of a 12-hour period.
    Hour12, pub(crate) HOUR12_INTERVAL = 43200, "hour12", ["12h", "hour12"]
);
