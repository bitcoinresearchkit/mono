use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};
use serde_json::{Map, Value};

use super::catalog::{Operation, ParameterLocation};

const MAX_URL_BYTES: usize = 32 * 1024;

/// The API path and query a tool call reads. The API judges the values; an error here is one only the bridge can see,
/// and names the argument and the fix for the model to act on.
pub fn api_path(operation: &Operation, arguments: &Map<String, Value>) -> Result<String, String> {
    let tool = &operation.name;
    let parameters = &operation.http.parameters;
    // Dropped silently, a misspelled filter would answer with unfiltered data.
    if let Some(unknown) = arguments.keys().find(|name| {
        !parameters
            .iter()
            .any(|parameter| parameter.argument == **name)
    }) {
        let known = parameters
            .iter()
            .map(|parameter| parameter.argument.as_str())
            .collect::<Vec<_>>();
        return Err(if known.is_empty() {
            format!("{tool} takes no arguments: remove `{unknown}`")
        } else {
            format!(
                "{tool} has no argument `{unknown}`: its arguments are {}",
                known.join(", ")
            )
        });
    }

    let mut path = operation.http.path.clone();
    let mut query = String::new();
    for parameter in parameters {
        let argument = &parameter.argument;
        let values = match arguments.get(argument) {
            None | Some(Value::Null) => Some(Vec::new()),
            Some(Value::Array(values)) => values.iter().map(scalar).collect(),
            Some(value) => scalar(value).map(|value| vec![value]),
        }
        .ok_or_else(|| {
            format!(
                "{tool}'s `{argument}` takes a string, a number or a boolean, or a list of them"
            )
        })?;
        match parameter.location {
            ParameterLocation::Path => {
                let [value] = values.as_slice() else {
                    return Err(format!("{tool} needs one value for `{argument}`"));
                };
                path = path.replace(
                    &format!("{{{}}}", parameter.name),
                    &encode_component(value).to_string(),
                );
            }
            ParameterLocation::Query => {
                for value in values {
                    query.push(if query.is_empty() { '?' } else { '&' });
                    query.extend(encode_component(&parameter.name));
                    query.push('=');
                    query.extend(encode_component(&value));
                }
            }
        }
    }

    let url = format!("{path}{query}");
    if url.len() > MAX_URL_BYTES {
        return Err(format!(
            "{tool}'s arguments make a URL over {MAX_URL_BYTES} bytes: pass fewer or shorter values"
        ));
    }
    Ok(url)
}

/// A scalar as a URL writes it; none for an object or a nested list.
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

/// Every byte but RFC 3986's unreserved characters is percent-encoded.
const ESCAPED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn encode_component(value: &str) -> PercentEncode<'_> {
    utf8_percent_encode(value, ESCAPED)
}
