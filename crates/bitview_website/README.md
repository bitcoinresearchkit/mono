# bitview_website

Serves Bitview's showcases, embedded or from a directory.

`/` serves `index.html` and `/<path>` the file it names, else that folder's own `index.html`;
anything else is 404. Release builds tag each response with an ETag of its content.

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
