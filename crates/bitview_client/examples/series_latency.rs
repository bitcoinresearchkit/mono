//! Sweep every series of a running `bitviewd` and rank them by latency.
//!
//! ```sh
//! cargo run --release -p bitview_client --example series_latency
//! cargo run --release -p bitview_client --example series_latency -- --filter supply_delta --repeat 3
//! cargo run --release -p bitview_client --example series_latency -- --index height --limit 2000
//! cargo run --release -p bitview_client --example series_latency -- --concurrency 8
//! ```
//!
//! Reads the catalog from `/api/series`, then requests every series that has
//! the chosen index the way the website does
//! (`/api/series/<name>/day1?start=-10000` by default). Each request records
//! the server's own timing (`X-Response-Time`), the client-side time and the
//! body size. Results are ranked from slowest to fastest.
//!
//! Defaults isolate the cost of each series:
//!
//! - `--concurrency 1`: series requests share bounded admission in the server
//!   (query workers = CPU count, 2 response bodies in flight), so concurrent
//!   requests queue behind each other. That is why trivial series such as
//!   `constant_50` can log ~18ms while a slow series runs. Raise it to
//!   reproduce browser-like contention instead.
//! - `--repeat 2`: the first request is cold (decoded-page cache not yet warm
//!   for this series), the best of the remaining requests is warm. Both are
//!   reported; ranking uses warm unless `--sort cold`.
//! - Requests never send `If-None-Match`, so the server always builds the body.
//!
//! Writes `<out>.md` (report) and `<out>.tsv` (every series), by default under
//! `tmp/series-latency/` relative to the current directory, and prints the
//! report.

use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Write as _},
    fs,
    path::Path,
    process::ExitCode,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use ureq::Agent;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const USAGE: &str = "\
Usage: series_latency [options]

  --url <url>            bitviewd base URL (default: http://localhost:3110)
  --index <index>        canonical index requested for every series (default: day1)
  --query <query>        query string for each request (default: start=-10000)
  --repeat <n>           requests per series; first = cold, best of rest = warm (default: 2)
  --concurrency <n>      parallel workers (default: 1)
  --filter <substring>   only series whose name contains it (repeatable, OR)
  --limit <n>            stop after n series (default: all)
  --sort <warm|cold>     ranking metric (default: warm)
  --top <n>              rows in the report tables (default: 50)
  --group-depth <n>      catalog path segments (leaf side) used to group series (default: 3)
  --timeout <seconds>    per-request timeout (default: 120)
  --out <prefix>         output path prefix (default: tmp/series-latency/<index>-<unix time>)
";

struct Options {
    url: String,
    index: String,
    query: String,
    repeat: usize,
    concurrency: usize,
    filters: Vec<String>,
    limit: usize,
    sort_cold: bool,
    top: usize,
    group_depth: usize,
    timeout: Duration,
    out: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            url: "http://localhost:3110".to_string(),
            index: "day1".to_string(),
            query: "start=-10000".to_string(),
            repeat: 2,
            concurrency: 1,
            filters: Vec::new(),
            limit: 0,
            sort_cold: false,
            top: 50,
            group_depth: 3,
            timeout: Duration::from_secs(120),
            out: None,
        }
    }
}

impl Options {
    fn parse() -> Result<Self> {
        let mut options = Self::default();
        let mut args = env::args().skip(1);
        while let Some(flag) = args.next() {
            if flag == "-h" || flag == "--help" {
                print!("{USAGE}");
                std::process::exit(0);
            }
            let mut value = || {
                args.next()
                    .ok_or_else(|| format!("{flag} needs a value\n\n{USAGE}"))
            };
            match flag.as_str() {
                "--url" => options.url = value()?.trim_end_matches('/').to_string(),
                "--index" => options.index = value()?,
                "--query" => options.query = value()?.trim_start_matches('?').to_string(),
                "--repeat" => options.repeat = value()?.parse()?,
                "--concurrency" => options.concurrency = value()?.parse()?,
                "--filter" => options.filters.push(value()?),
                "--limit" => options.limit = value()?.parse()?,
                "--sort" => {
                    options.sort_cold = match value()?.as_str() {
                        "warm" => false,
                        "cold" => true,
                        other => {
                            return Err(format!("--sort must be warm or cold, got {other}").into());
                        }
                    }
                }
                "--top" => options.top = value()?.parse()?,
                "--group-depth" => options.group_depth = value()?.parse()?,
                "--timeout" => options.timeout = Duration::from_secs_f64(value()?.parse()?),
                "--out" => options.out = Some(value()?),
                _ => return Err(format!("unknown argument: {flag}\n\n{USAGE}").into()),
            }
        }
        if options.repeat == 0 || options.concurrency == 0 || options.group_depth == 0 {
            return Err("--repeat, --concurrency and --group-depth must be at least 1".into());
        }
        Ok(options)
    }

