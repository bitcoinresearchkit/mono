mod args;
mod command;
mod generated;
mod parameter;
mod request_body;

use std::{
    error::Error,
    io::{self, Write},
    process::ExitCode,
};

use args::Args;
use generated::COMMANDS;

pub(crate) const PROGRAM_NAME: &str = env!("CARGO_BIN_NAME");

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{PROGRAM_NAME}: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let raw = std::env::args().skip(1).collect::<Vec<_>>();
    match raw.as_slice() {
        [] => {
            command::print_help(COMMANDS)?;
            return Ok(());
        }
        [flag] if matches!(flag.as_str(), "-h" | "--help" | "help") => {
            command::print_help(COMMANDS)?;
            return Ok(());
        }
        [flag] if matches!(flag.as_str(), "-V" | "--version") => {
            println!("{PROGRAM_NAME} {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        [help, name] if help == "help" => {
            print_command_help(name)?;
            return Ok(());
        }
        [name, help] if help == "help" => {
            print_command_help(name)?;
            return Ok(());
        }
        _ => {}
    }

    let args = Args::parse(raw, COMMANDS)?;
    if args.help {
        args.command.print_help()?;
        return Ok(());
    }

    execute(args)
}

fn print_command_help(name: &str) -> Result<(), Box<dyn Error>> {
    let command = COMMANDS
        .iter()
        .find(|command| command.name == name)
        .ok_or_else(|| format!("unknown command {name:?}"))?;
    command.print_help()?;
    Ok(())
}

fn execute(args: Args) -> Result<(), Box<dyn Error>> {
    let url = args.url();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .user_agent(concat!(
            env!("CARGO_BIN_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .http_status_as_error(false)
        .build()
        .into();
    let request = ureq::http::Request::builder()
        .method(args.command.method)
        .uri(url);
    let mut response = match args.command.request_body {
        Some(body) => agent.run(
            request
                .header("Content-Type", body.content_type)
                .body(args.body.unwrap_or_default())?,
        )?,
        None => agent.run(request.body(())?)?,
    };
    let status = response.status();
    let bytes = response.body_mut().read_to_vec()?;

    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(format!("HTTP {status}: {}", body.trim()).into());
    }

    let stdout = io::stdout();
    let mut output = stdout.lock();
    if !args.pretty {
        output.write_all(&bytes)?;
    } else if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
        serde_json::to_writer_pretty(&mut output, &value)?;
        writeln!(output)?;
    } else {
        output.write_all(&bytes)?;
    }
    Ok(())
}
