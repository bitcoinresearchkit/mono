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

Each run is written to `<bitviewdir>/benches/<machine>/<revision>/<run-id>/`,
regardless of the installation method or working directory, the layout of the
workspace's `benches/`, so committing a run is copying its directory. Runs are
never replaced: repetitions of one revision on one machine sit side by side.

- `<machine>` is the hardware: the CPU, the RAM and the data directory's
  storage, `internal`, `external-tb` (Thunderbolt or USB4, macOS),
  `external-usb`, `external` (another or undetected bus) or `unknown`
  (`apple-m6-16gb-external-tb`);
- `<revision>` is the tag at HEAD (`v0.13.0`), else the short commit, else the
  crate version, with `-dirty` when the sources (`crates/`, the Cargo manifest,
  lockfile and configuration, the toolchain file) have local changes. It is
  captured at compile time, with the compiler, profile, target and flags, so a
  run records the code that was built, not the checkout it runs next to;
- `<run-id>` is the run's UTC start, `20261014T153012Z`.

One benchmark runs per data directory at a time (`benches/.lock`).

The results path is printed before bootstrap starts and again after completion.
For example, `--bitviewdir /Volumes/External/bitview` writes reports beneath
`/Volumes/External/bitview/benches/`:

```text
metadata.txt        # build (revision, dirty, profile, rustc, flags, target), host (name, CPU, RAM, data and blocks storage with used/total volume bytes), chain, paths, cache budget
memory.csv          # footprint, peak, resident size and system swap, every 5 s
io.csv              # cumulative bytes read and written and page-ins, every 5 s
cpu.csv             # cumulative user and system CPU time, every 5 s
disk.csv            # physical data-directory size, excluding reports, every minute
disk_breakdown.csv  # final size per component (`plugins/<id>` and top-level entries)
progress.csv        # heights from production log events, with their source (`indexer`, ...)
run.csv             # total bootstrap duration and status (complete, failed or aborted)
timings.csv         # every production plugin import and compute interval
```

On macOS the footprint and its peak are the kernel's (`physical_bytes`,
`peak_physical_bytes`). Linux has no footprint: `physical_bytes` is anonymous
plus shared resident memory, and its peak column is `peak_resident_bytes`
(`VmHWM`, the peak of the whole resident size).

Every row is flushed as it is written, so a crash or a killed process keeps
everything recorded up to that point.

Bitcoin Core synchronization and the initial and final recursive disk scans
are outside the timed interval. The per-minute scans run inside it, on their own
thread, so their small metadata reads are part of the recorded I/O.
