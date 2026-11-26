const DEFAULT_SEPARATORS = "_- :/";
const DEFAULT_TRIGRAM_BUDGET = 6;
const DEFAULT_LIMIT = 100;
const DEFAULT_MIN_SCORE = 2;

/** Item IDs, ascending. @typedef {number[] | Uint32Array} Ids */

/** Tweak `trigramBudget` to trade speed for typo tolerance. */
export class QuickMatchConfig {
  /** Word separators in items. @type {string} */
  separators = DEFAULT_SEPARATORS;

  /** @type {number} */
  limit = DEFAULT_LIMIT;

  /** 0: off; max 20. @type {number} */
  trigramBudget = DEFAULT_TRIGRAM_BUDGET;

  /** Higher: fewer false positives. @type {number} */
  minScore = DEFAULT_MIN_SCORE;

  /** Whether disjoint known words fall back to their union. */
  unionFallback = true;

  /** @param {boolean} enabled */
  withUnionFallback(enabled) {
    this.unionFallback = enabled;
    return this;
  }

  /** @param {number} n */
  withLimit(n) {
    this.limit = Math.max(1, n);
    return this;
  }

  /** @param {number} n */
  withTrigramBudget(n) {
    this.trigramBudget = Math.max(0, Math.min(20, n));
    return this;
  }

  /** @param {string} s */
  withSeparators(s) {
    this.separators = s;
    return this;
  }

  /** @param {number} n */
  withMinScore(n) {
    this.minScore = Math.max(1, n);
    return this;
  }
}

