# bitview_bencher

Resource monitoring for long-running Bitcoin indexing operations.

## What It Enables

Track disk usage, memory consumption (current + peak), and I/O throughput during indexing runs. Progress tracking hooks into brk_logger to record processing milestones automatically.

## Key Features

- **Multi-metric monitoring**: Disk, memory (RSS + peak), I/O read/write
- **Progress tracking**: Integrates with logging to capture block heights as they're processed
- **Run comparison**: Outputs timestamped CSVs for comparing multiple runs
- **macOS optimized**: Uses libproc for accurate process metrics on macOS
- **Non-blocking**: Monitors in background thread with 5-second sample interval

## Core API

```rust,ignore
let mut bencher = Bencher::from_cargo_env("bitview_plugin_indexer", &data_path)?;
bencher.start()?;

// ... run indexing ...

bencher.stop()?;
```

## Output Structure

```
benches/
└── bitview_plugin_indexer/
    └── 1703001234/
        ├── disk.csv      # timestamp_ms, bytes
        ├── memory.csv    # timestamp_ms, current, peak
        ├── io.csv        # timestamp_ms, read, written
        └── progress.csv  # timestamp_ms, height
```

## Built On

- `brk_error` for error handling
- `brk_logger` for progress hook integration
