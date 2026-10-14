use std::{
    env, error::Error, io::Write, path::PathBuf, process::ExitCode, result::Result as StdResult,
};

use tempfile::NamedTempFile;

mod analysis;
mod group;
mod input;
mod record;
mod report;
mod routes;

type Result<T> = StdResult<T, Box<dyn Error>>;

fn run() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let (Some(dir), None) = (args.next().map(PathBuf::from), args.next()) else {
        return Err("usage: bitviewd_latency <bitviewdir>".into());
    };
    let files = input::discover(&dir.join("logs"))?;
    let analysis = analysis::Analysis::read(&files)?;
    let report = report::render(&analysis)?;
    let path = dir.join("latency.md");
    // Replace the report without following an existing symlink or truncating a
    // hard-linked source file. A failed write leaves the previous report intact.
    let mut output = NamedTempFile::new_in(&dir)?;
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
