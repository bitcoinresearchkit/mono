//! Address-specific content owned by a website folder, loaded and rendered once when its router starts.
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

use axum::body::Bytes;
use flate2::read::GzDecoder;
use include_dir::Dir;
use serde::Deserialize;
use tracing::info;

use crate::{
    Website,
    website::{EMBEDDED, hash},
};

const NAME: &str = "page-info.json.gz";
/// Beside a folder's page info, where its addresses once were: a line each, the old address and where it opens now,
/// tab-separated, both folder-relative (the second may carry a fragment). Written by the folder's own generator.
const MOVED: &str = "moved.txt";
/// A folder's page, at the folder's address.
const INDEX: &str = "index.html";
/// What a page's address takes to name its Markdown twin, a folder's page being its `index.html`
/// (`/studio/index.html.md`, `/studio/presets/price/all-time-high.md`): where llmstxt.org has readers look.
const TWIN: &str = ".md";
const HEAD: (&str, &str) = ("<!-- page-info-head -->", "<!-- /page-info-head -->");
const BODY: (&str, &str) = ("<!-- page-info-body -->", "<!-- /page-info-body -->");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// Where a page's two slots begin and end in its folder's index: the head's, then the body's.
type Slots = ((usize, usize), (usize, usize));

/// The address of the Markdown twin of the page at `address`.
pub(crate) fn twin(address: &str) -> String {
    let index = if address.ends_with('/') { INDEX } else { "" };
    format!("{address}{index}{TWIN}")
}

/// The address of the page whose Markdown twin `address` would be.
pub(crate) fn twin_page(address: &str) -> Option<&str> {
    let page = address.strip_suffix(TWIN)?;
    Some(match page.strip_suffix(INDEX) {
        Some(folder) if folder.ends_with('/') => folder,
        _ => page,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    title: String,
    description: String,
    canonical: String,
    json_ld: serde_json::Value,
    markdown: String,
}

struct Page {
    head: String,
    body: String,
    markdown: Bytes,
    /// The page's validator and its twin's, made as the site loads when its files can't change after.
    tags: Option<[String; 2]>,
}

/// A folder's index with its slots found, kept for a site whose files can't change after it loads (the embedded
/// one): a page is then three slices of it around the page's two strings, and nothing is searched per request.
struct Template {
    html: Bytes,
    slots: Slots,
}

#[derive(Default)]
struct Folder {
    template: Option<Template>,
    pages: HashMap<String, Page>,
    /// Each old address, as requested, and where it opens now (`MOVED`).
    moved: HashMap<String, String>,
}

/// The two ways a listed address is sent: the page, and its Markdown twin at an address of its own (`twin`).
#[derive(Clone, Copy)]
pub(crate) enum Form {
    Html,
    Markdown,
}

impl Form {
    pub(crate) fn content_type(self) -> &'static str {
        match self {
            Self::Html => "text/html; charset=utf-8",
            Self::Markdown => "text/markdown; charset=utf-8",
        }
    }
}

/// What a folder's page info makes of an address its index answers.
pub(crate) enum Listing<'a> {
    /// The folder keeps no page info: its index as it is.
    Unlisted,
    /// One of the folder's addresses.
    Listed(Listed<'a>),
    /// Not one of the folder's addresses, which its page info lists in full: nothing is there.
    Unknown,
}

/// One of a folder's addresses: its content, to write into the folder's index or to send as Markdown.
pub(crate) struct Listed<'a> {
    template: Option<&'a Template>,
    page: &'a Page,
}

impl Listed<'_> {
    /// The validator of a form, where it is known without its bytes.
    pub(crate) fn tag(&self, form: Form) -> Option<&str> {
        let [html, markdown] = self.page.tags.as_ref()?;
        Some(match form {
            Form::Html => html,
            Form::Markdown => markdown,
        })
    }

    /// The bytes of a form. `index` is the folder's index as read for this request, searched for its slots only
    /// when none was kept.
    pub(crate) fn contents(&self, form: Form, index: &[u8]) -> Result<Bytes> {
        Ok(match (form, self.template) {
            (Form::Markdown, _) => self.page.markdown.clone(),
            (Form::Html, Some(template)) => self.page.written_into(&template.html, template.slots),
            (Form::Html, None) => {
                self.page.written_into(index, slots(std::str::from_utf8(index)?)?)
            }
        })
    }
}

#[derive(Default)]
pub(crate) struct Pages(HashMap<PathBuf, Folder>);

