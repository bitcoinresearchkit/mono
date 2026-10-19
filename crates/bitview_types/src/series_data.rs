use bitview_primitives::{Date, Index};
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

/// JSON response of `GET /api/series/{series}/{index}`; bulk queries return an array of these.
#[derive(Debug, JsonSchema, Deserialize)]
pub struct SeriesData<T = Value> {
    pub version: u32,
    pub index: Index,
    /// Value type (e.g. "Ratio", "Sats", "Cents")
    #[serde(rename = "type", default)]
    pub value_type: String,
    /// Start index (inclusive) of the returned range
    pub start: usize,
    /// End index (exclusive) of the returned range
    pub end: usize,
    /// The series data. A value is `null` where it is missing (a period without blocks) or
    /// undefined (e.g. NaN, a ratio over zero); CSV leaves such a cell empty.
    pub data: Vec<T>,
}

impl<T> SeriesData<T> {
    fn dates(&self) -> Option<impl Iterator<Item = Date> + '_> {
        // Check first index to verify date conversion works (sub-daily returns None)
        self.index.index_to_date(self.start)?;
        let index = self.index;
        Some((self.start..self.end).map(move |i| index.index_to_date(i).unwrap()))
    }

    fn timestamps(&self) -> Option<impl Iterator<Item = Timestamp> + '_> {
        if !self.index.is_date_based() {
            return None;
        }
        let index = self.index;
        Some((self.start..self.end).map(move |i| index.index_to_timestamp(i).unwrap()))
    }

    /// `None` for non-date-based and sub-daily indexes (use `iter_timestamps()` instead).
    pub fn iter_dates(&self) -> Option<impl Iterator<Item = (Date, &T)> + '_> {
        Some(self.dates()?.zip(self.data.iter()))
    }

    /// Works for all date-based indexes including sub-daily; `None` for the others.
    pub fn iter_timestamps(&self) -> Option<impl Iterator<Item = (Timestamp, &T)> + '_> {
        Some(self.timestamps()?.zip(self.data.iter()))
    }
}