/** Instant search over a list of strings: exact words, prefixes, joined words both ways ("hashrate" ↔ "hash_rate"), typos. Exact first, then by specificity. */
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
    // Postings per word and per joined adjacent pair, matching the Rust crate's lists.
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
        // A joined-word query ("hashrate") can be longer than any word: capping at the longest index key keeps the DDoS guard data-bounded.
        if (previous.length + word.length > maxWordLen) maxWordLen = previous.length + word.length;
        addToIndex(compoundItems, `${previous.length}|${previous}${word}`, idx);
      }
      this._itemOffsets[idx + 1] = itemWords.length;

      if (words.length > maxWords) maxWords = words.length;
    }

    // Sorted, so the words sharing a prefix sit together.
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
    /** Sorted; each matches keys at least `from` long. */
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
    /** @type {Map<string, Uint32Array>} */
    this._trigramWords = new Map([...trigramWords].map(([tri, words]) => [tri, Uint32Array.from(words)]));

    // A word may be typed as two ("hash rate"): a space more each.
    this.maxWordLen = maxWordLen + 4;
    this.maxQueryLen = maxQueryLen + maxWords + 6;
    this.maxWords = 2 * maxWords + 2;
  }

  /** @private @param {string} key @returns {Uint32Array | null} */
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

  /** Whole word only (a plural's word, not two joined). @private @param {string} key @returns {Uint32Array | null} */
  _wordPostings(key) {
    const at = lowerBound(this._words, key);
    return this._words[at] === key ? this._wordItems[at] : null;
  }

  /** @private @param {string} tri @returns {Uint32Array | null} */
  _trigramPostings(tri) {
    const cached = this._trigramCache.get(tri);
    if (cached !== undefined) return cached;
    const words = this._trigramWords.get(tri);
    return remember(this._trigramCache, tri, words ? this._merge(Array.from(words, w => this._wordItems[w])) : null);
  }

  /**
   * @private @param {number} idx @param {string[]} qwords @param {Joins} joins @param {Uint8Array} sep @param {boolean} exact
   * @returns {[number, number, number]}
   */
  _wordMatch(idx, qwords, joins, sep, exact) {
    if (sep !== this._sepLookup) return wordMatch(this.items[idx], qwords, sep, exact);
    const words = this._words, ids = this._itemWords;
    const from = this._itemOffsets[idx], to = this._itemOffsets[idx + 1];
    let matched = 0, whole = 0;
    let position = to - from;
    for (let j = 0; j < qwords.length; j++) {
      const qw = qwords[j], own = qw.charCodeAt(0), pairs = joins[j];
      let found = false;
      for (let k = from; k < to; k++) {
        const word = words[ids[k]], previous = k > from ? words[ids[k - 1]] : null;
        const letter = word.charCodeAt(0), letterBefore = previous === null ? -1 : previous.charCodeAt(0);
        // Either fits a word starting as it does, or the one before it: most are passed by a letter.
        let how = letter === own || letterBefore === own ? fit(qw, word, previous, exact, 0) : 0;
        for (let p = 0; p < pairs.length; p++) {
          const pair = pairs[p];
          if (letter === pair[2] || letterBefore === pair[2]) how |= fit(pair[0], word, previous, exact, pair[1]);
        }
        if (how & FITS) {
          if (!found) {
            found = true;
            matched++;
            position = Math.min(position, how & JOINED ? k - from - 1 : k - from);
          }
          // Held only as a prefix so far: a later word may hold it whole.
          if (how & WHOLE) {
            whole++;
            break;
          }
        }
      }
    }
    return [matched, whole, position];
  }

  /** @private @param {Uint32Array[]} lists @returns {Uint32Array | null} */
  _merge(lists) {
    if (lists.length <= 1) return lists[0] ?? null;
    if (lists.length === 2) {
      const [a, b] = lists, merged = new Uint32Array(a.length + b.length);
      let i = 0, j = 0, n = 0;
      while (i < a.length && j < b.length) merged[n++] = a[i] < b[j] ? a[i++] : a[i] > b[j] ? b[j++] : (j++, a[i++]);
      while (i < a.length) merged[n++] = a[i++];
      while (j < b.length) merged[n++] = b[j++];
      return n === merged.length ? merged : merged.slice(0, n);
    }
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

  /**
   * @param {string} query @param {QuickMatchConfig} [config]
   * @returns {[number, number][]}
   */
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
    if (!qwords.length || qwords.length > this.maxWords) return [];
    if (!exactWords && !config.unionFallback && splitWords(q, sep, Infinity).length !== qwords.length) return [];

    const known = [];
    const unknown = [];
    // Unindexed words of 3+ letters must each be a typo of a candidate's word: trigrams probe only some.
    const unmatched = [];

    // `slip`: held only by a word one letter shorter ("pice" past "pic"), not as a word of its own.
    // A pair the index holds nothing under fits nothing, so none when split otherwise than the index.
    const joins = joinsOf(splitWords(q, sep, Infinity, true), qwords, (pair, from) => sep === this._sepLookup && (this._postings(pair) !== null || (!exactWords && pair.length >= 4 && pair.length - 1 > from && this._wordPostings(pair.slice(0, -1)) !== null)));
    const held = qwords.map((w, i) => {
      /** @type {Uint32Array[]} */
      const lists = [];
      let firm = false;
      // `from`: a plural of two joined takes more than the first ("hash rates", not "tatmaspool h").
      /** @type {[string, number][]} */
      const keys = [[w, 0]];
      for (const [pair, from, , second] of joins[i]) if (second) keys.push([pair, from]);
      for (const [key, from] of keys) {
        const own = this._postings(key);
        if (own) (firm = true), lists.push(own);
        const shorter = !exactWords && key.length >= 4 && key.length - 1 > from ? this._wordPostings(key.slice(0, -1)) : null;
        if (shorter) lists.push(shorter);
      }
      const hits = this._merge(lists);
      return hits ? { hits, slip: !firm } : null;
    });
    // A slip leaving the words nothing in common is read as a typo instead ("pice" as "price", not "pic…").
    const all = held.flatMap((h) => (h ? [h.hits] : [])), firm = held.flatMap((h) => (h && !h.slip ? [h.hits] : []));
    const slipsAsTypos = firm.length !== all.length && intersect(all)?.length === 0 && intersect(firm)?.length !== 0;
    const slipLists = [];
    qwords.forEach((w, i) => {
      const h = held[i];
      if (h && !(h.slip && slipsAsTypos)) {
        known.push(h.hits);
      } else if (w.length >= 3) {
        if (h) slipLists.push(h.hits);
        unmatched.push(w);
        if (unknown.length < trigramBudget) unknown.push(w);
      }
    });

    if (exactWords) {
      if (!config.unionFallback && known.length !== qwords.length) return [];
      const candidates = (config.unionFallback ? union(known) : intersect(known) || []).filter(id => {
        const [matched] = this._wordMatch(id, qwords, joins, sep, true);
        return matched > 0 && (config.unionFallback || matched === qwords.length);
      });
      return this._rank(candidates, null, qwords, joins, sep, limit, bestOnly, true);
    }

    const pool = intersect(known);
    // Disjoint known words: no fuzzy match satisfies them all, so straight to the union fallback.
    const disjoint = pool !== null && pool.length === 0;

    // A swapped pair often shares no trigrams ("prcie" → "price"): probe indexed corrections first.
    if (!disjoint && unknown.length && trigramBudget && (config.unionFallback || unknown.length === unmatched.length)) {
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
            const exact = hits.filter(id => this._wordMatch(id, [corrected], [[]], sep, true)[0]);
            if (exact.length) corrections[i].push(exact);
          }
        }
      }
      if (corrections.every(lists => lists.length)) {
        const candidates = intersect([...known, ...corrections.map(lists => union(lists).sort((a, b) => a - b))]);
        if (candidates?.length) return this._rank(candidates, null, qwords, joins, sep, limit, bestOnly);
      }
    }

    if (!disjoint && unknown.length && trigramBudget) {
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
        // A repeated or spaced-out word doesn't raise the floor ("bitoin bitoin" finds what "bitoin" finds).
        Math.max(0, qwords.reduce((len, w) => len + w.length, qwords.length - 1) - 3),
      );
      const minScore = Math.max(config.minScore, Math.ceil(hitCount / 2));
      // Shared trigrams only nominate: each unknown word must be a typo of one of the item's words, not letters scattered across several.
      const checks = unmatched.map((word) => new Typo(word)), indexed = sep === this._sepLookup;
      const typos = dirty.filter((i) => scores[i] >= minScore && checks.every((typo) => (indexed ? typo.ofIndexed(this, i) : typo.ofText(this.items[i], sep))));
      const result = this._rank(typos, minScore, qwords, joins, sep, limit, bestOnly);

      for (const i of dirty) scores[i] = 0;
      dirty.length = 0;

      if (result.length > 0) return result;
    }

    // Every word is required: one neither held nor a typo of anything leaves nothing, not the others' matches.
    if (!config.unionFallback && unmatched.length) return [];

    const candidates = pool?.length ? pool : config.unionFallback ? union([...known, ...slipLists]) : [];
    return candidates.length > 0
      ? this._rank(candidates, null, qwords, joins, sep, limit, bestOnly)
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
   * @param {Joins} joins
   * @param {Uint8Array} sep
   * @param {number} limit
   * @param {boolean} bestOnly
   * @param {boolean} [exactWords]
   * @returns {[number, number][]}
   */
  _rank(indices, minScore, qwords, joins, sep, limit, bestOnly, exactWords = false) {
    const { items, _scores: scores } = this;
    /** @type {[number, number, number][][]} */
    const buckets = Array.from({ length: qwords.length + 1 }, () => []);

    for (let i = 0; i < indices.length; i++) {
      const idx = indices[i];
      if (minScore !== null && scores[idx] < minScore) continue;
      const [matched, whole, position] = this._wordMatch(idx, qwords, joins, sep, exactWords);
      buckets[matched].push([idx, whole, position]);
    }

    /** @type {[number, number][]} */
    const results = [];
    for (let ps = buckets.length - 1; ps >= 0 && results.length < limit; ps--) {
      const bucket = buckets[ps];
      if (!bucket.length) continue;
      bucket.sort(
        ([a, wa, pa], [b, wb, pb]) =>
          scores[b] - scores[a] ||
          wb - wa || // query words matched whole, desc ("1" holds "1" before "13")
          pa - pb ||
          items[a].length - items[b].length ||
          (items[a] < items[b] ? -1 : items[a] > items[b] ? 1 : a - b),
      );
      const take = Math.min(bucket.length, limit - results.length);
      for (let i = 0; i < take; i++) results.push([bucket[i][0], ps]);
      if (bestOnly) break;
    }

    return results;
  }
}

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
 * `all`: every word as written, not once each within `maxLen`.
 * @param {string} text
 * @param {Uint8Array} sep
 * @param {number} maxLen
 * @param {boolean} [all]
 */
