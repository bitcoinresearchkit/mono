# bitviewd_bench_visualizer

SVG charts for `bitviewd_bench` runs, and `benches/index.html`, one self-contained page to
browse them all (published at [perf.bitview.dev](https://perf.bitview.dev)).

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

- `index.html`: every machine's runs in one file, their data inside it, no other file or
  network access needed: the latest complete run, each complete run's time and peak memory,
  a table of every run, and a run's curves (height, memory, CPU, disk, data directory) on one
  time axis beside another run's, with time per plugin and the data directory by component.
  Curves keep 400 points a run. To publish it, deploy the one file (`bunx wrangler pages
  deploy <dir holding it> --project-name bitview-perf`).

In the SVGs, rates come from cumulative counters over exactly the trailing 60 seconds, so
they start after the first minute; totals are the counters' growth over the run. The page
keeps 400 points a curve, so its rates are over the trailing minute as closely as those
points allow, from its second point on.
