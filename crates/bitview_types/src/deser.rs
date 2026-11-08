use serde::{Deserialize, Deserializer, de::DeserializeOwned, de::Error};

/// A number as a URL must write it, `None` for any other spelling: digits alone, which JSON's
/// grammar allows in one form, with no leading zeros. A CDN keys on the URL's text, so `0600` or
/// `+600` would each be one more way past its cache for the same answer.
pub(crate) fn plain_number<T: DeserializeOwned>(text: &str) -> Option<T> {
    text.bytes()
        .all(|byte| byte.is_ascii_digit())
        .then(|| serde_json::from_str(text).ok())
        .flatten()
}

/// Deserializes a number from a URL's path or query, in its [`plain_number`] spelling alone.
pub fn de_plain<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let text = String::deserialize(deserializer)?;
    plain_number(&text).ok_or_else(|| Error::custom(format!("invalid number: `{text}`")))
}
