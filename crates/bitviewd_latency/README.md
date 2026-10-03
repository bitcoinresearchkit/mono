# bitviewd_latency

Identify endpoints worth optimizing from existing `bitviewd` access logs.

```sh
cargo run --release -p bitviewd_latency
```

No flags. Uses the existing `bitviewd::Config::load()` to obtain the daemon's
configured data directory. Reads `<bitviewdir>/logs` and atomically replaces
`<bitviewdir>/latency.md`, then prints its path. Normally these are
`~/.bitview/logs` and `~/.bitview/latency.md`.

The analyzer does not parse TOML or maintain its own configuration. It uses the
same persisted settings loader and server-path resolution as the daemon, without
requiring Bitcoin directories or RPC credentials. Loading settings does not create
directories, start a node, or make RPC calls.
Daemon command-line overrides are not persisted; use the daemon config file to
keep both programs pointed at the same data directory.

Reads every available day's combined `YYYY-MM-DD.txt` once. If absent, reads
that day's per-level files. Logs are streamed; durations are retained for exact
nearest-rank percentiles, with three full slow examples per endpoint/status group.
Memory scales with request count.

Each HTTP status code (200, 304, 400, 404, etc.) has one table of up to 20 endpoints
sorted by P95, a histogram and up to 20 slow request examples.
Tables include count, median/P50, P95, P99, P99.9, average, maximum, accumulated
time and endpoint error percentage. Error rates use all captured statuses for
that endpoint. Groups below 100 samples are marked; P99.9 is omitted below 1,000
samples and marked low-confidence below 10,000.

Route templates come from the generated CLI API catalog embedded at build time.
Literal routes take precedence over parameter routes. Unknown paths are marked
`[unmatched]` in endpoint tables and summarized by request count across statuses,
with query strings removed. Slow examples retain query strings. Rebuild after
regenerating the API catalog. No extra fields are added to server logs.

## Interpretation

- Supports the current `brk_logger` plain-text access format. Non-access lines and
  malformed access-shaped records are counted separately. Unsupported older
  formats may appear as non-access lines; inspect coverage before interpreting results.
- HTTP methods are omitted by the logger, so methods cannot be separated.
- File logging caps events at 100/second/level. Filtering and rate limiting mean
  these are captured-traffic statistics, not guaranteed whole-traffic percentiles.
- Historical 304/400 debug events may be absent from info-level logs.
- Durations end when a server response is produced, before complete body transfer.
  They include waiting; accumulated duration is not CPU time.
- Timestamps use the logger's local wall clock without timezone offsets.
- A live log is read to the EOF observed during the scan. A partial final access
  line is counted as malformed.
- The daemon normally retains seven days of logs. Each run replaces the report;
  the analyzer does not maintain historical reports.

Aggregation lives in `analysis.rs` and `group.rs`; `report.rs` renders their result
as Markdown without reading logs or configuration.
