//! Generate import maps with hashed URLs for cache busting.

use std::{
    collections::BTreeMap,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use rapidhash::v3::rapidhash_v3;

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
    pub const EXTENSIONS: &[&str] = &["js", "mjs", "css"];
    pub const HASH_LEN: usize = 8;
    pub const MARKER_OPEN: &str = "<!-- IMPORTMAP -->";
    pub const MARKER_CLOSE: &str = "<!-- /IMPORTMAP -->";

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
            } else if let Ok(relative) = path.strip_prefix(root) {
                self.process_file(relative, &fs::read(&path)?, base_url);
            }
        }
        Ok(())
    }

    /// Process a file and insert into imports if it should be included.
    fn process_file(&mut self, path: &Path, contents: &[u8], base_url: &str) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !Self::EXTENSIONS.contains(&ext) {
            return;
        }

        // Skip JS files at root (e.g. service-worker.js)
        if ext == "js" && path.parent().is_none_or(|p| p == Path::new("")) {
            return;
        }

        // Skip development builds and test files
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains(".development.") || name.contains(".dev.") || name.contains(".test.") {
            return;
        }

        // Skip underscore-prefixed files (partials/internal)
        if name.starts_with('_') {
            return;
        }

        // Skip test files
        if path.components().any(|c| c.as_os_str() == "tests") {
            return;
        }

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
        let (mut css, js): (BTreeMap<_, _>, Vec<_>) = self.0.iter().fold(
            (BTreeMap::new(), Vec::new()),
            |(mut css, mut js), (k, v)| {
                match Path::new(k).extension().and_then(|e| e.to_str()) {
                    Some("css") => {
                        css.insert(k.clone(), v.clone());
                    }
                    Some("js" | "mjs") => js.push((k, v)),
                    _ => {}
                }
                (css, js)
            },
        );

        // Only include CSS already in HTML (preserves order for cascade correctness)
        let css_urls: Vec<_> = Self::extract_href_values(html, "stylesheet")
            .into_iter()
            .filter_map(|url| css.remove(&url))
            .collect();

        // Generate output
        let stylesheets = css_urls
            .iter()
            .map(|url| format!(r#"<link rel="stylesheet" href="{url}">"#))
            .collect::<Vec<_>>()
            .join("\n");

        let js_map: BTreeMap<_, _> = js.iter().map(|(k, v)| (*k, *v)).collect();
        let json = serde_json::to_string_pretty(&serde_json::json!({ "imports": js_map })).ok()?;
        let script = format!("<script type=\"importmap\">\n{json}\n</script>");

        let preloads = js
            .iter()
            .map(|(_, url)| format!(r#"<link rel="modulepreload" href="{url}">"#))
            .collect::<Vec<_>>()
            .join("\n");

        let content = [stylesheets, script, preloads]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Self::replace_between_markers(html, &content)
    }

    /// Extract href values from link tags with the given rel attribute.
    fn extract_href_values(html: &str, rel: &str) -> Vec<String> {
        let start = html.find(Self::MARKER_OPEN).unwrap_or(0);
        let end = html.find(Self::MARKER_CLOSE).unwrap_or(html.len());

        html[start..end]
            .lines()
            .filter(|line| {
                line.contains(&format!(r#"rel="{rel}""#))
                    || line.contains(&format!(r#"rel='{rel}'"#))
            })
            .filter_map(|line| {
                let href_start = line.find("href=\"").or_else(|| line.find("href='"))? + 6;
                let quote = line.as_bytes().get(href_start - 1).copied()? as char;
                let href_end = line[href_start..].find(quote)?;
                Some(line[href_start..href_start + href_end].to_string())
            })
            .collect()
    }

    fn replace_between_markers(html: &str, content: &str) -> Option<String> {
        let start_pos = html.find(Self::MARKER_OPEN)?;
        let after_open = start_pos + Self::MARKER_OPEN.len();
        let end_pos = html[after_open..].find(Self::MARKER_CLOSE)? + after_open;

        let line_start = html[..start_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let indent = &html[line_start..start_pos];

        let indented: String = content
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{indent}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Some(format!(
            "{}\n{}\n{}{}",
            &html[..after_open],
            indented,
            indent,
            &html[end_pos..]
        ))
    }
}
