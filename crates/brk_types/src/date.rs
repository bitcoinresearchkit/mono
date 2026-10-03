use std::{borrow::Cow, fmt, str::FromStr};

use brk_error::{Error, Result as ErrorResult};
use itoa::Buffer;
use jiff::{Span, Timestamp as JiffTimestamp, civil::Date as Date_, tz::TimeZone};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, Visitor},
};

use super::{Day1, Month1, Month3, Month6, Timestamp, Week1, Year1, Year10};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Date in YYYYMMDD format stored as u32
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Date(u32);

impl JsonSchema for Date {
    fn schema_name() -> Cow<'static, str> {
        "Date".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Calendar date in YYYY-MM-DD format.",
            "type": "string",
            "format": "date",
            "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
            "examples": ["2009-01-03", "2024-04-20"]
        })
    }
}

impl Date {
    pub(crate) const INDEX_ZERO: Self = Self(20090101);
    pub(crate) const INDEX_ZERO_: Date_ = Date_::constant(2009, 1, 1);

    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self(year as u32 * 1_00_00 + month as u32 * 1_00 + day as u32)
    }

    pub fn year(&self) -> u16 {
        (self.0 / 1_00_00) as u16
    }

    pub fn month(&self) -> u8 {
        ((self.0 % 1_00_00) / 1_00) as u8
    }

    pub fn day(&self) -> u8 {
        (self.0 % 1_00) as u8
    }

    pub(crate) fn into_jiff(self) -> Date_ {
        self.into()
    }

    /// Validate dates constructed directly or decoded from persisted values.
    pub fn try_into_jiff(self) -> ErrorResult<Date_> {
        let year = i16::try_from(self.0 / 10_000).map_err(|_| Error::UnindexableDate)?;
        Date_::new(year, self.month() as i8, self.day() as i8).map_err(|_| Error::UnindexableDate)
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::INDEX_ZERO
    }
}

impl From<Date_> for Date {
    #[inline]
    fn from(value: Date_) -> Self {
        Self::new(value.year() as u16, value.month() as u8, value.day() as u8)
    }
}

impl From<Date> for Date_ {
    #[inline]
    fn from(value: Date) -> Self {
        Self::new(value.year() as i16, value.month() as i8, value.day() as i8).unwrap()
    }
}

impl From<Date> for JiffTimestamp {
    #[inline]
    fn from(value: Date) -> Self {
        Self::from(Timestamp::from(value))
    }
}

impl From<Timestamp> for Date {
    #[inline]
    fn from(value: Timestamp) -> Self {
        Self::from(Date_::from(
            JiffTimestamp::from(value).to_zoned(TimeZone::UTC),
        ))
    }
}

impl From<Day1> for Date {
    #[inline]
    fn from(value: Day1) -> Self {
        Self::from(
            Self::INDEX_ZERO_
                .checked_add(Span::new().days(i64::from(value)))
                .unwrap(),
        )
    }
}

impl From<Week1> for Date {
    #[inline]
    fn from(value: Week1) -> Self {
        // Week 0 starts at 2009-01-01, add i weeks
        Self::from(
            Self::INDEX_ZERO_
                .checked_add(Span::new().weeks(i64::from(u16::from(value))))
                .unwrap(),
        )
    }
}

impl From<Month1> for Date {
    #[inline]
    fn from(value: Month1) -> Self {
        // Month 0 is January 2009, add i months
        Self::from(
            Date_::constant(2009, 1, 1)
                .checked_add(Span::new().months(i64::from(u16::from(value))))
                .unwrap(),
        )
    }
}

impl From<Year1> for Date {
    #[inline]
    fn from(value: Year1) -> Self {
        // Year 0 is 2009
        let year = 2009i16 + usize::from(value) as i16;
        Self::from(Date_::constant(year, 1, 1))
    }
}

impl From<Month3> for Date {
    #[inline]
    fn from(value: Month3) -> Self {
        // Quarter 0 is Q1 2009, add i*3 months
        Self::from(
            Date_::constant(2009, 1, 1)
                .checked_add(Span::new().months(usize::from(value) as i64 * 3))
                .unwrap(),
        )
    }
}

impl From<Month6> for Date {
    #[inline]
    fn from(value: Month6) -> Self {
        // Semester 0 is H1 2009, add i*6 months
        Self::from(
            Date_::constant(2009, 1, 1)
                .checked_add(Span::new().months(usize::from(value) as i64 * 6))
                .unwrap(),
        )
    }
}

impl From<Year10> for Date {
    #[inline]
    fn from(value: Year10) -> Self {
        // Decade 0 is 2009, add i*10 years
        let year = 2009i16 + usize::from(value) as i16 * 10;
        Self::from(Date_::constant(year, 1, 1))
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DateVisitor;

        impl Visitor<'_> for DateVisitor {
            type Value = Date;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a date string in YYYY-MM-DD format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(DateVisitor)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Buffer::new();

        let year = buf.format(self.year());
        for _ in year.len()..4 {
            f.write_str("0")?;
        }
        f.write_str(year)?;
        f.write_str("-")?;

        let month = self.month();
        if month < 10 {
            f.write_str("0")?;
        }
        f.write_str(buf.format(month))?;
        f.write_str("-")?;

        let day = self.day();
        if day < 10 {
            f.write_str("0")?;
        }

        f.write_str(buf.format(day))
    }
}

impl FromStr for Date {
    type Err = &'static str;

    /// Parse a date from YYYY-MM-DD format.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes
                .iter()
                .enumerate()
                .any(|(i, byte)| i != 4 && i != 7 && !byte.is_ascii_digit())
        {
            return Err("expected YYYY-MM-DD format");
        }
        let year: u16 = s[0..4].parse().map_err(|_| "invalid year")?;
        let month: u8 = s[5..7].parse().map_err(|_| "invalid month")?;
        let day: u8 = s[8..10].parse().map_err(|_| "invalid day")?;
        Date_::new(year as i16, month as i8, day as i8)
            .map(Self::from)
            .map_err(|_| "invalid calendar date")
    }
}

#[cfg(feature = "storage")]
impl Formattable for Date {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
