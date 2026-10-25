# bitview_website

Serves Bitview's showcases, embedded or from a directory.

`/` serves `index.html` and `/<path>` the file it names, else that folder's own `index.html`;
anything else is 404. Release builds tag each response with an ETag of its content, and with the same hash in a
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

See the `website` example for a complete standalone server with compression, tracing, and other middleware.

```sh
cargo run -p bitview_website --example website
```

Pass a folder to serve it instead of the showcases:

```sh
cargo run -p bitview_website --example website -- path/to/site
```

Open `http://localhost:3110` (or port 3111 if 3110 is occupied) for Bitview's front page and its showcases.
