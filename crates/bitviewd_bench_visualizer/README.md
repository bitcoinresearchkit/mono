# bitviewd_bench_visualizer

SVG chart generation for benchmark visualization.

## What It Enables

Turn benchmark CSV data into publication-ready SVG charts showing disk usage, memory (current/peak), progress, and I/O over time. Compare multiple runs side-by-side with automatic color coding.

## Key Features

- **Multi-run comparison**: Overlay multiple benchmark runs with distinct colors
- **Dual-axis charts**: Memory charts show both current and peak usage (solid vs dashed lines)
- **Smart scaling**: Automatic unit conversion for bytes (KB/MB/GB) and time (seconds/minutes/hours)
- **Per-run trimming**: Aligns data by progress cutoffs for fair comparison
- **Dark theme**: Clean, readable charts with monospace fonts

## Core API

```rust,ignore
let viz = Visualizer::from_cargo_env()?;
viz.generate()?;
```

Or run `cargo run --release -p bitviewd_bench_visualizer`.

## Chart Types

- `disk.svg` - Storage consumption over time
- `memory.svg` - Current + peak memory usage
- `progress.svg` - Processing progress (e.g., blocks indexed)
- `io_read.svg` / `io_write.svg` - I/O throughput

## Input Format

Reads CSV files from `benches/bitviewd/<run_id>/`:
- `disk.csv`, `memory.csv`, `progress.csv`, `io.csv`
