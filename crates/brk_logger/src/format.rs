use std::fmt::{Debug, Display, Result, Write};

use jiff::{Timestamp, tz};
use owo_colors::{AnsiColors, OwoColorize};
use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

// Don't remove, used to know the target of unwanted logs
const WITH_TARGET: bool = false;
// const WITH_TARGET: bool = true;

const fn level_str(level: Level) -> &'static str {
    match level {
        Level::ERROR => "error",
        Level::WARN => "warn ",
        Level::INFO => "info ",
        Level::DEBUG => "debug",
        Level::TRACE => "trace",
    }
}

pub struct Formatter<const ANSI: bool>;

impl<S, N, const ANSI: bool> FormatEvent<S, N> for Formatter<ANSI>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> Result {
        let ts = Timestamp::now()
            .to_zoned(tz::TimeZone::system())
            .strftime("%Y-%m-%d %H:%M:%S");

        let level = *event.metadata().level();
        let level_str = level_str(level);

        if ANSI {
            write!(writer, "{} ", ts.bright_black())?;
            if WITH_TARGET {
                write!(writer, "{} ", event.metadata().target())?;
            }
            write!(
                writer,
                "{} {} ",
                "-".bright_black(),
                level_str.color(level_color(level))
            )?;
        } else if WITH_TARGET {
            write!(writer, "{ts} {} - {level_str} ", event.metadata().target())?;
        } else {
            write!(writer, "{ts} - {level_str} ")?;
        }

        let mut visitor = FieldVisitor::<ANSI>::new();
        event.record(&mut visitor);
        visitor.write_to(&mut writer)?;
        writeln!(writer)
    }
}

const fn level_color(level: Level) -> AnsiColors {
    match level {
        Level::ERROR => AnsiColors::Red,
        Level::WARN => AnsiColors::Yellow,
        Level::INFO => AnsiColors::Green,
        Level::DEBUG => AnsiColors::Blue,
        Level::TRACE => AnsiColors::Cyan,
    }
}

const fn status_color(status: u64) -> AnsiColors {
    match status {
        200..=299 => AnsiColors::Green,
        300..=399 => AnsiColors::BrightBlack,
        _ => AnsiColors::Red,
    }
}

struct FieldVisitor<const ANSI: bool> {
    message: String,
    fields: String,
    method: Option<String>,
    status: Option<u64>,
    uri: Option<String>,
    latency: Option<String>,
}

impl<const ANSI: bool> FieldVisitor<ANSI> {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: String::new(),
            method: None,
            status: None,
            uri: None,
            latency: None,
        }
    }

    /// Access events (with `status`) print `status [uri] [latency]`; others print
    /// `message — fields`, with method/uri/latency appended as plain fields.
    fn write_to(self, w: &mut impl Write) -> Result {
        if let Some(status) = self.status {
            if ANSI {
                write!(w, "{}", status.color(status_color(status)))?;
            } else {
                write!(w, "{status}")?;
            }
            if let Some(uri) = self.uri {
                write!(w, " {uri}")?;
            }
            if let Some(latency) = self.latency {
                if ANSI {
                    write!(w, " {}", latency.bright_black())?;
                } else {
                    write!(w, " {latency}")?;
                }
            }
            return Ok(());
        }

        w.write_str(&self.message)?;
        let mut sep = if self.message.is_empty() { "" } else { " — " };
        if !self.fields.is_empty() {
            write!(w, "{sep}{}", self.fields)?;
            sep = " ";
        }
        for (name, value) in [
            ("method", self.method),
            ("uri", self.uri),
            ("latency", self.latency),
        ] {
            if let Some(value) = value {
                write!(w, "{sep}{name}={value}")?;
                sep = " ";
            }
        }
        Ok(())
    }

    fn record_field(&mut self, name: &str, value: impl Display) {
        if !self.fields.is_empty() {
            self.fields.push(' ');
        }
        let _ = write!(self.fields, "{name}={value}");
    }
}

impl<const ANSI: bool> Visit for FieldVisitor<ANSI> {
    fn record_u64(&mut self, field: &Field, value: u64) {
        let name = field.name();
        if name == "status" {
            self.status = Some(value);
        } else if !is_internal(name) {
            self.record_field(name, value);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        let name = field.name();
        if !is_internal(name) {
            self.record_field(name, value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        match name {
            "latency" => self.latency = Some(value.to_owned()),
            "message" => self.message.push_str(value),
            "method" => self.method = Some(value.to_owned()),
            "uri" => self.uri = Some(value.to_owned()),
            _ if is_internal(name) => {}
            _ => self.record_field(name, value),
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        let name = field.name();
        match name {
            "method" => self.method = Some(format!("{value:?}")),
            "uri" => self.uri = Some(format!("{value:?}")),
            "latency" => self.latency = Some(format!("{value:?}")),
            "message" => {
                let _ = write!(self.message, "{value:?}");
            }
            _ if is_internal(name) => {}
            _ => self.record_field(name, format_args!("{value:?}")),
        }
    }
}

fn is_internal(name: &str) -> bool {
    name.starts_with("log.") || name.starts_with("internal.")
}
