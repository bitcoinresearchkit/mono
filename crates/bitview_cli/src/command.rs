use std::io::{self, Write};

use crate::{
    PROGRAM_NAME,
    args::{BASE_URL_ENV, DEFAULT_BASE_URL},
    parameter::Parameter,
    request_body::RequestBody,
};

pub(crate) struct Command {
    pub name: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub path_parameters: &'static [Parameter],
    pub query_parameters: &'static [Parameter],
    pub request_body: Option<RequestBody>,
}

impl Command {
    pub fn query_parameter(&self, flag: &str) -> Option<&Parameter> {
        self.query_parameters
            .iter()
            .find(|parameter| parameter.name == flag)
    }

    pub fn print_help(&self) -> io::Result<()> {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        write!(output, "Usage: {PROGRAM_NAME} [OPTIONS] {}", self.name)?;
        for parameter in self.path_parameters {
            write!(output, " <{}>", parameter.name)?;
        }
        writeln!(output, " [COMMAND OPTIONS]\n")?;
        if !self.summary.is_empty() {
            writeln!(output, "{}\n", self.summary)?;
        }
        if !self.description.is_empty() && self.description != self.summary {
            writeln!(output, "{}\n", self.description)?;
        }
        writeln!(output, "{} {}\n", self.method, self.path)?;

        if !self.path_parameters.is_empty() {
            writeln!(output, "Path parameters:")?;
            for parameter in self.path_parameters {
                write!(
                    output,
                    "  <{:<20} {:<16}",
                    format!("{}>", parameter.name),
                    parameter.value_name
                )?;
                if let Some(description) = parameter.description {
                    write!(output, " {description}")?;
                }
                writeln!(output)?;
            }
            writeln!(output)?;
        }

        if !self.query_parameters.is_empty() {
            writeln!(output, "Query parameters:")?;
            for parameter in self.query_parameters {
                write!(
                    output,
                    "  --{:<20} {:<16}",
                    parameter.name,
                    format!("<{}>", parameter.value_name)
                )?;
                if parameter.required {
                    write!(output, " required")?;
                }
                if parameter.repeatable {
                    write!(output, " repeatable")?;
                }
                if let Some(description) = parameter.description {
                    write!(output, "  {description}")?;
                }
                writeln!(output)?;
            }
            writeln!(output)?;
        }

        if let Some(body) = self.request_body {
            writeln!(
                output,
                "Request body:\n  --body <{}> or --body-file <PATH>{}\n  Content-Type: {}",
                body.value_name,
                if body.required { " (required)" } else { "" },
                body.content_type,
            )?;
            writeln!(output)?;
        }

        writeln!(output, "Options:")?;
        writeln!(
            output,
            "  -u, --url <URL>          API origin [{BASE_URL_ENV} or {DEFAULT_BASE_URL}]"
        )?;
        writeln!(
            output,
            "  -p, --pretty             Pretty-print JSON responses"
        )?;
        writeln!(output, "  -h, --help               Show this help")?;
        Ok(())
    }
}

pub(crate) fn print_help(commands: &[Command]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "{PROGRAM_NAME} {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        output,
        "Generated command-line client for the Bitview API\n"
    )?;
    writeln!(output, "Usage: {PROGRAM_NAME} [OPTIONS] <COMMAND> [ARGS]\n")?;
    writeln!(output, "Options:")?;
    writeln!(
        output,
        "  -u, --url <URL>          API origin [{BASE_URL_ENV} or {DEFAULT_BASE_URL}]"
    )?;
    writeln!(
        output,
        "  -p, --pretty             Pretty-print JSON responses"
    )?;
    writeln!(output, "  -V, --version            Show version")?;
    writeln!(output, "  -h, --help               Show this help\n")?;
    writeln!(output, "Commands:")?;
    for command in commands {
        writeln!(output, "  {:<42} {}", command.name, command.summary)?;
    }
    writeln!(
        output,
        "\nRun `{PROGRAM_NAME} help <COMMAND>` for command details."
    )?;
    Ok(())
}
