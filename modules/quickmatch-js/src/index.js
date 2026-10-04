const DEFAULT_SEPARATORS = "_- :/";
const DEFAULT_TRIGRAM_BUDGET = 6;
const DEFAULT_LIMIT = 100;
const DEFAULT_MIN_SCORE = 2;

/** Item IDs, ascending. @typedef {number[] | Uint32Array} Ids */

/**
 * Search configuration.
 *
 * Defaults work well for most use cases.
 * Tweak `trigramBudget` to trade speed for typo tolerance.
 */
export class QuickMatchConfig {
  /** Characters that separate words in items (e.g. "hash_rate" → ["hash", "rate"]).
   * @type {string} */
  separators = DEFAULT_SEPARATORS;

  /** Max results returned per query.
   * @type {number} */
  limit = DEFAULT_LIMIT;

  /** How hard to try matching typos (0 = off, 3–6 = fast, 9–15 = thorough, max 20).
   * @type {number} */
  trigramBudget = DEFAULT_TRIGRAM_BUDGET;

  /** Min overlap required for a typo match. Higher = fewer false positives.
   * @type {number} */
  minScore = DEFAULT_MIN_SCORE;

  /** Whether disjoint known words fall back to their union. */
  unionFallback = true;

  /** @param {boolean} enabled */
  withUnionFallback(enabled) {
    this.unionFallback = enabled;
    return this;
  }

  /** @param {number} n - Max results (default: 100, min: 1) */
  withLimit(n) {
    this.limit = Math.max(1, n);
    return this;
  }

  /** @param {number} n - Trigram budget (0-20, default: 6) */
  withTrigramBudget(n) {
    this.trigramBudget = Math.max(0, Math.min(20, n));
    return this;
  }

  /** @param {string} s - Separator characters (default: '_- :/') */
  withSeparators(s) {
    this.separators = s;
    return this;
  }

  /** @param {number} n - Min trigram score (default: 2, min: 1) */
  withMinScore(n) {
    this.minScore = Math.max(1, n);
    return this;
  }
}

/**
 * Instant search over a list of strings.
 *
 * Supports exact words, prefixes ("dom" → "dominance"), joined words
 * ("hashrate" → "hash_rate"), and typo tolerance ("suply" → "supply").
 * Results are ranked: exact matches first, then by specificity.
 */
