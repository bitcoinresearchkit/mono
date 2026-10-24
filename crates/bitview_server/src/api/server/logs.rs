use axum::{
    Json,
    extract::Query,
    response::{IntoResponse, Response},
};
use bitview_types::{LogLevel, LogLine, Logs};
use brk_logger::{Line, Recent};
use tracing::Level;

use crate::{extended::HeaderMapExtended, params::LogsQuery};

pub async fn get(Query(query): Query<LogsQuery>) -> Response {
    // A `seq` from another process (before a restart) counts nothing here: every kept line.
    let after = query
        .after
        .filter(|_| query.started_unix_ms == Some(brk_logger::started().as_millisecond()));
    let Recent { started, lines } = brk_logger::recent(after);
    let mut response = Json(Logs {
        started_unix_ms: started.as_millisecond(),
        lines: lines.into_iter().map(line).collect(),
    })
    .into_response();
    let headers = response.headers_mut();
    headers.insert_cache_control("no-store");
    headers.insert_cdn_cache_control("no-store");
    response
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
