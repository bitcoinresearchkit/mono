#![doc = include_str!("../README.md")]

use std::{backtrace::Backtrace, env, io, panic, path::Path};

use file::LogFile;
pub use file::log_file_date;
pub use format::{MalformedRequestLine, RequestLine};
use layer::Logger;
pub use layer::register_hook;
use retention::delete_old_daily;
pub use tail::{Line, Recent, recent, started};
use tracing::{error, field};
use tracing_log::LogTracer;
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    filter::Targets, layer::SubscriberExt, registry as TracingSubscriberRegistry,
    util::SubscriberInitExt,
};

mod file;
mod format;
mod layer;
mod retention;
mod tail;

/// Initialize the global tracing subscriber, which writes colorized lines to the console.
///
/// If `dir` is `Some`, it also writes each day's lines to `YYYY-MM-DD.txt` in that directory. The
/// directory is created if it does not exist, and its log files older than 7 days are deleted at
/// start, then once a day.
pub fn init(dir: Option<&Path>) -> io::Result<()> {
    init_with_default_level(
        dir,
        if cfg!(debug_assertions) {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        },
    )
}

/// Initialize the logger with a caller-selected fallback level.
///
/// `RUST_LOG` (a full filter) takes precedence over `LOG` (a level), which takes precedence over
/// `default_level`. A value that doesn't parse is an error, not a silent fallback: a typo there
/// would otherwise mute errors too.
pub fn init_with_default_level(dir: Option<&Path>, default_level: LevelFilter) -> io::Result<()> {
    LogTracer::init().ok();

    let invalid = |name: &str, value: &str, error: &dyn std::fmt::Display| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name}={value}: {error}"),
        )
    };
    // Set but empty is unset (an empty LOG would parse as `error`).
    let var = |name| {
        env::var(name)
            .ok()
            .filter(|value: &String| !value.is_empty())
    };
    let filter: Targets = match var("RUST_LOG") {
        Some(value) => value
            .parse()
            .map_err(|error| invalid("RUST_LOG", &value, &error))?,
        None => {
            let level = match var("LOG") {
                Some(value) => value.parse::<LevelFilter>().map_err(|_| {
                    invalid(
                        "LOG",
                        &value,
                        &"not a level (off, error, warn, info, debug, trace)",
                    )
                })?,
                None => default_level,
            };
            let storage = level.min(LevelFilter::WARN);
            format!(
                "{level},bitcoin=off,corepc=off,tracing=off,aide=off,fjall={storage},lsm_tree={storage},tower_http=off,rmcp=warn"
            )
            .parse()
            .expect("the default filter parses")
        }
    };

    let file = dir.map(LogFile::new).transpose()?;
    TracingSubscriberRegistry()
        .with(filter)
        .with(Logger { file })
        .init();
    if let Some(dir) = dir {
        delete_old_daily(dir);
    }

    install_panic_hook();

    Ok(())
}

/// Chains the previous hook so panics always reach stderr whatever the log
/// filters say, then records them through tracing (log files included).
fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        previous(info);
        let location = info.location().map(field::display);
        let msg = info.payload_as_str().unwrap_or("Box<dyn Any>");
        let backtrace = Backtrace::capture();
        error!(location, backtrace = %backtrace, "panic: {msg}");
    }));
}
