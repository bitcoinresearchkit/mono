# bitviewd_bench_visualizer

SVG charts for `bitviewd_bench` runs.

## Usage

To chart the runs in a Bitview data directory, pass the path used for
`bitviewd_bench --bitviewdir`:

```sh
cargo run --release -p bitviewd_bench_visualizer -- /Volumes/External/bitview
```

With no argument, it charts the workspace's `benches/bitviewd/` collection. The
equivalent API call is `Visualizer::new(bitviewdir).generate()?`.

Every directory below `benches/bitviewd/` that holds a `memory.csv` is a run,
at any depth (`<machine-branch-commit>/run-<ms>/` included). Files are read by
column name, so runs recorded by older recorders chart what they have.

## Charts

- `<run>/dashboard.svg`: one page per run, every panel on the same time axis with
  the bootstrap cycles (indexing, compute, catch-up) shaded. It shows:
  - a summary header (host, revision, status, duration, peak memory, I/O totals);
  - a plugin timeline, one row per plugin (imports grey, computes coloured);
  - the indexed height and memory (footprint, peak, resident, swap);
  - CPU cores busy, disk read and write rates, and page-ins;
  - the data-directory size, time per plugin, and the final size per component.
- `compare.svg`: complete runs side by side.
  - A summary table lists every run.
  - Height, memory, CPU and I/O charts show the six most recent.
  - A table gives compute time per plugin and run.

Rates come from cumulative counters over exactly the trailing 60 seconds, so they start
after the first minute; totals are the counters' growth over the run.
