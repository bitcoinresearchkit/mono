use std::ops::Deref;

use brk_types::Timestamp;
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error as _},
};

use bitview_types::SeriesData;

/// `SeriesData` whose index is date-based, so [`iter_timestamps`](Self::iter_timestamps) cannot
/// fail. Date methods, through `Deref`, stay optional for sub-daily indexes.
#[derive(Debug)]
pub struct DateSeriesData<T>(SeriesData<T>);

impl<T> DateSeriesData<T> {
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
        if inner.index.is_date_based() {
            Ok(Self(inner))
        } else {
            Err(D::Error::custom(format!(
                "expected date-based index, got {:?}",
                inner.index
            )))
        }
    }
}
