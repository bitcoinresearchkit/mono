#![doc = include_str!("../README.md")]

use std::{backtrace::Backtrace, env, fs, io, panic, path::Path, time::Duration};

use format::Formatter;
use hook::HookLayer;
use rate_limit::RateLimitedFile;
pub use rate_limit::{LEVEL_NAMES, log_file_date};
pub use tail::{Line, Recent, recent, started};
use tail::TailLayer;
use tracing::{Event, Level, error, field};
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter::Targets, fmt, layer::SubscriberExt, registry as TracingSubscriberRegistry,
    util::SubscriberInitExt,
};

mod format;
mod hook;
mod rate_limit;
mod tail;

const MAX_LOG_AGE_DAYS: u64 = 7;

/// Initialize the global tracing subscriber with a colorized console layer.
///
/// If `dir` is `Some`, also writes daily log files to that directory:
/// `YYYY-MM-DD.txt` for the combined log and `YYYY-MM-DD_<level>.txt` for each
/// tracing level. The directory is created if it does not exist, and those
/// files older than 7 days are pruned on startup.
pub fn init(dir: Option<&Path>) -> io::Result<()> {
    #[cfg(debug_assertions)]
    const DEFAULT_LEVEL: &str = "debug";
    #[cfg(not(debug_assertions))]
    const DEFAULT_LEVEL: &str = "info";

    init_with_default_level(dir, DEFAULT_LEVEL)
}

/// Initialize the logger with a caller-selected fallback level.
///
/// `RUST_LOG` takes precedence over `LOG`, which takes precedence over `default_level`.
pub fn init_with_default_level(dir: Option<&Path>, default_level: &str) -> io::Result<()> {
    LogTracer::init().ok();

    let level = env::var("LOG").unwrap_or_else(|_| default_level.to_string());

    let directives = env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "{level},bitcoin=off,corepc=off,tracing=off,aide=off,fjall=off,lsm_tree=off,tower_http=off,rmcp=warn"
        )
    });

    let filter: Targets = directives
        .parse()
        .unwrap_or_else(|_| Targets::new().with_default(Level::INFO));

    let registry = TracingSubscriberRegistry()
        .with(filter)
        .with(fmt::layer().event_format(Formatter::<true>))
        .with(HookLayer)
        .with(TailLayer);

    if let Some(dir) = dir {
        let writer = RateLimitedFile::new(dir)?;

        cleanup_old_logs(dir);

        registry
            .with(
                fmt::layer()
                    .event_format(Formatter::<false>)
                    .with_writer(writer),
            )
            .init();
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

fn cleanup_old_logs(dir: &Path) {
    let max_age = Duration::from_secs(MAX_LOG_AGE_DAYS * 24 * 60 * 60);
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if log_file_date(name).is_none() {
            continue;
        }

        if let Ok(meta) = path.metadata()
            && let Ok(modified) = meta.modified()
            && let Ok(age) = modified.elapsed()
            && age > max_age
        {
            let _ = fs::remove_file(&path);
        }
    }
}