export class QuickMatch {
  /** @param {string[]} items - Searchable items (lowercase) @param {QuickMatchConfig} [config] */
  constructor(items, config = new QuickMatchConfig()) {
    this.config = Object.assign(new QuickMatchConfig(), config);
    this.items = items = items.slice();
    this._indexSeparators = config.separators;
    this._sepLookup = sepLookup(config.separators);
    this._scores = new Uint32Array(items.length);
    /** @type {number[]} */
    this._dirty = [];
    // Merging lists marks each item once per merge.
    this._seen = new Uint32Array(items.length);
    this._stamp = 0;
    /** @type {Map<string, Uint32Array | null>} */
    this._prefixCache = new Map();
    /** @type {Map<string, Uint32Array | null>} */
    this._trigramCache = new Map();

    let maxWordLen = 0;
    let maxQueryLen = 0;
    let maxWords = 0;
    const sep = this._sepLookup;
    // The items of each distinct word, and of each distinct joined pair of
    // adjacent words (keyed by the first word's length, then the text), in
    // ascending order. Prefixes and trigrams resolve against these at query
    // time, matching the Rust crate's per-prefix and per-trigram lists
    // exactly while keeping memory to one entry per word occurrence. Each
    // item keeps its words too, as numbers, for ranking to read.
    /** @type {Map<string, number>} */
    const wordIds = new Map();
    /** @type {string[]} */
    const wordTexts = [];
    /** @type {number[][]} */
    const wordLists = [];
    /** @type {number[]} */
    const itemWords = [];
    this._itemOffsets = new Uint32Array(items.length + 1);
    /** @type {Map<string, number[]>} */
    const compoundItems = new Map();

    for (let idx = 0; idx < items.length; idx++) {
      const item = items[idx];
      if (item.length > maxQueryLen) maxQueryLen = item.length;

      /** @type {string[]} */
      const words = [];
      let start = 0;
      for (let i = 0; i <= item.length; i++) {
        if (i < item.length && !sep[item.charCodeAt(i)]) continue;
        if (i > start) words.push(item.slice(start, i));
        start = i + 1;
      }

      for (let i = 0; i < words.length; i++) {
        const word = words[i];
        if (word.length > maxWordLen) maxWordLen = word.length;
        let id = wordIds.get(word);
        if (id === undefined) {
          wordIds.set(word, (id = wordTexts.length));
          wordTexts.push(word);
          wordLists.push([idx]);
        } else if (wordLists[id][wordLists[id].length - 1] !== idx) {
          wordLists[id].push(idx);
        }
        itemWords.push(id);
        if (i === 0) continue;
        const previous = words[i - 1];
        // A joined-word query ("hashrate") can be longer than any single
        // word. Capping at the longest index key keeps the DDoS guard
        // data-bounded while still letting it match.
        if (previous.length + word.length > maxWordLen) maxWordLen = previous.length + word.length;
        addToIndex(compoundItems, `${previous.length}|${previous}${word}`, idx);
      }
      this._itemOffsets[idx + 1] = itemWords.length;

      if (words.length > maxWords) maxWords = words.length;
    }

    // Distinct words, sorted, so the words sharing a prefix sit together; items' words renumbered to match.
    const order = wordTexts.map((_, id) => id).sort((a, b) => (wordTexts[a] < wordTexts[b] ? -1 : wordTexts[a] > wordTexts[b] ? 1 : 0));
    const renumbered = new Uint32Array(order.length);
    order.forEach((id, sorted) => (renumbered[id] = sorted));
    this._words = order.map(id => wordTexts[id]);
    this._wordItems = order.map(id => Uint32Array.from(wordLists[id]));
    this._itemWords = new (order.length <= 0x10000 ? Uint16Array : Uint32Array)(itemWords.length);
    for (let k = 0; k < itemWords.length; k++) this._itemWords[k] = renumbered[itemWords[k]];
    const compounds = [...compoundItems.keys()]
      .map(key => {
        const bar = key.indexOf("|");
        return /** @type {[string, number, string]} */ ([key.slice(bar + 1), Number(key.slice(0, bar)) + 1, key]);
      })
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    /** Distinct joined pairs, sorted; each matches keys at least `from` long. */
    this._compounds = compounds.map(([text]) => text);
    this._compoundFrom = Uint32Array.from(compounds, ([, from]) => from);
    this._compoundItems = compounds.map(([, , key]) => Uint32Array.from(/** @type {number[]} */ (compoundItems.get(key))));
    /** @type {Map<string, number[]>} */
    const trigramWords = new Map();
    this._words.forEach((word, w) => {
      for (let k = 0; k <= word.length - 3; k++) {
        addToIndex(trigramWords, word[k] + word[k + 1] + word[k + 2], w);
      }
    });
    /** The words holding each trigram. @type {Map<string, Uint32Array>} */
    this._trigramWords = new Map([...trigramWords].map(([tri, words]) => [tri, Uint32Array.from(words)]));

    this.maxWordLen = maxWordLen + 4;
    this.maxQueryLen = maxQueryLen + 6;
    this.maxWords = maxWords + 2;
  }

  /**
   * The items with a word, or a joined pair of adjacent words, starting with
   * `key`: ascending and distinct, or null when there are none.
   * @private @param {string} key @returns {Uint32Array | null}
   */
  _postings(key) {
    const cached = this._prefixCache.get(key);
    if (cached !== undefined) return cached;
    /** @type {Uint32Array[]} */
    const lists = [];
    const words = this._words;
    for (let i = lowerBound(words, key); i < words.length && words[i].startsWith(key); i++) {
      lists.push(this._wordItems[i]);
    }
    const compounds = this._compounds;
    for (let i = lowerBound(compounds, key); i < compounds.length && compounds[i].startsWith(key); i++) {
      if (key.length >= this._compoundFrom[i]) lists.push(this._compoundItems[i]);
    }
    return remember(this._prefixCache, key, this._merge(lists));
  }

