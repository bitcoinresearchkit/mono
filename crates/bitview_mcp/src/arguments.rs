use crate::config::{api_url, public_url};

#[derive(Debug)]
pub struct Arguments {
    pub api_url: String,
    pub public_url: String,
    pub display_name: String,
}

impl Arguments {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut api = None;
        let mut url = None;
        let mut name = None;
        let mut arguments = arguments.into_iter();

        while let Some(flag) = arguments.next() {
            let target = match flag.as_str() {
                "--api" => &mut api,
                "--url" => &mut url,
                "--name" => &mut name,
                _ if flag.starts_with('-') => return Err(format!("unknown option: {flag}")),
                _ => return Err(format!("unexpected positional argument: {flag}")),
            };
            if target.is_some() {
                return Err(format!("duplicate option: {flag}"));
            }
            let value = arguments
                .next()
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("missing value for {flag}"))?;
            *target = Some(value);
        }

        let api_url = api_url(&required(api, "--api")?)?;
        let public_url = public_url(&required(url, "--url")?)?;
        let display_name = required(name, "--name")?.trim().to_owned();
        if display_name.is_empty() {
            return Err("--name must not be empty".to_owned());
        }

        Ok(Self {
            api_url,
            public_url,
            display_name,
        })
    }
}

fn required(value: Option<String>, flag: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("missing required option: {flag}"))
}
