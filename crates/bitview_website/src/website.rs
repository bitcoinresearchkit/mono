use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{Formatter, Result as FmtResult},
    fs,
    path::{Path, PathBuf},
    result::Result as StdResult,
    str::FromStr,
    sync::OnceLock,
};

use axum::body::Bytes;
use include_dir::{Dir, File as EmbeddedFile, include_dir};
use rapidhash::v3::rapidhash_v3;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{error, info};

use crate::Error;

static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/showcases");

/// Embedded files' content hashes by path, computed once on first use.
static EMBEDDED_HASHES: OnceLock<HashMap<&'static Path, String>> = OnceLock::new();

/// A file to serve, with the content type its name declares. Embedded content carries its hash
/// (release only), computed once.
pub(crate) struct File {
    pub(crate) contents: Bytes,
    pub(crate) hash: Option<&'static str>,
    pub(crate) content_type: Option<&'static str>,
}

impl File {
    fn fresh(contents: Vec<u8>, path: &Path) -> Self {
        Self {
            contents: contents.into(),
            hash: None,
            content_type: content_type(path),
        }
    }

    fn embedded(file: &'static EmbeddedFile<'static>) -> Self {
        Self {
            contents: Bytes::from_static(file.contents()),
            hash: (!cfg!(debug_assertions)).then(|| embedded_hash(file)),
            content_type: content_type(file.path()),
        }
    }
}

/// The hash a file's validator names.
pub(crate) fn hash(contents: &[u8]) -> String {
    format!("{:016x}", rapidhash_v3(contents))
}

/// The content type a file's name declares, for the kinds the showcases use.
fn content_type(path: &Path) -> Option<&'static str> {
    Some(match path.extension()?.to_str()? {
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "html" => "text/html",
        "txt" => "text/plain",
        "xml" => "application/xml",
        "woff2" => "font/woff2",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webmanifest" => "application/manifest+json",
        _ => return None,
    })
}

/// Website configuration:
/// - `true` or omitted: `./showcases` in debug builds when it exists, else the embedded showcases
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

    /// The directory to serve from; `None` for the embedded showcases (or when disabled).
    pub(crate) fn filesystem_path(&self) -> Option<&Path> {
        match self {
            Self::Disabled => None,
            Self::Default => {
                let local = Path::new("./showcases");
                (cfg!(debug_assertions) && local.exists()).then_some(local)
            }
            Self::Filesystem(p) => Some(p),
        }
    }

    /// The file `path` names, else the `index.html` of the folder it names (`""` is the root's).
    pub(crate) fn get_file(&self, path: &str) -> Result<File, Error> {
        // `router` registers no route for a disabled website.
        debug_assert!(self.is_enabled());
        let path = Path::new(path);
        let page = path.join("index.html");
        let file = match self.filesystem_path() {
            None => EMBEDDED
                .get_file(path)
                .or_else(|| EMBEDDED.get_file(&page))
                .map(File::embedded),
            Some(base) => [base.join(path), base.join(&page)]
                .into_iter()
                .find(|file| file.is_file())
                .and_then(|file| match fs::read(&file) {
                    Ok(contents) => Some(File::fresh(contents, &file)),
                    Err(e) => {
                        error!("{}: {e}", file.display());
                        None
                    }
                }),
        };
        file.ok_or_else(|| Error::not_found("File not found"))
    }

    /// Log which website source is being used (call once at startup)
    pub fn log(&self) {
        match self {
            Self::Disabled => info!("Website disabled"),
            Self::Default => {
                if let Some(p) = self.filesystem_path() {
                    info!("Serving website from {}", p.display());
                } else {
                    info!("Using embedded website");
                }
            }
            Self::Filesystem(p) => info!("Serving website from {}", p.display()),
        }
    }
}

fn embedded_hash(file: &EmbeddedFile<'static>) -> &'static str {
    fn collect(dir: &'static Dir<'static>, hashes: &mut HashMap<&'static Path, String>) {
        for file in dir.files() {
            hashes.insert(file.path(), hash(file.contents()));
        }
        for dir in dir.dirs() {
            collect(dir, hashes);
        }
    }
    let hashes = EMBEDDED_HASHES.get_or_init(|| {
        let mut hashes = HashMap::new();
        collect(&EMBEDDED, &mut hashes);
        hashes
    });
    &hashes[file.path()]
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