impl Pages {
    pub(crate) fn load(website: &Website) -> Result<Self> {
        fn embedded(dir: &Dir<'_>, files: &mut Vec<PathBuf>) {
            files.extend(
                dir.files()
                    .filter(|file| file.path().file_name().is_some_and(|name| name == NAME))
                    .map(|file| file.path().to_owned()),
            );
            for dir in dir.dirs() {
                embedded(dir, files);
            }
        }
        fn filesystem(base: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let kind = entry.file_type()?;
                if kind.is_dir() {
                    filesystem(base, &entry.path(), files)?;
                } else if kind.is_file() && entry.file_name() == NAME {
                    files.push(entry.path().strip_prefix(base)?.to_owned());
                }
            }
            Ok(())
        }
        let start = Instant::now();
        let mut files = Vec::new();
        match website.filesystem_path() {
            Some(base) => filesystem(base, base, &mut files)?,
            None => embedded(&EMBEDDED, &mut files),
        }
        // The embedded site's files are what they are for good: its templates and tags are made here, once.
        let fixed = website.filesystem_path().is_none();
        let mut pages = Self::default();
        for file in files {
            let load = || -> Result<Folder> {
                let contents = website
                    .get_file(file.to_str().ok_or("Non-UTF-8 page-info path")?)
                    .map_err(|_| "Cannot read page-info file")?;
                let mut decoded = Vec::new();
                GzDecoder::new(contents.contents.as_ref()).read_to_end(&mut decoded)?;
                let sources: HashMap<String, Source> = serde_json::from_slice(&decoded)?;
                let index = file.with_file_name(INDEX);
                let template = website
                    .get_file(index.to_str().ok_or("Non-UTF-8 index path")?)
                    .map_err(|_| "Cannot read page-info index.html")?;
                let slots = slots(std::str::from_utf8(&template.contents)?)?;
                let template_hash = template.hash.filter(|_| fixed);
                // Kept by the address as requested: the folder's own path, then the key.
                let folder = file
                    .parent()
                    .and_then(Path::to_str)
                    .ok_or("Non-UTF-8 website folder")?;
                let prefix = if folder.is_empty() {
                    String::new()
                } else {
                    format!("/{folder}")
                };
                let pages = sources
                    .into_iter()
                    .map(|(address, source)| {
                        if !address.starts_with('/') || address.contains(['?', '#']) {
                            return Err("Page-info keys must be folder-relative addresses starting with /, without query or fragment".into());
                        }
                        Ok((format!("{prefix}{address}"), Page::new(source, template_hash)))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;
                let moved = match website.get_file(
                    file.with_file_name(MOVED).to_str().ok_or("Non-UTF-8 moved path")?,
                ) {
                    Ok(moved) => std::str::from_utf8(&moved.contents)?
                        .lines()
                        .map(|line| {
                            let (from, to) = line
                                .split_once('\t')
                                .filter(|(from, to)| from.starts_with('/') && to.starts_with('/'))
                                .ok_or_else(|| {
                                    format!("{MOVED}: not an old and a new address: {line}")
                                })?;
                            let (from, to) = (format!("{prefix}{from}"), format!("{prefix}{to}"));
                            // (Each to a page of the folder: never a redirect to a 404, or to another.)
                            let place = to.split_once('#').map_or(to.as_str(), |(place, _)| place);
                            if !pages.contains_key(place) {
                                return Err(format!("{MOVED}: {from} opens no page: {to}").into());
                            }
                            Ok((from, to))
                        })
                        .collect::<Result<_>>()?,
                    Err(error) if error.is_not_found() => HashMap::new(),
                    Err(error) => return Err(format!("Cannot read {MOVED}: {error:?}").into()),
                };
                Ok(Folder {
                    template: fixed.then(|| Template {
                        html: template.contents,
                        slots,
                    }),
                    pages,
                    moved,
                })
            };
            let folder = load().map_err(|error| format!("{}: {error}", file.display()))?;
            info!(file = %file.display(), entries = folder.pages.len(), "Loaded website page info");
            pages.0.insert(file.with_file_name(INDEX), folder);
        }
        info!(elapsed = ?start.elapsed(), "Website page info ready");
        Ok(pages)
    }

    /// Whether the folder of `index` lists `address`.
    pub(crate) fn lists(&self, index: &Path, address: &str) -> bool {
        self.0
            .get(index)
            .is_some_and(|folder| folder.pages.contains_key(address))
    }

    /// Where the folder of `index` says `address` opens now, if it once was one of its addresses (`MOVED`).
    pub(crate) fn moved(&self, index: &Path, address: &str) -> Option<&str> {
        self.0.get(index)?.moved.get(address).map(String::as_str)
    }

    /// What the folder of `index`, the file that answers `address`, makes of it: only that folder owns the
    /// address's content.
    pub(crate) fn find(&self, index: &Path, address: &str) -> Listing<'_> {
        let Some(folder) = self.0.get(index) else {
            return Listing::Unlisted;
        };
        match folder.pages.get(address) {
            Some(page) => Listing::Listed(Listed {
                template: folder.template.as_ref(),
                page,
            }),
            None => Listing::Unknown,
        }
    }
}

