# quickmatch

Fuzzy string matching for Rust and JavaScript, for autocomplete, command palettes and
search-as-you-type.

[crates.io](https://crates.io/crates/quickmatch) · [Rust API](https://docs.rs/quickmatch) · [npm](https://www.npmjs.com/package/quickmatch-js)

## Install

```bash
# rust
cargo add quickmatch

# js
npm install quickmatch-js
```

## Usage

Corpus items must be lowercase ASCII.
This is a precondition, only checked in Rust debug builds: normalize or validate
dynamic input before building a matcher. Query text is
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

qm.matches("file name");  // ["file_name"]
qm.matches("filename");   // ["file_name"]  (compound match)
qm.matches("filenme");    // ["file_name", "file_size"]  (trigram fuzzy)

const config = new QuickMatchConfig()
  .withLimit(5)
  .withTrigramBudget(10)
  .withSeparators("_- ");
const qm2 = new QuickMatch(items, config);
```

## How it works

Query words match in any order through indexed word lists, including prefixes and joined
adjacent words (`hashrate` matches `hash_rate`). Unknown words first try bounded adjacent-letter
corrections against whole indexed words, then trigram matching. If typo matching yields nothing,
results come from the known words' intersection, or their union with the union fallback.

Results rank by matched-word count and fuzzy score (descending), then first match position, item
length, text and original index (ascending).

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
