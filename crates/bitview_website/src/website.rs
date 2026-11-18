use std::{
    collections::HashMap,
    fmt::{Formatter, Result as FmtResult, Write},
    fs,
    io::ErrorKind,
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

pub(crate) static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/showcases");

/// The embedded site as served, made once on first use: each file's bytes and content hash, its pages with the
/// release's addresses written in (`versioned`), and the release itself, one hash of every file's path and hash.
struct Served {
    release: String,
    files: HashMap<&'static Path, (Bytes, String)>,
}

static SERVED: OnceLock<Served> = OnceLock::new();

/// The folder whose next segment names a release (`/v/<release>/<path>`): the file at `<path>` as that release serves
/// it, which never changes, so it's cached for good.
pub(crate) const RELEASES: &str = "v/";

/// The same files at a second address (`/w/<release>/<path>`), for workers: in WebKit, a page's module whose address a
/// worker has also loaded comes from the network on each reload after, so a worker's modules have addresses of their
/// own (`showcases/studio/workers.js`).
pub(crate) const WORKERS: &str = "w/";

/// A file to serve, with the content type its name declares. Embedded content carries its hash, computed once.
pub(crate) struct File {
    /// The file found (a folder's `index.html` for its address).
    pub(crate) path: PathBuf,
    pub(crate) contents: Bytes,
    pub(crate) hash: Option<&'static str>,
    pub(crate) content_type: &'static str,
}

impl File {
    fn fresh(contents: Vec<u8>, path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            contents: contents.into(),
            hash: None,
            content_type: content_type(path),
        }
    }

    fn embedded(file: &'static EmbeddedFile<'static>) -> Self {
        let (contents, hash) = &served().files[file.path()];
        Self {
            path: file.path().to_owned(),
            contents: contents.clone(),
            hash: Some(hash),
            content_type: content_type(file.path()),
        }
    }
}

/// The hash a file's validator names.
pub(crate) fn hash(contents: &[u8]) -> String {
    format!("{:016x}", rapidhash_v3(contents))
}

/// The content type a file's name declares: the kinds a site holds, bytes otherwise.
fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
    {
        "css" => "text/css",
        "wasm" => "application/wasm",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "html" => "text/html",
        "txt" => "text/plain",
        // (A file has no charset of its own to declare, as a page's meta does.)
        "md" => "text/markdown; charset=utf-8",
        "xml" => "application/xml",
        "woff2" => "font/woff2",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webmanifest" => "application/manifest+json",
        _ => "application/octet-stream",
    }
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

    /// The release the site's files are served under (`RELEASES`): the embedded site's alone, as a folder's files may
    /// change while it's served.
    pub(crate) fn release(&self) -> Option<&'static str> {
        self.filesystem_path()
            .is_none()
            .then(|| served().release.as_str())
    }

    /// The file `path` names, else the `index.html` of the folder it names (`""` is the root's). A path
    /// naming neither, without an extension, gets the page of the nearest folder above it that has one,
    /// never the root's: an app's own addresses (Studio's presets, `studio/presets/...`), while a
    /// missing file (`studio/x.js`) or an unknown address at the root stays not found.
    pub(crate) fn get_file(&self, path: &str) -> Result<File, Error> {
        // `router` registers no route for a disabled website.
        debug_assert!(self.is_enabled());
        let path = Path::new(path);
        let above = path
            .ancestors()
            .skip(1)
            .take_while(|folder| !folder.as_os_str().is_empty())
            .filter(|_| path.extension().is_none());
        let mut candidates = [path.to_path_buf(), path.join("index.html")]
            .into_iter()
            .chain(above.map(|folder| folder.join("index.html")));
        let file = match self.filesystem_path() {
            None => {
                candidates.find_map(|candidate| EMBEDDED.get_file(candidate).map(File::embedded))
            }
            Some(base) => {
                let failed = |file: &Path, e: std::io::Error| {
                    error!("{}: {e}", file.display());
                    Error::internal("Failed to read website file")
                };
                for file in candidates.map(|candidate| base.join(candidate)) {
                    let metadata = match fs::metadata(&file) {
                        Ok(metadata) => metadata,
                        Err(e)
                            if matches!(
                                e.kind(),
                                ErrorKind::NotFound | ErrorKind::NotADirectory
                            ) =>
                        {
                            continue;
                        }
                        Err(e) => return Err(failed(&file, e)),
                    };
                    if metadata.is_file() {
                        let contents = fs::read(&file).map_err(|e| failed(&file, e))?;
                        let path = file.strip_prefix(base).unwrap();
                        let contents = if matches!(self, Self::Default)
                            && content_type(path) == "text/html"
                        {
                            let html = std::str::from_utf8(&contents)
                                .map_err(|e| Error::internal(format!("{}: {e}", file.display())))?;
                            versioned(html, "", |path| {
                                !path.ends_with(".html") && base.join(path).is_file()
                            })
                            .map_err(|e| Error::internal(format!("{}: {e}", file.display())))?
                            .into_bytes()
                        } else {
                            contents
                        };
                        return Ok(File::fresh(contents, path));
                    }
                }
                None
            }
        };
        file.ok_or_else(Error::not_found)
    }

    /// Log which website source is being used (call once at startup)
    pub fn log(&self) {
        match self {
            Self::Disabled => info!("Website disabled"),
            Self::Default => {
                if let Some(p) = self.filesystem_path() {
                    info!("Serving showcases from {}", p.display());
                } else {
                    info!("Using embedded website");
                }
            }
            Self::Filesystem(p) => info!("Serving website from {}", p.display()),
        }
    }
}

