# Changelog

## [v0.4.0](https://github.com/nym21/quickmatch/releases/tag/v0.4.0) - 2026-03-22

### Breaking Changes

#### Rust & JavaScript
- Word indexing now stores all prefixes of each word and compound, increasing memory usage proportionally to total word character count — previously only full words and full compounds were indexed ([Rust source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/index.js))

### New Features

#### Rust & JavaScript
- Added prefix indexing: every prefix of each word is now stored in the word index (e.g., "dominance" registers "d", "do", "dom", ..., "dominance"), so short queries like "dom" directly match items containing "dominance" without relying on trigram fallback ([Rust source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/index.js))
- Added prefix indexing for compound words: adjacent word pairs index all prefixes starting from the second word's first character (e.g., "hash" + "rate" indexes "hashr", "hashra", ..., "hashrate"), enabling partial compound matching
- Changed `prefix_score` from exact word matching to prefix matching (`starts_with` in Rust, length comparison in JS), so query word "pric" is recognized as a prefix of item word "price" and scored accordingly ([Rust source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/index.js))
- Added union fallback: when word intersection yields no common items and trigram matching produces no results, the algorithm falls back to union of all known word sets, returning partial matches instead of empty results ([Rust source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/index.js))
- Changed search flow to try trigram matching first when unknown words exist; if trigram results are found they're returned immediately, otherwise falls back to ranking known-word candidates (intersection or union)

#### Examples
- Added `examples/compare.rs` with 80+ test queries covering exact matches, compound words, typos, short prefixes, acronyms, reverse word order, and unknown terms against a metrics dataset ([source](https://github.com/nym21/quickmatch/blob/v0.4.0/examples/compare.rs))

### Internal Changes

#### Rust
- Extracted ranking logic into a standalone `fn rank()` method that takes an iterator of `(*const str, usize)` candidates and produces bucket-sorted results ([source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/lib.rs))
- Added `intersect_sets()` method that returns `Option<FxHashSet>` (returning `None` instead of empty set to distinguish "no known words" from "empty intersection")
- Added `union_sets()` method that merges multiple `FxHashSet` references into a single set
- Changed query word deduplication from a separate `FxHashSet` to inline `seen.insert()` filtering during collection, then dropping the set immediately
- Used lifetime elision (`QuickMatch<'_>`) in `Send` and `Sync` trait implementations
- Removed doc comments from `prefix_score` free function

#### JavaScript
- Replaced flat sorting by `(prefixScore, trigramScore, length)` with a 3-bucket ranking approach: items are bucketed by prefix score (0, 1, 2), each bucket is independently sorted by `(score, length)`, and results are filled from the highest bucket first until the limit is reached ([source](https://github.com/nym21/quickmatch/blob/v0.4.0/src/index.js))
- Added `union()` helper function that merges multiple index arrays into a deduplicated result using a `Set`
- Changed `intersect()` to return `null` instead of an empty array when intersection is empty, distinguishing "no results" from "no input"
- Added duplicate prevention in `addToIndex()`: checks `arr[arr.length - 1] !== value` before pushing, preventing duplicate entries from prefix indexing
- Removed standalone `indexTrigrams()` function; trigram indexing is now inlined in the constructor alongside prefix indexing
- Condensed JSDoc class-level documentation to include behavioral descriptions (e.g., "Supports exact words, prefixes, joined words, and typo tolerance")

[View changes](https://github.com/nym21/quickmatch/compare/v0.3.2...v0.4.0)

## [v0.3.2](https://github.com/nym21/quickmatch/releases/tag/v0.3.2) - 2026-03-21

### Breaking Changes

#### Rust & JavaScript
- Default separators changed from `['_', '-', ' ']` to `['_', '-', ' ', ':', '/']`, meaning colon-separated and path-like strings are now split into words by default ([Rust source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/config.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))

### New Features

#### Rust & JavaScript
- Added compound word indexing: adjacent words in items are indexed as concatenated pairs (e.g., "hash" + "rate" → "hashrate"), so queries like `hashrate` now match items like `hash_rate` without relying on trigram fallback ([Rust source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))
- Added prefix-based result ranking via a new `prefix_score` function that scores items as exact match (2), prefix match (1), or no match (0) — results are now sorted by prefix score first, then trigram score, then length ([Rust source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/lib.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))
- Added `with_min_score(n)` (Rust) / `withMinScore(n)` (JS) configuration option to set the minimum trigram score required for fuzzy matches — higher values require more trigram overlap, reducing noise in results (default: 2) ([Rust source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/config.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))

#### Documentation
- Added `docs/README.md` with usage examples for both Rust and JavaScript, a "How it works" section explaining the 3-stage matching pipeline (word → compound → trigram), a configuration options table, and performance benchmarks (~26μs/query Rust, ~29μs/query JS) ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/docs/README.md))

#### Examples
- Added `examples/colon_test.rs` and `examples/colon_test.mjs` demonstrating separator behavior with colon-containing items and queries under both default and custom separator configurations ([Rust source](https://github.com/nym21/quickmatch/blob/v0.3.2/examples/colon_test.rs), [JS source](https://github.com/nym21/quickmatch/blob/v0.3.2/examples/colon_test.mjs))

#### CI
- Added `.github/workflows/outdated.yml` workflow that runs daily (and on manual trigger) to detect outdated Rust dependencies using `cargo-outdated` ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/.github/workflows/outdated.yml))

### Bug Fixes

#### Rust
- Fixed `max_word_len` calculation that was incorrectly using the full item length instead of individual word length, which could cause valid long words to be rejected from queries ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/lib.rs))
- Fixed query normalization ordering so that emptiness and length checks now happen after trimming whitespace and lowercasing, correctly rejecting whitespace-only or non-ASCII-only queries ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/lib.rs))

