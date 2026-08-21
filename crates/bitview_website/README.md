# bitview_website

Embedded and filesystem-backed website serving for Bitview.

## Features

- **Embedded assets**: Website files compiled into binary
- **Filesystem mode**: Serve from custom path for development
- **SPA support**: Routes without extensions fallback to index.html
- **ImportMap**: Auto-generates import maps for hashed assets

## Usage

```rust,ignore
use bitview_website::{Website, router};

// Create router for website
let website_router = router(Website::Default);

// Merge with your app
let app = your_api_router.merge(website_router);
```

## Website Enum

| Variant | Description |
|---------|-------------|
| `Default` | Filesystem in debug, embedded in release |
| `Filesystem(path)` | Always serve from specified path |
| `Disabled` | No routes registered |

## Standalone Server

See the `website` example for a complete standalone server with compression, tracing, and other middleware.

```sh
cargo run -p bitview_website --example website
```

## Dependencies

- `axum` - HTTP routing
- `include_dir` - embedded assets
- `importmap` - asset hashing
