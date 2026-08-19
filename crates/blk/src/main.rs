mod args;
mod fields;
mod formatter;
mod mode;
mod path;
mod selector;
mod usage;

use std::process::ExitCode;

use brk_reader::Reader;

use args::Args;
use fields::Ctx;
use formatter::Formatter;
use mode::Mode;
use selector::Selector;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("blk: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> brk_error::Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.is_empty() || raw.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
        usage::print();
        return Ok(());
    }
    let args = Args::parse(raw)?;

    let client = args.rpc()?;
    let (start, end) = Selector::parse(&args.selector, &client)?;
    let network = client.get_network()?;

    let mode = Mode::pick(args.pretty, args.compact, args.paths.len())?;
    let reader = Reader::new(args.blocks_dir(), &client);
    let formatter = Formatter::new(mode, args.paths);
    let parser_threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        / 2)
    .max(1);
    for block in reader.range_with(start, end, parser_threads)?.iter() {
        let block = block?;
        let line = formatter.format(&Ctx::new(&block, network))?;
        if !line.is_empty() {
            println!("{line}");
        }
    }
    Ok(())
}
