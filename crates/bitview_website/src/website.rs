use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use importmap::ImportMap;
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::Error;

/// Embedded website assets
pub static EMBEDDED_WEBSITE: Dir = include_dir!("$CARGO_MANIFEST_DIR/website");

struct CachedIndex {
    html: Vec<u8>,
    etag: String,
}

/// Cached index.html with importmap injected
static INDEX_HTML: OnceLock<CachedIndex> = OnceLock::new();

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

    /// Returns the cached index.html etag (None in debug mode or before first request)
    pub fn index_etag(&self) -> Option<&str> {
        if cfg!(debug_assertions) {
            return None;
        }
        INDEX_HTML.get().map(|cached| cached.etag.as_str())
    }

    /// Returns the filesystem path if available, None means use embedded
    pub fn filesystem_path(&self) -> Option<PathBuf> {
        match self {
            Self::Disabled => None,
            Self::Default => {
                if cfg!(debug_assertions) {
                    let local = PathBuf::from("./website");
                    local.exists().then_some(local)
                } else {
                    None
                }
            }
            Self::Filesystem(p) => Some(p.clone()),
        }
    }

    /// Get file content by path (handles hash-stripping, SPA fallback, importmap)
    ///
    /// Returns an error if the website is disabled.
    pub fn get_file(&self, path: &str) -> Result<Vec<u8>, Error> {
        if !self.is_enabled() {
            return Err(Error::not_found("Website is disabled"));
        }
        match self.filesystem_path() {
            None => self.get_embedded(path),
            Some(base) => self.get_filesystem(&base, path),
        }
    }

    /// Log which website source is being used (call once at startup)
    pub fn log(&self) {
        match self {
            Self::Disabled => info!("website: disabled"),
            Self::Default => {
                if let Some(p) = self.filesystem_path() {
                    info!("website: filesystem ({})", p.display());
                } else {
                    info!("website: embedded");
                }
            }
            Self::Filesystem(p) => info!("website: filesystem ({})", p.display()),
        }
    }

    fn get_index(&self) -> Result<Vec<u8>, Error> {
        // Debug mode: no importmap, no cache
        if cfg!(debug_assertions) {
            return match self.filesystem_path() {
                Some(base) => {
                    fs::read(base.join("index.html")).map_err(|e| Error::not_found(e.to_string()))
                }
                None => {
                    let file = EMBEDDED_WEBSITE
                        .get_file("index.html")
                        .expect("index.html must exist in embedded website");
                    Ok(file.contents().to_vec())
                }
            };
        }

        // Release mode: cache with importmap
        let cached = INDEX_HTML.get_or_init(|| {
            let html = match self.filesystem_path() {
                None => {
                    let file = EMBEDDED_WEBSITE
                        .get_file("index.html")
                        .expect("index.html must exist in embedded website");

                    let html = std::str::from_utf8(file.contents())
                        .expect("index.html must be valid UTF-8");

                    let importmap = ImportMap::scan_embedded(&EMBEDDED_WEBSITE, "");
                    importmap
                        .transform_html(html)
                        .unwrap_or_else(|| html.to_string())
                }
                Some(base) => {
                    let html =
                        fs::read_to_string(base.join("index.html")).expect("index.html must exist");

                    match ImportMap::scan(&base, "") {
                        Ok(importmap) => importmap.transform_html(&html).unwrap_or(html),
                        Err(e) => {
                            error!("Failed to scan for importmap: {e}");
                            html
                        }
                    }
                }
            };

            let mut hasher = DefaultHasher::new();
            html.hash(&mut hasher);
            let etag = format!("\"{}\"", hasher.finish());

            CachedIndex {
                html: html.into_bytes(),
                etag,
            }
        });

        Ok(cached.html.clone())
    }

    fn get_embedded(&self, path: &str) -> Result<Vec<u8>, Error> {
        // Index.html
        if path.is_empty() || path == "index.html" {
            return self.get_index();
        }

        // Try direct lookup, then with hash stripped
        let file = EMBEDDED_WEBSITE.get_file(path).or_else(|| {
            ImportMap::strip_hash(Path::new(path))
                .and_then(|unhashed| EMBEDDED_WEBSITE.get_file(unhashed.to_str()?))
        });

        if let Some(file) = file {
            return Ok(file.contents().to_vec());
        }

        // SPA fallback: no extension -> index.html
        if Path::new(path).extension().is_none() {
            return self.get_index();
        }

        Err(Error::not_found("File not found"))
    }

    fn get_filesystem(&self, base: &Path, path: &str) -> Result<Vec<u8>, Error> {
        // Index.html
        if path.is_empty() {
            return self.get_index();
        }

        let mut file_path = base.join(path);

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

        // Explicit index.html request
        if file_path.file_name().is_some_and(|n| n == "index.html") {
            return self.get_index();
        }

        fs::read(&file_path).map_err(|e| {
            error!("{e}");
            Error::not_found("File not found")
        })
    }
}

impl FromStr for Website {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Self::Default,
            "false" | "0" | "no" | "off" => Self::Disabled,
            _ => Self::Filesystem(PathBuf::from(s)),
        })
    }
}

impl Serialize for Website {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Disabled => serializer.serialize_bool(false),
            Self::Default => serializer.serialize_bool(true),
            Self::Filesystem(p) => p.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Website {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        use serde::de::{self, Visitor};

        struct WebsiteVisitor;

        impl<'de> Visitor<'de> for WebsiteVisitor {
            type Value = Website;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a boolean or a path string")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Self::Value, E> {
                Ok(if v {
                    Website::Default
                } else {
                    Website::Disabled
                })
            }

            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
                Ok(Website::Filesystem(PathBuf::from(v)))
            }

            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Self::Value, E> {
                Ok(Website::Filesystem(PathBuf::from(v)))
            }
        }

        deserializer.deserialize_any(WebsiteVisitor)
    }
}
