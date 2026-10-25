use axum::{
    Json,
    extract::Query,
    response::{IntoResponse, Response},
};
use bitview_types::{LogLevel, LogLine, Logs};
use brk_logger::{Line, Recent};
use tracing::Level;

use crate::{
    Error, cache::CDN_LIVE, error::Result, extended::HeaderMapExtended, params::LogsQuery,
};

pub async fn get(Query(query): Query<LogsQuery>) -> Result<Response> {
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
    let mut response = Json(Logs {
        started_unix_ms: started,
        lines: lines.into_iter().map(line).collect(),
        next: format!("{started}-{next}"),
    })
    .into_response();
    let headers = response.headers_mut();
    // A second, for browsers and the CDN alike: readers polling together share one answer.
    headers.insert_cache_control(CDN_LIVE);
    headers.insert_cdn_cache_control(CDN_LIVE);
    Ok(response)
}

/// A `next`, read: the process it's of, and the `seq` it reads on from.
fn cursor(after: &str) -> Result<(i64, u64)> {
    after
        .rsplit_once('-')
        .and_then(|(process, seq)| Some((process.parse().ok()?, seq.parse().ok()?)))
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
