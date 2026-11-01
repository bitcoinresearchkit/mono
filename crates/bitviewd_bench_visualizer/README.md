# bitviewd_bench_visualizer

SVG charts for `bitviewd_bench` runs. The page to browse them all is
[bitview.space/benchmarks](https://bitview.space/benchmarks/) (`showcases/benchmarks/`), which
reads the runs committed to `benches/` from GitHub in the browser, by the same rules as here.

## Usage

To chart the runs in a Bitview data directory, pass the path used for
`bitviewd_bench --bitviewdir`:

```sh
cargo run --release -p bitviewd_bench_visualizer -- /Volumes/External/bitview
```

With no argument, it charts the workspace's `benches/` collection. The
equivalent API call is `bitviewd_bench_visualizer::generate(bitviewdir)?`.

Each directory of `benches/` is a machine (`apple-m6-16gb-external-tb`), and every directory
below it that holds a `memory.csv` is one of its runs (`<revision>/<run-id>/`,
`<revision>/` for older ones), labelled by that path. Files are read by column
name, so runs recorded by older recorders chart what they have. A run whose files
cannot be read, or without memory samples, is reported and skipped.

## Charts

- `<run>/dashboard.svg`: one page per run, every panel on the same time axis with
  the bootstrap cycles (indexing, compute, catch-up) shaded. It shows:
  - a summary header (host, revision, status, duration, peak memory, I/O totals);
  - a plugin timeline, one row per plugin (imports grey, computes coloured);
  - the indexed height and memory (footprint, peak, resident, swap);
  - CPU cores busy, disk read and write rates, and page-ins;
  - the data-directory size, time per plugin, and the final size per component.
- `<machine>/compare.svg`: the machine's complete runs side by side (runs of
  different hardware do not compare).
  - A summary table lists every run.
  - Height, memory, CPU and I/O charts show the six most recent.
  - A table gives compute time per plugin and run.

In the SVGs, rates come from cumulative counters over exactly the trailing 60 seconds, so
they start after the first minute; totals are the counters' growth over the run. The page
keeps 400 points a curve, so its rates are over the trailing minute as closely as those
points allow, from its second point on. A change to how runs are read here (`src/data.rs`)
belongs in the page's reading too.
