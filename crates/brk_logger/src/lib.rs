#![doc = include_str!("../README.md")]

use std::{backtrace::Backtrace, env, io, panic, path::Path};

use format::Formatter;
use hook::HookLayer;
use rate_limit::RateLimitedFile;
use retention::delete_old_daily;
pub use rate_limit::{LEVEL_NAMES, log_file_date};
use tail::TailLayer;
pub use tail::{Line, Recent, recent, started};
use tracing::{Event, error, field};
use tracing_log::LogTracer;
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    filter::Targets, fmt, layer::SubscriberExt, registry as TracingSubscriberRegistry,
    util::SubscriberInitExt,
};

mod format;
mod hook;
mod rate_limit;
mod retention;
mod tail;

/// Initialize the global tracing subscriber with a colorized console layer.
///
/// If `dir` is `Some`, also writes daily log files to that directory:
/// `YYYY-MM-DD.txt` for the combined log and `YYYY-MM-DD_<level>.txt` for each
/// tracing level. The directory is created if it does not exist, and those
/// files older than 7 days are deleted at start, then once a day.
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
            format!(
            "{level},bitcoin=off,corepc=off,tracing=off,aide=off,fjall=off,lsm_tree=off,tower_http=off,rmcp=warn"
        )
            .parse()
            .expect("the default filter parses")
        }
    };

    // Layers see an event in this order: the hook and the tail read the clock before the
    // console and the log file write it out.
    let registry = TracingSubscriberRegistry()
        .with(filter)
        .with(HookLayer)
        .with(TailLayer)
        .with(fmt::layer().event_format(Formatter::<true>));

    if let Some(dir) = dir {
        let writer = RateLimitedFile::new(dir)?;
        registry
            .with(
                fmt::layer()
                    .event_format(Formatter::<false>)
                    .with_writer(writer),
            )
            .init();
        delete_old_daily(dir);
    } else {
        registry.init();
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

/// Registers the process's single hook, which receives every event the filter admits; errors if
/// one is already registered.
pub fn register_hook<F>(hook: F) -> Result<(), &'static str>
where
    F: for<'a> Fn(&Event<'a>) + Send + Sync + 'static,
{
    hook::register(hook)
}
