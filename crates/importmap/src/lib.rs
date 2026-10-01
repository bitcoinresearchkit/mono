//! Generate import maps with hashed URLs for cache busting.

use std::{
    collections::BTreeMap,
    fmt::Write,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use rapidhash::v3::rapidhash_v3;
use serde_json::{json as SerdeJsonJson, to_string_pretty};

#[cfg(feature = "embedded")]
mod include_dir;

/// Import map structure matching the web standard.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportMap(BTreeMap<String, String>);

impl Deref for ImportMap {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ImportMap {
    const EXTENSIONS: &[&str] = &["js", "mjs", "css"];
    const HASH_LEN: usize = 8;
    const MARKER_OPEN: &str = "<!-- IMPORTMAP -->";
    const MARKER_CLOSE: &str = "<!-- /IMPORTMAP -->";

    /// Create an empty import map (useful for dev mode).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Scan a directory and generate an import map.
    pub fn scan(dir: &Path, base_url: &str) -> io::Result<Self> {
        let mut map = Self::empty();
        let base_url = base_url.trim_end_matches('/');
        map.scan_fs(dir, dir, base_url)?;
        Ok(map)
    }

    fn scan_fs(&mut self, root: &Path, dir: &Path, base_url: &str) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                self.scan_fs(root, &path, base_url)?;
            } else if let Ok(relative) = path.strip_prefix(root)
                && Self::includes(relative)
            {
                self.process_file(relative, &fs::read(&path)?, base_url);
            }
        }
        Ok(())
    }

    /// Shared file selection for filesystem and embedded scans.
    fn includes(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !Self::EXTENSIONS.contains(&ext) {
            return false;
        }

        // Skip JS files at root (e.g. service-worker.js)
        if ext == "js" && path.parent().is_none_or(|p| p == Path::new("")) {
            return false;
        }

        // Skip development builds and test files
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains(".development.") || name.contains(".dev.") || name.contains(".test.") {
            return false;
        }

        // Skip underscore-prefixed files (partials/internal)
        if name.starts_with('_') {
            return false;
        }

        // Skip test files
        if path.components().any(|c| c.as_os_str() == "tests") {
            return false;
        }

        path.file_stem().and_then(|s| s.to_str()).is_some()
    }

    /// Process a file and insert into imports if it should be included.
    fn process_file(&mut self, path: &Path, contents: &[u8], base_url: &str) {
        if !Self::includes(path) {
            return;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };

        let hash = rapidhash_v3(contents);
        let hash_hex = format!("{:016x}", hash);
        let short_hash = &hash_hex[..Self::HASH_LEN];

        let original_url = format!("{}/{}", base_url, path.display());
        let parent = path.parent().filter(|p| *p != Path::new(""));

        let hashed_url = match parent {
            Some(p) => format!(
                "{}/{}/{}.{}.{}",
                base_url,
                p.display(),
                stem,
                short_hash,
                ext
            ),
            None => format!("{}/{}.{}.{}", base_url, stem, short_hash, ext),
        };

        self.0.insert(original_url, hashed_url);
    }

    /// Strip hash from filename: `foo.abc12345.js` -> `foo.js`
    pub fn strip_hash(path: &Path) -> Option<PathBuf> {
        let stem = path.file_stem()?.to_str()?;
        let ext = path.extension()?.to_str()?;

        if !Self::EXTENSIONS.contains(&ext) {
            return None;
        }

        let dot_pos = stem.rfind('.')?;
        let hash = &stem[dot_pos + 1..];

        if hash.len() == Self::HASH_LEN && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            let name = &stem[..dot_pos];
            Some(path.with_file_name(format!("{}.{}", name, ext)))
        } else {
            None
        }
    }

    /// Update an HTML file in place between `<!-- IMPORTMAP -->` and `<!-- /IMPORTMAP -->` markers.
    pub fn update_html_file(&self, path: &Path) -> io::Result<bool> {
        let html = fs::read_to_string(path)?;
        match self.transform_html(&html) {
            Some(updated) if updated != html => {
                fs::write(path, updated)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Transform HTML content between `<!-- IMPORTMAP -->` and `<!-- /IMPORTMAP -->` markers.
    pub fn transform_html(&self, html: &str) -> Option<String> {
        if self.0.is_empty() {
            return Self::replace_between_markers(html, "");
        }

        // Partition by file type
        let mut css = BTreeMap::new();
        let mut js = BTreeMap::new();
        for (url, hashed) in &self.0 {
            match Path::new(url).extension().and_then(|e| e.to_str()) {
                Some("css") => {
                    css.insert(url.as_str(), hashed);
                }
                Some("js" | "mjs") => {
                    js.insert(url, hashed);
                }
                _ => {}
            }
        }

        // Only include CSS already in HTML (preserves order for cascade correctness)
        let mut content = String::new();
        for url in Self::extract_href_values(html, "stylesheet") {
            if let Some(hashed) = css.remove(url) {
                writeln!(content, r#"<link rel="stylesheet" href="{hashed}">"#).unwrap();
            }
        }

        let json = to_string_pretty(&SerdeJsonJson!({ "imports": js })).ok()?;
        write!(content, "<script type=\"importmap\">\n{json}\n</script>").unwrap();
        for url in js.values() {
            write!(content, "\n<link rel=\"modulepreload\" href=\"{url}\">").unwrap();
        }

        Self::replace_between_markers(html, &content)
    }

    /// Extract href values from link tags with the given rel attribute.
    fn extract_href_values<'a>(html: &'a str, rel: &str) -> impl Iterator<Item = &'a str> {
        let start = html.find(Self::MARKER_OPEN).unwrap_or(0);
        let end = html.find(Self::MARKER_CLOSE).unwrap_or(html.len());

        let double_quoted = format!(r#"rel="{rel}""#);
        let single_quoted = format!(r#"rel='{rel}'"#);
        html.get(start..end)
            .unwrap_or("")
            .lines()
            .filter(move |line| line.contains(&double_quoted) || line.contains(&single_quoted))
            .filter_map(|line| {
                let href_start = line.find("href=\"").or_else(|| line.find("href='"))? + 6;
                let quote = line.as_bytes().get(href_start - 1).copied()? as char;
                let href_end = line[href_start..].find(quote)?;
                Some(&line[href_start..href_start + href_end])
            })
    }

    fn replace_between_markers(html: &str, content: &str) -> Option<String> {
        let start_pos = html.find(Self::MARKER_OPEN)?;
        let after_open = start_pos + Self::MARKER_OPEN.len();
        let end_pos = html[after_open..].find(Self::MARKER_CLOSE)? + after_open;

        let line_start = html[..start_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let indent = &html[line_start..start_pos];

        let mut output = String::with_capacity(html.len() + content.len());
        output.push_str(&html[..after_open]);
        output.push('\n');
        for (i, line) in content.lines().enumerate() {
            if i != 0 {
                output.push('\n');
            }
            if !line.is_empty() {
                output.push_str(indent);
                output.push_str(line);
            }
        }
        output.push('\n');
        output.push_str(indent);
        output.push_str(&html[end_pos..]);
        Some(output)
    }
}