  /**
   * The items with a word holding the trigram `tri`: ascending and distinct,
   * or null when there are none.
   * @private @param {string} tri @returns {Uint32Array | null}
   */
  _trigramPostings(tri) {
    const cached = this._trigramCache.get(tri);
    if (cached !== undefined) return cached;
    const words = this._trigramWords.get(tri);
    return remember(this._trigramCache, tri, words ? this._merge(Array.from(words, w => this._wordItems[w])) : null);
  }

  /**
   * `wordMatch` for item `idx`, read from its indexed words; a query that
   * splits words differently from the index reads the item itself.
   * @private @param {number} idx @param {string[]} qwords @param {Uint8Array} sep @param {boolean} exact
   * @returns {[number, number]}
   */
  _wordMatch(idx, qwords, sep, exact) {
    if (sep !== this._sepLookup) return wordMatch(this.items[idx], qwords, sep, exact);
    const words = this._words, ids = this._itemWords;
    const from = this._itemOffsets[idx], to = this._itemOffsets[idx + 1];
    let matched = 0;
    let position = to - from;
    for (const qw of qwords) {
      for (let k = from; k < to; k++) {
        const word = words[ids[k]];
        let joined = false;
        if (k > from) {
          const previous = words[ids[k - 1]], rest = qw.length - previous.length;
          joined = rest > 0 && qw.startsWith(previous)
            && (exact ? word.length === rest : word.length >= rest) && word.startsWith(qw.slice(previous.length));
        }
        if ((exact ? word === qw : word.startsWith(qw)) || joined) {
          matched++;
          position = Math.min(position, joined ? k - from - 1 : k - from);
          break;
        }
      }
    }
    return [matched, position];
  }

  /**
   * The distinct items of ascending lists, ascending; null for no lists.
   * @private @param {Uint32Array[]} lists @returns {Uint32Array | null}
   */
  _merge(lists) {
    if (lists.length <= 1) return lists[0] ?? null;
    if (this._stamp === 0xffffffff) {
      this._seen.fill(0);
      this._stamp = 0;
    }
    const stamp = ++this._stamp;
    const seen = this._seen;
    /** @type {number[]} */
    const merged = [];
    for (const list of lists) {
      for (let j = 0; j < list.length; j++) {
        const id = list[j];
        if (seen[id] !== stamp) {
          seen[id] = stamp;
          merged.push(id);
        }
      }
    }
    return Uint32Array.from(merged).sort();
  }

  /** @param {string} query */
  matches(query) {
    return this.matchesWith(query, this.config);
  }

  /**
   * @param {string} query
   * @param {QuickMatchConfig} config
   */
  matchesWith(query, config) {
    return this.matchesWithMatchedWords(query, config).map(([item]) => item);
  }

  /** @param {string} query @param {QuickMatchConfig} [config]
   * @returns {[string, number][]} */
  matchesWithMatchedWords(query, config = this.config) {
    return this.matchesWithIdsAndMatchedWords(query, config)
      .map(([id, matched]) => [this.items[id], matched]);
  }

  /** Original zero-based item IDs and matched query-word counts.
   * @param {string} query @param {QuickMatchConfig} [config]
   * @returns {[number, number][]} */
  matchesWithIdsAndMatchedWords(query, config = this.config) {
    return this._matches(query, config, false);
  }

  /** Only the highest matched-query-word tier.
   * @param {string} query @param {QuickMatchConfig} [config]
   * @returns {[number, number][]} */
  matchesBestWithIdsAndMatchedWords(query, config = this.config) {
    return this._matches(query, config, true);
  }

