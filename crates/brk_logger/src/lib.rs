#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

mod format;
mod hook;
mod rate_limit;

use std::{backtrace::Backtrace, env, fs, io, panic, path::Path, time::Duration};

use tracing::Event;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use format::Formatter;
use hook::HookLayer;
use rate_limit::{RateLimitedFile, is_log_file};

/// Days to keep log files before cleanup
const MAX_LOG_AGE_DAYS: u64 = 7;

/// Initialize the global tracing subscriber with a colorized console layer.
///
/// If `dir` is `Some`, also writes daily log files to that directory:
/// `YYYY-MM-DD.txt` for the combined log and `YYYY-MM-DD_<level>.txt` for each
/// tracing level. The directory is created if it does not exist, and any
/// `*.txt` file older than 7 days is pruned on startup.
pub fn init(dir: Option<&Path>) -> io::Result<()> {
    #[cfg(debug_assertions)]
    const DEFAULT_LEVEL: &str = "debug";
    #[cfg(not(debug_assertions))]
    const DEFAULT_LEVEL: &str = "info";

    init_with_default_level(dir, DEFAULT_LEVEL)
}

/// Initialize the logger with a caller-selected fallback level.
///
/// `LOG` and `RUST_LOG` still take precedence. This is useful for services
/// whose normal debug traffic is too verbose for their default execution mode.
pub fn init_with_default_level(dir: Option<&Path>, default_level: &str) -> io::Result<()> {
    tracing_log::LogTracer::init().ok();
    install_panic_hook();

    let level = env::var("LOG").unwrap_or_else(|_| default_level.to_string());

    let directives = env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "{level},bitcoin=off,corepc=off,tracing=off,aide=off,fjall=off,lsm_tree=off,tower_http=off,rmcp=warn"
        )
    });

    let filter: Targets = directives
        .parse()
        .unwrap_or_else(|_| Targets::new().with_default(tracing::Level::INFO));

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().event_format(Formatter::<true>))
        .with(HookLayer);

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

    Ok(())
}

fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<&str>()
            .copied()
            .map(str::to_owned)
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Box<dyn Any>".to_owned());
        let backtrace = Backtrace::capture();
        tracing::error!(location, backtrace = %backtrace, "panic: {msg}");
    }));
}

/// Register a hook that receives every tracing event.
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
        if !is_log_file(name) {
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
