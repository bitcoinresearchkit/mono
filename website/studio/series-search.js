// @ts-check
// Studio's series search (see seriesSearch): run in its worker (search-worker.js), or on the page when workers can't
// run. Its imports are relative, so in the worker they stay in the release it was served from.
import { QuickMatch, QuickMatchConfig } from "../modules/quickmatch-js/src/index.js";
import { initialsOf, initialsWord } from "./initials.js";

/**
 * Mirrors the server's normalization (crates/bitview_query/src/vecs/normalize.rs).
 * @param {string} text
 */
function normalizeQuery(text) {
  return (text.match(/(?:[A-Za-z0-9<>=+%]|(?<=[0-9])\.(?=[0-9]))+/g) ?? []).join(" ").toLowerCase();
}
/** @param {string} catalog @param {readonly string[]} dated */
export async function seriesSearch(catalog, dated) {
  const byBytes = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
  const index = (async () => {
    const response = await fetch(catalog, { signal: AbortSignal.timeout(30_000) });
    if (!response.ok) throw new Error(`The catalog answered ${response.status}`);
    /** @type {Map<string, string>} */
    const described = new Map();
    /** @type {Map<string, { description: string, indexes: string[], type: string, accumulation: string }>} */
    const about = new Map();
    /** @type {Set<string>} */
    const byBlock = new Set();
    const stack = [await response.json()];
    while (stack.length) {
      const node = stack.pop();
      if (typeof node?.name === "string" && Array.isArray(node.indexes)) {
        // A name listed in several places has its own intervals under each: all of them, as its info has them.
        const had = about.get(node.name);
        if (had) {
          had.indexes = [...new Set([...had.indexes, ...node.indexes])];
          continue;
        }
        // A text (a date, a hash, an address) or an object isn't a value to chart: not searched for.
        const text = node.type === "string" || node.type === "object";
        if (!text) described.set(node.name, typeof node.description === "string" ? node.description : "");
        about.set(node.name, { description: typeof node.description === "string" ? node.description : "", indexes: node.indexes, type: typeof node.kind === "string" ? node.kind : "", accumulation: node.accumulation });
      } else if (node && typeof node === "object") for (const child of Object.values(node)) stack.push(child);
    }
    for (const [name, { indexes }] of about) if (!dated.some((id) => indexes.includes(id))) byBlock.add(name);
    // In the server's orders: names by length then text, descriptions by text.
    const names = [...described.keys()].sort((a, b) => a.length - b.length || byBytes(a, b));
    const ids = new Map(names.map((name, id) => [name, id]));
    /** @type {Map<string, number[]>} */
    const byDescription = new Map();
    names.forEach((name, id) => {
      const text = normalizeQuery(described.get(name) ?? "");
      if (text) (byDescription.get(text) ?? byDescription.set(text, []).get(text))?.push(id);
    });
    const descriptions = [...byDescription.keys()].sort(byBytes), nameInitials = names.map((name) => initialsOf([name])), descriptionInitials = descriptions.map((text) => initialsOf([text]));
    const grouped = (/** @type {string[]} */ initials) => {
      const groups = /** @type {Map<string, number[]>} */ (new Map());
      initials.forEach((text, id) => (groups.get(text) ?? groups.set(text, []).get(text))?.push(id));
      return groups;
    };
    return {
      names,
      about,
      nameInitials,
      byNameInitials: grouped(nameInitials),
      descriptionInitials,
      byDescriptionInitials: grouped(descriptionInitials),
      wordCounts: names.map((name) => name.split("_").length),
      seriesOf: descriptions.map((text) => /** @type {number[]} */ (byDescription.get(text))),
      matcher: new QuickMatch(names),
      descriptionMatcher: new QuickMatch(descriptions),
      blocked: names.map((name) => byBlock.has(name)),
      daily: names.map((name) => (byBlock.has(name) ? ((unit) => ["24h", "sum_24h", "avg_24h"].flatMap((end) => [`${name}_${end}`, ...(unit ? [`${unit[1]}_${end}_${unit[2]}`] : [])]))(/^(.*)_(usd|btc)$/.exec(name)).map((other) => ids.get(other) ?? -1).find((id) => id >= 0 && !byBlock.has(names[id])) ?? -1 : -1)),
    };
  })();
  /** @param {Awaited<typeof index>} found @param {string} query @param {number} max @returns {string[]} */
  function search(found, query, max) {
    const limit = Math.min(max, found.names.length);
    if (!limit) return [];
    const words = normalizeQuery(query);
    if (!words) return [];
    /** @type {number[]} */
    const result = [], last = [];
    const seen = new Set();
    /** @param {Iterable<number>} ids */
    const append = (ids) => {
      for (const id of ids) {
        if (result.length === limit) break;
        if (seen.has(id)) continue;
        seen.add(id);
        if (!found.blocked[id]) result.push(id);
        else {
          if (last.length < limit) last.push(id);
          const day = found.daily[id];
          if (day >= 0 && !seen.has(day)) (seen.add(day), result.push(day));
        }
      }
    };
    const run = stages(found, words, limit);
    for (let stage = 0; stage < run.length; stage++) if (result.length < limit) append(run[stage]());
    return [...result, ...last].slice(0, limit).map((id) => found.names[id]);
  }
  /** @param {Awaited<typeof index>} found @param {string} query @param {number} limit @returns {(() => number[])[]} */
  function stages({ names, nameInitials, byNameInitials, descriptionInitials, byDescriptionInitials, wordCounts, seriesOf, matcher, descriptionMatcher }, query, limit) {
    const room = Math.min(names.length, limit * 3), config = new QuickMatchConfig().withLimit(room).withUnionFallback(false);
    // As the server orders them: the larger share of a name's words in the query first, then by id.
    /** @param {number[]} descriptions @param {Map<number, number>} nameWords */
    const ranked = (descriptions, nameWords) => descriptions
      .flatMap((description) => seriesOf[description].map((id) => [id, nameWords.get(id) ?? 0, wordCounts[id]]))
      .sort((a, b) => b[1] * a[2] - a[1] * b[2] || a[0] - b[0])
      .slice(0, room)
      .map(([id]) => id);
    /** @type {Map<number, number> | null} */
    let nameWords = null;
    const words = () => (nameWords ??= new Map(matcher.matchesExactWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(names.length))));
    // Counted as the matcher splits them: a name matching all of them, by word, is a whole match.
    const count = new Set(query.split(/[_\-\s:/]+/).filter(Boolean)).size;
    /** @type {[number, number][] | null} */
    let fuzzy = null;
    const slips = () => (fuzzy ??= matcher.matchesWithIdsAndMatchedWords(query, config));
    // A description is found by whole words only, singular or plural, as on the server.
    const described = () => {
      const [first, ...rest] = descriptionMatcher.wordHits(query);
      const others = rest.map((ids) => new Set(ids));
      return Array.from(first ?? []).filter((id) => others.every((ids) => ids.has(id)));
    };
    const typed = query.split(/[_\-\s:/]+/).filter(Boolean), initialWords = typed.map((word) => initialsWord(word) ? word : "\0");
    const initialMatches = (/** @type {string[]} */ initials, /** @type {Map<string, number[]>} */ groups, /** @type {(id: number, word: number) => boolean} */ holds) => {
      const candidates = /** @type {Set<number>} */ (new Set());
      for (const word of typed) {
        if (!initialsWord(word)) continue;
        for (const [letters, ids] of groups) if (letters.includes(word)) for (const id of ids) candidates.add(id);
      }
      const found = /** @type {number[]} */ ([]);
      for (const i of candidates) {
        let held = true;
        for (let k = 0; k < typed.length; k++) if (!holds(i, k) && !initials[i].includes(initialWords[k])) { held = false; break; }
        if (held) found.push(i);
      }
      return found.sort((a, b) => a - b);
    };
    const nameInitialMatches = () => initialMatches(nameInitials, byNameInitials, (i, k) => names[i].includes(typed[k]));
    const descriptionInitialMatches = () => {
      // (Read at the first candidate: most queries have none.)
      /** @type {Set<number>[] | undefined} */
      let held;
      return initialMatches(descriptionInitials, byDescriptionInitials, (i, k) => (held ??= typed.map((word) => new Set(descriptionMatcher.wordHits(word)[0])))[k].has(i));
    };
    return [
      () => matcher.matchesExactWithIdsAndMatchedWords(query, config).map(([id]) => id),
      () => initialsWord(query) ? byNameInitials.get(query) ?? [] : [],
      () => slips().filter(([, matched]) => matched >= count).map(([id]) => id),
      () => ranked(described(), words()),
      () => [...nameInitialMatches(), ...ranked(descriptionInitialMatches(), words())],
      () => slips().filter(([, matched]) => matched < count).map(([id]) => id),
    ];
  }
  const found = await index;
  return (/** @type {{ query?: string, limit?: number, about?: string }} */ { query = "", limit = 0, about }) => about !== undefined ? { about: found.about.get(about) ?? null } : { names: search(found, query, limit) };
}
