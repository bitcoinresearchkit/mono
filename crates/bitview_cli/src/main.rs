use std::{
    env,
    error::Error,
    io::{self, BufReader, BufWriter, Read, Write},
    process::ExitCode,
};

use args::Args;
use generated::COMMANDS;
use ureq::{
    Agent,
    http::{Request, header::CONTENT_TYPE},
};

mod args;
mod command;
mod generated;
mod parameter;
mod request_body;

const PROGRAM_NAME: &str = env!("CARGO_BIN_NAME");
const MAX_ERROR_BYTES: u64 = 64 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        // A closed pipe (e.g. `bitview-cli ... | head`) is a normal end of output.
        Err(error)
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe) =>
        {
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{PROGRAM_NAME}: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let raw = env::args().skip(1).collect::<Vec<_>>();
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
    let agent: Agent = Agent::config_builder()
        .user_agent(concat!(
            env!("CARGO_BIN_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .http_status_as_error(false)
        .build()
        .into();
    let request = Request::builder().method(args.command.method).uri(url);
    let mut response = match args.command.request_body {
        Some(body) => agent.run(
            request
                .header("Content-Type", body.content_type)
                .body(args.body.unwrap_or_default())?,
        )?,
        None => agent.run(request.body(())?)?,
    };
    let status = response.status();
    let json = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"));
    let mut body = response.body_mut().as_reader();

    if !status.is_success() {
        let mut detail = Vec::new();
        body.take(MAX_ERROR_BYTES).read_to_end(&mut detail)?;
        let detail = String::from_utf8_lossy(&detail);
        return Err(format!("HTTP {status}: {}", detail.trim()).into());
    }

    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    if args.pretty && json {
        write_pretty(body, &mut output)?;
    } else {
        io::copy(&mut body, &mut output)?;
    }
    output.flush()?;
    Ok(())
}

/// Streams `input` re-indented like serde_json's pretty printer. Tokens are copied byte for byte, so numbers beyond
/// u64/f64 (u128 hashrates) stay exact and nothing is buffered.
fn write_pretty(input: impl Read, output: &mut impl Write) -> io::Result<()> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut opened = false;
    for byte in BufReader::new(input).bytes() {
        let byte = byte?;
        if in_string {
            in_string = escaped || byte != b'"';
            escaped = !escaped && byte == b'\\';
            output.write_all(&[byte])?;
            continue;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        if opened {
            opened = false;
            // Empty containers stay `[]` / `{}`.
            if matches!(byte, b']' | b'}') {
                depth -= 1;
                output.write_all(&[byte])?;
                continue;
            }
            newline(output, depth)?;
        }
        match byte {
            b'[' | b'{' => {
                depth += 1;
                opened = true;
                output.write_all(&[byte])?;
            }
            b']' | b'}' => {
                depth -= 1;
                newline(output, depth)?;
                output.write_all(&[byte])?;
            }
            b',' => {
                output.write_all(b",")?;
                newline(output, depth)?;
            }
            b':' => output.write_all(b": ")?,
            b'"' => {
                in_string = true;
                output.write_all(b"\"")?;
            }
            _ => output.write_all(&[byte])?,
        }
    }
    writeln!(output)
}

fn newline(output: &mut impl Write, depth: usize) -> io::Result<()> {
    output.write_all(b"\n")?;
    for _ in 0..depth {
        output.write_all(b"  ")?;
    }
    Ok(())
}
