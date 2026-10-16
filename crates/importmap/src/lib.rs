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
    fn empty() -> Self {
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

        let hash = Self::hash(contents);

        let original_url = format!("{}/{}", base_url, path.display());
        let parent = path.parent().filter(|p| *p != Path::new(""));

        let hashed_url = match parent {
            Some(p) => format!("{}/{}/{}.{}.{}", base_url, p.display(), stem, hash, ext),
            None => format!("{}/{}.{}.{}", base_url, stem, hash, ext),
        };

        self.0.insert(original_url, hashed_url);
    }

    /// Strip hash from filename: `foo.abc12345.js` -> `foo.js`
    pub fn strip_hash(path: &Path) -> Option<PathBuf> {
        let (name, _, ext) = Self::split_hash(path)?;
        Some(path.with_file_name(format!("{}.{}", name, ext)))
    }

    /// Whether the filename carries this content hash (`foo.abc12345.js`), so it never changes.
    pub fn names_hash(path: &Path, hash: &str) -> bool {
        Self::split_hash(path).is_some_and(|(_, named, _)| named == hash)
    }

    /// `foo.abc12345.js` -> (`foo`, `abc12345`, `js`) for a hashed extension and a well-formed hash.
    fn split_hash(path: &Path) -> Option<(&str, &str, &str)> {
        let stem = path.file_stem()?.to_str()?;
        let ext = path.extension()?.to_str()?;
        if !Self::EXTENSIONS.contains(&ext) {
            return None;
        }
        let (name, hash) = stem.rsplit_once('.')?;
        (hash.len() == Self::HASH_LEN && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .then_some((name, hash, ext))
    }

    /// The content hash hashed URLs carry: 8 hex chars.
    pub fn hash(contents: &[u8]) -> String {
        let mut hash = format!("{:016x}", rapidhash_v3(contents));
        hash.truncate(Self::HASH_LEN);
        hash
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

    /// Transform HTML content between `<!-- IMPORTMAP -->` and `<!-- /IMPORTMAP -->` markers:
    /// its stylesheet links stay, in order and with their attributes, the hrefs of scanned files
    /// hashed; the import map and module preloads are regenerated after them.
    pub fn transform_html(&self, html: &str) -> Option<String> {
        let start = html.find(Self::MARKER_OPEN)? + Self::MARKER_OPEN.len();
        let end = html[start..].find(Self::MARKER_CLOSE)? + start;

        let mut content = String::new();
        for tag in Self::link_tags(&html[start..end]) {
            if let Some(tag) = self.stylesheet_link(tag) {
                writeln!(content, "{tag}").unwrap();
            }
        }

        let js: BTreeMap<&str, &str> = self
            .0
            .iter()
            .filter(|(url, _)| {
                matches!(
                    Path::new(url).extension().and_then(|e| e.to_str()),
                    Some("js" | "mjs")
                )
            })
            .map(|(url, hashed)| (url.as_str(), hashed.as_str()))
            .collect();
        if !js.is_empty() {
            let json = to_string_pretty(&SerdeJsonJson!({ "imports": js })).ok()?;
            write!(content, "<script type=\"importmap\">\n{json}\n</script>").unwrap();
            for url in js.values() {
                write!(content, "\n<link rel=\"modulepreload\" href=\"{url}\">").unwrap();
            }
        }

        Self::replace_between_markers(html, &content)
    }

    /// Each `<link ...>` tag (any case, may span lines) outside HTML comments.
    fn link_tags(html: &str) -> impl Iterator<Item = &str> {
        let mut rest = html;
        std::iter::from_fn(move || {
            loop {
                let tag = &rest[rest.find('<')?..];
                if let Some(comment) = tag.strip_prefix("<!--") {
                    rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
                    continue;
                }
                let is_link = tag
                    .get(1..5)
                    .is_some_and(|name| name.eq_ignore_ascii_case("link"))
                    && tag[5..]
                        .starts_with(|c: char| c.is_ascii_whitespace() || c == '/' || c == '>');
                if !is_link {
                    rest = &tag[1..];
                    continue;
                }
                let len = Self::outside_quotes(tag)
                    .find(|&(_, c)| c == '>')
                    .map_or(tag.len(), |(i, _)| i + 1);
                rest = &tag[len..];
                return Some(&tag[..len]);
            }
        })
    }

    /// A stylesheet link on one line, its href hashed when it names a scanned file (hashed
    /// already or not); `None` for other links, which are regenerated.
    fn stylesheet_link(&self, tag: &str) -> Option<String> {
        let mut tag = Self::one_line(tag);
        let mut rel = None;
        let mut href = None;
        let attributes = tag["<link".len()..]
            .trim_end_matches('>')
            .trim_end_matches('/');
        let mut quote = None;
        for attribute in attributes.split(|c| {
            match quote {
                Some(q) if c == q => quote = None,
                None if c == '"' || c == '\'' => quote = Some(c),
                _ => {}
            }
            c == ' ' && quote.is_none()
        }) {
            let (name, value) = attribute.split_once('=').unwrap_or((attribute, ""));
            if name.eq_ignore_ascii_case("rel") {
                rel = Some(Self::unquote(value));
            } else if name.eq_ignore_ascii_case("href") {
                let offset = attribute.as_ptr() as usize - tag.as_ptr() as usize;
                href = Some((offset..offset + attribute.len(), Self::unquote(value)));
            }
        }
        if !rel?
            .split_ascii_whitespace()
            .any(|rel| rel.eq_ignore_ascii_case("stylesheet"))
        {
            return None;
        }
        if let Some((range, url)) = href
            && let Some(hashed) = self.get(url).or_else(|| {
                let original = Self::strip_hash(Path::new(url))?;
                self.get(original.to_str()?)
            })
        {
            tag.replace_range(range, &format!("href=\"{hashed}\""));
        }
        Some(tag)
    }

    /// Whitespace outside quoted values collapsed to one space, and dropped around `=` and
    /// before `>`.
    fn one_line(tag: &str) -> String {
        let mut out = String::with_capacity(tag.len());
        let mut quote = None;
        let mut space = false;
        for c in tag.chars() {
            match quote {
                Some(q) => {
                    if c == q {
                        quote = None;
                    }
                }
                None if c.is_whitespace() => {
                    space = true;
                    continue;
                }
                None => {
                    if space && c != '=' && c != '>' && !out.ends_with('=') {
                        out.push(' ');
                    }
                    if c == '"' || c == '\'' {
                        quote = Some(c);
                    }
                }
            }
            space = false;
            out.push(c);
        }
        out
    }

    /// Characters with their byte offsets, skipping quoted values.
    fn outside_quotes(s: &str) -> impl Iterator<Item = (usize, char)> {
        let mut quote = None;
        s.char_indices().filter(move |&(_, c)| match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
                false
            }
            None => {
                if c == '"' || c == '\'' {
                    quote = Some(c);
                }
                quote.is_none()
            }
        })
    }

    fn unquote(value: &str) -> &str {
        value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(value)
    }

    fn replace_between_markers(html: &str, content: &str) -> Option<String> {
        let start_pos = html.find(Self::MARKER_OPEN)?;
        let after_open = start_pos + Self::MARKER_OPEN.len();
        let end_pos = html[after_open..].find(Self::MARKER_CLOSE)? + after_open;

        // The marker's indentation: what precedes it on its line, when that is only whitespace.
        let line_start = html[..start_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prefix = &html[line_start..start_pos];
        let indent = if prefix.chars().all(char::is_whitespace) {
            prefix
        } else {
            ""
        };

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
