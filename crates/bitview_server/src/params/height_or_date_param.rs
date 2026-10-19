use bitview_primitives::Date;
use brk_types::Height;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize, JsonSchema)]
pub struct HeightOrDateParam {
    /// Confirmed block height as decimal digits (`840000`) or calendar date in
    /// `YYYY-MM-DD` format.
    #[schemars(example = &"840000")]
    pub point: String,
}

/// A parsed [`HeightOrDateParam`]: a block height or a calendar day.
#[derive(Clone, Copy)]
pub enum HeightOrDate {
    Height(Height),
    Date(Date),
}

impl HeightOrDateParam {
    /// Parses `point` as a `YYYY-MM-DD` date or a `u32` block height; anything else is a 400.
    pub fn resolve(&self) -> Result<HeightOrDate, Error> {
        if let Ok(date) = self.point.parse::<Date>() {
            Ok(HeightOrDate::Date(date))
        } else if let Ok(height) = self.point.parse::<u32>() {
            Ok(HeightOrDate::Height(Height::from(height)))
        } else {
            Err(Error::bad_request(format!(
                "expected a block height or YYYY-MM-DD date, got `{}`",
                self.point
            )))
        }
    }
}
