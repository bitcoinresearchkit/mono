use brk_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::Date;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Bitcoin year (2009, 2010, ..., 2025+)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Year(u16);

brk_types::impl_integer_index!(Year, u16);

impl From<Timestamp> for Year {
    #[inline]
    fn from(value: Timestamp) -> Self {
        Self(Date::from(value).year())
    }
}

impl From<Date> for Year {
    #[inline]
    fn from(value: Date) -> Self {
        Self(value.year())
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Year {
    fn to_string() -> &'static str {
        "year"
    }
}
