use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, VecValue};

pub trait DailyValue: VecValue + Formattable + JsonSchema + Serialize {}

impl<T> DailyValue for T where T: VecValue + Formattable + JsonSchema + Serialize {}
