use axum::{extract::Query, http::HeaderMap, response::Response};
use bitview_types::{LogLevel, LogLine, Logs};
use brk_logger::{Line, Recent};
use tracing::Level;

use crate::{
    Error,
    answer::{Answer, Content},
    error::Result,
    params::LogsQuery,
    request_state::RequestState,
};

pub async fn get(
    headers: HeaderMap,
    Query(query): Query<LogsQuery>,
    RequestState(state): RequestState,
) -> Result<Response> {
    let started = brk_logger::started().as_millisecond();
    // A cursor from another process (before a restart) counts nothing here: every kept line.
    let from = match query.after.as_deref().map(cursor).transpose()? {
        Some((process, seq)) if process == started => Some(seq),
        _ => None,
    };
    let Recent {
        started,
        next,
        lines,
    } = brk_logger::recent(from);
    let started = started.as_millisecond();
    let logs = Logs {
        started_unix_ms: started,
        lines: lines.into_iter().map(line).collect(),
        next: format!("{started}-{next}"),
    };
    // Readers polling together share one answer for a second, as any live one.
    Ok(state.respond_now(&headers, Answer::built(Content::json(&logs)?)))
}

/// A `next`, read: the process it's of, and the `seq` it reads on from.
fn cursor(after: &str) -> Result<(i64, u64)> {
    after
        .rsplit_once('-')
        .and_then(|(process, seq)| Some((process.parse().ok()?, seq.parse().ok()?)))
        .filter(|(process, seq)| format!("{process}-{seq}") == after)
        .ok_or_else(|| Error::bad_request("`after` takes an answer's `next`, as it is"))
}

fn line(line: Line) -> LogLine {
    LogLine {
        seq: line.seq,
        unix_ms: line.time.as_millisecond(),
        level: match line.level {
            Level::ERROR => LogLevel::Error,
            Level::WARN => LogLevel::Warn,
            Level::INFO => LogLevel::Info,
            Level::DEBUG => LogLevel::Debug,
            Level::TRACE => LogLevel::Trace,
        },
        text: line.text,
    }
}
