use jiff::civil::DateTime;

#[derive(Clone)]
pub struct Record {
    pub timestamp: DateTime,
    pub status: u16,
    pub uri: String,
    pub nanos: u64,
}

impl Record {
    // Current brk_logger::Formatter file output. Non-access lines are ignored;
    // access-shaped lines with invalid timestamps/durations are reported separately.
    pub fn parse(line: &str) -> Result<Option<Self>, ()> {
        let Some((prefix, message)) = line.split_once(" - ") else {
            return Ok(None);
        };
        let mut fields = message.split_whitespace();
        let Some(level) = fields.next() else {
            return Ok(None);
        };
        if !["info", "debug", "error", "warn", "trace"].contains(&level) {
            return Ok(None);
        }
        let Some(status) = fields.next().and_then(|s| s.parse::<u16>().ok()) else {
            return Ok(None);
        };
        if !(100..600).contains(&status) {
            return Ok(None);
        }
        let uri = fields.next().ok_or(())?;
        if !uri.starts_with('/') {
            return Ok(None);
        }
        let nanos = duration(fields.next().ok_or(())?).ok_or(())?;
        if fields.next().is_some() {
            return Err(());
        }
        let timestamp = prefix.parse().map_err(|_| ())?;
        Ok(Some(Self {
            timestamp,
            status,
            uri: uri.to_string(),
            nanos,
        }))
    }
}

fn duration(text: &str) -> Option<u64> {
    for (unit, scale) in [
        ("ns", 1.),
        ("µs", 1e3),
        ("μs", 1e3),
        ("us", 1e3),
        ("ms", 1e6),
        ("s", 1e9),
    ] {
        if let Some(value) = text.strip_suffix(unit) {
            let n = value.parse::<f64>().ok()? * scale;
            return (n.is_finite() && n >= 0. && n < u64::MAX as f64).then(|| n.round() as u64);
        }
    }
    None
}
