use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{Formatter, Result as FmtResult},
    fs,
    path::{Path, PathBuf},
    result::Result as StdResult,
    str::{self as StdStr, FromStr},
    sync::OnceLock,
};

use axum::body::Bytes;
use importmap::ImportMap;
use include_dir::{Dir, File as EmbeddedFile, include_dir};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{error, info};

use crate::Error;

/// Embedded website assets
static EMBEDDED_WEBSITE: Dir = include_dir!("$CARGO_MANIFEST_DIR/website");

/// Cached embedded index.html with importmap injected.
static INDEX_HTML: OnceLock<(Bytes, String)> = OnceLock::new();

/// Embedded files' content hashes by path, computed once on first use.
static EMBEDDED_HASHES: OnceLock<HashMap<&'static Path, String>> = OnceLock::new();

/// A file to serve. Embedded content carries its hash (release only), computed once.
pub(crate) struct File {
    pub(crate) contents: Bytes,
    pub(crate) hash: Option<&'static str>,
}

impl File {
    fn fresh(contents: Vec<u8>) -> Self {
        Self {
            contents: contents.into(),
            hash: None,
        }
    }

    fn embedded(file: &'static EmbeddedFile<'static>) -> Self {
        Self {
            contents: Bytes::from_static(file.contents()),
            hash: (!cfg!(debug_assertions)).then(|| embedded_hash(file)),
        }
    }
}

/// Website configuration:
/// - `true` or omitted: serve embedded website
/// - `false`: disable website serving
/// - `"/path/to/website"`: serve custom website from path
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Website {
    Disabled,
    #[default]
    Default,
    Filesystem(PathBuf),
}

impl Website {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Returns the filesystem path if available, None means use embedded
    pub(crate) fn filesystem_path(&self) -> Option<&Path> {
        match self {
            Self::Disabled => None,
            Self::Default => {
                let local = Path::new("./website");
                (cfg!(debug_assertions) && local.exists()).then_some(local)
            }
            Self::Filesystem(p) => Some(p),
        }
    }

    /// Get file content by path (handles hash-stripping, SPA fallback, importmap)
    ///
    /// Returns an error if the website is disabled.
    pub(crate) fn get_file(&self, path: &str) -> Result<File, Error> {
        if !self.is_enabled() {
            return Err(Error::not_found("Website is disabled"));
        }
        match self.filesystem_path() {
            None => self.get_embedded(path),
            Some(base) => self.get_filesystem(base, path),
        }
    }

    /// Log which website source is being used (call once at startup)
    pub fn log(&self) {
        match self {
            Self::Disabled => info!("Website disabled"),
            Self::Default => {
                if let Some(p) = self.filesystem_path() {
                    info!("Website loaded from {}", p.display());
                } else {
                    info!("Using embedded website");
                }
            }
            Self::Filesystem(p) => info!("Website loaded from {}", p.display()),
        }
    }

    fn get_index(&self) -> Result<File, Error> {
        // Filesystem content is mutable and must never enter the embedded cache.
        if let Some(base) = self.filesystem_path() {
            if cfg!(debug_assertions) {
                return fs::read(base.join("index.html"))
                    .map(File::fresh)
                    .map_err(|e| Error::not_found(e.to_string()));
            }
            let html = fs::read_to_string(base.join("index.html"))
                .map_err(|e| Error::not_found(e.to_string()))?;
            let html = match ImportMap::scan(base, "") {
                Ok(importmap) => importmap.transform_html(&html).unwrap_or(html),
                Err(e) => {
                    error!("Failed to scan for importmap: {e}");
                    html
                }
            };
            return Ok(File::fresh(html.into_bytes()));
        }

        let file = EMBEDDED_WEBSITE
            .get_file("index.html")
            .expect("index.html must exist in embedded website");
        if cfg!(debug_assertions) {
            return Ok(File::embedded(file));
        }

        let (html, hash) = INDEX_HTML.get_or_init(|| {
            let html = StdStr::from_utf8(file.contents()).expect("index.html must be valid UTF-8");
            let importmap = ImportMap::scan_embedded(&EMBEDDED_WEBSITE, "");
            let html = importmap
                .transform_html(html)
                .unwrap_or_else(|| html.to_string());
            let hash = ImportMap::hash(html.as_bytes());
            (html.into(), hash)
        });

        Ok(File {
            contents: html.clone(),
            hash: Some(hash),
        })
    }

