# Server test flakiness investigation — 2026-09-18

The confirmed intermittent failure is `chart_reads_survive_concurrent_cache_eviction_and_reorgs`. It depends on fixture concurrency and available resources: full pipeline updates can outlast the five-second HTTP deadline. Cache eviction is not required to reproduce it. The earlier explanation placed too much emphasis on cache clearing.

## Controlled local runs

A temporary test binary recorded publication-call boundaries, chart request times, and cache-clear duration. Runs were sequential, each in a fresh process. Parallel runs explicitly used 12 test threads and bypassed the recently added fixture mutex; production request deadlines and assertions were unchanged. The no-eviction run disabled only the eviction loop's `clear()` call. Instrumentation was removed from the source files before running the experiments.

| Configuration | Result | Longest update in the affected fixture | Slowest recorded chart request | Peak resident memory |
| --- | --- | --- | --- | --- |
| Affected test alone, eviction enabled, three runs | 3/3 passed | 0.508 / 0.737 / 0.749 s | 0.965 / 1.048 / 1.322 s | 1.27–1.31 GiB |
| Full suite, 12 threads, eviction disabled | 91 passed, affected test failed | 13.389 s | 5.012 s; HTTP 504 | 13.69 GiB |
| Full suite, 12 threads, eviction enabled | 91 passed, affected test failed | 17.393 s | 5.282 s; HTTP 504 | 15.16 GiB |
| Prior full-suite run with fixture isolation | 92 passed | Not instrumented | Not instrumented | Not measured |

Update maxima include both fixture preparation and the four concurrent branch replacements. The no-eviction loop recorded zero clears, confirming that control. Each isolated run completed all 67 chart requests. The full suites each had four HTTP reader failures; one diagnostic line in the eviction-enabled run interleaved with panic output, so the automatic timing parser recovered only three of those four failure records.

The eviction-enabled full run spent 25.786 seconds cumulatively inside 13,243 cache-clear calls, with a maximum individual clear of 20 ms. This is additional overlapping work, not an additive estimate of suite slowdown. These single full-suite samples do not precisely quantify eviction's performance cost.

## A concrete failure timeline

In the run with eviction disabled, relative to the diagnostic clock:

- 85.828 s: the concurrent branch update starts.
- 85.833–85.847 s: four chart requests start.
- 90.844–90.849 s: all four return HTTP 504 after approximately five seconds.
- 91.745 s: the update completes, after 5.917 seconds.

Every failed request's entire lifetime falls inside that update call. The preceding successful chart requests took 1–13 ms. Later updates completed normally. No mixed-publication assertion failed in these runs.

## Why this happens

1. [The test requires HTTP 200 for every chart read](/Users/k/Developer/mono/crates/bitview_server/tests/unit/cache_reorg.rs:55), while four readers overlap four real branch replacements and a cache eviction loop.
2. [An update closes the publication barrier across compute and commit](/Users/k/Developer/mono/crates/bitview_runtime/src/update.rs:32). The selected derived series uses [publication protection](/Users/k/Developer/mono/crates/bitview_query/src/impl/series/read.rs:43), so reads can wait for that work to finish.
3. [HTTP requests have a five-second deadline](/Users/k/Developer/mono/crates/bitview_server/src/lib.rs:87). When resource contention stretches an update beyond that budget, returning 504 is expected deadline behavior; the test's unconditional 200 assertion fails.
4. These fixtures import the full plugin pipeline. [Query construction deliberately retains its catalog and read-only plugin composition for the process lifetime](/Users/k/Developer/mono/crates/bitview_query/src/lib.rs:179), matching the production daemon lifecycle. Repeated fixture construction in a single test process retains those objects, and the measured parallel process footprint is substantial.
5. [Cache clearing scans all registered live caches](/Users/k/Developer/mono/crates/vecdb/src/cache/budget.rs:55), including caches outside the active fixture. This is real cross-test interference, even though disabling it did not prevent the timeout. The separate cached-source assertion can also race with another test's global clear between `collect()` and `read_cached_into_at()`; that race was identified by inspection, not observed in these runs.

The measurements establish concurrency-dependent update latency and deadline failures. They do not isolate how much of the slowdown comes from CPU scheduling, memory pressure, filesystem work, or individual lock waits. The diagnostic boundaries cover the update call, not a profiler trace of every lock. These runs found no evidence of inconsistent returned data, but cannot prove the absence of all data races.

## What the existing isolation does

The fixture mutex removes competing full-pipeline fixtures while preserving concurrency within each scenario. The earlier full suite passed all 92 tests with it. It addresses test-to-test contention; an overloaded host could still exhaust a wall-clock deadline. It does not change production deadline or publication behavior.

Longer-term improvements should target fixture cost and deterministic concurrency: use the smallest real plugin composition needed by each scenario, coordinate required overlap with explicit events, and keep heavy full-pipeline tests resource-isolated. Cache-residency assertions require exclusive control over global eviction. Deadline tests should continue to validate timeout behavior explicitly.

Other tests contain short real-time budgets and sleeps, making them candidates for review under extreme load; none failed in these comparison runs. Earlier TypeScript errors were deterministic. Sandbox socket permission failures and the live Python test's HTTP 500 are separate failure classes, not demonstrated timing flakes.

## Evidence

- [Isolated run 1](/tmp/mono-flake-isolated-1.log), [run 2](/tmp/mono-flake-isolated-2.log), [run 3](/tmp/mono-flake-isolated-3.log)
- [Parallel without eviction](/tmp/mono-flake-parallel-no-eviction.log)
- [Parallel with eviction](/tmp/mono-flake-parallel-eviction.log)
- [Machine-readable measurements](/tmp/mono-flake-summary.json)
- [Prior isolated-fixture full suite](/tmp/mono-fix-server-after.log)

Persistent changes from this investigation are this report, a correction to the cleanup report, and a more accurate fixture-isolation comment. Diagnostic controls and timing prints are not retained in source.
