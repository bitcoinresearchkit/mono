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
    let log = git(&["rev-parse", "--path-format=absolute", "--git-path", "logs/HEAD"]);
    println!("cargo::rerun-if-changed={log}");
    git(&["rev-parse", "--short=7", "HEAD"])
}
