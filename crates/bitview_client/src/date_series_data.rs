use std::ops::Deref;

use brk_types::Timestamp;
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error as _},
};

use bitview_types::SeriesData;

/// Series data that is guaranteed to use a date-based index.
///
/// This is a newtype around `SeriesData<T>` that guarantees `is_date_based()` is true,
/// making timestamp methods infallible. Date methods are inherited through `Deref`
/// and remain optional for sub-daily indexes.
#[derive(Debug)]
pub struct DateSeriesData<T>(SeriesData<T>);

impl<T> DateSeriesData<T> {
    /// Create a `DateSeriesData` from a `SeriesData`, returning `Err` if the index is not date-based.
    fn try_new(inner: SeriesData<T>) -> Result<Self, SeriesData<T>> {
        if inner.is_date_based() {
            Ok(Self(inner))
        } else {
            Err(inner)
        }
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
