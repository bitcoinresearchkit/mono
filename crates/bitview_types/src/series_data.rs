use std::ops::Range;

use bitview_primitives::{Date, Index};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

/// Series data with range information.
///
/// All series data endpoints return this structure when format is JSON.
#[derive(Debug, JsonSchema, Deserialize)]
pub struct SeriesData<T = Value> {
    /// Version of the series data
    pub version: u32,
    /// The index type used for this query
    pub index: Index,
    /// Value type (e.g. "Ratio", "Sats", "Cents")
    #[serde(rename = "type", default)]
    pub value_type: String,
    /// Start index (inclusive) of the returned range
    start: usize,
    /// End index (exclusive) of the returned range
    end: usize,
    /// ISO 8601 timestamp of when the response was generated
    pub stamp: String,
    /// The series data. A value is `null` where it is missing (a period without blocks) or
    /// undefined (e.g. NaN, a ratio over zero); CSV leaves such a cell empty.
    pub data: Vec<T>,
}

impl<T> SeriesData<T> {
    /// Returns an iterator over the index range.
    fn indexes(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Returns true if this series uses a date-based index.
    pub fn is_date_based(&self) -> bool {
        self.index.is_date_based()
    }

    /// Returns an iterator over dates for the index range.
    /// Returns `None` for non-date-based and sub-daily indexes (use `timestamps()` instead).
    fn dates(&self) -> Option<impl Iterator<Item = Date> + '_> {
        // Check first index to verify date conversion works (sub-daily returns None)
        self.index.index_to_date(self.start)?;
        let index = self.index;
        Some(self.indexes().map(move |i| index.index_to_date(i).unwrap()))
    }

    /// Returns an iterator over timestamps for the index range.
    /// Works for all date-based indexes including sub-daily.
    /// Returns `None` for non-date-based indexes.
    fn timestamps(&self) -> Option<impl Iterator<Item = Timestamp> + '_> {
        if !self.is_date_based() {
            return None;
        }
        let index = self.index;
        Some(
            self.indexes()
                .map(move |i| index.index_to_timestamp(i).unwrap()),
        )
    }

    /// Iterate over (date, &value) pairs.
    /// Returns `None` for non-date-based and sub-daily indexes (use `iter_timestamps()` instead).
    pub fn iter_dates(&self) -> Option<impl Iterator<Item = (Date, &T)> + '_> {
        Some(self.dates()?.zip(self.data.iter()))
    }

    /// Iterate over (timestamp, &value) pairs.
    /// Works for all date-based indexes including sub-daily.
    /// Returns `None` for non-date-based indexes.
    pub fn iter_timestamps(&self) -> Option<impl Iterator<Item = (Timestamp, &T)> + '_> {
        Some(self.timestamps()?.zip(self.data.iter()))
    }
}