    fn sort_label(&self) -> &'static str {
        if self.sort_cold { "cold" } else { "warm" }
    }
}

struct Series {
    name: String,
    path: Vec<String>,
    kind: String,
}

impl Series {
    fn path(&self) -> String {
        self.path.join(".")
    }

    fn group(&self, depth: usize) -> String {
        if self.path.is_empty() {
            return self.name.clone();
        }
        self.path[self.path.len().saturating_sub(depth)..].join(".")
    }
}

#[derive(Clone)]
struct Sample {
    bytes: usize,
    server_ms: Option<f64>,
    client_ms: f64,
}

impl Sample {
    /// Server timing when the header is present, client timing otherwise.
    fn ms(&self) -> f64 {
        self.server_ms.unwrap_or(self.client_ms)
    }
}

struct Outcome {
    status: Option<u16>,
    cold: Option<Sample>,
    warm: Option<Sample>,
    error: Option<String>,
}

impl Outcome {
    fn ranked(&self, options: &Options) -> Option<&Sample> {
        if options.sort_cold {
            self.cold.as_ref()
        } else {
            self.warm.as_ref()
        }
    }
}

struct Response {
    status: u16,
    body: Vec<u8>,
    server_ms: Option<f64>,
    client_ms: f64,
}

fn get(agent: &Agent, url: &str) -> Result<Response> {
    let start = Instant::now();
    let mut response = agent.get(url).call()?;
    let status = response.status().as_u16();
    let server_ms = response
        .headers()
        .get("x-response-time")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_response_time);
    let body = response
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_vec()?;
    Ok(Response {
        status,
        body,
        server_ms,
        client_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Parses the server's `X-Response-Time` header (`"1234us"`) into milliseconds.
fn parse_response_time(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(us) = value.strip_suffix("us") {
        return us.parse::<f64>().ok().map(|us| us / 1000.0);
    }
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.parse().ok();
    }
    value.parse().ok()
}

/// Collects catalog leaves with their path. A leaf is an object with a string
/// `name` and an `indexes` array; every other object is a branch.
fn collect_leaves(node: &Value, path: &mut Vec<String>, out: &mut Vec<(Series, Vec<String>)>) {
    let Value::Object(map) = node else {
        return;
    };
    if let (Some(Value::String(name)), Some(Value::Array(indexes))) =
        (map.get("name"), map.get("indexes"))
    {
        let indexes = indexes
            .iter()
            .filter_map(|index| index.as_str().map(str::to_string))
            .collect();
        let kind = map
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        out.push((
            Series {
                name: name.clone(),
                path: path.clone(),
                kind,
            },
            indexes,
        ));
        return;
    }
    for (key, child) in map {
        path.push(key.clone());
        collect_leaves(child, path, out);
        path.pop();
    }
}

struct Catalog {
    series: Vec<Series>,
    total: usize,
    without_index: usize,
    fetch_ms: f64,
}

fn load_catalog(agent: &Agent, options: &Options) -> Result<Catalog> {
    let response = get(agent, &format!("{}/api/series", options.url))?;
    if response.status != 200 {
        return Err(format!("GET /api/series returned {}", response.status).into());
    }
    let tree: Value = serde_json::from_slice(&response.body)?;
    let mut leaves = Vec::new();
    collect_leaves(&tree, &mut Vec::new(), &mut leaves);

    let mut seen = std::collections::HashSet::new();
    let mut series = Vec::new();
    let mut without_index = 0;
    for (leaf, indexes) in leaves {
        if !seen.insert(leaf.name.clone()) {
            continue;
        }
        if !options.filters.is_empty()
            && !options
                .filters
                .iter()
                .any(|filter| leaf.name.contains(filter.as_str()))
        {
            continue;
        }
        if !indexes.contains(&options.index) {
            without_index += 1;
            continue;
        }
        series.push(leaf);
    }
    if options.limit > 0 {
        series.truncate(options.limit);
    }
    Ok(Catalog {
        series,
        total: seen.len(),
        without_index,
        fetch_ms: response.client_ms,
    })
}

fn measure_one(agent: &Agent, options: &Options, series: &Series) -> Outcome {
    let mut url = format!(
        "{}/api/series/{}/{}",
        options.url, series.name, options.index
    );
    if !options.query.is_empty() {
        url.push('?');
        url.push_str(&options.query);
    }

    let mut samples: Vec<Sample> = Vec::with_capacity(options.repeat);
    for _ in 0..options.repeat {
        let response = match get(agent, &url) {
            Ok(response) => response,
            Err(error) => {
                return Outcome {
                    status: None,
                    cold: samples.first().cloned(),
                    warm: None,
                    error: Some(error.to_string()),
                };
            }
        };
        if response.status != 200 {
            let end = response.body.len().min(300);
            let message = String::from_utf8_lossy(&response.body[..end]).replace(['\n', '\t'], " ");
            return Outcome {
                status: Some(response.status),
                cold: samples.first().cloned(),
                warm: None,
                error: Some(message),
            };
        }
        samples.push(Sample {
            bytes: response.body.len(),
            server_ms: response.server_ms,
            client_ms: response.client_ms,
        });
    }

    let cold = samples[0].clone();
    let warm = samples[1..]
        .iter()
        .min_by(|a, b| a.ms().total_cmp(&b.ms()))
        .cloned()
        .unwrap_or_else(|| cold.clone());
    Outcome {
        status: Some(200),
        cold: Some(cold),
        warm: Some(warm),
        error: None,
    }
}

fn measure(agent: &Agent, options: &Options, series: &[Series]) -> (Vec<Outcome>, Duration) {
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<Outcome>>> = series.iter().map(|_| Mutex::new(None)).collect();
    let start = Instant::now();
    let total = series.len();

    thread::scope(|scope| {
        for _ in 0..options.concurrency {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(entry) = series.get(i) else {
                        break;
                    };
                    let outcome = measure_one(agent, options, entry);
                    *results[i].lock().unwrap() = Some(outcome);
                    let finished = done.fetch_add(1, Ordering::Relaxed) + 1;
                    if finished.is_multiple_of(200) || finished == total {
                        let elapsed = start.elapsed().as_secs_f64();
                        let left = elapsed / finished as f64 * (total - finished) as f64;
                        eprint!("\r{finished}/{total} series  {elapsed:7.1}s elapsed  ~{left:7.1}s left   ");
                    }
                }
            });
        }
    });
    eprintln!();

    let outcomes = results
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap()
                .expect("every series is measured")
        })
        .collect();
    (outcomes, start.elapsed())
}

