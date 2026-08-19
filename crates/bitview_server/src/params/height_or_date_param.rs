use brk_types::{Date, Height};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Error;

/// Path parameter accepting either a block height (`840000`) or a calendar date
/// (`YYYY-MM-DD`). The handler resolves it and dispatches to the per-height or
/// per-day variant, choosing the matching cache strategy.
#[derive(Deserialize, JsonSchema)]
pub struct HeightOrDateParam {
    /// Confirmed block height as decimal digits (`840000`) or calendar date in
    /// `YYYY-MM-DD` format.
    #[schemars(example = &"840000")]
    pub point: String,
}

/// A resolved [`HeightOrDateParam`]: a confirmed block height or a calendar day.
pub enum HeightOrDate {
    Height(Height),
    Date(Date),
}

impl HeightOrDateParam {
    /// Parses the raw `point`: a `YYYY-MM-DD` string is a [`Date`], an all-digit
    /// string is a [`Height`], anything else is a 400. Dates are tried first
    /// because their dashes keep them from parsing as a height.
    pub fn resolve(&self) -> Result<HeightOrDate, Error> {
        if let Ok(date) = self.point.parse::<Date>() {
            Ok(HeightOrDate::Date(date))
        } else if let Ok(height) = self.point.parse::<usize>() {
            Ok(HeightOrDate::Height(Height::from(height)))
        } else {
            Err(Error::bad_request(format!(
                "expected a block height or YYYY-MM-DD date, got `{}`",
                self.point
            )))
        }
    }
}