function splitWords(text, sep, maxLen, all = false) {
  /** @type {string[]} */
  const words = [];
  const seen = new Set();
  let start = 0;
  for (let i = 0; i <= text.length; i++) {
    if (i < text.length && !sep[text.charCodeAt(i)]) continue;
    if (i > start) {
      const w = text.slice(start, i);
      if (all) words.push(w);
      else if (w.length <= maxLen && !seen.has(w)) {
        seen.add(w);
        words.push(w);
      }
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

/** @param {string[]} list @param {string} key */
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
 * Bounded, so memory stays flat.
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

/** Null when there are no lists (no constraint); empty when they're disjoint. @param {Ids[]} arrays @returns {Ids|null} */
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
    if (!w) return result;
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
 * Never across a word left out, nor a word with itself; `live` drops a pair that can't fit.
 * @typedef {[string, number, number, boolean][][]} Joins
 * @param {string[]} typed @param {string[]} qwords @param {(pair: string, from: number) => boolean} live @returns {Joins}
 */
function joinsOf(typed, qwords, live) {
  const joins = qwords.map(() => /** @type {Joins[number]} */ ([]));
  for (let k = 1; k < typed.length; k++) {
    const a = qwords.indexOf(typed[k - 1]), b = qwords.indexOf(typed[k]), pair = typed[k - 1] + typed[k], from = typed[k - 1].length;
    if (a < 0 || b < 0 || a === b || joins[b].some(([other, first]) => other === pair && first === from) || !live(pair, from)) continue;
    joins[a].push([pair, from, pair.charCodeAt(0), false]);
    joins[b].push([pair, from, pair.charCodeAt(0), true]);
  }
  return joins;
}

const FITS = 1, WHOLE = 2, JOINED = 4;
/**
 * WHOLE: not a prefix nor a plural; JOINED: takes the word before. `from`: the first word's length when `qw` is two joined, else 0.
 * @param {string} qw @param {string} word @param {string | null} previous @param {boolean} exact @param {number} from
 */
function fit(qw, word, previous, exact, from) {
  if (word.length < qw.length - 1 && (previous === null || previous.length >= qw.length)) return 0;
  const rest = previous !== null && qw.length > previous.length && qw.startsWith(previous) ? qw.length - previous.length : -1;
  const joined = rest > 0 && (exact ? word.length === rest : word.length >= rest) && word.startsWith(qw.slice(qw.length - rest));
  if (!joined && !(exact ? word === qw : word.startsWith(qw) || (past(qw, word) && word.length > from))) return 0;
  return FITS | (word === qw || (joined && word.length === rest) ? WHOLE : 0) | (joined ? JOINED : 0);
}

/**
 * @param {string} item @param {string[]} qwords @param {Uint8Array} sep
 * @param {boolean} [exact] Whole words only: validates typo corrections.
 * @returns {[number, number, number]} Query words matched, whole, and first item-word position.
 */
function wordMatch(item, qwords, sep, exact = false) {
  const words = [];
  let start = 0;
  for (let i = 0; i <= item.length; i++) {
    if (i < item.length && !sep[item.charCodeAt(i)]) continue;
    if (i > start) words.push(item.slice(start, i));
    start = i + 1;
  }
  let matched = 0, whole = 0;
  let position = words.length;
  for (const qw of qwords) {
    let found = false;
    for (let i = 0; i < words.length; i++) {
      const how = fit(qw, words[i], i > 0 ? words[i - 1] : null, exact, 0);
      if (how & FITS) {
        if (!found) {
          found = true;
          matched++;
          position = Math.min(position, how & JOINED ? i - 1 : i);
        }
        // Held only as a prefix so far: a later word may hold it whole.
        if (how & WHOLE) {
          whole++;
          break;
        }
      }
    }
  }
  return [matched, whole, position];
}

/** One letter past the word's end, from four letters: a plural, or a slip there. @param {string} qw @param {string} word */
function past(qw, word) {
  return qw.length >= 4 && qw.length === word.length + 1 && qw.startsWith(word);
}

/**
 * Within `most` edits of a run of an item's adjacent words (a swap counts as one) or of its start; never letters scattered across words.
 */
class Typo {
  /** @param {string} word */
  constructor(word) {
    this.codes = Int32Array.from(word, (c) => c.charCodeAt(0));
    this.most = word.length >= 7 ? 2 : 1;
    /** 0 not asked yet, 1 no, 2 yes, by word number. @type {Uint8Array | null} */
    this.single = null;
    /** @type {Map<number, boolean>} */
    this.runs = new Map();
    /** As `single`. @type {Uint8Array | null} */
    this.starts = null;
  }

  /** If no start of this one is within reach of `word`, no run it begins is (cut after it, its alignment costs no more). @param {string} word */
  startsRun(word) {
    if (letters.length < word.length) letters = new Int32Array(word.length * 2);
    for (let c = 0; c < word.length; c++) letters[c] = word.charCodeAt(c);
    return editsWithin(letters, word.length, this.codes, this.codes.length, this.most, true);
  }

  /** @param {QuickMatch} index @param {number} idx */
  ofIndexed(index, idx) {
    const words = index._words, ids = index._itemWords, from = index._itemOffsets[idx], to = index._itemOffsets[idx + 1];
    const n = this.codes.length, count = words.length, single = (this.single ??= new Uint8Array(count)), starts = (this.starts ??= new Uint8Array(count));
    for (let first = from; first < to; first++) {
      const id = ids[first], length = words[id].length;
      if (length + this.most >= n) {
        if (single[id] === 0) single[id] = this.closeRun(words, ids, first, first, length) ? 2 : 1;
        if (single[id] === 2) return true;
      }
      if (length > n || first + 1 === to) continue;
      if (starts[id] === 0) starts[id] = this.startsRun(words[id]) ? 2 : 1;
      if (starts[id] === 1) continue;
      let len = length;
      for (let last = first + 1; last < to; last++) {
        if (len > n) break;
        len += words[ids[last]].length;
        if (len + this.most < n) continue;
        // Pairs memoized by a key kept a small integer (exact); longer runs are rare.
        const key = last - first === 1 && count < 94906266 ? ids[first] * count + ids[last] : -1;
        let close = key >= 0 ? this.runs.get(key) : undefined;
        if (close === undefined) {
          close = this.closeRun(words, ids, first, last, len);
          if (key >= 0) this.runs.set(key, close);
        }
        if (close) return true;
      }
    }
    return false;
  }

  /** @param {string} item @param {Uint8Array} sep */
  ofText(item, sep) {
    /** @type {string[]} */
    const words = [];
    let start = 0;
    for (let i = 0; i <= item.length; i++) {
      if (i < item.length && !sep[item.charCodeAt(i)]) continue;
      if (i > start) words.push(item.slice(start, i));
      start = i + 1;
    }
    const ids = Uint32Array.from(words, (_, k) => k), n = this.codes.length;
    for (let first = 0; first < words.length; first++) {
      const length = words[first].length;
      if (length + this.most >= n && this.closeRun(words, ids, first, first, length)) return true;
      if (length > n || first + 1 === words.length || !this.startsRun(words[first])) continue;
      let len = length;
      for (let last = first + 1; last < words.length; last++) {
        if (len > n) break;
        len += words[last].length;
        if (len + this.most < n) continue;
        if (this.closeRun(words, ids, first, last, len)) return true;
      }
    }
    return false;
  }

  /** @param {string[]} words @param {ArrayLike<number>} ids @param {number} first @param {number} last @param {number} len */
  closeRun(words, ids, first, last, len) {
    const n = this.codes.length, cut = Math.min(len, n + this.most);
    if (letters.length < cut) letters = new Int32Array(cut * 2);
    let k = 0;
    for (let w = first; w <= last && k < cut; w++) {
      const word = words[ids[w]];
      for (let c = 0; c < word.length && k < cut; c++) letters[k++] = word.charCodeAt(c);
    }
    return (cut === len && editsWithin(this.codes, this.codes.length, letters, cut, this.most, false)) || (n >= 5 && editsWithin(this.codes, this.codes.length, letters, Math.min(cut, n + 1), 1, true));
  }
}

/** Reused from one comparison to the next. */
let letters = new Int32Array(32);
let before = new Int32Array(32);
let row = new Int32Array(32);
let next = new Int32Array(32);

/**
 * Gives up as soon as `a` and `b` can't be within `most` edits (swaps count as one); `start`: `a` against the closest start of `b`
 * (the last row's least, which no row lowers).
 * @param {Int32Array} a @param {number} n @param {Int32Array} b @param {number} m @param {number} most @param {boolean} start
 */
function editsWithin(a, n, b, m, most, start) {
  if (!start && Math.abs(n - m) > most) return false;
  if (row.length <= m) {
    before = new Int32Array((m + 1) * 2);
    row = new Int32Array((m + 1) * 2);
    next = new Int32Array((m + 1) * 2);
  }
  let x = before;
  let y = row;
  let z = next;
  for (let j = 0; j <= m; j++) y[j] = j;
  for (let i = 1; i <= n; i++) {
    z[0] = i;
    let least = i;
    for (let j = 1; j <= m; j++) {
      let value = Math.min(y[j] + 1, z[j - 1] + 1, y[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) value = Math.min(value, x[j - 2] + 1);
      z[j] = value;
      if (value < least) least = value;
    }
    if (least > most) return false;
    const t = x;
    x = y;
    y = z;
    z = t;
  }
  return start || y[m] <= most;
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
