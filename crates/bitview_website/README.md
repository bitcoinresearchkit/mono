# bitview_website

Embedded and filesystem-backed website serving for Bitview.

Routes without extensions fall back to `index.html`, and a folder with its own `index.html` is
served as a standalone page. Release builds generate the root index's import map for hashed
assets.

## Usage

```rust,ignore
use bitview_website::{Website, router};

let website_router = router(Website::Default);
let app = your_api_router.merge(website_router);
```

## Website Enum

| Variant | Description |
|---------|-------------|
| `Default` | `./website` in debug builds when it exists, embedded otherwise |
| `Filesystem(path)` | Always serve from specified path |
| `Disabled` | No routes registered |

## Standalone Server

See the `website` example for a complete standalone server with compression, tracing, and other middleware.

```sh
cargo run -p bitview_website --example website
```

Pass a folder to serve it instead of the default website:

```sh
cargo run -p bitview_website --example website -- showcases
```

Open `http://localhost:3110` (or port 3111 if 3110 is occupied) for the showcases gallery.
