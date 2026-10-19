use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use bitview_primitives::{Date, Index};
use bitview_types::SeriesData;
use brk_types::Timestamp;
use serde::de::DeserializeOwned;

use crate::{BitviewClientBase, BitviewError, DateSeriesData, Result, client_base::path_segment};

/// Parameterized by element type `T` and response type `D` (defaults to `SeriesData<T>`).
/// For date-based indexes, `DateSeriesEndpoint<T>` sets `D = DateSeriesData<T>`. Selectors
/// narrow the request; the last one of a kind wins.
pub struct SeriesEndpoint<T, D = SeriesData<T>> {
    client: Arc<BitviewClientBase>,
    name: Arc<str>,
    index: Index,
    start: Option<i64>,
    end: Option<i64>,
    limit: Option<usize>,
    _marker: PhantomData<fn() -> (T, D)>,
}

/// Date-based series endpoint, with date and timestamp selectors.
pub type DateSeriesEndpoint<T> = SeriesEndpoint<T, DateSeriesData<T>>;

impl<T, D: DeserializeOwned> SeriesEndpoint<T, D> {
    pub fn new(client: Arc<BitviewClientBase>, name: Arc<str>, index: Index) -> Self {
        Self {
            client,
            name,
            index,
            start: None,
            end: None,
            limit: None,
            _marker: PhantomData,
        }
    }

    /// Select a specific index position.
    pub fn get(self, index: usize) -> Self {
        self.range(index..=index)
    }

    /// # Examples
    /// ```ignore
    /// endpoint.range(..10)      // first 10
    /// endpoint.range(100..110)  // indices 100-109
    /// endpoint.range(100..)     // from 100 to end
    /// ```
    pub fn range<R: RangeBounds<usize>>(mut self, range: R) -> Self {
        self.start = match range.start_bound() {
            Bound::Included(&n) => Some(signed(n)),
            Bound::Excluded(&n) => Some(signed(n).saturating_add(1)),
            Bound::Unbounded => None,
        };
        self.end = match range.end_bound() {
            Bound::Included(&n) => Some(signed(n).saturating_add(1)),
            Bound::Excluded(&n) => Some(signed(n)),
            Bound::Unbounded => None,
        };
        self.limit = None;
        self
    }

    /// Take n items from the selected start (the first n without one).
    pub fn take(mut self, n: usize) -> Self {
        match self.end {
            // The server gives `end` precedence over `limit`: narrow the end instead.
            Some(end) => {
                self.end = Some(end.min(self.start.unwrap_or(0).saturating_add(signed(n))))
            }
            None => self.limit = Some(n),
        }
        self
    }

    pub fn last(mut self, n: usize) -> Self {
        (self.start, self.end) = if n == 0 {
            (None, Some(0))
        } else {
            (Some(-signed(n)), None)
        };
        self.limit = None;
        self
    }

    /// Skip the first n items. Chain with `take(n)` for a count.
    pub fn skip(mut self, n: usize) -> Self {
        self.start = Some(signed(n));
        self
    }

    pub fn fetch(&self) -> Result<D> {
        self.client.get_json(&self.query_path(None))
    }

    pub fn fetch_csv(&self) -> Result<String> {
        self.client.get_text(&self.query_path(Some("csv")))
    }

    /// Total number of data points for this series.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<i64> {
        self.client.get_json(&format!("{}/len", self.path()))
    }

    pub fn version(&self) -> Result<u32> {
        self.client.get_json(&format!("{}/version", self.path()))
    }

    /// The base endpoint path.
    pub fn path(&self) -> String {
        format!(
            "/api/series/{}/{}",
            path_segment(&self.name),
            self.index.name()
        )
    }

    /// Counted selections send `limit`: the server ends them after resolving a negative start.
    fn query_path(&self, format: Option<&str>) -> String {
        let mut params = Vec::new();
        if let Some(start) = self.start {
            params.push(format!("start={start}"));
        }
        if let Some(end) = self.end {
            params.push(format!("end={end}"));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(format) = format {
            params.push(format!("format={format}"));
        }
        let path = self.path();
        if params.is_empty() {
            path
        } else {
            format!("{path}?{}", params.join("&"))
        }
    }
}

/// A count or position as a query value; past `i64::MAX` it saturates (the range ends there).
fn signed(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn required_date_index(position: Option<usize>) -> Result<usize> {
    position
        .ok_or_else(|| BitviewError::new("Date or timestamp is not representable for this index"))
}

impl<T: DeserializeOwned> DateSeriesEndpoint<T> {
    /// Select a specific date position (for day-precision or coarser indexes).
    pub fn get_date(self, date: Date) -> Result<Self> {
        let index = required_date_index(self.index.date_to_index(date))?;
        Ok(self.get(index))
    }

    /// Select a date range (for day-precision or coarser indexes).
    pub fn date_range(self, start: Date, end: Date) -> Result<Self> {
        let start = required_date_index(self.index.date_to_index(start))?;
        let end = required_date_index(self.index.date_to_index(end))?;
        Ok(self.range(start..end))
    }

    /// Select a specific timestamp position (works for all date-based indexes including sub-daily).
    pub fn get_timestamp(self, timestamp: Timestamp) -> Result<Self> {
        let index = required_date_index(self.index.timestamp_to_index(timestamp))?;
        Ok(self.get(index))
    }

    /// Select a timestamp range (works for all date-based indexes including sub-daily).
    pub fn timestamp_range(self, start: Timestamp, end: Timestamp) -> Result<Self> {
        let start = required_date_index(self.index.timestamp_to_index(start))?;
        let end = required_date_index(self.index.timestamp_to_index(end))?;
        Ok(self.range(start..end))
    }
}
