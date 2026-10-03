# QuickMatch JS

Dependency-free fuzzy search over lowercase ASCII item names, matching the Rust
`crates/quickmatch` implementation. Queries may contain Unicode: surrounding
Unicode whitespace is trimmed, non-ASCII characters are removed, and ASCII
letters are lowercased, in that order.

```js
import { QuickMatch, QuickMatchConfig } from 'quickmatch-js';

const config = new QuickMatchConfig().withLimit(50).withUnionFallback(false);
const matcher = new QuickMatch(['price', 'realized_price'], config);
matcher.matches('price'); // ['price', 'realized_price']
matcher.matchesWithMatchedWords('price'); // [['price', 1], ['realized_price', 1]]
matcher.matchesWithIdsAndMatchedWords('price'); // [[0, 1], [1, 1]]
matcher.matchesBestWithIdsAndMatchedWords('price'); // [[0, 1], [1, 1]]
```

`matchesWith(query, config)` overrides the search configuration. The three
metadata methods also accept an optional configuration. IDs are positions in
the original input array; duplicate names retain separate IDs. The constructor
copies the input array and configuration.

Query words match in any order, including prefixes and joined adjacent words.
Bounded adjacent-letter corrections run before the trigram fallback and must match whole indexed words.
Results sort by matched-word count, fuzzy score, first match position, item
length, then text. Identical items tie-break by original ID in both languages.
`matchesExactWithIdsAndMatchedWords(query, config)` matches whole words only,
with no prefixes or typos. Set `unionFallback` to `false` to require every query word.
The best-tier method returns only results with the highest matched-word count,
subject to the same limit. Union fallback defaults to `true`, as in Rust.

## Development and releases

`scripts/js-publish.sh <workspace-version>` updates both package
versions to the workspace release, then publishes missing versions of Bitview
Client and QuickMatch. Published versions are skipped; registry and
authentication errors stop the release. The versioned `0.5.0` directory is a
historical browser dependency; new consumers use `quickmatch-js/src/index.js`.

Studio imports this source directly through its `modules` symlink.
