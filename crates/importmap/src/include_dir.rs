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
