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
rebuild. Existing data is reused: such a run is marked resumed, its times aren't a
full sync's, and it skips the serial recompute below.

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

What a run is judged on (memory perf-work.md, "What judges what"): per-stage and per-plugin results by retired
instructions per process, which co-running work barely moves; whole-sync wall time only against runs that
held the mini's lock, from a fresh boot, on a clean machine (`m6-2-lock run`): brain's `bench-20261120/run.sh` appends
`locked=true` to the copied recording's `metadata.txt`, and the page and `bench-best.mjs` take a wall time into their
bests only from such a run. The binary measures whatever machine it runs on and judges nothing: `metadata.txt` records
the CPU, RAM, both volumes' used and total bytes (`data_volume`, `blocks_volume`, `internal_volume`) and the pool's
width (`rayon_threads` beside `parallelism`, the cores: narrower when `RAYON_NUM_THREADS` is set). A narrower pool is
a rejected configuration, not a measure of the machine, so the page and `bench-best.mjs` leave such a run out of every
best, instructions too (the earlier rule, carried over).

The results path is printed before bootstrap starts and again after completion.
For example, `--bitviewdir /Volumes/External/bitview` writes reports beneath
`/Volumes/External/bitview/benches/`:

```text
metadata.txt        # build (revision, dirty, profile, rustc, flags, target), host (name, CPU, RAM, data and blocks storage with used/total volume bytes), chain, paths, cache budget
memory.csv          # footprint, peak, resident size and system swap, every 5 s
io.csv              # cumulative bytes read and written and page-ins, every 5 s
cpu.csv             # cumulative user and system CPU time, and on macOS retired instructions and cycles, every 5 s
disk.csv            # physical data-directory size, excluding reports, every minute
disk_breakdown.csv  # final size per component (`plugins/<id>` and top-level entries)
progress.csv        # heights from production log events, with their source (`indexer`, ...)
run.csv             # the bootstrap's duration and status (complete, failed or aborted), then the serial recompute's
timings.csv         # every completed production plugin import and compute interval
serial_compute.csv  # each plugin's retired instructions, cycles and CPU time computing from zero over the run's indexed chain
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

### Serial recompute

The timed run's plugins overlap on threads, and a plugin waiting in a rayon join runs other
plugins' stolen work, so their durations say nothing about what each one spends (Market read
327 s and 91 s in two runs of one commit). So after a fresh run, every plugin's data but the
indexer's is removed and the bootstrap runs again on the indexed chain, with the composition's
forks run one branch at a time: each plugin computes all of it from zero alone, and the process's
counters (`proc_pid_rusage`: retired instructions and cycles; `getrusage`: user plus system CPU
time) between its start and end are its own, less the steps nested in it (the shared history walk
is its own `walk` step; Age, Profitability and URPD run inside it). A plugin's internal
parallelism is unchanged, and what the indexer leaves running in the background (ingestion and
compaction, freeing the blocks) is joined into its step; it only indexes the blocks that arrived
meanwhile. The counters are macOS's: on Linux the recompute fails, naming that (`perf_event_open`
waits until a Linux bench exists), and `cpu.csv` has no counter columns.

`serial_compute.csv` has `plugin,instructions,cycles,cpu_ms`, then `total` (the whole recompute,
the imports included) and `unattributed` (`total` less the plugins: the imports, the schedule,
logging and helpers between steps). Instructions are what a run is judged on, and why they and not cycles or CPU
time is in memory perf-work.md, "What judges what". The recompute took about 32 minutes on m6-2, with 101.8 trillion
instructions of which 12.8 billion were unattributed; `run.csv`'s `serial_compute` row holds its wall time and status
(`complete`, `failed` or `aborted`; absent when it never began). The compaction a plugin's compute
schedules for 5 s later (`bitview_plugin/src/update_context.rs`) runs at once, in the plugin's own
step, so it is counted as that plugin's (`bitview_default::serial::compact`).

A resumed run has no recompute: its times aren't a full sync's, so nothing compares them.
