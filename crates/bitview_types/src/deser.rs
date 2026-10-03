use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

pub(crate) fn de_unquote_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(value) = Option::<Value>::deserialize(deserializer)? else {
        return Ok(None);
    };

    if let Some(s) = value.as_str() {
        let s = s
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(s);
        if s == "null" || s.is_empty() {
            return Ok(None);
        }
        s.parse().map(Some).map_err(Error::custom)
    } else {
        value
            .as_u64()
            .map(|number| Some(number as usize))
            .ok_or_else(|| Error::custom("expected a string or number"))
    }
}
