use std::fmt::Write;

use jiff::{Timestamp, tz};
use owo_colors::OwoColorize;
use tracing::{Event, Level, Subscriber, field::Field};
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
    ) -> std::fmt::Result {
        let ts = Timestamp::now()
            .to_zoned(tz::TimeZone::system())
            .strftime("%Y-%m-%d %H:%M:%S")
            .to_string();

        let level = *event.metadata().level();
        let level_str = level_str(level);

        if ANSI {
            let level_colored = match level {
                Level::ERROR => level_str.red().to_string(),
                Level::WARN => level_str.yellow().to_string(),
                Level::INFO => level_str.green().to_string(),
                Level::DEBUG => level_str.blue().to_string(),
                Level::TRACE => level_str.cyan().to_string(),
            };
            if WITH_TARGET {
                write!(
                    writer,
                    "{} {} {} {level_colored} ",
                    ts.bright_black(),
                    event.metadata().target(),
                    "-".bright_black(),
                )?;
            } else {
                write!(
                    writer,
                    "{} {} {level_colored} ",
                    ts.bright_black(),
                    "-".bright_black()
                )?;
            }
        } else if WITH_TARGET {
            write!(writer, "{ts} {} - {level_str} ", event.metadata().target())?;
        } else {
            write!(writer, "{ts} - {level_str} ")?;
        }

        let mut visitor = FieldVisitor::<ANSI>::new();
        event.record(&mut visitor);
        write!(writer, "{}", visitor.finish())?;
        writeln!(writer)
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

    fn finish(self) -> String {
        if let Some(status) = self.status {
            let status_str = if ANSI {
                match status {
                    200..=299 => status.green().to_string(),
                    300..=399 => status.bright_black().to_string(),
                    _ => status.red().to_string(),
                }
            } else {
                status.to_string()
            };

            let mut parts = Vec::with_capacity(3);
            parts.push(status_str);
            if let Some(uri) = self.uri {
                parts.push(uri);
            }
            if let Some(latency) = self.latency {
                parts.push(if ANSI {
                    latency.bright_black().to_string()
                } else {
                    latency
                });
            }

            parts.join(" ")
        } else {
            let mut fields = self.fields;
            if let Some(method) = self.method {
                push_field(&mut fields, "method", method);
            }
            if let Some(uri) = self.uri {
                push_field(&mut fields, "uri", uri);
            }
            if let Some(latency) = self.latency {
                push_field(&mut fields, "latency", latency);
            }

            if self.message.is_empty() {
                fields
            } else if fields.is_empty() {
                self.message
            } else {
                format!("{} — {fields}", self.message)
            }
        }
    }

    fn record_field(&mut self, name: &str, value: impl std::fmt::Display) {
        push_field(&mut self.fields, name, value);
    }
}

fn push_field(fields: &mut String, name: &str, value: impl std::fmt::Display) {
    if !fields.is_empty() {
        fields.push(' ');
    }
    let _ = write!(fields, "{name}={value}");
}

impl<const ANSI: bool> tracing::field::Visit for FieldVisitor<ANSI> {
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

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
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

#[cfg(test)]
mod tests {
    use super::{FieldVisitor, is_internal};

    #[test]
    fn hides_logger_and_internal_fields() {
        assert!(is_internal("log.target"));
        assert!(is_internal("internal.duration_ns"));
        assert!(!is_internal("duration_ns"));
    }

    #[test]
    fn separates_messages_from_structured_fields() {
        let mut visitor = FieldVisitor::<false>::new();
        visitor.message = "Starting server".to_owned();
        visitor.fields = "bind=127.0.0.1:3110 tools=42".to_owned();

        assert_eq!(
            visitor.finish(),
            "Starting server — bind=127.0.0.1:3110 tools=42"
        );
    }

    #[test]
    fn formats_http_access_events() {
        let mut visitor = FieldVisitor::<false>::new();
        visitor.method = Some("GET".to_owned());
        visitor.status = Some(200);
        visitor.uri = Some("/api".to_owned());
        visitor.latency = Some("1.25ms".to_owned());

        assert_eq!(visitor.finish(), "200 /api 1.25ms");
    }

    #[test]
    fn retains_special_fields_without_an_http_status() {
        let mut visitor = FieldVisitor::<false>::new();
        visitor.message = "Request failed".to_owned();
        visitor.fields = "error=timeout".to_owned();
        visitor.latency = Some("1.25ms".to_owned());

        assert_eq!(
            visitor.finish(),
            "Request failed — error=timeout latency=1.25ms"
        );
    }
}