  /** Whole query words without prefixes or typos. Disable unionFallback to require every word.
   * @param {string} query @param {QuickMatchConfig} [config]
   * @returns {[number, number][]} */
  matchesExactWithIdsAndMatchedWords(query, config = this.config) {
    return this._matches(query, config, false, true);
  }

  /**
   * @private
   * @param {string} query
   * @param {QuickMatchConfig} config
   * @param {boolean} bestOnly
   * @param {boolean} [exactWords]
   * @returns {[number, number][]}
   */
  _matches(query, config, bestOnly, exactWords = false) {
    const { limit, trigramBudget } = config;
    const sep =
      config.separators === this._indexSeparators
        ? this._sepLookup
        : sepLookup(config.separators);

    const q = normalize(query);
    if (!q || (!exactWords && q.length > this.maxQueryLen)) return [];

    const qwords = splitWords(q, sep, exactWords ? Infinity : this.maxWordLen);
    if (!qwords.length || (!exactWords && qwords.length > this.maxWords)) return [];

    const known = [];
    const unknown = [];

    for (const w of qwords) {
      const hits = this._postings(w);
      if (hits) {
        known.push(hits);
      } else if (w.length >= 3 && unknown.length < trigramBudget) {
        unknown.push(w);
      }
    }

    if (exactWords) {
      if (!config.unionFallback && known.length !== qwords.length) return [];
      const candidates = (config.unionFallback ? union(known) : intersect(known) || []).filter(id => {
        const [matched] = this._wordMatch(id, qwords, sep, true);
        return matched > 0 && (config.unionFallback || matched === qwords.length);
      });
      return this._rank(candidates, null, qwords, sep, limit, bestOnly, true);
    }

    const pool = intersect(known);

    // A swapped pair often shares no trigrams ("prcie" → "price").
    // Probe indexed corrections before the broader trigram fallback.
    if (unknown.length && trigramBudget) {
      /** @type {Ids[][]} */
      const corrections = unknown.map(() => []);
      let budget = trigramBudget;
      for (let round = 0; round < trigramBudget && budget > 0; round++) {
        for (let i = 0; i < unknown.length && budget > 0; i++) {
          const word = unknown[i];
          if (round >= word.length - 1) continue;
          const at = round % 2 === 0 ? round / 2 : word.length - 2 - Math.floor(round / 2);
          if (word[at] === word[at + 1]) continue;
          budget--;
          const corrected = word.slice(0, at) + word[at + 1] + word[at] + word.slice(at + 2);
          const hits = this._postings(corrected);
          if (hits) {
            const exact = hits.filter(id => this._wordMatch(id, [corrected], sep, true)[0]);
            if (exact.length) corrections[i].push(exact);
          }
        }
      }
      if (corrections.every(lists => lists.length)) {
        const candidates = intersect([...known, ...corrections.map(lists => union(lists).sort((a, b) => a - b))]);
        if (candidates?.length) return this._rank(candidates, null, qwords, sep, limit, bestOnly);
      }
    }

    // Try typo matching for unknown words
    if (unknown.length && trigramBudget) {
      const { _scores: scores, _dirty: dirty } = this;

      if (pool) {
        for (const i of pool) {
          scores[i] = 1;
          dirty.push(i);
        }
      }

      const hitCount = this._scoreTrigrams(
        unknown,
        trigramBudget,
        pool !== null,
        Math.max(0, q.length - 3),
      );
      const minScore = Math.max(config.minScore, Math.ceil(hitCount / 2));
      const result = this._rank(dirty, minScore, qwords, sep, limit, bestOnly);

      for (const i of dirty) scores[i] = 0;
      dirty.length = 0;

      if (result.length > 0) return result;
    }

    // Rank known candidates (intersection, or union as fallback)
    const candidates = pool || (config.unionFallback ? union(known) : []);
    return candidates.length > 0
      ? this._rank(candidates, null, qwords, sep, limit, bestOnly)
      : [];
  }

