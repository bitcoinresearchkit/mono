use std::{
    fmt::{self, Debug, Display, Write},
    result::Result as StdResult,
};

use owo_colors::{AnsiColors, OwoColorize};
use tracing::{
    Level,
    field::{Field, Visit},
};

use crate::file::LEVEL_NAMES;

// Don't remove: set to true to print each event's target, to find the source of unwanted logs.
pub const WITH_TARGET: bool = false;

pub const fn level_str(level: Level) -> &'static str {
    match level {
        Level::ERROR => "error",
        Level::WARN => "warn ",
        Level::INFO => "info ",
        Level::DEBUG => "debug",
        Level::TRACE => "trace",
    }
}

pub const fn level_color(level: Level) -> AnsiColors {
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

/// An event's fields, read once for every place the event is written.
#[derive(Default)]
pub struct Fields {
    message: String,
    fields: String,
    method: Option<String>,
    status: Option<u64>,
    uri: Option<String>,
    latency: Option<String>,
}

impl Fields {
    /// What a line shows after its level, coloured for the console or plain for the log file.
    /// Access events (with `status`) print `status [method] [uri] [latency] [fields]`; others print
    /// `message — fields`, with method/uri/latency appended as plain fields.
    /// [`RequestLine::parse`] reads the plain access form back: the two change together.
    pub fn text(&self, ansi: bool) -> impl Display {
        fmt::from_fn(move |w| {
            if let Some(status) = self.status {
                if ansi {
                    write!(w, "{}", status.color(status_color(status)))?;
                } else {
                    write!(w, "{status}")?;
                }
                for part in [&self.method, &self.uri].into_iter().flatten() {
                    write!(w, " {part}")?;
                }
                if let Some(latency) = &self.latency {
                    if ansi {
                        write!(w, " {}", latency.bright_black())?;
                    } else {
                        write!(w, " {latency}")?;
                    }
                }
                if !self.fields.is_empty() {
                    write!(w, " {}", self.fields)?;
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
                ("method", &self.method),
                ("uri", &self.uri),
                ("latency", &self.latency),
            ] {
                if let Some(value) = value {
                    write!(w, "{sep}{name}={value}")?;
                    sep = " ";
                }
            }
            Ok(())
        })
    }

    fn record_field(&mut self, name: &str, value: impl Display) {
        if !self.fields.is_empty() {
            self.fields.push(' ');
        }
        let _ = write!(self.fields, "{name}={value}");
    }
}

/// A request's line in a log file, `<time> - <level> <status> [<method>] <uri> <latency> [<name>=<value>…]`:
/// what the log file holds for the server's request events ([`Fields::text`] after the time and level).
pub struct RequestLine<'a> {
    /// `YYYY-MM-DD HH:MM:SS`, in the logger's local time.
    pub time: &'a str,
    pub status: u16,
    /// `None` on lines written before requests logged their method (2026-11-17).
    pub method: Option<&'a str>,
    pub uri: &'a str,
    /// A `Duration` as `{:?}` prints it.
    pub latency: &'a str,
}

/// A line that starts as a request's (a status after the level) and doesn't go on as one.
pub struct MalformedRequestLine;

impl<'a> RequestLine<'a> {
    /// `Ok(None)` for every other line of the log: only a request's text starts with a status.
    pub fn parse(line: &'a str) -> StdResult<Option<Self>, MalformedRequestLine> {
        let Some((time, text)) = line.split_once(" - ") else {
            return Ok(None);
        };
        let mut words = text.split_ascii_whitespace();
        if !words.next().is_some_and(|level| LEVEL_NAMES.contains(&level)) {
            return Ok(None);
        }
        let status = words.next().and_then(|word| word.parse().ok());
        let Some(status) = status.filter(|status| (100..600).contains(status)) else {
            return Ok(None);
        };
        let mut word = || words.next().ok_or(MalformedRequestLine);
        let first = word()?;
        let (method, uri) = if first.starts_with('/') { (None, first) } else { (Some(first), word()?) };
        Ok(Some(Self { time, status, method, uri, latency: word()? }))
    }
}

impl Visit for Fields {
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
