use std::{env, path::Path, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace = Path::new(&manifest).ancestors().nth(2).unwrap();
    let version = if workspace.join(".git").exists() {
        commit(workspace)
    } else {
        format!("v{}", env!("CARGO_PKG_VERSION"))
    };
    println!("cargo::rustc-env=BITVIEW_SERVER_VERSION={version}");
}

fn commit(workspace: &Path) -> String {
    let git = |args: &[&str]| {
        let output = Command::new("git").current_dir(workspace).args(args).output()
            .expect("running git to determine the Bitview server version");
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim_end().to_owned()
    };
    let git_path = |path: &str| git(&["rev-parse", "--path-format=absolute", "--git-path", path]);
    let watch = |path: &str| println!("cargo::rerun-if-changed={path}");
    let watch_ref = |reference| {
        let log = git_path(&format!("logs/{reference}"));
        let exists = Path::new(&log).is_file();
        // A reflog tracks one ref even when its loose file is packed.
        watch(&if exists { log } else { git_path(reference) });
        exists
    };
    watch_ref("HEAD");
    let head = git(&["rev-parse", "--symbolic-full-name", "HEAD"]);
    if head != "HEAD" && !watch_ref(&head) {
        let packed = git_path("packed-refs");
        if Path::new(&packed).is_file() {
            watch(&packed);
        }
    }
    git(&["rev-parse", "--short=7", "HEAD"])
}