/// Nearest-rank percentile of sorted values.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

fn fmt_ms(value: Option<f64>) -> String {
    match value {
        None => "–".to_string(),
        Some(v) if v >= 100.0 => format!("{v:.0}"),
        Some(v) if v >= 10.0 => format!("{v:.1}"),
        Some(v) => format!("{v:.2}"),
    }
}

fn fmt_bytes(bytes: Option<usize>) -> String {
    let Some(bytes) = bytes else {
        return "–".to_string();
    };
    let mut value = bytes as f64;
    for unit in ["B", "KB", "MB"] {
        if value < 1024.0 {
            return if unit == "B" {
                format!("{bytes} B")
            } else {
                format!("{value:.1} {unit}")
            };
        }
        value /= 1024.0;
    }
    format!("{value:.1} GB")
}

fn report(
    options: &Options,
    catalog: &Catalog,
    outcomes: &[Outcome],
    order: &[usize],
    wall: Duration,
) -> std::result::Result<String, fmt::Error> {
    let series = &catalog.series;
    let failed: Vec<usize> = (0..outcomes.len())
        .filter(|&i| outcomes[i].error.is_some())
        .collect();
    let label = options.sort_label();
    let mut ranked: Vec<f64> = order
        .iter()
        .filter_map(|&i| outcomes[i].ranked(options).map(Sample::ms))
        .collect();
    ranked.sort_by(f64::total_cmp);
    let mut cold: Vec<f64> = order
        .iter()
        .filter_map(|&i| outcomes[i].cold.as_ref().map(Sample::ms))
        .collect();
    cold.sort_by(f64::total_cmp);

    let mut r = String::new();
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    writeln!(r, "# Series latency sweep (unix time {unix})")?;
    writeln!(r)?;
    writeln!(
        r,
        "- Target: `{}` — `/api/series/<name>/{}?{}`",
        options.url, options.index, options.query
    )?;
    writeln!(
        r,
        "- Catalog: {} series ({} without `{}` skipped), fetched in {} ms",
        catalog.total,
        catalog.without_index,
        options.index,
        fmt_ms(Some(catalog.fetch_ms))
    )?;
    writeln!(
        r,
        "- Measured: {} ok, {} failed; repeat={}, concurrency={}, wall time {:.1} s",
        order.len(),
        failed.len(),
        options.repeat,
        options.concurrency,
        wall.as_secs_f64()
    )?;
    writeln!(
        r,
        "- Ranked by **{label}** server time (`X-Response-Time`; client time when the header is missing)"
    )?;
    writeln!(r)?;

    writeln!(r, "## Distribution (ms)")?;
    writeln!(r)?;
    writeln!(r, "| | p50 | p90 | p99 | p99.9 | max | sum |")?;
    writeln!(r, "|---|---:|---:|---:|---:|---:|---:|")?;
    let mut rows = vec![(label, &ranked)];
    if !options.sort_cold && options.repeat > 1 {
        rows.push(("cold", &cold));
    }
    for (name, values) in rows {
        writeln!(
            r,
            "| {name} | {} | {} | {} | {} | {} | {} |",
            fmt_ms(Some(percentile(values, 50.0))),
            fmt_ms(Some(percentile(values, 90.0))),
            fmt_ms(Some(percentile(values, 99.0))),
            fmt_ms(Some(percentile(values, 99.9))),
            fmt_ms(values.last().copied()),
            fmt_ms(Some(values.iter().sum())),
        )?;
    }
    writeln!(r)?;
    writeln!(r, "| {label} ms | series |")?;
    writeln!(r, "|---|---:|")?;
    let mut low = 0.0;
    for high in [1.0, 10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, f64::INFINITY] {
        let count = ranked.iter().filter(|&&v| v >= low && v < high).count();
        if high.is_finite() {
            writeln!(r, "| {low}–{high} | {count} |")?;
        } else {
            writeln!(r, "| ≥ {low} | {count} |")?;
        }
        low = high;
    }
    writeln!(r)?;

    writeln!(r, "## Slowest {} series", options.top.min(order.len()))?;
    writeln!(r)?;
    writeln!(
        r,
        "| # | series | warm ms | cold ms | client warm ms | size | kind | path |"
    )?;
    writeln!(r, "|---:|---|---:|---:|---:|---:|---|---|")?;
    for (rank, &i) in order.iter().take(options.top).enumerate() {
        let (s, o) = (&series[i], &outcomes[i]);
        writeln!(
            r,
            "| {} | `{}` | {} | {} | {} | {} | {} | {} |",
            rank + 1,
            s.name,
            fmt_ms(o.warm.as_ref().map(Sample::ms)),
            fmt_ms(o.cold.as_ref().map(Sample::ms)),
            fmt_ms(o.warm.as_ref().map(|w| w.client_ms)),
            fmt_bytes(o.cold.as_ref().map(|c| c.bytes)),
            s.kind,
            s.path(),
        )?;
    }
    writeln!(r)?;

    let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
    for &i in order {
        if let Some(sample) = outcomes[i].ranked(options) {
            groups
                .entry(series[i].group(options.group_depth))
                .or_default()
                .push(sample.ms());
        }
    }
    let mut groups: Vec<(String, Vec<f64>)> = groups.into_iter().collect();
    for (_, values) in &mut groups {
        values.sort_by(f64::total_cmp);
    }
    groups.sort_by(|a, b| {
        let total = |values: &[f64]| values.iter().sum::<f64>();
        total(&b.1).total_cmp(&total(&a.1))
    });
    writeln!(
        r,
        "## Slowest groups (last {} catalog path segments, by total {label} time)",
        options.group_depth
    )?;
    writeln!(r)?;
    writeln!(
        r,
        "A group collects the same metric across cohorts, so one slow vec type shows up once."
    )?;
    writeln!(r)?;
    writeln!(r, "| # | group | series | median ms | max ms | total ms |")?;
    writeln!(r, "|---:|---|---:|---:|---:|---:|")?;
    for (rank, (group, values)) in groups.iter().take(options.top).enumerate() {
        writeln!(
            r,
            "| {} | `{group}` | {} | {} | {} | {} |",
            rank + 1,
            values.len(),
            fmt_ms(Some(percentile(values, 50.0))),
            fmt_ms(values.last().copied()),
            fmt_ms(Some(values.iter().sum())),
        )?;
    }
    writeln!(r)?;

    if !failed.is_empty() {
        writeln!(r, "## Failed ({})", failed.len())?;
        writeln!(r)?;
        writeln!(r, "| series | status | error |")?;
        writeln!(r, "|---|---:|---|")?;
        for &i in failed.iter().take(200) {
            let o = &outcomes[i];
            let error = o.error.as_deref().unwrap_or_default();
            writeln!(
                r,
                "| `{}` | {} | {} |",
                series[i].name,
                o.status.map_or("–".to_string(), |s| s.to_string()),
                error.chars().take(160).collect::<String>(),
            )?;
        }
        if failed.len() > 200 {
            writeln!(r, "| … {} more in the TSV | | |", failed.len() - 200)?;
        }
        writeln!(r)?;
    }
    Ok(r)
}

