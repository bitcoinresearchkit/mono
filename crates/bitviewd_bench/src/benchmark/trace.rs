use std::{
    fmt::{Debug, Write as _},
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    time::Instant,
};

use parking_lot::Mutex;
use tracing::{
    Event,
    field::{Field, Visit},
};

const PROGRESS_PATTERNS: [&str; 2] = ["block ", "chain at "];

pub struct TraceMonitor(Mutex<State>);

struct State {
    started_at: Option<Instant>,
    timings: BufWriter<File>,
    progress: BufWriter<File>,
    error: Option<io::Error>,
}

#[derive(Default)]
struct Fields {
    message: String,
    phase: Option<String>,
    plugin: Option<String>,
    duration_ns: Option<u64>,
}

impl TraceMonitor {
    pub fn new(path: &Path) -> io::Result<Self> {
        let mut timings = BufWriter::new(File::create(path.join("timings.csv"))?);
        writeln!(timings, "phase,plugin,start_ms,duration_ms")?;
        let mut progress = BufWriter::new(File::create(path.join("progress.csv"))?);
        writeln!(progress, "timestamp_ms,height")?;

        Ok(Self(Mutex::new(State {
            started_at: None,
            timings,
            progress,
            error: None,
        })))
    }

    pub fn start(&self, started_at: Instant) {
        self.0.lock().started_at = Some(started_at);
    }

    pub fn record(&self, event: &Event<'_>) {
        let mut state = self.0.lock();
        let Some(started_at) = state.started_at else {
            return;
        };

        let mut fields = Fields::default();
        event.record(&mut fields);
        let end_ms = started_at.elapsed().as_secs_f64() * 1_000.0;

        let result = if let (Some(phase), Some(plugin), Some(duration_ns)) =
            (fields.phase, fields.plugin, fields.duration_ns)
        {
            let duration_ms = duration_ns as f64 / 1_000_000.0;
            let start_ms = (end_ms - duration_ms).max(0.0);
            writeln!(
                state.timings,
                "{phase},{plugin},{start_ms:.3},{duration_ms:.3}"
            )
        } else if let Some(height) = progress(&fields.message)
            && height % 10 == 0
        {
            writeln!(state.progress, "{end_ms:.3},{height}")
        } else {
            Ok(())
        };

        if let Err(error) = result
            && state.error.is_none()
        {
            state.error = Some(error);
        }
    }

    pub fn finish(&self) -> io::Result<()> {
        let mut state = self.0.lock();
        state.started_at = None;
        state.timings.flush()?;
        state.progress.flush()?;
        match state.error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Visit for Fields {
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "internal.duration_ns" {
            self.duration_ns = Some(value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => self.message.push_str(value),
            "internal.phase" => self.phase = Some(value.to_owned()),
            "internal.plugin" => self.plugin = Some(value.to_owned()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        }
    }
}

fn progress(message: &str) -> Option<u64> {
    PROGRESS_PATTERNS.iter().find_map(|pattern| {
        let start = message.find(pattern)? + pattern.len();
        let value = &message[start..];
        let end = value
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(value.len());
        if end == 0 {
            None
        } else {
            value[..end].parse().ok()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::progress;

    #[test]
    fn extracts_supported_progress_messages() {
        assert_eq!(progress("Indexing block 123..."), Some(123));
        assert_eq!(progress("Processing chain at 456..."), Some(456));
        assert_eq!(progress("Imported blocks"), None);
    }
}
