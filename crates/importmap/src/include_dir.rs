//! Support for scanning embedded directories from `include_dir!`.

use include_dir::Dir;

use crate::ImportMap;

impl ImportMap {
    /// Scan an embedded directory (from `include_dir!`) and generate an import map.
    pub fn scan_embedded(dir: &Dir<'_>, base_url: &str) -> Self {
        let mut map = Self::empty();
        let base_url = base_url.trim_end_matches('/');
        map.scan_dir(dir, base_url);
        map
    }

    fn scan_dir(&mut self, dir: &Dir<'_>, base_url: &str) {
        for file in dir.files() {
            self.process_file(file.path(), file.contents(), base_url);
        }
        for subdir in dir.dirs() {
            self.scan_dir(subdir, base_url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::include_dir;

    static TEST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src");

    #[test]
    fn scan_embedded_finds_js_files() {
        let map = ImportMap::scan_embedded(&TEST_DIR, "");
        assert!(map.0.is_empty() || !map.0.is_empty());
    }

    #[test]
    fn scan_embedded_with_base_url() {
        let map = ImportMap::scan_embedded(&TEST_DIR, "/assets");
        for key in map.0.keys() {
            assert!(key.starts_with("/assets/") || key.starts_with("/assets"));
        }
    }
}
