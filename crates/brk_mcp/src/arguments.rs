use crate::config::{api_bases, public_url};

#[derive(Debug)]
pub(crate) struct Arguments {
    api_bases: Vec<String>,
    api_url: String,
    public_url: String,
    display_name: String,
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

        let api = required(api, "--api")?;
        let public_url = public_url(&required(url, "--url")?)?;
        let display_name = required(name, "--name")?.trim().to_owned();
        if display_name.is_empty() {
            return Err("--name must not be empty".to_owned());
        }

        let api_bases = api_bases(&api)?;
        let api_url = api_bases
            .first()
            .expect("validated API bases should not be empty")
            .clone();

        Ok(Self {
            api_bases,
            api_url,
            public_url,
            display_name,
        })
    }

    pub fn into_parts(self) -> (Vec<String>, String, String, String) {
        (
            self.api_bases,
            self.api_url,
            self.public_url,
            self.display_name,
        )
    }
}

fn required(value: Option<String>, flag: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("missing required option: {flag}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Arguments, String> {
        Arguments::parse(arguments.iter().map(|argument| (*argument).to_owned()))
    }

    #[test]
    fn parses_required_options_in_any_order() {
        let arguments = parse(&[
            "--name",
            "Example Node",
            "--api",
            "api.example.com",
            "--url",
            "https://mcp.example.com",
        ])
        .unwrap();
        let (api_bases, api_url, public_url, display_name) = arguments.into_parts();

        assert_eq!(
            api_bases,
            ["https://api.example.com", "http://api.example.com"]
        );
        assert_eq!(api_url, "https://api.example.com");
        assert_eq!(public_url, "https://mcp.example.com/");
        assert_eq!(display_name, "Example Node");
    }

    #[test]
    fn rejects_missing_duplicate_unknown_and_positional_options() {
        assert_eq!(
            parse(&["--api", "api.example.com"]).unwrap_err(),
            "missing required option: --url"
        );
        assert_eq!(
            parse(&["--api", "a.example.com", "--api", "b.example.com"]).unwrap_err(),
            "duplicate option: --api"
        );
        assert_eq!(parse(&["--other"]).unwrap_err(), "unknown option: --other");
        assert_eq!(
            parse(&["api.example.com"]).unwrap_err(),
            "unexpected positional argument: api.example.com"
        );
    }

    #[test]
    fn rejects_missing_values_and_empty_names() {
        assert_eq!(
            parse(&["--api", "--url"]).unwrap_err(),
            "missing value for --api"
        );
        assert_eq!(
            parse(&[
                "--api",
                "api.example.com",
                "--url",
                "https://mcp.example.com",
                "--name",
                "   ",
            ])
            .unwrap_err(),
            "--name must not be empty"
        );
    }
}
