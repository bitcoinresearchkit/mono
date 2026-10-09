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
rebuild. Existing indexed data is reused.

Each run is written below `<bitviewdir>/benches/bitviewd/run-<unix timestamp>/`,
regardless of the installation method or working directory. The results path is
printed before bootstrap starts and again after completion. For example,
`--bitviewdir /Volumes/External/bitview` writes reports beneath
`/Volumes/External/bitview/benches/bitviewd/`:

```text
metadata.txt        # build, host (name, CPU, RAM), chain, revision, path, and cache budget
memory.csv          # footprint, peak footprint, resident size and system swap, every 5 s
io.csv              # cumulative bytes read and written and page-ins, every 5 s
cpu.csv             # cumulative user and system CPU time, every 5 s
disk.csv            # physical data-directory size, excluding reports, every minute
disk_breakdown.csv  # final size per component (`plugins/<id>` and top-level entries)
progress.csv        # heights from production log events, with their source (`indexer`, ...)
run.csv             # total bootstrap duration and status (complete, failed or aborted)
timings.csv         # every production plugin import and compute interval
```

Every row is flushed as it is written, so a crash or a killed process keeps
everything recorded up to that point.

Bitcoin Core synchronization and the initial and final recursive disk scans
are outside the timed interval. The per-minute scans run inside it, on their own
thread, so their small metadata reads are part of the recorded I/O.