fn served() -> &'static Served {
    SERVED.get_or_init(|| {
        fn collect(dir: &'static Dir<'static>, files: &mut Vec<&'static EmbeddedFile<'static>>) {
            files.extend(dir.files());
            for dir in dir.dirs() {
                collect(dir, files);
            }
        }
        let mut files = Vec::new();
        collect(&EMBEDDED, &mut files);
        files.sort_unstable_by_key(|file| file.path());
        let hashes: Vec<String> = files.iter().map(|file| hash(file.contents())).collect();
        // (The content type too: how a file is served is part of what the release holds for good.)
        let mut every = String::new();
        for (file, file_hash) in files.iter().zip(&hashes) {
            let (path, kind) = (file.path().display(), content_type(file.path()));
            writeln!(every, "{path}\0{kind}\0{file_hash}").unwrap();
        }
        let release = hash(every.as_bytes());
        let is_file = |path: &str| !path.ends_with(".html") && EMBEDDED.get_file(path).is_some();
        let mut errors = Vec::new();
        let files = files
            .into_iter()
            .zip(hashes)
            .filter_map(|(file, file_hash)| {
                let result = if content_type(file.path()) == "text/html" {
                    std::str::from_utf8(file.contents())
                        .map_err(|e| e.to_string())
                        .and_then(|html| versioned(html, &release, is_file))
                        .map(|page| {
                            let page_hash = hash(page.as_bytes());
                            (Bytes::from(page), page_hash)
                        })
                } else {
                    Ok((Bytes::from_static(file.contents()), file_hash))
                };
                match result {
                    Ok(contents) => Some((file.path(), contents)),
                    Err(e) => {
                        errors.push(format!("{}: {e}", file.path().display()));
                        None
                    }
                }
            })
            .collect();
        assert!(
            errors.is_empty(),
            "Invalid website pages:\n{}",
            errors.join("\n")
        );
        Served { release, files }
    })
}

/// A page as a release serves it: its import map first in its head, sending every module the page loads (static,
/// dynamic, relative or absolute; addresses already a release's excepted) to the release's address, and each of the
/// site's files it names by its path from the root (in `src`, `srcset`, `url()`, a `<link>`'s `href`, a script's
/// string) written at the release's address. A link's (`<a href>`) stays where readers go.
fn versioned(html: &str, release: &str, is_file: impl Fn(&str) -> bool) -> Result<String, String> {
    for tag in html
        .split("<script")
        .skip(1)
        .filter_map(|rest| rest.split('>').next())
    {
        if tag.contains("type=\"module\"")
            && let Some((_, rest)) = tag.split_once(" src=\"")
        {
            let src = rest.split('"').next().unwrap_or_default();
            if !src
                .strip_prefix('/')
                .is_some_and(|p| !p.contains(['?', '#']) && is_file(p))
            {
                return Err(format!("Cannot rewrite module src {src:?}"));
            }
        }
    }
    let (prefix, workers) = if release.is_empty() {
        (String::new(), String::new())
    } else {
        (
            format!("/{RELEASES}{release}"),
            format!("/{WORKERS}{release}"),
        )
    };
    let mut page = String::with_capacity(html.len() + 4096);
    let mut last = 0;
    for (at, _) in html.match_indices('/') {
        let before = &html[..at];
        if !(before.ends_with(['"', '\'', '(']) || before.ends_with(", ")) {
            continue;
        }
        let rest = &html[at + 1..];
        let path = &rest[..rest
            .find(|c: char| matches!(c, '"' | '\'' | ')' | ',' | '?' | '#') || c.is_whitespace())
            .unwrap_or(rest.len())];
        if is_file(path) && !is_link(before) {
            page.push_str(&html[last..at]);
            page.push_str(&prefix);
            last = at;
        }
    }
    page.push_str(&html[last..]);
    let head = page.find("<head>").ok_or("Page has no <head>")? + "<head>".len();
    page.insert_str(
        head,
        &format!(
            r#"<script type="importmap">{{"imports":{{"/":"{prefix}/","/{RELEASES}":"/{RELEASES}","worker/":"{workers}/"}}}}</script>"#
        ),
    );
    Ok(page)
}

/// Whether the attribute value that `before` ends opening is a link's (`<a href="`).
fn is_link(before: &str) -> bool {
    before
        .strip_suffix(['"', '\''])
        .and_then(|before| before.strip_suffix("href="))
        .and_then(|before| before.rfind('<').map(|tag| &before[tag + 1..]))
        .is_some_and(|tag| tag.split_whitespace().next() == Some("a"))
}

impl FromStr for Website {
    type Err = !;

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