  /** @private @param {string[]} unknown @param {number} budget @param {boolean} poolOnly @param {number} minLen */
  _scoreTrigrams(unknown, budget, poolOnly, minLen) {
    const { _scores: scores, _dirty: dirty, items } = this;
    const visited = new Set();
    const maxRounds = budget;
    let hits = 0;

    outer: for (let round = 0; round < maxRounds; round++) {
      for (const word of unknown) {
        if (budget <= 0) break outer;

        const pos = trigramPosition(word.length, round);
        if (pos < 0) continue;

        const tri = word[pos] + word[pos + 1] + word[pos + 2];
        if (visited.has(tri)) continue;
        visited.add(tri);
        budget--;

        const matched = this._trigramPostings(tri);
        if (!matched) continue;
        hits++;

        if (poolOnly) {
          for (let j = 0; j < matched.length; j++) {
            const i = matched[j];
            if (scores[i] > 0) scores[i]++;
          }
        } else {
          for (let j = 0; j < matched.length; j++) {
            const i = matched[j];
            if (items[i].length >= minLen) {
              if (scores[i] === 0) dirty.push(i);
              scores[i]++;
            }
          }
        }
      }
    }

    return hits;
  }

  /**
   * @private
   * @param {Ids} indices
   * @param {number|null} minScore
   * @param {string[]} qwords
   * @param {Uint8Array} sep
   * @param {number} limit
   * @param {boolean} bestOnly
   * @param {boolean} [exactWords]
   * @returns {[number, number][]}
   */
  _rank(indices, minScore, qwords, sep, limit, bestOnly, exactWords = false) {
    const { items, _scores: scores } = this;
    /** @type {[number, number][][]} */
    const buckets = Array.from({ length: qwords.length + 1 }, () => []);

    for (let i = 0; i < indices.length; i++) {
      const idx = indices[i];
      if (minScore !== null && scores[idx] < minScore) continue;
      const [matched, position] = this._wordMatch(idx, qwords, sep, exactWords);
      buckets[matched].push([idx, position]);
    }

    /** @type {[number, number][]} */
    const results = [];
    for (let ps = buckets.length - 1; ps >= 0 && results.length < limit; ps--) {
      const bucket = buckets[ps];
      if (!bucket.length) continue;
      bucket.sort(
        ([a, pa], [b, pb]) =>
          scores[b] - scores[a] ||
          pa - pb ||
          items[a].length - items[b].length ||
          (items[a] < items[b] ? -1 : items[a] > items[b] ? 1 : a - b), // item text, asc (total order)
      );
      const take = Math.min(bucket.length, limit - results.length);
      for (let i = 0; i < take; i++) results.push([bucket[i][0], ps]);
      if (bestOnly) break;
    }

    return results;
  }
}

// --- Helpers ---

/** @param {string} query */
function normalize(query) {
  // Rust str::trim uses Unicode White_Space (unlike JS trim, notably BOM/NEL).
  query = query.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, "");
  let out = "";
  let start = 0;
  let end = query.length;
  for (let i = start; i < end; i++) {
    const c = query.charCodeAt(i);
    if (c >= 128) continue;
    out += c >= 65 && c <= 90 ? String.fromCharCode(c + 32) : query[i];
  }
  return out;
}

/** @param {string} separators */
function sepLookup(separators) {
  const t = new Uint8Array(128);
  for (let i = 0; i < separators.length; i++) {
    const c = separators.charCodeAt(i);
    if (c < 128) t[c] = 1;
  }
  return t;
}

/**
 * @param {string} text
 * @param {Uint8Array} sep
 * @param {number} maxLen
 */
function splitWords(text, sep, maxLen) {
  /** @type {string[]} */
  const words = [];
  let start = 0;
  for (let i = 0; i <= text.length; i++) {
    if (i < text.length && !sep[text.charCodeAt(i)]) continue;
    if (i > start) {
      const w = text.slice(start, i);
      if (w.length <= maxLen && !words.includes(w)) words.push(w);
    }
    start = i + 1;
  }
  return words;
}

