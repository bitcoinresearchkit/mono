# bitviewd_bench

One-shot instrumentation of the real default `bitviewd` bootstrap pipeline.

The benchmark always records the complete bootstrap and every plugin import and
compute executed within it. Plugin timings come from the production schedule,
so parallel work and drop/reimport cycles are preserved rather than reproduced
by a benchmark-specific pipeline.

Run it with:

```sh
cargo run --release -p bitviewd_bench
```

It reads `~/.bitview/config.toml` and accepts the same command-line options as
`bitviewd`. Pass `--bitviewdir <EMPTY_PATH>` to measure a complete historical
rebuild. The benchmark never deletes existing data.

Each run is written below `benches/bitviewd/run-<unix timestamp>/`:

```text
disk.csv       # physical data-directory size before and after bootstrap
metadata.txt   # build, host, chain, revision, and path context
memory.csv     # current and peak physical memory sampled every five seconds
io.csv         # process disk I/O sampled from the same OS call
progress.csv   # indexed heights observed through production log events
run.csv        # total bootstrap duration and completion status
timings.csv    # every production plugin import and compute interval
```

Bitcoin Core synchronization and both recursive disk scans are deliberately
outside the timed and process-sampled interval.
