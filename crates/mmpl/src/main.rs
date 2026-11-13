use std::{
    env,
    io::{self, BufWriter, Write},
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_mempool::Mempool;
use emitter::Emitter;
use event::Event;

mod args;
mod emitter;
mod event;
mod usage;

const PERIOD: Duration = Duration::from_millis(500);
/// The longest wait between cycles while they keep failing.
const BACKOFF_MAX: Duration = Duration::from_secs(4);

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
    let client = args::parse(raw)?.client()?;
    let mut mempool = Mempool::new(&client);
    let published = mempool.read_only_clone();

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut emitter = Emitter::default();

    let mut wait = PERIOD;
    loop {
        let started = Instant::now();
        let failed = match mempool.tick() {
            // A failed RPC, a block mid-read, the node reloading its mempool.
            Err(e) => Some(e),
            Ok(()) => match emitter.emit(&mut out, &client, &published.load(), started.elapsed()) {
                Ok(()) => None,
                // Broken pipe (e.g. `mmpl | head`) is a normal end-of-stream.
                Err(Error::IO(e)) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                Err(Error::IO(e)) => return Err(e.into()),
                // A publication moving on mid-read, or a failed RPC: the whole cycle again.
                Err(e) => Some(e),
            },
        };
        wait = match failed {
            None => PERIOD,
            // On the stream, so a stall reads apart from an idle pool.
            Some(e) => {
                match emitter::write_line(&mut out, &Event::stall(e.to_string()))
                    .and_then(|()| out.flush())
                {
                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                    written => written?,
                }
                // A tip moving mid-cycle is a race, not a failure: no backoff.
                match e {
                    Error::StateUpdating(_) => PERIOD,
                    _ => (wait * 2).min(BACKOFF_MAX),
                }
            }
        };
        if let Some(rest) = wait.checked_sub(started.elapsed()) {
            thread::sleep(rest);
        }
    }
}
