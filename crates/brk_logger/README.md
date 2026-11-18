# brk_logger

Tracing setup: colorized console output with local timestamps, plus optional daily log files.

With a directory, `init` also writes each day's lines to `YYYY-MM-DD.txt` in it. Each level gets at
most 100 lines a second there, so a chatty level can neither fill the disk nor crowd out the
others. Log files modified more than seven days ago are removed at start, then once a day by a
thread of their own, so however long a process runs, busy or quiet, it keeps about a week of them.

Default filters turn off bitcoin, corepc, tracing, aide, fjall, lsm_tree and tower_http, and limit
rmcp to warnings. `register_hook` installs the process's single hook, which receives every event
the filter admits.

The last 1,000 lines stay in memory: `recent` returns them, or those after a given line. Each line
has a `seq` counting the process's lines from 0, so a reader asking again gets each line once, and
a jump in `seq` shows lines that left memory before it asked.

`RequestLine::parse` reads a request's line back from a log file: the layout is written and read here.

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
