use crate::{checkout_commit, commit_installed};

#[test]
fn version_from_install_record() {
    // As cargo writes it: a crates.io install, a `cargo install --git` one (its full commit after
    // `#`), another release's, and one whose commit isn't one.
    let version = env!("CARGO_PKG_VERSION");
    let record = format!(
        r#"{{"installs":{{
            "bitviewd {version} (registry+https://github.com/rust-lang/crates.io-index)":{{"bins":["bitviewd"]}},
            "bitview_mcp {version} (git+https://github.com/bitcoinresearchkit/mono?rev=a1b2c3d#a1b2c3d4e5f60718293a4b5c6d7e8f9012345678)":{{"bins":["bitview_mcp"]}},
            "blk 0.0.1 (git+https://github.com/bitcoinresearchkit/mono#a1b2c3d4e5f60718293a4b5c6d7e8f9012345678)":{{"bins":["blk"]}},
            "odd {version} (git+https://example.com/odd#a1b2c3\"\n)":{{"bins":["odd"]}}
        }}}}"#
    );
    let commit = |name| commit_installed(record.as_bytes(), name);
    assert_eq!(commit("bitviewd"), None);
    assert_eq!(commit("bitview_mcp").as_deref(), Some("a1b2c3d"));
    assert_eq!(commit("blk"), None);
    assert_eq!(commit("odd"), None);
    assert_eq!(commit_installed(b"not json", "bitviewd"), None);
}

#[test]
fn version_from_checkout() {
    // A checkout's HEAD: on a branch (its ref loose, then packed), detached, none.
    let root = std::env::temp_dir().join(format!("bitview-checkout-{}", std::process::id()));
    let git = root.join(".git");
    std::fs::create_dir_all(git.join("refs/heads")).unwrap();
    let commit = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";
    std::fs::write(git.join("HEAD"), "ref: refs/heads/next\n").unwrap();
    std::fs::write(git.join("refs/heads/next"), format!("{commit}\n")).unwrap();
    assert_eq!(checkout_commit(&root).as_deref(), Some("a1b2c3d"));
    std::fs::remove_file(git.join("refs/heads/next")).unwrap();
    std::fs::write(git.join("packed-refs"), format!("# pack-refs\n{commit} refs/heads/next\n")).unwrap();
    assert_eq!(checkout_commit(&root).as_deref(), Some("a1b2c3d"));
    std::fs::write(git.join("HEAD"), format!("{commit}\n")).unwrap();
    assert_eq!(checkout_commit(&root).as_deref(), Some("a1b2c3d"));
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(checkout_commit(&root), None);
}
