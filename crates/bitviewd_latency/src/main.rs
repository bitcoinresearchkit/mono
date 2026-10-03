use std::{env, error::Error, io::Write, process::ExitCode, result::Result as StdResult};

use bitview::Config;
use tempfile::NamedTempFile;

mod analysis;
mod group;
mod input;
mod record;
mod report;
mod routes;

type Result<T> = StdResult<T, Box<dyn Error>>;

fn run() -> Result<()> {
    if env::args_os().len() != 1 {
        return Err("no arguments supported; paths come from bitview::Config".into());
    }
    let config = Config::load()?.server_config();
    let parent = &config.data_path;
    let files = input::discover(&config.logs_path())?;
    let analysis = analysis::Analysis::read(&files)?;
    let report = report::render(&analysis)?;
    let path = parent.join("latency.md");
    // Replace the report without following an existing symlink or truncating a
    // hard-linked source file. A failed write leaves the previous report intact.
    let mut output = NamedTempFile::new_in(parent)?;
    output.write_all(report.as_bytes())?;
    output.persist(&path)?;
    println!("Report written to {}", path.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bitviewd_latency: {e}");
            ExitCode::FAILURE
        }
    }
}
