# bitview_website

Serves Bitview's showcases, embedded or from a directory.

`/` serves `index.html` and `/<path>` the file it names, else that folder's own `index.html`;
an address no file answers that is served elsewhere answers 308 to the same place under its new folder (`/address/…`
to the explorer's), or to the page that replaced it (`/charts` to Studio's; `MOVED` in `src/handlers.rs`), each 308
revalidated as the files are, so a redirect changed with a release reaches browsers; anything else is 404, with
the site's `404.html` for a request whose `Accept` lists `text/html` (a browser's), a plain message otherwise, both
`no-store` since the answer depends on `Accept`. A `Filesystem` website without a `404.html` answers browsers with the
plain message too (behind the API server, its JSON problem document). Release builds tag each response with an ETag of its content, and with the same hash in a
`Bitview-Version` header, which an open page reads to tell when a newer version of it is served
(`showcases/assets/version`).

## Behind Cloudflare

Cloudflare drops pages' ETag whenever one of its features that process HTML is on for the zone, whether or not it
changes anything: browsers then download every page again in full instead of revalidating it (a 304 with no body).
Automatic HTTPS Rewrites alone does it (turned back on for a test, every page lost its ETag within seconds); Email
Address Obfuscation, Server Side Excludes and Replace insecure JavaScript libraries are of the same kind. The
showcases need none of them (no `http://` links, no email addresses, no `<!--sse-->` blocks, no third-party
scripts): bitview.space has them all off. Scripts, images and JSON keep their ETag either way. Cloudflare drops the
ETag on every `HEAD` request too, but keeps the server's own headers (`Bitview-Version`) on `GET` and `HEAD` alike.

## Usage

```rust,ignore
use bitview_website::{Website, router};

let website_router = router(Website::Default);
let app = your_api_router.merge(website_router);
```

## Website Enum

| Variant | Description |
|---------|-------------|
| `Default` | `./showcases` in debug builds when it exists, the embedded showcases otherwise |
| `Filesystem(path)` | Always serve from specified path |
| `Disabled` | No routes registered |

## Standalone Server

See the `website` example for a complete standalone server with tracing, and other middleware.

```sh
cargo run -p bitview_website --example website
```

Pass a folder to serve it instead of the showcases:

```sh
cargo run -p bitview_website --example website -- path/to/site
```

Open `http://localhost:3110` (or port 3111 if 3110 is occupied) for Bitview's front page and its showcases.

## Address-specific page info

A folder can publish `page-info.json.gz`: a gzip-compressed UTF-8 JSON object keyed by addresses relative to that
folder (`"/"`, `"/articles/example"`, etc.). Each entry has `title`, `description`, `canonical`, `json_ld` (a JSON value)
and `markdown`. Only entries belonging to the `index.html` actually selected by the nearest-folder rule apply.
Missing files or entries keep the ordinary response. Invalid data or templates stop router initialization with the file's name.

That index provides exactly one pair of each marker, in document order:

```html
<head>
  <!-- page-info-head -->
  <!-- The generic title, description, canonical, og:title/description/url and JSON-LD go here. -->
  <!-- /page-info-head -->
</head>
<body>
  <article id="page-info"><!-- page-info-body --><!-- /page-info-body --></article>
</body>
```

The server replaces the head slot and fills the body slot. The Markdown subset supports headings, paragraphs,
unordered lists and HTTP(S), root-relative or fragment links; raw HTML is escaped. Rendered entries load once when
the router starts, including for filesystem websites: restart after replacing the data. Gzip keeps large generated
catalogs small on disk and in embedded builds; requests never decompress or render Markdown.

An app removes its body article when it starts, leaving that article readable when JavaScript cannot run. HTML and
Markdown carry the same content for every user agent. An explicit `Accept: text/markdown` preference over HTML selects
the entry's Markdown instead; both variants send `Vary: Accept`, including on 304. Release ETags hash the final response
bytes, so addresses and representations validate independently. Negotiated responses are private and must revalidate, so shared caches that ignore `Accept` cannot mix formats.
