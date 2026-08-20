use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn de_unquote_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;

    if value.is_none() {
        return Ok(None);
    }

    let value = value.unwrap();

    if let Some(mut s) = value.as_str().map(|s| s.to_string()) {
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s = s[1..s.len() - 1].to_string();
        }
        if s == "null" || s.is_empty() {
            return Ok(None);
        }
        s.parse::<i64>().map(Some).map_err(serde::de::Error::custom)
    } else if let Some(n) = value.as_i64() {
        Ok(Some(n))
    } else {
        Err(serde::de::Error::custom("expected a string or number"))
    }
}

pub fn de_unquote_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;

    if value.is_none() {
        return Ok(None);
    }

    let value = value.unwrap();

    if let Some(mut s) = value.as_str().map(|s| s.to_string()) {
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s = s[1..s.len() - 1].to_string();
        }
        if s == "null" || s.is_empty() {
            return Ok(None);
        }
        s.parse::<usize>()
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else if let Some(n) = value.as_u64() {
        Ok(Some(n as usize))
    } else {
        Err(serde::de::Error::custom("expected a string or number"))
    }
}
