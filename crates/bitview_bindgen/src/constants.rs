//! The constants the JavaScript and Python clients embed: version, indexes, mining pools, cohort
//! names, date calendars and address payload lengths.

use std::collections::BTreeMap;

use bitview_cohort::{
    AGE_RANGE_NAMES, AMOUNT_RANGE_NAMES, ENTRY_NAMES, HALVING_EPOCH_NAMES,
    PROFITABILITY_RANGE_NAMES, SPENDABLE_TYPE_NAMES, TERM_NAMES, VINTAGE_NAMES,
};
use bitview_primitives::{Calendar, Index, pools};
use brk_types::ADDRESS_PAYLOAD_LENGTHS;
use serde::Serialize;
use serde_json::{Map, Value, json, to_string_pretty};

use crate::{VERSION, to_camel_case};

pub struct ClientConstants {
    pub version: String,
    pub indexes: Vec<&'static str>,
    pub pool_map: BTreeMap<String, &'static str>,
}

impl ClientConstants {
    pub fn collect() -> Self {
        let indexes = Index::all();
        let indexes: Vec<&'static str> = indexes.iter().map(|i| i.name()).collect();

        let pool_map: BTreeMap<String, &'static str> = pools()
            .iter()
            .map(|p| (p.slug.to_string(), p.name))
            .collect();

        Self {
            version: format!("v{}", VERSION),
            indexes,
            pool_map,
        }
    }
}

/// Each date-based index's name and calendar as `(unit, size, phase)`: the clients convert
/// dates from these.
pub fn calendars() -> Vec<(&'static str, (&'static str, u32, u32))> {
    Index::all()
        .into_iter()
        .filter_map(|index| {
            let calendar = match index.calendar()? {
                Calendar::Seconds(size) => ("second", size, 0),
                Calendar::Days { size, phase } => ("day", size, phase),
                Calendar::Months { size, phase } => ("month", size, phase),
            };
            Some((index.name(), calendar))
        })
        .collect()
}

/// Each address type's name and payload length in bytes: the clients validate raw payloads
/// against these.
pub fn address_payload_lengths() -> impl Iterator<Item = (String, usize)> {
    ADDRESS_PAYLOAD_LENGTHS
        .iter()
        .map(|&(output_type, len)| (output_type.to_string(), len))
}

pub fn cohort_constants() -> Vec<(&'static str, Value)> {
    vec![
        ("TERM_NAMES", json!(TERM_NAMES)),
        ("HALVING_EPOCH_NAMES", json!(HALVING_EPOCH_NAMES)),
        ("VINTAGE_NAMES", json!(VINTAGE_NAMES)),
        ("ENTRY_NAMES", json!(ENTRY_NAMES)),
        ("SPENDABLE_TYPE_NAMES", json!(SPENDABLE_TYPE_NAMES)),
        ("AGE_RANGE_NAMES", json!(AGE_RANGE_NAMES)),
        ("AMOUNT_RANGE_NAMES", json!(AMOUNT_RANGE_NAMES)),
        (
            "PROFITABILITY_RANGE_NAMES",
            json!(PROFITABILITY_RANGE_NAMES),
        ),
    ]
}

/// Convert top-level keys of a JSON object to camelCase.
pub fn camel_case_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let new_map: Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (to_camel_case(&k), v))
                .collect();
            Value::Object(new_map)
        }
        other => other,
    }
}

/// Format a JSON value as a pretty-printed string.
pub fn format_json<T: Serialize>(value: &T) -> String {
    to_string_pretty(value).unwrap()
}
