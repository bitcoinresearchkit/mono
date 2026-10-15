use std::{
    env,
    io::{self, BufWriter},
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

use args::Args;
use brk_error::{Error, Result};
use brk_mempool::Mempool;
use emitter::Emitter;

mod args;
mod emitter;
mod event;
mod usage;

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

fn run() -> Result<()> {
    let raw: Vec<String> = env::args().skip(1).collect();
    if raw.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
        usage::print();
        return Ok(());
    }
    let args = Args::parse(raw)?;
    let client = args.connect.client()?;
    let mut mempool = Mempool::new(&client);
    let published = mempool.read_only_clone();

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut emitter = Emitter::default();

    loop {
        let started = Instant::now();
        match mempool.tick() {
            Ok(()) => match emitter.emit(&mut out, &client, &published.load(), started.elapsed()) {
                Ok(()) => {}
                // Broken pipe (e.g. `mmpl | head`) is a normal end-of-stream.
                Err(Error::IO(e)) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                Err(Error::IO(e)) => return Err(e.into()),
                // A publication moving on mid-read, or a failed RPC: retry the whole cycle.
                Err(e) => eprintln!("mmpl: cycle skipped: {e}"),
            },
            // Transient RPC failure - log, then retry on the next tick.
            Err(e) => eprintln!("mmpl: tick failed: {e}"),
        }
        if let Some(rest) = PERIOD.checked_sub(started.elapsed()) {
            thread::sleep(rest);
        }
    }
}
