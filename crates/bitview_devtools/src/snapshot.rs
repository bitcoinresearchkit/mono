use std::{
    collections::BTreeMap,
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::ExitCode,
};

use brk_error::Result;

/// One rendered snapshot file, relative to `snapshots/`.
pub struct Snapshot {
    pub(crate) file: &'static str,
    pub(crate) contents: String,
}

/// Records the snapshots as the local baseline, or with `--check` compares against it.
pub fn run(tool: &str, render: impl FnOnce() -> Result<Vec<Snapshot>>) -> ExitCode {
    let check = match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [arg] if arg == "--check" => true,
        _ => {
            eprintln!("usage: cargo {tool} [-- --check]");
            return ExitCode::FAILURE;
        }
    };
    match render().and_then(|snapshots| apply(tool, check, &snapshots)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn apply(tool: &str, check: bool, snapshots: &[Snapshot]) -> Result<bool> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("snapshots");
    fs::create_dir_all(&dir)?;
    let mut current = true;
    let mut recorded = 0;
    for snapshot in snapshots {
        let path = dir.join(snapshot.file);
        let new = sibling(&path, "new");
        let baseline = match fs::read_to_string(&path) {
            Ok(baseline) => Some(baseline),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        if baseline.as_deref() == Some(snapshot.contents.as_str()) {
            remove_if_exists(&new)?;
            continue;
        }
        if check {
            current = false;
            fs::write(&new, &snapshot.contents)?;
            match &baseline {
                Some(baseline) => {
                    report_diff(&path, baseline, &snapshot.contents);
                    eprintln!("  full diff: diff -u {} {}", path.display(), new.display());
                }
                None => eprintln!("{}: no baseline yet, run `cargo {tool}`", path.display()),
            }
        } else {
            recorded += 1;
            if let Some(baseline) = baseline {
                fs::write(sibling(&path, "prev"), baseline)?;
            }
            fs::write(&path, &snapshot.contents)?;
            remove_if_exists(&new)?;
            eprintln!("recorded {}", path.display());
        }
    }
    if !check {
        eprintln!(
            "{tool}: recorded {recorded}, unchanged {}",
            snapshots.len() - recorded
        );
    } else if current {
        eprintln!("{tool} snapshots match the baseline");
    } else {
        eprintln!("{tool} snapshots differ from the baseline: review, then run `cargo {tool}`");
    }
    Ok(current)
}

fn sibling(path: &Path, extension: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".");
    name.push(extension);
    PathBuf::from(name)
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

/// Lines only on one side, with multiplicity; a pure reorder is called out as such.
fn report_diff(path: &Path, baseline: &str, current: &str) {
    let mut counts = BTreeMap::<&str, isize>::new();
    for line in baseline.lines() {
        *counts.entry(line).or_default() -= 1;
    }
    for line in current.lines() {
        *counts.entry(line).or_default() += 1;
    }
    let removed = counts
        .iter()
        .filter(|(_, count)| **count < 0)
        .collect::<Vec<_>>();
    let added = counts
        .iter()
        .filter(|(_, count)| **count > 0)
        .collect::<Vec<_>>();
    eprintln!(
        "{}: -{} +{} distinct lines",
        path.display(),
        removed.len(),
        added.len()
    );
    if removed.is_empty() && added.is_empty() {
        match baseline
            .lines()
            .zip(current.lines())
            .position(|(a, b)| a != b)
        {
            Some(index) => eprintln!(
                "  same lines, different order (first difference at line {})",
                index + 1
            ),
            None => eprintln!("  same lines; only line endings or the trailing newline differ"),
        }
    }
    for (line, count) in removed.iter().take(20) {
        eprintln!("  - {line}{}", times(**count));
    }
    for (line, count) in added.iter().take(20) {
        eprintln!("  + {line}{}", times(**count));
    }
}

fn times(count: isize) -> String {
    match count.unsigned_abs() {
        1 => String::new(),
        n => format!("  (×{n})"),
    }
}
