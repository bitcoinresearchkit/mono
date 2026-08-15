# importmap

Generate import maps with content-hashed URLs. No bundler required.

## Problem

ES modules load sequentially. Browser fetches `main.js`, parses it, discovers imports, fetches those, parses them, discovers more imports... This creates a waterfall that gets worse with deeper dependency trees.

Cache invalidation is also tricky. Change one file and users might get stale cached versions of files that import it.

## Solution

This crate:

1. Scans your JS/MJS/CSS files and hashes their contents
2. Generates an [import map](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script/type/importmap) that maps clean URLs to hashed URLs
3. Generates `<link rel="modulepreload">` tags for all modules
4. Updates your HTML in place

The import map lets you write clean imports (`./utils.js`) while the browser fetches hashed URLs (`./utils.a1b2c3d4.js`). Modulepreload tells the browser to fetch everything in parallel, eliminating the waterfall.

## Usage

Add markers to your HTML:

```html
<head>
    <!-- IMPORTMAP -->
    <!-- /IMPORTMAP -->
</head>
```

Run the CLI:

```sh
importmap path/to/site
```

Or use as a library:

```rust
use importmap::ImportMap;

let map = ImportMap::scan(dir, "")?;
let html = fs::read_to_string("index.html")?;
if let Some(updated) = map.update_html(&html) {
    fs::write("index.html", updated)?;
}
```

## Output

The markers get replaced with:

```html
<head>
    <!-- IMPORTMAP -->
    <link rel="modulepreload" href="/scripts/main.a1b2c3d4.js">
    <link rel="modulepreload" href="/scripts/utils.e5f6g7h8.js">
    <script type="importmap">
    {
      "imports": {
        "/scripts/main.js": "/scripts/main.a1b2c3d4.js",
        "/scripts/utils.js": "/scripts/utils.e5f6g7h8.js"
      }
    }
    </script>
    <!-- /IMPORTMAP -->
</head>
```

Your source files stay unchanged. The browser:
1. Sees modulepreload → fetches all modules in parallel
2. Sees `import "./utils.js"` → checks import map → requests `./utils.a1b2c3d4.js`
3. Gets a cache hit (already preloaded)

## Server Setup

Your server must handle hashed URLs by stripping the hash and serving the original file:

```
Request:  /scripts/main.a1b2c3d4.js
Serve:    /scripts/main.js
```

The hash is always 8 hex characters before the extension. Set `Cache-Control: immutable, max-age=31536000` for hashed URLs—the content will never change for that hash.

## Details

- Hash: 8 hex chars from [rapidhash](https://crates.io/crates/rapidhash) of file contents
- Extensions: `.js`, `.mjs`, `.css`
- Skips: root-level `.js` files (service workers), files with `.development.` or `.dev.` in the name
- Symlinks: followed (useful for `node_modules` links)
- Indentation: preserved from the opening marker

## License

MIT
