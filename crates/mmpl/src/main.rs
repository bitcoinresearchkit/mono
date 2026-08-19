mod args;
mod emitter;
mod event;
mod usage;

use std::{
    io::{self, BufWriter},
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

use brk_mempool::Mempool;

use args::Args;
use emitter::Emitter;

const PERIOD: Duration = Duration::from_millis(500);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mmpl: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> brk_error::Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
        usage::print();
        return Ok(());
    }
    let args = Args::parse(raw)?;
    let client = args.rpc()?;
    let mempool = Mempool::new(&client);

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut emitter = Emitter::default();

    loop {
        let started = Instant::now();
        match mempool.tick() {
            Ok(cycle) => match emitter.emit(&mut out, &cycle) {
                Ok(()) => {}
                // Broken pipe (e.g. `mmpl | head`) is a normal end-of-stream.
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                Err(e) => return Err(e.into()),
            },
            // Transient RPC failure - log, then retry on the next tick.
            Err(e) => eprintln!("mmpl: tick failed: {e}"),
        }
        if let Some(rest) = PERIOD.checked_sub(started.elapsed()) {
            thread::sleep(rest);
        }
    }
}
