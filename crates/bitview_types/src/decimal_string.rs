//! An unsigned integer written as a decimal string, as mempool.space writes large amounts.

use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serializer, de::Error};

pub fn serialize<T: Display, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

pub fn deserialize<'de, T: From<u64>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map(T::from)
        .map_err(D::Error::custom)
}