### Internal Changes

#### Rust
- Changed `query_words` collection from `FxHashSet` to `Vec` to preserve word ordering, which is required for the new prefix scoring algorithm ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/lib.rs))
- Removed doc comments from `matches()` and `matches_with()` public methods

#### JavaScript
- Replaced string-based separator checking (`separators.includes()`) with a `Uint8Array(128)` lookup table for O(1) separator detection instead of O(n) string scanning ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))
- Replaced `Map<number, number>` score tracking with a pre-allocated `Uint32Array(items.length)` and a dirty index list, eliminating Map allocation overhead during each search ([source](https://github.com/nym21/quickmatch/blob/v0.3.2/src/index.js))
- Merged `sortedByLength()` and `rankedResults()` into a single `_rank()` method that handles both exact-match and fuzzy-match result paths
- Renamed internal functions for brevity: `normalizeQuery` → `normalize`, `parseWords` → `splitWords`, `addTrigramsToIndex` → `indexTrigrams`, `intersectAll` → `intersect`, `binarySearch` → `bsearch`, `pickTrigramPosition` → `trigramPosition`, `scoreByTrigrams` → `_scoreTrigrams`
- Condensed JSDoc annotations to single-line `@param` format

#### Workspace
- Reorganized project files: moved `LICENSE` to `docs/LICENSE.md`, set `Cargo.toml` readme field to `docs/README.md`
- Added `*metrics*` pattern to `.gitignore`
- Added `examples` to `jsconfig.json` exclude list

[View changes](https://github.com/nym21/quickmatch/compare/v0.3.1...v0.3.2)

## [v0.3.1](https://github.com/nym21/quickmatch/releases/tag/v0.3.1) - 2026-01-22

### New Features

#### JavaScript
- Added JSDoc type annotations for `QuickMatchConfig` class properties (`separators`, `limit`, `trigramBudget`) and all public methods (`withLimit()`, `withTrigramBudget()`, `withSeparators()`, `matches()`, `matchesWith()`) ([source](https://github.com/nym21/quickmatch/blob/v0.3.1/src/index.js))
- Added `@private` annotations for internal methods (`scoreByTrigrams()`, `sortedByLength()`, `rankedResults()`) to indicate non-public API
- Added explicit type annotations to Map types: `Map<string, number[]>` for `wordIndex` and `trigramIndex`
- Added `jsconfig.json` with TypeScript-style type checking enabled (`checkJs: true`, `strict: true`) for improved IDE support and static analysis ([source](https://github.com/nym21/quickmatch/blob/v0.3.1/jsconfig.json))

### Internal Changes

#### JavaScript
- Renamed `trigramCount` variable to `hitCount` for consistency with Rust implementation
- Normalized string quotes to double quotes throughout the codebase
- Reformatted multi-line function parameters for readability

[View changes](https://github.com/nym21/quickmatch/compare/v0.3.0...v0.3.1)

## [v0.3.0](https://github.com/nym21/quickmatch/releases/tag/v0.3.0) - 2026-01-22

### New Features

#### JavaScript
- Added complete JavaScript port of the library with identical API to Rust, published as `quickmatch-js` npm package ([source](https://github.com/nym21/quickmatch/blob/v0.3.0/src/index.js))
- Implemented `QuickMatch` class with word indexing and trigram-based fuzzy matching
- Implemented `QuickMatchConfig` class with builder methods (`withLimit()`, `withTrigramBudget()`, `withSeparators()`)
- Implemented `matches()` and `matchesWith()` methods returning results sorted by relevance
- Uses ES modules (`"type": "module"`) for modern JavaScript compatibility

#### Documentation
- Added JavaScript installation instructions to README (`npm install quickmatch-js`)

### Internal Changes

#### Rust
- Renamed `trigram_count` to `hit_count` to clarify it tracks successful trigram matches rather than total attempts ([source](https://github.com/nym21/quickmatch/blob/v0.3.0/src/lib.rs))
- Changed trigram budget tracking from increment-based (`trigram_count += 1`) to decrement-based (`budget -= 1`) for clearer remaining budget visibility
- Simplified trigram processing loop by removing `processed_trigrams` flag and using direct budget check (`if budget == 0`) for early termination

[View changes](https://github.com/nym21/quickmatch/compare/v0.2.0...v0.3.0)

## [v0.2.0](https://github.com/nym21/quickmatch/releases/tag/v0.2.0) - 2026-01-14

### New Features

#### Rust
- Initial release of `quickmatch` - a fuzzy string matching library for autocomplete and search-as-you-type interfaces
- Added `QuickMatch` struct implementing hybrid matching with word-level indexing and trigram-based fuzzy fallback ([source](https://github.com/nym21/quickmatch/blob/v0.2.0/src/lib.rs))
- Added `QuickMatchConfig` for customizing match behavior with builder pattern methods ([source](https://github.com/nym21/quickmatch/blob/v0.2.0/src/config.rs)):
  - `with_limit(n)` - Set maximum results (default: 100)
  - `with_trigram_budget(n)` - Set fuzzy matching depth (default: 6, range: 0-20)
  - `with_separators(chars)` - Set word separator characters (default: `_`, `-`, ` `)
- Added `new()` constructor for default configuration and `new_with()` for custom configuration
- Added `matches()` and `matches_with()` methods returning results ranked by match quality and length
- Implemented thread-safe design with `Send` and `Sync` traits for concurrent usage
- Added zero-copy string storage using pointer-based indexing for memory efficiency
- Implemented bounded trigram selection (first, last, middle, then alternating) for fuzzy matching
- Added interactive autocomplete example demonstrating product search functionality ([source](https://github.com/nym21/quickmatch/blob/v0.2.0/examples/autocomplete.rs))

[View changes](https://github.com/nym21/quickmatch/compare/e310f20...v0.2.0)