/// The two explicit replacement slots, each exactly once, in document order.
fn slots(html: &str) -> Result<Slots> {
    let slot = |(open, close): (&str, &str)| -> Result<(usize, usize)> {
        if html.matches(open).count() != 1 || html.matches(close).count() != 1 {
            return Err(format!("Expected one {open} and {close}").into());
        }
        let start = html.find(open).unwrap() + open.len();
        let end = html.find(close).unwrap();
        if start > end {
            return Err(format!("Reversed {open} slot").into());
        }
        Ok((start, end))
    };
    let head = slot(HEAD)?;
    let body = slot(BODY)?;
    if head.1 >= body.0 {
        return Err("Page-info head must precede body".into());
    }
    Ok((head, body))
}

impl Page {
    /// A page's content as written into its folder's index. With the hash of an index that can't change after,
    /// its validators too: the page's from the index's and what is written in, so no page is rendered to name it.
    fn new(source: Source, template_hash: Option<&str>) -> Self {
        let title = escape(&source.title);
        let description = escape(&source.description);
        let canonical = escape(&source.canonical);
        let twin = escape(&twin(&source.canonical));
        // JSON inside a script is raw text: escaping '<' prevents any case of </script> ending it.
        let json = source.json_ld.to_string().replace('<', "\\u003c");
        let head = format!(
            "\n<title>{title}</title>\n<meta name=\"description\" content=\"{description}\">\n<link rel=\"canonical\" href=\"{canonical}\">\n<link rel=\"alternate\" type=\"text/markdown\" href=\"{twin}\">\n<meta property=\"og:title\" content=\"{title}\">\n<meta property=\"og:description\" content=\"{description}\">\n<meta property=\"og:url\" content=\"{canonical}\">\n<script type=\"application/ld+json\">{json}</script>\n"
        );
        let body = markdown(&source.markdown);
        let tags = template_hash.map(|template| {
            let written = [template, "\0", &head, "\0", &body].concat();
            [hash(written.as_bytes()), hash(source.markdown.as_bytes())]
        });
        Self {
            head,
            body,
            markdown: source.markdown.into(),
            tags,
        }
    }

    /// The folder's index `html` with this page's content in its `slots`.
    fn written_into(&self, html: &[u8], (head, body): Slots) -> Bytes {
        let mut page = Vec::with_capacity(html.len() + self.head.len() + self.body.len());
        page.extend_from_slice(&html[..head.0]);
        page.extend_from_slice(self.head.as_bytes());
        page.extend_from_slice(&html[head.1..body.0]);
        page.extend_from_slice(self.body.as_bytes());
        page.extend_from_slice(&html[body.1..]);
        page.into()
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Inline text and links only; HTML and unsafe link schemes are always plain text.
fn inline(mut text: &str) -> String {
    let mut html = String::new();
    while let Some(start) = text.find('[') {
        html.push_str(&escape(&text[..start]));
        text = &text[start..];
        if let Some((label, rest)) = text[1..].split_once("](")
            && let Some((url, after)) = rest.split_once(')')
            && (url.starts_with("https://")
                || url.starts_with("http://")
                || url.starts_with('/')
                || url.starts_with('#'))
        {
            html.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape(url),
                escape(label)
            ));
            text = after;
        } else {
            html.push('[');
            text = &text[1..];
        }
    }
    html.push_str(&escape(text));
    html
}

/// A deliberately small Markdown subset: headings, paragraphs, unordered lists and links.
fn markdown(text: &str) -> String {
    let mut html = String::new();
    let mut open = "";
    for line in text.lines().map(str::trim) {
        let heading = line.bytes().take_while(|b| *b == b'#').count();
        let tag = if line.is_empty() {
            ""
        } else if line.starts_with("- ") {
            "ul"
        } else {
            "p"
        };
        if open != tag || (1..=6).contains(&heading) && line.as_bytes().get(heading) == Some(&b' ')
        {
            if !open.is_empty() {
                html.push_str(&format!("</{open}>\n"));
            }
            open = "";
        }
        if (1..=6).contains(&heading) && line.as_bytes().get(heading) == Some(&b' ') {
            html.push_str(&format!(
                "<h{heading}>{}</h{heading}>\n",
                inline(&line[heading + 1..])
            ));
        } else if !tag.is_empty() {
            if open.is_empty() {
                html.push_str(&format!("<{tag}>"));
                open = tag;
            }
            if tag == "ul" {
                html.push_str(&format!("<li>{}</li>\n", inline(&line[2..])));
            } else {
                html.push_str(&inline(line));
                html.push('\n');
            }
        }
    }
    if !open.is_empty() {
        html.push_str(&format!("</{open}>\n"));
    }
    html
}