fn tsv(
    series: &[Series],
    outcomes: &[Outcome],
    order: &[usize],
) -> std::result::Result<String, fmt::Error> {
    let number = |value: Option<f64>| value.map_or(String::new(), |v| format!("{v:.3}"));
    let mut t = String::new();
    writeln!(
        t,
        "rank\tname\tstatus\tserver_warm_ms\tserver_cold_ms\tclient_warm_ms\tclient_cold_ms\tbytes\tkind\tpath\terror"
    )?;
    // Ranked rows first, then failures without a rank.
    let ranked = order
        .iter()
        .enumerate()
        .map(|(rank, &i)| (Some(rank + 1), i));
    let failed = (0..outcomes.len())
        .filter(|&i| outcomes[i].error.is_some())
        .map(|i| (None, i));
    for (rank, i) in ranked.chain(failed) {
        let (s, o) = (&series[i], &outcomes[i]);
        let rank = rank.map_or(String::new(), |rank| rank.to_string());
        writeln!(
            t,
            "{rank}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            s.name,
            o.status.map_or(String::new(), |s| s.to_string()),
            number(o.warm.as_ref().and_then(|w| w.server_ms)),
            number(o.cold.as_ref().and_then(|c| c.server_ms)),
            number(o.warm.as_ref().map(|w| w.client_ms)),
            number(o.cold.as_ref().map(|c| c.client_ms)),
            o.cold
                .as_ref()
                .map_or(String::new(), |c| c.bytes.to_string()),
            s.kind,
            s.path(),
            o.error.as_deref().unwrap_or_default().replace('\t', " "),
        )?;
    }
    Ok(t)
}

