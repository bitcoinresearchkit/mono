use std::{
    env, fs,
    io::{self, Read},
};

use crate::command::Command;

pub(crate) const BASE_URL_ENV: &str = "BITVIEW_URL";
pub(crate) const DEFAULT_BASE_URL: &str = "http://localhost:3110";

pub(crate) struct Args {
    pub command: &'static Command,
    pub base_url: String,
    pub path_values: Vec<String>,
    pub query: Vec<(&'static str, String)>,
    pub body: Option<Vec<u8>>,
    pub pretty: bool,
    pub help: bool,
}

impl Args {
    pub fn parse(raw: Vec<String>, commands: &'static [Command]) -> Result<Self, String> {
        let mut command: Option<&'static Command> = None;
        let mut base_url = env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
        let mut path_values = Vec::new();
        let mut query = Vec::new();
        let mut body = None;
        let mut pretty = false;
        let mut help = false;
        let mut index = 0;

        while index < raw.len() {
            let argument = &raw[index];
            if let Some(value) = argument.strip_prefix("--url=") {
                base_url = value.to_owned();
                index += 1;
                continue;
            }
            if command.is_some()
                && let Some(value) = argument.strip_prefix("--body=")
            {
                set_body(&mut body, value.as_bytes().to_vec())?;
                index += 1;
                continue;
            }
            if command.is_some()
                && let Some(path) = argument.strip_prefix("--body-file=")
            {
                set_body(&mut body, read_body(path)?)?;
                index += 1;
                continue;
            }
            match argument.as_str() {
                "-u" | "--url" => {
                    base_url = next_value(&raw, &mut index, argument)?;
                }
                "-p" | "--pretty" => pretty = true,
                "-h" | "--help" => help = true,
                "--body" if command.is_some() => {
                    set_body(
                        &mut body,
                        next_value(&raw, &mut index, argument)?.into_bytes(),
                    )?;
                }
                "--body-file" if command.is_some() => {
                    let path = next_value(&raw, &mut index, argument)?;
                    set_body(&mut body, read_body(&path)?)?;
                }
                value if value.starts_with("--") && command.is_some() => {
                    let current = command.unwrap();
                    let (flag, inline_value) = value[2..]
                        .split_once('=')
                        .map_or((&value[2..], None), |(flag, value)| (flag, Some(value)));
                    let parameter = current
                        .query_parameter(flag)
                        .ok_or_else(|| format!("unknown option --{flag} for {}", current.name))?;
                    let value = match inline_value {
                        Some(value) => value.to_owned(),
                        None => next_value(&raw, &mut index, argument)?,
                    };
                    if !parameter.repeatable
                        && query.iter().any(|(name, _)| *name == parameter.api_name)
                    {
                        return Err(format!("--{} may only be provided once", parameter.name));
                    }
                    query.push((parameter.api_name, value));
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown option {value}"));
                }
                value if command.is_none() => {
                    command = commands.iter().find(|candidate| candidate.name == value);
                    if command.is_none() {
                        return Err(format!("unknown command {value:?}"));
                    }
                }
                value => path_values.push(value.to_owned()),
            }
            index += 1;
        }

        let command = command.ok_or_else(|| "missing command".to_owned())?;
        if help {
            return Ok(Self {
                command,
                base_url,
                path_values,
                query,
                body,
                pretty,
                help,
            });
        }

        if path_values.len() != command.path_parameters.len() {
            return Err(format!(
                "{} expects {} path argument(s), received {}",
                command.name,
                command.path_parameters.len(),
                path_values.len()
            ));
        }
        for parameter in command.query_parameters {
            if parameter.required
                && !query
                    .iter()
                    .any(|(provided, _)| *provided == parameter.api_name)
            {
                return Err(format!("missing required option --{}", parameter.name));
            }
        }
        match (command.request_body, body.is_some()) {
            (Some(body), false) if body.required => {
                return Err("missing required --body or --body-file".to_owned());
            }
            (None, true) => return Err(format!("{} does not accept a request body", command.name)),
            _ => {}
        }

        Ok(Self {
            command,
            base_url: base_url.trim_end_matches('/').to_owned(),
            path_values,
            query,
            body,
            pretty,
            help,
        })
    }

    pub fn url(&self) -> String {
        let mut path = self.command.path.to_owned();
        for (parameter, value) in self.command.path_parameters.iter().zip(&self.path_values) {
            path = path.replace(
                &format!("{{{}}}", parameter.api_name),
                &encode_component(value),
            );
        }
        if !self.query.is_empty() {
            path.push('?');
            path.push_str(
                &self
                    .query
                    .iter()
                    .map(|(name, value)| {
                        format!("{}={}", encode_component(name), encode_component(value))
                    })
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        format!("{}{path}", self.base_url)
    }
}

fn next_value(raw: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    *index += 1;
    raw.get(*index)
        .cloned()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn set_body(body: &mut Option<Vec<u8>>, value: Vec<u8>) -> Result<(), String> {
    if body.replace(value).is_some() {
        return Err("request body may only be provided once".to_owned());
    }
    Ok(())
}

fn read_body(path: &str) -> Result<Vec<u8>, String> {
    if path == "-" {
        let mut body = Vec::new();
        io::stdin()
            .read_to_end(&mut body)
            .map_err(|error| format!("failed to read request body from stdin: {error}"))?;
        Ok(body)
    } else {
        fs::read(path)
            .map_err(|error| format!("failed to read request body from {path:?}: {error}"))
    }
}

fn encode_component(value: &str) -> String {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").unwrap();
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::Args;
    use crate::{command::Command, parameter::Parameter, request_body::RequestBody};

    static COMMANDS: &[Command] = &[
        Command {
            name: "get-example",
            method: "GET",
            path: "/api/example/{name}",
            summary: "",
            description: "",
            path_parameters: &[Parameter {
                api_name: "name",
                name: "name",
                required: true,
                value_name: "string",
                repeatable: false,
                description: None,
            }],
            query_parameters: &[Parameter {
                api_name: "txId[]",
                name: "tx-id",
                required: false,
                value_name: "string[]",
                repeatable: true,
                description: None,
            }],
            request_body: None,
        },
        Command {
            name: "post-example",
            method: "POST",
            path: "/api/example",
            summary: "",
            description: "",
            path_parameters: &[],
            query_parameters: &[],
            request_body: Some(RequestBody {
                value_name: "string",
                required: true,
                content_type: "text/plain",
            }),
        },
    ];

    #[test]
    fn builds_encoded_url_and_repeated_array_query() {
        let args = Args::parse(
            [
                "--url=https://example.test/",
                "get-example",
                "a/b",
                "--tx-id",
                "one two",
                "--tx-id=three",
            ]
            .map(str::to_owned)
            .to_vec(),
            COMMANDS,
        )
        .unwrap();

        assert_eq!(
            args.url(),
            "https://example.test/api/example/a%2Fb?txId%5B%5D=one%20two&txId%5B%5D=three"
        );
    }

    #[test]
    fn accepts_inline_request_body() {
        let args = Args::parse(
            ["post-example", "--body=02000000"]
                .map(str::to_owned)
                .to_vec(),
            COMMANDS,
        )
        .unwrap();

        assert_eq!(args.body.as_deref(), Some(b"02000000".as_slice()));
    }
}
