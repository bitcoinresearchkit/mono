use std::{
    env,
    io::{self, BufWriter, Write},
    process::ExitCode,
};

use args::Args;
use brk_error::{Error, Result};
use brk_reader::Reader;
use fields::Ctx;
use formatter::Formatter;
use mode::Mode;

mod args;
mod fields;
mod formatter;
mod mode;
mod path;
mod selector;
mod step;
mod usage;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        // Broken pipe (e.g. `blk … | head`) is a normal end-of-stream.
        Err(Error::IO(e)) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("blk: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let raw: Vec<String> = env::args().skip(1).collect();
    if raw.is_empty() || raw.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
        usage::print();
        return Ok(());
    }
    let args = Args::parse(raw)?;

    let client = args.connect.client()?;
    let (start, end) = selector::parse(&args.selector, &client)?;
    let tip = client.get_last_height()?;
    if end > tip {
        return Err(Error::OutOfRange(
            format!("range end {end} is past current tip {tip}").into(),
        ));
    }
    let before = match start.decremented() {
        Some(height) => client.get_block_hashes_range(*height, *height)?.pop(),
        None => None,
    };

    let mode = Mode::pick(args.pretty, args.compact, args.paths.len())?;
    let reader = Reader::connect(&args.connect, &client)?;
    let network = reader.chain().network;
    let formatter = Formatter::new(mode, args.paths);
    let mut out = BufWriter::new(io::stdout().lock());
    for block in reader.after(before, end)? {
        formatter.write(&Ctx::new(&block?, network), &mut out)?;
    }
    Ok(out.flush()?)
}
