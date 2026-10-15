use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    fmt::{Result as FmtResult, Write},
};

use crate::{Result, analysis::Analysis, group::Group};

fn cell(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace('`', "&#96;")
        .replace(['\n', '\r'], " ")
}

/// Histogram buckets: inclusive upper bound in nanoseconds, and label. Slower requests are `>5s`.
const BUCKETS: [(u64, &str); 8] = [
    (1_000_000, "≤1ms"),
    (5_000_000, "(1, 5]ms"),
    (10_000_000, "(5, 10]ms"),
    (50_000_000, "(10, 50]ms"),
    (100_000_000, "(50, 100]ms"),
    (500_000_000, "(100, 500]ms"),
    (1_000_000_000, "(500, 1000]ms"),
    (5_000_000_000, "(1, 5]s"),
];

pub fn render(analysis: &Analysis) -> Result<String> {
    let Analysis {
        ref files,
        ref groups,
        lines,
        ignored,
        malformed,
        unmatched,
        count,
        errors,
        first,
        last,
    } = *analysis;
    // Error rates include every status for an endpoint.
    let mut endpoint_counts: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for ((endpoint, status), group) in groups {
        let (requests, errors) = endpoint_counts.entry(endpoint).or_default();
        *requests += group.durations.len();
        if *status >= 400 {
            *errors += group.durations.len();
        }
    }
    let mut out = String::from("# Bitview HTTP latency report\n\n");
    writeln!(
        out,
        "Read {} files, {lines} lines: **{count} captured requests**, {ignored} non-access lines, {malformed} malformed access lines.\n",
        files.len()
    )?;
    if let (Some(first), Some(last)) = (first, last) {
        writeln!(
            out,
            "Captured range: **{first} to {last}**, in the logger's local time.\n"
        )?;
    }
    out.push_str("This measures server response preparation, excluding complete body transfer and network latency. File logging is capped at 100 events/second/level; dropped or filtered events cannot be recovered. HTTP methods are absent from this log format. Historical 304/400 requests may be missing at info level. Percentiles describe captured traffic, not a controlled benchmark.\n\n");
    writeln!(
        out,
        "{unmatched} requests did not match the compiled API catalog; their paths are kept verbatim (query strings removed). Catalog changes can affect historical grouping.\n"
    )?;
    if count == 0 {
        out.push_str("**No access requests found in the logs.** Check the log directory and server logging level.\n\n");
    } else {
        writeln!(
            out,
            "Errors (4xx + 5xx): **{errors} / {count} ({:.2}%)**.\n",
            100. * errors as f64 / count as f64
        )?;
        out.push_str("Percentiles use exact nearest-rank samples. Groups with fewer than 100 samples are marked †; their P99 is especially sensitive to individual requests. P99.9 is omitted below 1,000 samples and marked ‡ (low sample confidence) below 10,000. Endpoint error % is (4xx + 5xx) / all captured requests for that endpoint, including 304s; it repeats across status rows. Total time sums overlapping request durations; it is not CPU time. Each HTTP status code has one table of up to 20 endpoints sorted by P95, plus a histogram and slow examples. Paths marked [unmatched] are absent from the API catalog.\n\n");
        if unmatched > 0 {
            let mut unknown: BTreeMap<&str, usize> = BTreeMap::new();
            for ((endpoint, _), group) in groups {
                if !group.matched {
                    *unknown.entry(endpoint).or_default() += group.durations.len();
                }
            }
            let mut unknown: Vec<_> = unknown.into_iter().collect();
            unknown.sort_by_key(|(path, count)| (Reverse(*count), *path));
            out.push_str("## Most frequent unmatched paths\n\nCounts combine all statuses. These may be website paths, removed endpoints or invalid requests.\n\n| Path | Requests |\n|---|---:|\n");
            for (path, count) in unknown.into_iter().take(20) {
                writeln!(out, "| {} | {count} |", cell(path))?;
            }
            out.push('\n');
        }
        let statuses: BTreeSet<_> = groups.keys().map(|(_, status)| *status).collect();
        for status in statuses {
            let mut status_groups: Vec<_> = groups
                .iter()
                .filter(|((_, code), _)| *code == status)
                .collect();
            let status_count: usize = status_groups.iter().map(|(_, g)| g.durations.len()).sum();
            writeln!(
                out,
                "## HTTP {status}\n\n{status_count} captured requests ({:.2}% of all captured requests).\n",
                100. * status_count as f64 / count as f64
            )?;
            status_groups.sort_by_key(|(key, g)| (Reverse(g.percentile(95)), *key));
            table(&mut out, &status_groups, &endpoint_counts)?;
            let mut histogram = [0usize; BUCKETS.len() + 1];
            for (_, group) in &status_groups {
                for nanos in &group.durations {
                    let bucket = BUCKETS.iter().position(|(bound, _)| nanos <= bound);
                    histogram[bucket.unwrap_or(BUCKETS.len())] += 1;
                }
            }
            out.push_str(
                "### Duration histogram\n\n| Duration | Requests | Share |\n|---|---:|---:|\n",
            );
            let labels = BUCKETS.iter().map(|(_, label)| *label).chain([">5s"]);
            for (label, n) in labels.zip(histogram) {
                writeln!(
                    out,
                    "| {label} | {n} | {:.2}% |",
                    100. * n as f64 / status_count as f64
                )?;
            }
            out.push_str("\n### Slow request examples\n\nUp to three per endpoint/status group, ranked by duration within this status.\n\n| Timestamp | Status | Duration ms | Request URI |\n|---|---:|---:|---|\n");
            let mut examples: Vec<_> = status_groups.iter().flat_map(|(_, g)| &g.slowest).collect();
            examples.sort_by_key(|r| Reverse(r.nanos));
            for record in examples.into_iter().take(20) {
                writeln!(
                    out,
                    "| {} | {} | {:.3} | {} |",
                    record.timestamp,
                    record.status,
                    record.nanos as f64 / 1e6,
                    cell(&record.uri)
                )?;
            }
        }
    }
    out.push_str("\n## Input files\n\n");
    for path in files {
        writeln!(out, "- {}", cell(&path.display().to_string()))?;
    }
    Ok(out)
}