    fn get_embedded(&self, path: &str) -> Result<File, Error> {
        // Index.html
        if path.is_empty() || path == "index.html" {
            return self.get_index();
        }

        // Try direct lookup, then with hash stripped
        if let Some(file) = embedded_file(path) {
            return Ok(File::embedded(file));
        }

        // SPA fallback: no extension -> index.html
        if Path::new(path).extension().is_none() {
            return self.get_index();
        }

        Err(Error::not_found("File not found"))
    }

    fn get_filesystem(&self, base: &Path, path: &str) -> Result<File, Error> {
        // Root index.html (importmap, SPA)
        if path.is_empty() || path == "index.html" {
            return self.get_index();
        }

        let mut file_path = base.join(path);

        // A folder with its own index.html is a standalone page
        let page = file_path.join("index.html");
        if page.is_file() {
            file_path = page;
        }

        // Try with hash stripped
        if !file_path.exists()
            && let Some(unhashed) = ImportMap::strip_hash(&file_path)
            && unhashed.exists()
        {
            file_path = unhashed;
        }

        // SPA fallback or missing file
        if !file_path.exists() || file_path.is_dir() {
            if file_path.extension().is_some() {
                return Err(Error::not_found("File not found"));
            }
            return self.get_index();
        }

        fs::read(&file_path).map(File::fresh).map_err(|e| {
            error!("{e}");
            Error::not_found("File not found")
        })
    }
}

fn embedded_hash(file: &EmbeddedFile<'static>) -> &'static str {
    fn collect(dir: &'static Dir<'static>, hashes: &mut HashMap<&'static Path, String>) {
        for file in dir.files() {
            hashes.insert(file.path(), ImportMap::hash(file.contents()));
        }
        for dir in dir.dirs() {
            collect(dir, hashes);
        }
    }
    let hashes = EMBEDDED_HASHES.get_or_init(|| {
        let mut hashes = HashMap::new();
        collect(&EMBEDDED_WEBSITE, &mut hashes);
        hashes
    });
    &hashes[file.path()]
}

/// A file, its unhashed name, or a folder's own index.html (standalone page).
fn embedded_file(path: &str) -> Option<&'static EmbeddedFile<'static>> {
    EMBEDDED_WEBSITE
        .get_file(path)
        .or_else(|| {
            ImportMap::strip_hash(Path::new(path))
                .and_then(|unhashed| EMBEDDED_WEBSITE.get_file(unhashed.to_str()?))
        })
        .or_else(|| EMBEDDED_WEBSITE.get_file(Path::new(path).join("index.html")))
}

impl FromStr for Website {
    type Err = Infallible;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Self::Default,
            "false" | "0" | "no" | "off" => Self::Disabled,
            _ => Self::Filesystem(PathBuf::from(s)),
        })
    }
}

impl Serialize for Website {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        match self {
            Self::Disabled => serializer.serialize_bool(false),
            Self::Default => serializer.serialize_bool(true),
            Self::Filesystem(p) => p.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Website {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        use serde::de::{self, Visitor};

        struct WebsiteVisitor;

        impl<'de> Visitor<'de> for WebsiteVisitor {
            type Value = Website;

            fn expecting(&self, f: &mut Formatter) -> FmtResult {
                f.write_str("a boolean or a path string")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> StdResult<Self::Value, E> {
                Ok(if v {
                    Website::Default
                } else {
                    Website::Disabled
                })
            }

            fn visit_str<E: de::Error>(self, v: &str) -> StdResult<Self::Value, E> {
                Ok(Website::Filesystem(PathBuf::from(v)))
            }

            fn visit_string<E: de::Error>(self, v: String) -> StdResult<Self::Value, E> {
                Ok(Website::Filesystem(PathBuf::from(v)))
            }
        }

        deserializer.deserialize_any(WebsiteVisitor)
    }
}
