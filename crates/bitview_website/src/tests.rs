use std::{env, fs, process};

use crate::Website;

#[test]
fn addresses_get_their_folders_page() {
    let root = env::temp_dir().join(format!("bitview-website-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("app/deep")).unwrap();
    fs::write(root.join("index.html"), "root").unwrap();
    fs::write(root.join("app/index.html"), "app").unwrap();
    fs::write(root.join("app/deep/file.txt"), "file").unwrap();
    let website = Website::Filesystem(root.clone());
    let read = |path: &str| {
        website
            .get_file(path)
            .ok()
            .map(|file| String::from_utf8(file.contents.to_vec()).unwrap())
    };
    assert_eq!(read("").as_deref(), Some("root"));
    assert_eq!(read("app").as_deref(), Some("app"));
    assert_eq!(read("app/deep/file.txt").as_deref(), Some("file"));
    assert_eq!(read("app/presets/valuation/mvrv").as_deref(), Some("app"));
    assert_eq!(read("app/presets/valuation/").as_deref(), Some("app"));
    assert_eq!(read("app/deep/gone.js"), None);
    assert_eq!(read("nowhere"), None);
    fs::remove_dir_all(root).unwrap();
}
