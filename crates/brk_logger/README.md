# brk_logger

Tracing setup: colorized console output with local timestamps, plus optional daily log files.

With a directory, `init` also writes `YYYY-MM-DD.txt` (combined) and `YYYY-MM-DD_<level>.txt`
files into it. Each level admits at most 100 writes per second, so a chatty level cannot starve
the others; the combined file gets exactly the admitted writes. On startup, log files modified more
than seven days ago are removed.

Default filters turn off bitcoin, corepc, tracing, aide, fjall, lsm_tree and tower_http, and limit
rmcp to warnings. `register_hook` installs the process's single hook, which receives every event
the filter admits.

## Environment variables

- `LOG` - log level (default: `info` in release, `debug` in dev). Example: `LOG=debug bitviewd`
- `RUST_LOG` - comma-separated `target=level` directives with an optional default level; replaces
  `LOG` and every default filter

## Usage

```rust,ignore
use std::path::Path;

use tracing::info;

fn main() -> std::io::Result<()> {
    brk_logger::init(Some(Path::new("logs")))?; // `None`: console only
    info!("Ready");
    Ok(())
}
```
