// @ts-check
// Studio's series search (see seriesSearch): run in its worker (search-worker.js), or on the page when workers can't
// run. Its imports are relative, so in the worker they stay in the release it was served from.
import { QuickMatch, QuickMatchConfig } from "../modules/quickmatch-js/src/index.js";
import { initialsOf, initialsWord } from "./initials.js";

/**
 * A query's words as the server normalizes them (crates/bitview_query/src/vecs/normalize.rs): words of letters, digits
 * and < > = + %, decimals kept, lowercase.
 * @param {string} text
 */
function normalizeQuery(text) {
  return (text.match(/(?:[A-Za-z0-9<>=+%]|(?<=[0-9])\.(?=[0-9]))+/g) ?? []).join(" ").toLowerCase();
}
/**
 * Studio's series search: the server's QuickMatch word matching, with consecutive initials before typos. Whole
 * words and prefixes lead initials; a description several series share ranks
 * them by how few other words their names carry. Series only by block (none of `dated`, the intervals Studio charts) come last, their day's (a 24h sum, an
 * average) in their place. It runs in a worker (search-worker.js; on the page when workers can't run): the catalog's
 * names and descriptions are indexed once, off the page, and each query is answered in milliseconds.
 * @param {string} catalog the catalog's URL @param {readonly string[]} dated
 */
export async function seriesSearch(catalog, dated) {
  const byBytes = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
  const index = (async () => {
    const response = await fetch(catalog, { signal: AbortSignal.timeout(30_000) });
    if (!response.ok) throw new Error(`The catalog answered ${response.status}`);
    /** @type {Map<string, string>} */
    const described = new Map();
    /** What the API says of each series (its description, intervals, type, accumulation), as its info answers it. @type {Map<string, { description: string, indexes: string[], type: string, accumulation: string }>} */
    const about = new Map();
    /** @type {Set<string>} */
    const byBlock = new Set();
    const stack = [await response.json()];
    while (stack.length) {
      const node = stack.pop();
      if (typeof node?.name === "string" && Array.isArray(node.indexes)) {
        // (A name the catalog lists in several places, each with its own intervals (an interval's first height,
        // under each): all of them, as its info has them.)
        const had = about.get(node.name);
        if (had) {
          had.indexes = [...new Set([...had.indexes, ...node.indexes])];
          continue;
        }
        // (A text (a date, a hash, an address) or an object isn't a value to chart: not searched for (one added says so).)
        const text = node.type === "string" || node.type === "object";
        if (!text) described.set(node.name, typeof node.description === "string" ? node.description : "");
        // (Its type as the API's info says it, "Dollars", "Sats": the catalog's kind; its "type" is the JSON one.)
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
      descriptions,
      nameInitials,
      byNameInitials: grouped(nameInitials),
      descriptionInitials,
      byDescriptionInitials: grouped(descriptionInitials),
      wordCounts: names.map((name) => name.split("_").length),
      seriesOf: descriptions.map((text) => /** @type {number[]} */ (byDescription.get(text))),
      matcher: new QuickMatch(names),
      descriptionMatcher: new QuickMatch(descriptions),
      blocked: names.map((name) => byBlock.has(name)),
      // A series by block, by its day: its own over 24 hours, their sum, their average (before a unit it ends in).
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
    // (Stage by stage: a name's whole words before its starts and slips.)
    const run = stages(found, words, limit);
    for (let stage = 0; stage < run.length; stage++) if (result.length < limit) append(run[stage]());
    return [...result, ...last].slice(0, limit).map((id) => found.names[id]);
  }
  /**
   * Word matches first (whole words, then prefixes, in names and descriptions), then initials, then typos.
   * Each stage runs only while there's room. @param {Awaited<typeof index>} found
   * @param {string} query @param {number} limit @returns {(() => number[])[]}
   */
  function stages({ names, descriptions, nameInitials, byNameInitials, descriptionInitials, byDescriptionInitials, wordCounts, seriesOf, matcher, descriptionMatcher }, query, limit) {
    // (More than fit: those by block give way to those by date.)
    const room = Math.min(names.length, limit * 3), config = new QuickMatchConfig().withLimit(room).withUnionFallback(false);
    /** Series by matching description, those whose names carry fewer other words first. @param {[number, number][]} matches @param {Map<number, number>} nameWords */
    const ranked = (matches, nameWords) => matches
      .flatMap(([description], rank) => seriesOf[description].map((id) => [id, rank, nameWords.get(id) ?? 0, wordCounts[id]]))
      .sort((a, b) => b[2] * a[3] - a[2] * b[3] || names[a[0]].length - names[b[0]].length || a[1] - b[1] || byBytes(names[a[0]], names[b[0]]))
      .slice(0, room)
      .map(([id]) => id);
    const descriptionConfig = new QuickMatchConfig().withLimit(seriesOf.length).withUnionFallback(false);
    /** @type {Map<number, number> | null} */
    let nameWords = null;
    const words = () => (nameWords ??= new Map(matcher.matchesExactWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(names.length))));
    // (The words as the matcher splits them, once each: a name matching all of them, by word, is a whole match.)
    const count = new Set(query.split(/[_\-\s:/]+/).filter(Boolean)).size;
    /** @type {[number, number][] | null} */
    let fuzzy = null;
    const slips = () => (fuzzy ??= matcher.matchesWithIdsAndMatchedWords(query, config));
    let descriptionSlips = /** @type {[number, number][] | null} */ (null);
    const described = () => (descriptionSlips ??= descriptionMatcher.matchesWithIdsAndMatchedWords(query, descriptionConfig));
    const typed = query.split(/[_\-\s:/]+/).filter(Boolean), initialWords = typed.map((word) => initialsWord(word) ? word : "\0");
    const initialMatches = (/** @type {string[]} */ texts, /** @type {string[]} */ initials, /** @type {Map<string, number[]>} */ groups) => {
      const candidates = /** @type {Set<number>} */ (new Set());
      for (const word of typed) {
        if (!initialsWord(word)) continue;
        for (const [letters, ids] of groups) if (letters.includes(word)) for (const id of ids) candidates.add(id);
      }
      const found = /** @type {[number, number][]} */ ([]);
      for (const i of candidates) {
        let held = true;
        for (let k = 0; k < typed.length; k++) if (!texts[i].includes(typed[k]) && !initials[i].includes(initialWords[k])) { held = false; break; }
        if (held) found.push([i, count]);
      }
      return found.sort((a, b) => a[0] - b[0]);
    };
    return [
      () => matcher.matchesExactWithIdsAndMatchedWords(query, config).map(([id]) => id),
      () => initialsWord(query) ? byNameInitials.get(query) ?? [] : [],
      () => slips().filter(([, matched]) => matched >= count).map(([id]) => id),
      () => ranked(descriptionMatcher.matchesExactWithIdsAndMatchedWords(query, descriptionConfig), words()),
      () => ranked(described().filter(([, matched]) => matched >= count), words()),
      () => [...initialMatches(names, nameInitials, byNameInitials).map(([id]) => id), ...ranked(initialMatches(descriptions, descriptionInitials, byDescriptionInitials), words())],
      () => slips().filter(([, matched]) => matched < count).map(([id]) => id),
      () => ranked(described().filter(([, matched]) => matched < count), words()),
    ];
  }
  const found = await index;
  return (/** @type {{ query?: string, limit?: number, about?: string }} */ { query = "", limit = 0, about }) => about !== undefined ? { about: found.about.get(about) ?? null } : { names: search(found, query, limit) };
}
