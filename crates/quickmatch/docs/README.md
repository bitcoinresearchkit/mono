# quickmatch

Fast fuzzy string matching for Rust and JavaScript.

Built for autocomplete, command palettes, and search-as-you-type interfaces.

[![Crates.io](https://img.shields.io/crates/v/quickmatch.svg)](https://crates.io/crates/quickmatch)
[![npm](https://img.shields.io/npm/v/quickmatch-js.svg)](https://www.npmjs.com/package/quickmatch-js)
[![Documentation](https://docs.rs/quickmatch/badge.svg)](https://docs.rs/quickmatch)

## Install

```bash
# rust
cargo add quickmatch

# js
npm install quickmatch-js
```

## Usage

Corpus items must be lowercase ASCII.
Constructors enforce these programmer preconditions with diagnostic panics;
normalize or validate dynamic input before building a matcher. Query text is
lowercased and non-ASCII query characters are ignored. This is not a Unicode
search engine.

**Rust**

```rust
use quickmatch::{QuickMatch, QuickMatchConfig};

let items = vec!["file_name", "file_size", "created_at", "updated_at"];
let qm = QuickMatch::new(&items);
let config = QuickMatchConfig::new().with_limit(5);

// Results contain (item index, matched word count); "filename" and "filenme"
// also match through compound and trigram fuzzy matching.
let results = qm.matches_with_ids_and_matched_words("file name", &config);

// Whole words only, in any order.
let exact = qm.matches_exact_with_ids_and_matched_words("name file", &config);
```

**JavaScript**

```js
import { QuickMatch, QuickMatchConfig } from "quickmatch-js";

const items = ["file_name", "file_size", "created_at", "updated_at"];
const qm = new QuickMatch(items);

qm.matches("file name");  // ["file_name", "file_size"]
qm.matches("filename");   // ["file_name", "file_size"]  (compound match)
qm.matches("filenme");    // ["file_name", "file_size"]  (trigram fuzzy)

// Custom config
const config = new QuickMatchConfig()
  .withLimit(5)
  .withTrigramBudget(10)
  .withSeparators("_- ");
const qm2 = new QuickMatch(items, config);
```

## How it works

Queries go through three matching stages:

1. **Word match** — query is split by separators and looked up in a word index
2. **Compound match** — adjacent words are indexed as compounds, so `hashrate` finds `hash_rate`
3. **Trigram fallback** — unknown words are matched via character trigrams for fuzzy/typo tolerance

Query words match in any order, including prefixes and joined adjacent words. Bounded adjacent-letter corrections run before the trigram fallback and must match whole indexed words. Results rank by matched-word count, fuzzy score, first match position, item length, text, then original index.

## Config

Rust exposes `with_limit(n)` (default 100) and `with_union_fallback(bool)` (default `true`).
The trigram budget (6), minimum fuzzy score (2) and separators (`_- :/`) are fixed.
The JS package additionally exposes `withTrigramBudget`, `withMinScore` and `withSeparators`.

## Performance

Benchmarked against ~5,000 metric names, 83 queries, averaged over 10K iterations:

| | Avg/query | Build time |
|---|-----------|------------|
| **Rust** | ~26 us | ~40 ms |
| **JS** | ~29 us | ~30 ms |

## License

MIT
