# bitviewd_bench

One-shot instrumentation of the real default `bitviewd` bootstrap pipeline.

The benchmark records the complete bootstrap and every plugin import and compute
that completes within it. Plugin timings come from the production schedule, so
parallel work and drop/reimport cycles are preserved rather than reproduced by a
benchmark-specific pipeline.

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
- `<revision>` is the short commit (`bd1ed8635`): the checkout's HEAD, or the
  commit a published package records, never a tag or a version; `-dirty` when
  the sources (`crates/`, the Cargo manifest, lockfile and configuration, the
  toolchain file) have local changes. A build with neither fails. It is
  captured at compile time, with the compiler, profile, target and flags, so a
  run records the code that was built, not the checkout it runs next to;
- `<run-id>` is the run's UTC start, `20261014T153012Z`.

The binary holds `/tmp/bitviewd-bench.lock` for its lifetime, across data directories.
It refuses to start beside `bitviewd`, `bitview_mcp`, another `bitviewd_bench` or a running `rustc`.
On a mini, stop those workloads with `~/Developer/dots/shared/ship stop bitviewd mcp`
and stop any other benchmark first. Bitcoin Core must remain running for the benchmark.
The lock is released by the OS on exit, including a crash or kill; leave its file in place.
A recording made by this binary passed that start check, which does not guarantee the box
stays quiet later. A recording known to have overlapped another workload says `comparable=false`
and a `comparable_reason` in its `metadata.txt`; an absent `comparable` means comparable.
The existing `benches/.lock` also guards each data directory.

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
timings.csv         # every completed production plugin import and compute interval
serial_cpu.csv      # per plugin, serial-replay CPU per block of the last 10 blocks: p50 and p95
serial_cpu_samples.csv # the serial-replay CPU of each plugin in each block
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

### Serial replay CPU

The timed run's plugins overlap on threads, so their durations say nothing about the CPU each
one spends. After the run, on a fresh data directory (a resumed one has no replay), the last 10
blocks (`UNDO_TAIL`, the undo the run already keeps) are rolled back and applied again one at a
time, with Bitcoin Core on its full chain. The first is unmeasured: it restores the plugins' live
state. Each of the others is one production update with the composition's forks run one branch at
a time, so the process CPU (`getrusage`, user plus system) between a plugin's start and end is its
own, less the steps nested in it (the shared history walk is its own `walk` step; Age,
Profitability and URPD run inside it). A plugin's internal parallelism is unchanged, and what an
update leaves running in the background (the indexer's ingestion and compaction, freeing the
blocks) is joined into the indexer's step.

`serial_cpu_samples.csv` has `height,plugin,serial_cpu_ms`, one row per plugin per block, and
`total` (the whole update) and `unattributed` (`total` less the plugins: the schedule, logging and
helper threads between steps). `serial_cpu.csv` has the nearest-rank p50 and p95 of each over the
9 measured blocks, so p95 is the largest.

These are serial-replay CPU per block, not production CPU. The vectors' delayed compaction, 5 s
after an update (`bitview_plugin/src/update_context.rs`), is not counted. Compare plugins with each
other and a plugin with its own earlier run, never with the timed run's durations, and gate nothing
on one plugin's number: nine blocks of one machine in one state.
