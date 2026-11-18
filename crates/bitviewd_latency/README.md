# bitviewd_latency

Identify endpoints worth optimizing from existing `bitviewd` access logs.

```sh
cargo run --release -p bitviewd_latency -- ~/.bitview
```

One argument: the daemon's data directory (`bitviewdir`). Reads
`<bitviewdir>/logs` and atomically replaces `<bitviewdir>/latency.md`, then
prints its path. It reads no configuration, creates no directories, and makes no
RPC calls.

Reads every available day's `YYYY-MM-DD.txt` once. Logs are streamed; durations are
retained for exact nearest-rank percentiles, with three full slow examples per
endpoint/status group. Memory scales with request count.

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

- Request lines are read by `brk_logger::RequestLine`, beside the code that writes them. A line whose
  text starts with a status is a request's: one that can't be read is counted as malformed, never
  dropped silently; every other line is counted as non-access.
- An endpoint is a method and a route. Lines written before 2026-11-17 carry no method and group apart.
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
