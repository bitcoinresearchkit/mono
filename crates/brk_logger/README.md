# brk_logger

Colorized, timestamped logging with optional file output and hooks.

## What It Enables

Drop-in logging initialization that silences noisy dependencies (bitcoin, fjall, rolldown, ...) while keeping your logs readable with color-coded levels and local timestamps.

## Key Features

- **Dual output**: Console (colorized) + optional file logging with daily rotation
- **Per-level files**: One combined log plus one file per tracing level (error/warn/info/debug/trace)
- **Per-level rate limit**: 100 writes/sec per level so a chatty level can't starve the others; combined file mirrors what the per-level files accept
- **Auto-cleanup**: `*.txt` files older than 7 days are pruned on startup
- **Log hooks**: Register callbacks to intercept log messages programmatically
- **Sensible defaults**: Pre-configured filters silence common verbose libraries,
  including RMCP transport lifecycle messages
- **Timestamp formatting**: Uses system timezone via jiff

## Environment Variables

- `LOG` - Set log level (default: `info` in release, `debug` in dev). Example: `LOG=debug your-binary`
- `RUST_LOG` - Full control over filtering (overrides all defaults)

## Core API

```rust,ignore
brk_logger::init(Some(Path::new("logs")))?;  // Console + files in logs/
brk_logger::init(None)?;                      // Console only

brk_logger::register_hook(|msg| {
    // React to log messages
})?;
```

## Usage

```rust,ignore
use tracing::info;

fn main() -> std::io::Result<()> {
    brk_logger::init(None)?;
    info!("Ready");
    Ok(())
}
```
