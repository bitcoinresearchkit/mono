# importmap

Generate import maps with content-hashed URLs. No bundler required.

## Problem

A browser discovers a module's imports only after fetching and parsing it, so deep dependency trees load as a waterfall of round trips.

Change one file and users might get stale cached versions of files that import it.

## Solution

This crate:

1. Scans your JS/MJS/CSS files and hashes their contents
2. Generates an [import map](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script/type/importmap) that maps clean URLs to hashed URLs
3. Generates `<link rel="modulepreload">` tags for all modules
4. Updates your HTML in place

The import map lets you write clean imports (`./utils.js`) while the browser fetches hashed URLs (`./utils.a1b2c3d4.js`). Modulepreload starts fetching every scanned module before its importers are parsed.

## Usage

Add markers to your HTML:

```html
<head>
    <!-- IMPORTMAP -->
    <link rel="stylesheet" href="/styles/main.css">
    <!-- /IMPORTMAP -->
</head>
```

Run the CLI:

```sh
importmap path/to/site
```

Or use as a library:

```rust
use std::path::Path;
use importmap::ImportMap;

let dir = Path::new("path/to/site");
let map = ImportMap::scan(dir, "")?;
map.update_html_file(&dir.join("index.html"))?;
```

## Output

The content between the markers becomes:

```html
<head>
    <!-- IMPORTMAP -->
    <link rel="stylesheet" href="/styles/main.9f8e7d6c.css">
    <script type="importmap">
    {
      "imports": {
        "/scripts/main.js": "/scripts/main.a1b2c3d4.js",
        "/scripts/utils.js": "/scripts/utils.e5f6a7b8.js"
      }
    }
    </script>
    <link rel="modulepreload" href="/scripts/main.a1b2c3d4.js">
    <link rel="modulepreload" href="/scripts/utils.e5f6a7b8.js">
    <!-- /IMPORTMAP -->
</head>
```

Stylesheets are kept only if their `<link rel="stylesheet">` is already between the markers, in the same order (cascade), with all their attributes (`media`, `crossorigin`...): only the href of a scanned file changes, to its hashed URL; external and unscanned stylesheets stay as written. Tags spanning lines are rewritten on one line. Everything else between the markers (the import map, module preloads, other links) is regenerated; comments are dropped, so a commented-out stylesheet stays out. Running the transform again is idempotent: already hashed hrefs are matched to their files and get the current hash.

## Server Setup

Your server must handle hashed URLs by stripping the hash and serving the original file:

```
Request:  /scripts/main.a1b2c3d4.js
Serve:    /scripts/main.js
```

The hash is always 8 hex characters before the extension. Mark a hashed URL immutable (`Cache-Control: public, max-age=31536000, immutable`) only when the requested hash matches the content you serve (`ImportMap::names_hash(path, &ImportMap::hash(&contents))`). After a deploy changes `main.js`, a request for its previous hashed URL still strips to `main.js` and gets the new bytes: answer it with a revalidating policy (an ETag and `no-cache` or a short `max-age`), or a year-long cache would keep the new content under the old hash.

## Details

- Hash: 8 hex chars from [rapidhash](https://crates.io/crates/rapidhash) of file contents
- Extensions: `.js`, `.mjs`, `.css`
- Skips: root-level `.js` files (service workers), names containing `.development.`, `.dev.` or `.test.`, underscore-prefixed names (partials), and paths through a `tests` directory
- Symlinks: followed (useful for `node_modules` links)
- Indentation: preserved from the opening marker

## License

MIT
