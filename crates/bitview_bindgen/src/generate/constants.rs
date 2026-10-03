//! Shared constant generation for static client data.
//!
//! Extracts common logic for generating INDEXES, POOL_ID_TO_POOL_NAME,
//! and cohort name constants across JavaScript and Python clients.

use std::collections::BTreeMap;

use bitview_cohort::{
    AGE_RANGE_NAMES, AMOUNT_RANGE_NAMES, CLASS_NAMES, ENTRY_NAMES, EPOCH_NAMES,
    PROFITABILITY_RANGE_NAMES, SPENDABLE_TYPE_NAMES, TERM_NAMES,
};
use bitview_primitives::{Index, pools};
use serde::Serialize;
use serde_json::{Map, Value, to_string_pretty, to_value as SerdeJsonToValue};

use crate::{VERSION, to_camel_case};

/// Collected constant data for client generation.
pub struct ClientConstants {
    pub version: String,
    pub indexes: Vec<&'static str>,
    pub pool_map: BTreeMap<String, &'static str>,
}

impl ClientConstants {
    /// Collect all constant data.
    pub fn collect() -> Self {
        let indexes = Index::all();
        let indexes: Vec<&'static str> = indexes.iter().map(|i| i.name()).collect();

        let pools = pools();
        let mut sorted_pools: Vec<_> = pools.iter().collect();
        sorted_pools.sort_by_key(|p| p.name.to_lowercase());
        let pool_map: BTreeMap<String, &'static str> = sorted_pools
            .iter()
            .map(|p| (p.slug().to_string(), p.name))
            .collect();

        Self {
            version: format!("v{}", VERSION),
            indexes,
            pool_map,
        }
    }
}

/// Get all cohort constants as name-value pairs for iteration.
pub fn cohort_constants() -> Vec<(&'static str, Value)> {
    fn to_value<T: Serialize>(v: &T) -> Value {
        SerdeJsonToValue(v).unwrap()
    }

    vec![
        ("TERM_NAMES", to_value(&TERM_NAMES)),
        ("EPOCH_NAMES", to_value(&EPOCH_NAMES)),
        ("CLASS_NAMES", to_value(&CLASS_NAMES)),
        ("ENTRY_NAMES", to_value(&ENTRY_NAMES)),
        ("SPENDABLE_TYPE_NAMES", to_value(&SPENDABLE_TYPE_NAMES)),
        ("AGE_RANGE_NAMES", to_value(&AGE_RANGE_NAMES)),
        ("AMOUNT_RANGE_NAMES", to_value(&AMOUNT_RANGE_NAMES)),
        (
            "PROFITABILITY_RANGE_NAMES",
            to_value(&PROFITABILITY_RANGE_NAMES),
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
