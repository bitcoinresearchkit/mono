//! The build's identity, embedded at compile time: a run records the code that was compiled, not
//! the checkout the binary happens to run next to.

use std::{env, path::Path, process::Command};

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
    // Commits, checkouts, staging and tags.
    let git_dir = git(&["rev-parse", "--absolute-git-dir"]);
    if !git_dir.is_empty() {
        for state in ["HEAD", "index", "logs/HEAD", "packed-refs", "refs/tags"] {
            rerun_if_changed(&Path::new(&git_dir).join(state));
        }
    }

    let dirty = !git(&[&["status", "--porcelain", "--"][..], &SOURCES].concat()).is_empty();
    // One directory name: the tag at HEAD (`v0.13.0`), else the short commit, else the crate
    // version; `-dirty` with local changes.
    let mut revision = git(&["describe", "--tags", "--exact-match", "HEAD"]);
    if revision.is_empty() {
        revision = git(&["rev-parse", "--short", "HEAD"]);
    }
    if revision.is_empty() {
        revision = format!("v{}", env::var("CARGO_PKG_VERSION").unwrap());
    }
    let mut revision = revision.replace('/', "-");
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
        ("COMMIT", git(&["rev-parse", "HEAD"])),
        ("DIRTY", dirty.to_string()),
        ("PROFILE", profile),
        ("RUSTC", output(Command::new(env::var_os("RUSTC").unwrap()).arg("-V"))),
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
