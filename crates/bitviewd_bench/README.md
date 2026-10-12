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

Each run is written to `<bitviewdir>/benches/<machine>/<revision>/`, regardless
of the installation method or working directory, the layout of the workspace's
`benches/`, so committing a run is copying its directory. A rerun of the same
revision on the same machine replaces the previous one.

- `<machine>` is the hardware: the CPU, the RAM and the data directory's
  storage, `internal`, `external-tb` (Thunderbolt or USB4, macOS),
  `external-usb`, `external` (another or undetected bus) or `unknown`
  (`apple-m6-16gb-external-tb`);
- `<revision>` is the tag at HEAD (`v0.13.0`), else the short commit, else the
  crate version, with `-dirty` when the code has local changes (`benches/`
  aside).

One benchmark runs per data directory at a time (`benches/.lock`).

The results path is printed before bootstrap starts and again after completion.
For example, `--bitviewdir /Volumes/External/bitview` writes reports beneath
`/Volumes/External/bitview/benches/`:

```text
metadata.txt        # build, host (name, CPU, RAM, data and blocks storage), chain, revision, paths, cache budget
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
