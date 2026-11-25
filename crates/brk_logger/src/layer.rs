use std::{
    io::{self, Write},
    sync::OnceLock,
};

use jiff::{Timestamp, tz::TimeZone};
use owo_colors::OwoColorize;
use tracing::{Event, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

use crate::{
    file::LogFile,
    format::{Fields, WITH_TARGET, level_color, level_str},
    tail,
};

type Hook = dyn for<'a> Fn(&Event<'a>) + Send + Sync;

static HOOK: OnceLock<Box<Hook>> = OnceLock::new();

/// Registers the process's single hook, which receives every event the filter admits; errors if
/// one is already registered.
pub fn register_hook<F>(hook: F) -> Result<(), &'static str>
where
    F: for<'a> Fn(&Event<'a>) + Send + Sync + 'static,
{
    HOOK.set(Box::new(hook))
        .map_err(|_| "Hook already registered")
}

/// Writes each event the filter admits to the console, to the day's log file when there is one, and
/// to the tail kept in memory, after the hook has seen it.
pub struct Logger {
    pub file: Option<LogFile>,
}

impl<S: Subscriber> Layer<S> for Logger {
    /// Runs on the thread that logs, a request's own for its line: the clock and the event's fields
    /// are read once, and each destination gets one write.
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        if let Some(hook) = HOOK.get() {
            hook(event);
        }

        let now = Timestamp::now();
        let time = now
            .to_zoned_datetime(TimeZone::system())
            .strftime("%Y-%m-%d %H:%M:%S")
            .to_string();
        let meta = event.metadata();
        let level = level_str(*meta.level());
        let (target, gap) = if WITH_TARGET { (meta.target(), " ") } else { ("", "") };
        let mut fields = Fields::default();
        event.record(&mut fields);

        let text = fields.text(false).to_string();
        let console = format!(
            "{} {target}{gap}{} {} {}\n",
            time.bright_black(),
            "-".bright_black(),
            level.color(level_color(*meta.level())),
            fields.text(true)
        );
        report("console", io::stdout().lock().write_all(console.as_bytes()));
        if let Some(file) = &self.file {
            let line = format!("{time} {target}{gap}- {level} {text}\n");
            report(
                "log file",
                file.write(now.as_second(), &time[..10], *meta.level(), &line),
            );
        }
        tail::push(now, *meta.level(), text);
    }
}

/// A line that can't be written is said on stderr, the one place left to say it.
fn report(what: &str, result: io::Result<()>) {
    if let Err(error) = result {
        let _ = writeln!(io::stderr(), "brk_logger: {what}: {error}");
    }
}
