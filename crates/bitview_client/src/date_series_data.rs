use std::ops::Deref;

use brk_types::{Date, Timestamp};
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error as _},
};

use bitview_types::SeriesData;

/// Series data that is guaranteed to use a date-based index.
///
/// This is a newtype around `SeriesData<T>` that guarantees `is_date_based()` is true,
/// making date methods infallible.
#[derive(Debug)]
pub struct DateSeriesData<T>(SeriesData<T>);

impl<T> DateSeriesData<T> {
    /// Create a `DateSeriesData` from a `SeriesData`, returning `Err` if the index is not date-based.
    pub fn try_new(inner: SeriesData<T>) -> Result<Self, SeriesData<T>> {
        if inner.is_date_based() {
            Ok(Self(inner))
        } else {
            Err(inner)
        }
    }

    /// Consume and return the inner `SeriesData`.
    pub fn into_inner(self) -> SeriesData<T> {
        self.0
    }

    /// Returns an iterator over dates for the index range.
    /// Returns `None` for sub-daily indexes (use `timestamps()` instead).
    pub fn dates(&self) -> Option<impl Iterator<Item = Date> + '_> {
        self.0.dates()
    }

    /// Iterate over (date, &value) pairs.
    /// Returns `None` for sub-daily indexes (use `iter_timestamps()` instead).
    pub fn iter_dates(&self) -> Option<impl Iterator<Item = (Date, &T)> + '_> {
        self.0.iter_dates()
    }

    /// Returns an iterator over timestamps for the index range (infallible).
    /// Works for all date-based indexes including sub-daily.
    pub fn timestamps(&self) -> impl Iterator<Item = Timestamp> + '_ {
        self.0
            .timestamps()
            .expect("DateSeriesData is always date-based")
    }

    /// Iterate over (timestamp, &value) pairs (infallible).
    /// Works for all date-based indexes including sub-daily.
    pub fn iter_timestamps(&self) -> impl Iterator<Item = (Timestamp, &T)> + '_ {
        self.0
            .iter_timestamps()
            .expect("DateSeriesData is always date-based")
    }
}

impl<T> Deref for DateSeriesData<T> {
    type Target = SeriesData<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for DateSeriesData<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = SeriesData::<T>::deserialize(deserializer)?;
        Self::try_new(inner).map_err(|message| {
            D::Error::custom(format!(
                "expected date-based index, got {:?}",
                message.index
            ))
        })
    }
}
