//! The build's identity, embedded at compile time: a run records the code that was compiled, not
//! the checkout the binary happens to run next to.

use std::{env, fs, path::Path, process::Command};

/// The workspace inputs of the binary: a change to one reruns this script, and only they make a
/// build `dirty` (reports copied into `benches/` are not code).
const SOURCES: [&str; 5] = [
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    "rust-toolchain.toml",
];

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace = Path::new(&manifest).ancestors().nth(2).unwrap();
    let git = |args: &[&str]| output(Command::new("git").arg("-C").arg(workspace).args(args));

    for source in SOURCES {
        rerun_if_changed(&workspace.join(source));
    }
    // Commits, checkouts and staging.
    let git_dir = git(&["rev-parse", "--absolute-git-dir"]);
    if !git_dir.is_empty() {
        for state in ["HEAD", "index", "logs/HEAD"] {
            rerun_if_changed(&Path::new(&git_dir).join(state));
        }
    }

    // The commit: the one a published package records (crates.io builds have no git), else the
    // checkout's HEAD. A run is filed under its commit alone, so none is guessed.
    let (commit, dirty) = published_commit(Path::new(&manifest))
        .or_else(|| {
            let commit = git(&["rev-parse", "HEAD"]);
            let dirty =
                || !git(&[&["status", "--porcelain", "--"][..], &SOURCES].concat()).is_empty();
            (!commit.is_empty()).then(|| (commit, dirty()))
        })
        .unwrap_or_else(|| {
            panic!(
                "bitviewd_bench: no commit to file runs under: build it from a git checkout or a \
                 published package"
            )
        });
    // One directory name: the short commit, `-dirty` with local changes.
    let mut revision = commit[..9].to_owned();
    if dirty {
        revision.push_str("-dirty");
    }

    // `<target>/[<triple>/]<profile>/build/<package>/out`: custom profiles keep their name, `dev`
    // builds into `debug`.
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .and_then(Path::file_name)
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());

    for (key, value) in [
        ("REVISION", revision),
        ("COMMIT", commit),
        ("DIRTY", dirty.to_string()),
        ("PROFILE", profile),
        (
            "RUSTC",
            output(Command::new(env::var_os("RUSTC").unwrap()).arg("-V")),
        ),
        (
            "RUSTFLAGS",
            env::var("CARGO_ENCODED_RUSTFLAGS")
                .unwrap_or_default()
                .replace('\x1f', " "),
        ),
        ("TARGET", env::var("TARGET").unwrap()),
    ] {
        println!("cargo::rustc-env=BENCH_{key}={value}");
    }
}

/// The commit and dirtiness `cargo package` records in `.cargo_vcs_info.json`, read without a JSON
/// dependency: `{"git": {"sha1": "…", "dirty": true}, "path_in_vcs": "…"}`.
fn published_commit(manifest: &Path) -> Option<(String, bool)> {
    let info = fs::read_to_string(manifest.join(".cargo_vcs_info.json")).ok()?;
    let sha1 = info.split("\"sha1\"").nth(1)?.split('"').nth(1)?;
    (sha1.len() == 40 && sha1.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| (sha1.to_owned(), info.contains("\"dirty\": true")))
}

/// A missing path would rerun the script on every build.
fn rerun_if_changed(path: &Path) {
    if path.exists() {
        println!("cargo::rerun-if-changed={}", path.display());
    }
}

fn output(command: &mut Command) -> String {
    command
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
        .unwrap_or_default()
}