fn run() -> Result<()> {
    let options = Options::parse()?;
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(options.timeout))
        .http_status_as_error(false)
        .user_agent(concat!(
            "bitview_client-series_latency/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .into();

    let catalog = load_catalog(&agent, &options)
        .map_err(|error| format!("cannot load the catalog from {}: {error}", options.url))?;
    if catalog.series.is_empty() {
        return Err("no series selected (check --index and --filter)".into());
    }
    eprintln!(
        "{} series at {} ({} request(s) each, concurrency {})",
        catalog.series.len(),
        options.index,
        options.repeat,
        options.concurrency
    );

    let (outcomes, wall) = measure(&agent, &options, &catalog.series);

    // Slowest first; failures are reported separately.
    let mut order: Vec<usize> = (0..outcomes.len())
        .filter(|&i| outcomes[i].error.is_none())
        .collect();
    order.sort_by(|&a, &b| {
        let ms = |i: usize| outcomes[i].ranked(&options).map_or(0.0, Sample::ms);
        ms(b).total_cmp(&ms(a))
    });

    let markdown = report(&options, &catalog, &outcomes, &order, wall)?;
    let table = tsv(&catalog.series, &outcomes, &order)?;

    let prefix = options.out.clone().unwrap_or_else(|| {
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        format!("tmp/series-latency/{}-{unix}", options.index)
    });
    if let Some(parent) = Path::new(&prefix)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(format!("{prefix}.md"), &markdown)?;
    fs::write(format!("{prefix}.tsv"), &table)?;

    print!("{markdown}");
    eprintln!("wrote {prefix}.md and {prefix}.tsv");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("series_latency: {error}");
            ExitCode::FAILURE
        }
    }
}
