use brk_logger::{MalformedRequestLine, RequestLine};
use jiff::civil::DateTime;

pub struct Record {
    pub timestamp: DateTime,
    pub status: u16,
    pub method: Option<String>,
    pub uri: String,
    pub nanos: u64,
}

impl Record {
    /// `Ok(None)` for a line that isn't a request's; `Err` for a request's line that can't be read.
    pub fn parse(line: &str) -> Result<Option<Self>, MalformedRequestLine> {
        let Some(request) = RequestLine::parse(line)? else {
            return Ok(None);
        };
        Ok(Some(Self {
            timestamp: request.time.parse().map_err(|_| MalformedRequestLine)?,
            status: request.status,
            method: request.method.map(str::to_string),
            uri: request.uri.to_string(),
            nanos: duration(request.latency).ok_or(MalformedRequestLine)?,
        }))
    }

    /// `path` (this request's URI or its route) after the method, when the line has one.
    pub fn with_method(&self, path: &str) -> String {
        match &self.method {
            Some(method) => format!("{method} {path}"),
            None => path.to_string(),
        }
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