type Row<'a> = (&'a (String, u16), &'a Group);

/// The 20 slowest endpoints of one status, by P95.
fn table(
    out: &mut String,
    rows: &[Row<'_>],
    endpoint_counts: &BTreeMap<&str, (usize, usize)>,
) -> FmtResult {
    writeln!(
        out,
        "### Endpoints by P95\n\n| Endpoint | Requests | P50 ms | P95 ms | P99 ms | P99.9 ms | Avg ms | Max ms | Total s | Endpoint error % |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )?;
    for ((endpoint, _), g) in rows.iter().take(20) {
        let n = g.durations.len();
        let (requests, errors) = endpoint_counts[endpoint.as_str()];
        writeln!(
            out,
            "| {} | {n}{} | {:.3} | {:.3} | {:.3} | {} | {:.3} | {:.3} | {:.3} | {:.2}% |",
            if g.matched {
                cell(endpoint)
            } else {
                format!("{} [unmatched]", cell(endpoint))
            },
            if n < 100 { "†" } else { "" },
            g.percentile(50) as f64 / 1e6,
            g.percentile(95) as f64 / 1e6,
            g.percentile(99) as f64 / 1e6,
            g.p999().map_or_else(
                || "—".into(),
                |value| format!(
                    "{:.3}{}",
                    value as f64 / 1e6,
                    if n < 10_000 { "‡" } else { "" }
                )
            ),
            g.total as f64 / n as f64 / 1e6,
            *g.durations.last().unwrap() as f64 / 1e6,
            g.total as f64 / 1e9,
            100. * errors as f64 / requests as f64
        )?;
    }
    out.push('\n');
    Ok(())
}
