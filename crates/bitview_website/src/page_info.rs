//! Address-specific content owned by a website folder, loaded and rendered once when its router starts.
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

use axum::http::{HeaderMap, header};
use flate2::read::GzDecoder;
use include_dir::Dir;
use serde::Deserialize;
use tracing::info;

use crate::{
    Website,
    website::{EMBEDDED, File},
};

const NAME: &str = "page-info.json.gz";
const HEAD: (&str, &str) = ("<!-- page-info-head -->", "<!-- /page-info-head -->");
const BODY: (&str, &str) = ("<!-- page-info-body -->", "<!-- /page-info-body -->");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
    markdown: String,
}

#[derive(Default)]
pub(crate) struct Pages(HashMap<PathBuf, HashMap<String, Page>>);

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
        let mut pages = Self::default();
        for file in files {
            let load = || -> Result<HashMap<String, Page>> {
                let contents = website
                    .get_file(file.to_str().ok_or("Non-UTF-8 page-info path")?)
                    .map_err(|_| "Cannot read page-info file")?;
                let mut decoded = Vec::new();
                GzDecoder::new(contents.contents.as_ref()).read_to_end(&mut decoded)?;
                let sources: HashMap<String, Source> = serde_json::from_slice(&decoded)?;
                let index = file.with_file_name("index.html");
                let template = website
                    .get_file(index.to_str().ok_or("Non-UTF-8 index path")?)
                    .map_err(|_| "Cannot read page-info index.html")?;
                let html = std::str::from_utf8(&template.contents)?;
                slots(html)?;
                sources.into_iter().map(|(address, source)| {
                    if !address.starts_with('/') || address.contains(['?', '#']) { return Err("Page-info keys must be folder-relative addresses starting with /, without query or fragment".into()); }
                    Ok((address, Page::new(source)))
                }).collect()
            };
            let entries = load().map_err(|error| format!("{}: {error}", file.display()))?;
            info!(file = %file.display(), entries = entries.len(), "Loaded website page info");
            pages.0.insert(file.with_file_name("index.html"), entries);
        }
        info!(elapsed = ?start.elapsed(), "Website page info ready");
        Ok(pages)
    }

    /// Only the folder whose index actually answered owns this address's content.
    pub(crate) fn apply(
        &self,
        file: &mut File,
        address: &str,
        headers: &HeaderMap,
    ) -> Result<bool> {
        let Some(entries) = self.0.get(&file.path) else {
            return Ok(false);
        };
        let folder = file
            .path
            .parent()
            .and_then(Path::to_str)
            .ok_or("Non-UTF-8 website folder")?;
        let prefix = if folder.is_empty() {
            String::new()
        } else {
            format!("/{folder}")
        };
        let Some(page) = address
            .strip_prefix(&prefix)
            .and_then(|address| entries.get(address))
        else {
            return Ok(false);
        };
        file.hash = None;
        if wants_markdown(headers) {
            file.contents = page.markdown.clone().into();
            file.content_type = "text/markdown; charset=utf-8";
        } else {
            file.content_type = "text/html; charset=utf-8";
            let html = std::str::from_utf8(&file.contents)?;
            let (head, body) = slots(html)?;
            let mut rendered =
                String::with_capacity(html.len() + page.head.len() + page.body.len());
            rendered.push_str(&html[..head.0]);
            rendered.push_str(&page.head);
            rendered.push_str(&html[head.1..body.0]);
            rendered.push_str(&page.body);
            rendered.push_str(&html[body.1..]);
            file.contents = rendered.into();
        }
        Ok(true)
    }
}

/// The two explicit replacement slots, each exactly once, in document order.
fn slots(html: &str) -> Result<((usize, usize), (usize, usize))> {
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
    fn new(source: Source) -> Self {
        let title = escape(&source.title);
        let description = escape(&source.description);
        let canonical = escape(&source.canonical);
        // JSON inside a script is raw text: escaping '<' prevents any case of </script> ending it.
        let json = source.json_ld.to_string().replace('<', "\\u003c");
        Self {
            head: format!(
                "\n<title>{title}</title>\n<meta name=\"description\" content=\"{description}\">\n<link rel=\"canonical\" href=\"{canonical}\">\n<meta property=\"og:title\" content=\"{title}\">\n<meta property=\"og:description\" content=\"{description}\">\n<meta property=\"og:url\" content=\"{canonical}\">\n<script type=\"application/ld+json\">{json}</script>\n"
            ),
            body: markdown(&source.markdown),
            markdown: source.markdown,
        }
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

/// HTML is the default. Markdown must be explicitly requested and preferred over HTML;
/// more specific ranges override wildcard ranges, including q=0 exclusions.
fn wants_markdown(headers: &HeaderMap) -> bool {
    let ranges: Vec<_> = headers
        .get_all(header::ACCEPT)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .collect();
    let quality = |kind: &str| {
        ranges
            .iter()
            .filter_map(|range| {
                let mut parts = range.split(';');
                let media = parts.next()?.trim();
                let specificity = if media.eq_ignore_ascii_case(kind) {
                    2
                } else if media.eq_ignore_ascii_case("text/*") {
                    1
                } else if media == "*/*" {
                    0
                } else {
                    return None;
                };
                let q = parts
                    .find_map(|part| {
                        part.trim()
                            .split_once('=')
                            .filter(|(key, _)| key.eq_ignore_ascii_case("q"))
                            .map(|(_, value)| value.trim().parse::<f32>().unwrap_or(0.0))
                    })
                    .unwrap_or(1.0);
                Some((specificity, if (0.0..=1.0).contains(&q) { q } else { 0.0 }))
            })
            .max_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map_or(0.0, |(_, q)| q)
    };
    ranges.iter().any(|range| {
        range
            .split(';')
            .next()
            .is_some_and(|media| media.trim().eq_ignore_ascii_case("text/markdown"))
    }) && quality("text/markdown") > quality("text/html")
}