/**
 * @param {Map<string, number[]>} index
 * @param {string} key
 * @param {number} value
 */
function addToIndex(index, key, value) {
  const arr = index.get(key);
  if (arr) {
    if (arr[arr.length - 1] !== value) arr.push(value);
  } else {
    index.set(key, [value]);
  }
}

/** The first position in sorted `list` whose value is not below `key`. @param {string[]} list @param {string} key */
function lowerBound(list, key) {
  let lo = 0,
    hi = list.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (list[mid] < key) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/**
 * Keeps a resolved list for the next lookups of the same key (a query looks
 * its words up more than once); a bounded few, so memory stays flat.
 * @template T @param {Map<string, T>} cache @param {string} key @param {T} value
 */
function remember(cache, key, value) {
  if (cache.size >= 64) cache.clear();
  cache.set(key, value);
  return value;
}

/** @param {Ids[]} arrays @returns {Ids} */
function union(arrays) {
  if (arrays.length <= 1) return arrays[0] || [];
  const seen = new Set();
  const result = [];
  for (const arr of arrays) {
    for (const idx of arr) {
      if (!seen.has(idx)) {
        seen.add(idx);
        result.push(idx);
      }
    }
  }
  return result;
}

/** @param {Ids[]} arrays @returns {Ids|null} */
function intersect(arrays) {
  if (arrays.length <= 1) return arrays[0] || null;

  let si = 0;
  for (let i = 1; i < arrays.length; i++) {
    if (arrays[i].length < arrays[si].length) si = i;
  }

  const result = Array.from(arrays[si]);
  for (let i = 0; i < arrays.length; i++) {
    if (i === si) continue;
    let w = 0;
    for (let j = 0; j < result.length; j++) {
      if (bsearch(arrays[i], result[j])) result[w++] = result[j];
    }
    result.length = w;
    if (!w) return null;
  }
  return result;
}

/**
 * @param {Ids} arr
 * @param {number} val
 */
function bsearch(arr, val) {
  let lo = 0,
    hi = arr.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (arr[mid] === val) return true;
    if (arr[mid] < val) lo = mid + 1;
    else hi = mid - 1;
  }
  return false;
}

/**
 * @param {string} item @param {string[]} qwords @param {Uint8Array} sep
 * @param {boolean} [exact] Require whole words when validating typo corrections.
 * @returns {[number, number]} Query words matched in any order (including
 *   joined adjacent words), and the first matching item-word position.
 */
function wordMatch(item, qwords, sep, exact = false) {
  const words = [];
  let start = 0;
  for (let i = 0; i <= item.length; i++) {
    if (i < item.length && !sep[item.charCodeAt(i)]) continue;
    if (i > start) words.push(item.slice(start, i));
    start = i + 1;
  }
  let matched = 0;
  let position = words.length;
  for (const qw of qwords) {
    for (let i = 0; i < words.length; i++) {
      const previous = words[i - 1];
      const joined = previous !== undefined && qw.length > previous.length
        && qw.startsWith(previous) && (exact ? words[i] === qw.slice(previous.length) : words[i].startsWith(qw.slice(previous.length)));
      if ((exact ? words[i] === qw : words[i].startsWith(qw)) || joined) {
        matched++;
        position = Math.min(position, joined ? i - 1 : i);
        break;
      }
    }
  }
  return [matched, position];
}

/** @param {number} len @param {number} round */
function trigramPosition(len, round) {
  const max = len - 3;
  if (max < 0) return -1;
  if (round === 0) return 0;
  if (round === 1 && max > 0) return max;
  if (round === 2 && max > 1) return max >> 1;
  if (max <= 2) return -1;

  const mid = max >> 1;
  const off = (round - 2) >> 1;
  const pos = round & 1 ? Math.max(0, mid - off) : mid + off;
  return pos === 0 || pos >= max || pos === mid ? -1 : pos;
}
