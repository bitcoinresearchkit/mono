// What a release breaks of the API, for the page beside it: two servers compared, one on the last release (prod) and
// one on what's about to be released (next), each answering for its own version. Their endpoints, from their OpenAPI
// specifications: those gone, and those whose parameters or answers changed. Their indexes. Their series ids: each one
// gone, where it went, found by the values both servers serve (a series renamed serves the same values under its new
// id, or the same in another unit, cents read as dollars), told as rules (see rules.js), the ids no rule explains
// written out, and those gone for good grouped by cohort and by measure. Run it before a release, both servers synced to
// the same tip:
//
//   bun showcases/changes/diff.mjs [old server] [new server]   writes <old version>.json, and lists it on the page
//   ... --to VERSION                                                 the version about to be released (the old one's
//                                                                    next minor, by default: 0.12.2 to 0.13.0)
//   ... --cache                                                      keeps what it fetched in target/changes, for
//                                                                    work on this script (never for a release)
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { index, read } from "./rules.js";

const HERE = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const toAt = args.indexOf("--to");
const servers = args.filter((arg, i) => !arg.startsWith("--") && i !== toAt + 1);
const OLD = (servers[0] ?? "https://bitview.space").replace(/\/$/, "");
const NEW = (servers[1] ?? "https://next.bitview.space").replace(/\/$/, "");
const CACHE = args.includes("--cache") ? join(HERE, "..", "..", "target", "changes") : null;
// (Requests at once, per server: gentle on a prod server that may be serving alone.)
const AT_ONCE = 4;
// (Values read per series: enough that two series match only if they're one, or one cohort's matches another's.)
const WINDOW = 30;
// What a series' values may have been multiplied by under its new id: units (cents to dollars, sats to bitcoin, ppm,
// basis points or percent to a ratio) and their inverses.
const FACTORS = [1, 1e-2, 1e2, 1e-8, 1e8, 1e-6, 1e6, 1e-4, 1e4, 1e-3, 1e3];

// ── Fetching ──

/** @param {string} url @returns {Promise<any>} */
async function get(url) {
  const cached = CACHE && join(CACHE, createHash("sha1").update(url).digest("hex") + ".json");
  if (cached) {
    try {
      return JSON.parse(readFileSync(cached, "utf8"));
    } catch {}
  }
  for (let attempt = 1; ; attempt++) {
    const response = await fetch(url, { headers: { accept: "application/json" } }).catch((error) => error);
    if (response instanceof Response && response.ok) {
      const body = await response.json();
      if (cached) {
        mkdirSync(CACHE, { recursive: true });
        writeFileSync(cached, JSON.stringify(body));
      }
      return body;
    }
    // (A 4xx is the request's fault: asking again won't help.)
    if (attempt === 3 || (response instanceof Response && response.status < 500)) {
      throw new Error(`${url}: ${response instanceof Response ? response.status : response.message}`);
    }
    await Bun.sleep(1000 * attempt);
  }
}

/**
 * `work` for each of `items`, `AT_ONCE` at a time.
 * @template T
 * @param {T[]} items
 * @param {(item: T) => Promise<void>} work
 */
async function each(items, work) {
  let next = 0;
  await Promise.all(
    Array.from({ length: AT_ONCE }, async () => {
      while (next < items.length) await work(items[next++]);
    }),
  );
}

/** @typedef {{ kind: string, json: string, indexes: string[], path: string[] }} Series A series: its value type, as named
 *   and as JSON reads it, its indexes, and where it is in the catalog. */

/**
 * Every series a server's catalog lists, by id, with where it is in the catalog's tree (the first place, if several).
 * @param {string} server
 */
async function catalog(server) {
  /** @type {Map<string, Series>} */
  const series = new Map();
  /** @param {any} node @param {string[]} path */
  const walk = (node, path) => {
    if (typeof node?.name === "string" && Array.isArray(node.indexes)) {
      if (!series.has(node.name)) series.set(node.name, { kind: node.kind, json: node.type, indexes: node.indexes, path });
      return;
    }
    for (const [key, child] of Object.entries(node)) if (child && typeof child === "object") walk(child, [...path, key]);
  };
  walk(await get(`${server}/api/series`), []);
  return series;
}

console.log(`Comparing ${OLD} (the last release) with ${NEW} (the next)...`);
const [oldSpec, newSpec, oldSeries, newSeries, oldIndexes, newIndexes] = await Promise.all([
  get(`${OLD}/openapi.json`),
  get(`${NEW}/openapi.json`),
  catalog(OLD),
  catalog(NEW),
  get(`${OLD}/api/series/indexes`),
  get(`${NEW}/api/series/indexes`),
]);
// (Releases as `v0.12.2`; the one being made, said, or the next minor: in 0.x, a release that breaks is one.)
const FROM = `v${String(oldSpec.info.version).replace(/^v/, "")}`;
const [major, minor] = FROM.slice(1).split(".").map(Number);
const TO = toAt >= 0 ? `v${args[toAt + 1].replace(/^v/, "")}` : `v${major}.${minor + 1}.0`;
const oldIds = [...oldSeries.keys()], newIds = [...newSeries.keys()];
const kept = oldIds.filter((id) => newSeries.has(id));
const gone = oldIds.filter((id) => !newSeries.has(id));
console.log(`${FROM}: ${oldIds.length} series; ${TO} (${newSpec.info.version}): ${newIds.length}; ${kept.length} ids kept, ${gone.length} gone`);

// ── Each series' values ──

// The same positions on both servers: the last days but today's (still filling), or blocks a thousand deep (the same
// whatever their tips), or anything else from its start.
const [oldDays, newDays, oldHeight, newHeight] = await Promise.all([
  get(`${OLD}/api/series/price_close/day1/len`),
  get(`${NEW}/api/series/price_close/day1/len`),
  get(`${OLD}/api/series/blockhash/height/len`),
  get(`${NEW}/api/series/blockhash/height/len`),
]);
if (oldDays !== newDays) throw new Error(`the servers are on different days (${oldDays}, ${newDays}): run it again once they aren't`);
const settled = Math.min(oldHeight, newHeight) - 1000;
// A deeper window, early 2015, for the ids renamed one by one: there blocks weren't full and most cohorts held coins at
// a loss, so two series that agree now (every percentile of full blocks' weight, a cohort's losses while its coins
// were idle) part there.
const DEEP = { day1: 2200, height: 340_000 };
/** The window read of a series with `indexes` (the deep one, if `deep`). @param {string[]} indexes */
const windowOf = (indexes, deep = false) =>
  indexes.includes("day1")
    ? { index: "day1", start: deep ? DEEP.day1 : oldDays - 1 - WINDOW }
    : indexes.includes("height")
      ? { index: "height", start: deep ? DEEP.height : settled - WINDOW }
      : { index: indexes[0], start: 0 };

/** @typedef {{ index: string, values: unknown[] }} Window */
// (Which window a series was read in: an index renamed reads its start as it did, so it's compared by that.)
/** @param {Window} window */
const windowKind = (window) => (window.index === "day1" || window.index === "height" ? window.index : "start");

/**
 * The values of each of `ids` on `server`, in the window its indexes give (the deep one, if `deep`); an id whose
 * values can't be read is left out, and counted.
 * @param {string} server
 * @param {Map<string, Series>} series
 * @param {string[]} ids
 */
async function windows(server, series, ids, deep = false) {
  /** @type {Map<string, Window>} */
  const read = new Map();
  /** @type {Map<string, string[]>} */
  const byIndex = new Map();
  for (const id of ids) {
    const { index } = windowOf(series.get(id)?.indexes ?? []);
    byIndex.set(index, [...(byIndex.get(index) ?? []), id]);
  }
  // (Batches within the 2,048 bytes a series list may take.)
  /** @type {[string, string[]][]} */
  const batches = [];
  for (const [index, all] of byIndex) {
    let batch = [], length = 0;
    for (const id of all) {
      if (length + id.length + 1 > 1900) batches.push([index, batch]), (batch = []), (length = 0);
      batch.push(id), (length += id.length + 1);
    }
    if (batch.length) batches.push([index, batch]);
  }
  let done = 0, unread = 0;
  /** @param {[string, string[]]} batch @returns {Promise<void>} */
  const fetchBatch = async ([index, batch]) => {
    const { start } = windowOf([index], deep);
    try {
      const answers = await get(`${server}/api/series/bulk?series=${batch.join(",")}&index=${index}&start=${start}&end=${start + WINDOW}`);
      batch.forEach((id, i) => read.set(id, { index, values: answers[i].data }));
    } catch {
      // (One series that can't be read fails its batch: halved until it's alone.)
      if (batch.length === 1) return void unread++;
      const half = Math.ceil(batch.length / 2);
      await fetchBatch([index, batch.slice(0, half)]);
      await fetchBatch([index, batch.slice(half)]);
    }
  };
  await each(batches, async (batch) => {
    await fetchBatch(batch);
    if (++done % 50 === 0 || done === batches.length) process.stdout.write(`\r  ${server}: ${done}/${batches.length} batches`);
  });
  console.log(unread ? `, ${unread} series unreadable` : "");
  return read;
}

console.log("Reading the values of every series gone and every series now:");
// (The ids kept too, to tell whether their values changed under them.)
const oldWindows = await windows(OLD, oldSeries, [...gone, ...kept]);
// (Every series now, not only those added: an id gone may now read as one that was there, its cents as its dollars.)
const newWindows = await windows(NEW, newSeries, newIds);

// ── Matching by values ──

/** A window's values in order, numbers and what isn't, an object's fields by name. @param {unknown} value */
function flat(value, out = /** @type {unknown[]} */ ([])) {
  if (Array.isArray(value)) value.forEach((item) => flat(item, out));
  else if (value && typeof value === "object") for (const key of Object.keys(value).sort()) flat(/** @type {any} */ (value)[key], out);
  else out.push(value);
  return out;
}
/** @param {unknown[]} values */
const sum = (values) => values.reduce((total, value) => total + (typeof value === "number" ? value : 0), /** @type {number} */ (0));
// (A series whose values take fewer than three values, a count that rarely moves, zeros, matches too many others to
// say which: it goes by the rules alone.)
/** @param {Window} window */
const strong = (window) => new Set(flat(window.values).map((v) => (typeof v === "number" ? v.toPrecision(6) : JSON.stringify(v)))).size >= 3;

// Two windows the same, `factor` applied to the first's numbers: each value within a part in ten thousand (the same
// series computed again, a rounding or a fix moving its last digits), or, for a fifth of them at most, a cent (cents
// rounded once rather than twice).
/** @param {Window} a @param {Window} b @param {number} factor */
function alike(a, b, factor) {
  const x = flat(a.values), y = flat(b.values);
  if (x.length !== y.length) return false;
  let loose = 0;
  for (let i = 0; i < x.length; i++) {
    const p = typeof x[i] === "number" ? /** @type {number} */ (x[i]) * factor : x[i], q = y[i];
    if (p === q) continue;
    if (typeof p !== "number" || typeof q !== "number") return false;
    const gap = Math.abs(p - q), size = Math.max(Math.abs(p), Math.abs(q));
    if (gap <= 1e-4 * size) continue;
    if (size < 1 || gap > 0.0100001 || ++loose > x.length / 5) return false;
  }
  return true;
}

/** @param {string} id */
const tokens = (id) => id.split("_");
/** How alike two ids read, 0 to 1: the share of their words in common, in order. @param {string} a @param {string} b */
function likeness(a, b) {
  const x = tokens(a), y = tokens(b), row = new Array(y.length + 1).fill(0);
  for (let i = 1; i <= x.length; i++) {
    let diagonal = 0;
    for (let j = 1; j <= y.length; j++) {
      const above = row[j];
      row[j] = x[i - 1] === y[j - 1] ? diagonal + 1 : Math.max(row[j], row[j - 1]);
      diagonal = above;
    }
  }
  return (2 * row[y.length]) / (x.length + y.length);
}
/** The same, letter by letter: what tells `pct25` nearer `p25` than `p10`. @param {string} a @param {string} b */
function spelling(a, b) {
  const row = new Array(b.length + 1).fill(0);
  for (let i = 1; i <= a.length; i++) {
    let diagonal = 0;
    for (let j = 1; j <= b.length; j++) {
      const above = row[j];
      row[j] = a[i - 1] === b[j - 1] ? diagonal + 1 : Math.max(row[j], row[j - 1]);
      diagonal = above;
    }
  }
  return (2 * row[b.length]) / (a.length + b.length);
}
/** Of the ids now that serve the same values, the one `id` reads most like: by words, then by spelling. @param {string} id @param {string[]} ids */
const likest = (id, ids) =>
  ids.reduce((a, b) => {
    const words = likeness(id, b) - likeness(id, a);
    return words > 0 || (words === 0 && spelling(id, b) > spelling(id, a)) ? b : a;
  });

// The series now, by index, in the order of their values' sum: where a series gone might be is a range of them.
/** @type {Map<string, { id: string, total: number }[]>} */
const bySum = new Map();
for (const [id, window] of newWindows) {
  if (!strong(window)) continue;
  bySum.set(windowKind(window), [...(bySum.get(windowKind(window)) ?? []), { id, total: sum(flat(window.values)) }]);
}
for (const list of bySum.values()) list.sort((a, b) => a.total - b.total);
/** The series now that series gone `id` serves the values of, `factor` applied; the likeliest by name, if several. @param {string} id */
function findByValues(id) {
  const window = oldWindows.get(id);
  if (!window || !strong(window)) return null;
  const list = bySum.get(windowKind(window)) ?? [], values = flat(window.values);
  for (const factor of FACTORS) {
    const total = sum(values) * factor, slack = Math.abs(total) * 1e-4 + 0.0100001 * values.length;
    let low = 0, high = list.length;
    while (low < high) {
      const middle = (low + high) >> 1;
      if (list[middle].total < total - slack) low = middle + 1;
      else high = middle;
    }
    /** @type {string[]} */
    const candidates = [];
    for (let i = low; i < list.length && list[i].total <= total + slack; i++) {
      if (alike(window, /** @type {Window} */ (newWindows.get(list[i].id)), factor)) candidates.push(list[i].id);
    }
    if (candidates.length) return { id: likest(id, candidates), factor };
  }
  return null;
}

/** @typedef {{ from: string, to: string, factor: number }} Pair */
/** @type {Pair[]} */
const pairs = [];
for (const id of gone) {
  const found = findByValues(id);
  if (found) pairs.push({ from: id, to: found.id, factor: found.factor });
}
console.log(`${pairs.length} of the ids gone found by their values`);
// (The ids kept say what didn't change: a cohort, a measure, a suffix kept as it was.)
const evidence = [...pairs, ...kept.map((id) => ({ from: id, to: id, factor: 1 }))];

// ── Rules ──

/** @param {Map<string, number>} map @param {string} key */
const bump = (map, key, by = 1) => map.set(key, (map.get(key) ?? 0) + by);
/** The words two lists of words start with alike. @param {string[][]} lists */
function common(lists) {
  let length = Math.min(...lists.map((list) => list.length));
  for (const list of lists) for (let i = 0; i < length; i++) if (list[i] !== lists[0][i]) length = i;
  return lists[0].slice(0, length);
}

// The cohorts now, from the catalog's tree, which holds each cohort's series under it (`age.ranges.2y_to_3y.…`): a node
// is one when a sibling holds mostly the same series under it as it does (the same paths below), its prefix the words
// every id under it starts with (`age_2y_to_3y_`). An id's cohort is the outermost holding it.
/** @type {Map<string, string>} */
const cohortNow = new Map();
{
  /** @type {Map<string, { ids: string[], below: Set<string> }>} */
  const nodes = new Map();
  for (const [id, { path }] of newSeries) {
    for (let depth = 1; depth < path.length; depth++) {
      const key = path.slice(0, depth).join("."), node = nodes.get(key) ?? { ids: [], below: new Set() };
      node.ids.push(id), node.below.add(path.slice(depth).join("."));
      nodes.set(key, node);
    }
  }
  /** @type {Map<string, string[]>} */
  const children = new Map();
  for (const key of nodes.keys()) {
    const parent = key.includes(".") ? key.slice(0, key.lastIndexOf(".")) : "";
    children.set(parent, [...(children.get(parent) ?? []), key]);
  }
  /** @param {Set<string>} a @param {Set<string>} b */
  const overlap = (a, b) => {
    let both = 0;
    for (const item of a) if (b.has(item)) both++;
    return both / (a.size + b.size - both);
  };
  /** @type {Map<string, string>} */
  const prefixOf = new Map();
  for (const siblings of children.values()) {
    for (const key of siblings) {
      const node = /** @type {{ ids: string[], below: Set<string> }} */ (nodes.get(key));
      if (node.below.size < 3) continue;
      const twin = siblings.some((other) => other !== key && overlap(node.below, /** @type {any} */ (nodes.get(other)).below) >= 0.5);
      const words = common(node.ids.map(tokens));
      if (twin && words.length && node.ids.every((id) => tokens(id).length > words.length)) prefixOf.set(key, words.join("_") + "_");
    }
  }
  for (const [id, { path }] of newSeries) {
    for (let depth = 1; depth < path.length; depth++) {
      const prefix = prefixOf.get(path.slice(0, depth).join("."));
      if (prefix && id.startsWith(prefix)) {
        cohortNow.set(id, prefix);
        break;
      }
    }
  }
}

// Each cohort's prefix as it was: words that three fifths of the ids that came to it start with, those whose ids read
// off them read most like the measures they became (`utxos_7y_to_8y_old_`: `matured_supply` is like `aged_out_supply`,
// `old_matured_supply` less so; `bedrock_cointime_`, though all that came to `awake_` were its cost bases); again for
// the rest, a cohort made of two (up to three: `utxos_under_5m_old_` came to `sth_` with it). A cohort found only now
// and then, its series matching another's where they hold the same coins (`utxos_under_1k_sats_` and
// `amount_100_to_1k_sats_` hold the same bitcoin, not the same outputs), isn't one: a sixth of the series it had whose
// values could tell, at least, and as a second, three quarters of the share the first came with.
/** @type {Map<string, number>} */
const telling = new Map();
for (const id of gone) {
  const window = oldWindows.get(id);
  if (!window || !strong(window)) continue;
  const words = tokens(id);
  for (let k = 1; k < words.length; k++) bump(telling, words.slice(0, k).join("_") + "_");
}
/** @type {Map<string, Pair[]>} */
const byCohortNow = new Map();
for (const pair of evidence) {
  const prefix = cohortNow.get(pair.to);
  if (prefix) byCohortNow.set(prefix, [...(byCohortNow.get(prefix) ?? []), pair]);
}
/** @type {import("./rules.js").Rename[]} */
const cohorts = [];
for (const [now, all] of byCohortNow) {
  let rest = all, first = 0;
  for (let round = 0; round < 3 && rest.length >= 3; round++) {
    /** @type {Map<string, number>} */
    const support = new Map();
    for (const { from } of rest) {
      const words = tokens(from);
      for (let k = 1; k < words.length; k++) bump(support, words.slice(0, k).join("_") + "_");
    }
    let best = "", most = 0, closest = -1;
    for (const [prefix, count] of support) {
      if (count < 0.6 * rest.length || count < 3) continue;
      const under = rest.filter(({ from }) => from.startsWith(prefix));
      const like = under.reduce((total, { from, to }) => total + likeness(from.slice(prefix.length), to.slice(now.length)), 0) / under.length;
      if (like > closest || (like === closest && prefix.length > best.length)) (best = prefix), (most = count), (closest = like);
    }
    if (!best) break;
    const share = most / (telling.get(best) || most);
    if (best === now) cohorts.push([best, now, most]);
    else if (share >= Math.max(1 / 6, 0.75 * first)) cohorts.push([best, now, most]), (first ||= share);
    rest = rest.filter(({ from }) => !from.startsWith(best));
  }
}
// (One prefix as it was, one cohort now: the one more of its series came to.)
const cohortRules = [...new Map(cohorts.sort((a, b) => a[2] - b[2]).map((rule) => [rule[0], rule])).values()];
const cohortIndex = index({ cohorts: cohortRules, measures: [], words: [], suffixes: [], ids: [] });
/** The cohort rule an id as it was falls under, if any. @param {string} id */
const cohortOf = (id) => cohortIndex.cohorts.find(([from]) => id.length > from.length && id.startsWith(from));

// What a pair says of measures, if it does: its id with its cohort's prefix read off, as it was and as it is (the whole
// id for a series in no cohort). One whose cohort as it was isn't the cohort it came to matched another cohort's values
// by chance; one from a cohort gone that came to a series in none matched the whole's (its series no different, every
// coin in it: `utxos_under_15y_old_`). Neither says anything.
/** @type {Map<string, number>} */
const idsOfPrefix = new Map();
for (const id of oldIds) {
  const words = tokens(id);
  for (let k = 1; k < words.length; k++) bump(idsOfPrefix, words.slice(0, k).join("_") + "_");
}
/** Whether an id as it was belongs to a cohort, by its words: twenty ids or more begin as it does. @param {string} id */
const inACohort = (id) => {
  const words = tokens(id);
  for (let k = 2; k < words.length; k++) if ((idsOfPrefix.get(words.slice(0, k).join("_") + "_") ?? 0) >= 20) return true;
  return false;
};
/** @param {Pair} pair */
function measuresOf({ from, to }) {
  const rule = cohortOf(from), now = cohortNow.get(to);
  if (rule ? rule[1] !== now : now || inACohort(from)) return null;
  return { before: rule ? from.slice(rule[0].length) : from, now: now ? to.slice(now.length) : to };
}
/** @typedef {Pair & { before: string, now: string }} MeasurePair */
/** @type {MeasurePair[]} */
const measurePairs = [];
for (const pair of evidence) {
  const measures = measuresOf(pair);
  if (measures) measurePairs.push({ ...pair, ...measures });
}

/**
 * Where two lists of words differ, each run as it was and as it is, in order (around the most words in common, in
 * order), with where it starts in the first.
 * @param {string[]} a
 * @param {string[]} b
 */
function differences(a, b) {
  const table = Array.from({ length: a.length + 1 }, () => new Array(b.length + 1).fill(0));
  for (let i = a.length - 1; i >= 0; i--) {
    for (let j = b.length - 1; j >= 0; j--) {
      table[i][j] = a[i] === b[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }
  /** @type {{ from: string[], to: string[], at: number }[]} */
  const runs = [];
  let i = 0, j = 0, startA = 0, startB = 0;
  const close = () => {
    if (i > startA || j > startB) runs.push({ from: a.slice(startA, i), to: b.slice(startB, j), at: startA });
  };
  while (i < a.length || j < b.length) {
    if (i < a.length && j < b.length && a[i] === b[j]) {
      close();
      i++, j++;
      (startA = i), (startB = j);
    } else if (j >= b.length || (i < a.length && table[i + 1][j] >= table[i][j + 1])) i++;
    else j++;
  }
  close();
  return runs;
}

// Rules to try, from the measures' differences, each with how many measures made it:
// - suffixes, a measure's last word where its values changed unit with it (`cents` to `usd` or dropped, by 0.01;
//   `sats` dropped, by 1e-8);
// - words, a run of them renamed (`addrs` to `address_count`, `matured` to `aged_out`), or added, then found by the
//   word before them (or after them, at the start);
// - and measures renamed whole (their suffix read off: `dominance` to `block_share_cumulative`, whatever its unit).
/** @param {Map<string, Set<string>>} votes @param {string} key @param {string} voter */
const vote = (votes, key, voter) => votes.set(key, (votes.get(key) ?? new Set()).add(voter));
/** @type {Map<string, Set<string>>} */
const suffixVotes = new Map(), wordVotes = new Map(), measureVotes = new Map();
for (const { before, now, factor } of measurePairs) {
  const a = tokens(before), b = tokens(now), last = factor === 1 ? undefined : differences(a, b).at(-1);
  if (last && last.at + last.from.length === a.length && last.from.length === 1 && last.to.length <= 1) {
    vote(suffixVotes, `${last.from[0]}\t${last.to.join("_")}\t${factor}`, before);
  }
}
// (Words and measures with their unit read off, as a suffix tried reads it: `dominance_ppm` is `dominance`, whatever
// else was renamed with it.)
const units = [...suffixVotes].filter(([, voters]) => voters.size >= 5).map(([key]) => key.split("\t"));
for (const { from, before, now, factor } of measurePairs) {
  let a = tokens(before), b = tokens(now), unit = 1;
  const suffix = units.find(([word, to, f]) => a.at(-1) === word && Math.abs(Number(f) - factor) <= 1e-9 * factor && (!to || b.at(-1) === to));
  if (suffix && a.length > 1) (a = a.slice(0, -1)), (b = suffix[1] ? b.slice(0, -1) : b), (unit = factor);
  for (const run of differences(a, b)) {
    const from = run.from.length ? run.from : run.at > 0 ? [a[run.at - 1]] : a.slice(0, 1);
    const to = run.from.length ? run.to : run.at > 0 ? [a[run.at - 1], ...run.to] : [...run.to, a[0]];
    if (from.length) vote(wordVotes, `${from.join("_")}\t${to.join("_")}`, before);
  }
  if (a.join("_") !== b.join("_")) vote(measureVotes, `${a.join("_")}\t${b.join("_")}\t${factor / unit}`, from);
}
/** @param {Map<string, Set<string>>} votes @param {number} least */
const candidates = (votes, least) =>
  [...votes]
    .filter(([, voters]) => voters.size >= least)
    .map(([key, voters]) => /** @type {[string[], number]} */ ([key.split("\t"), voters.size]))
    .sort((a, b) => b[1] - a[1]);

// Each rule kept only if, added to those kept, it reads more of the ids found by their values, and of the ids kept, as
// they are now than it misreads (one reading first a series the next release serves but isn't it: `in_profit` dropped,
// found where a cohort's coins are all in profit, would read every `supply_in_profit` as `supply`). Units first, then
// words, then measures whole; again until none is added, as a rule may need another to read anything right (`ratio`,
// by 100, needs `rate` read as `percent`).
/** @type {import("./rules.js").Rules} */
const rules = { cohorts: cohortRules, measures: [], words: [], suffixes: [], ids: [] };
let rulesIndex = index(rules);
/** @param {number} a @param {number} b */
const sameFactor = (a, b) => Math.abs(a - b) <= 1e-9 * Math.max(a, b);
/** What `id` reads as now: the first reading the next release serves. @param {string} id */
const now = (id) => read(id, rulesIndex).find((reading) => newSeries.has(reading.id));
const checked = evidence.filter((pair) => measuresOf(pair));
/** @param {Pair[]} affected */
const right = (affected) =>
  affected.filter(({ from, to, factor }) => {
    const reading = now(from);
    return reading && reading.id === to && sameFactor(reading.factor, factor);
  }).length;
/**
 * @param {any[]} list
 * @param {any} rule
 * @param {(id: string) => boolean} touches
 */
function tryRule(list, rule, touches) {
  const affected = checked.filter(({ from }) => touches(from));
  const before = right(affected);
  list.push(rule);
  rulesIndex = index(rules);
  if (right(affected) - before >= 2) return true;
  list.pop();
  rulesIndex = index(rules);
  return false;
}
for (let added = true; added; ) {
  added = false;
  for (const [[from, to, factor], count] of candidates(suffixVotes, 5)) {
    if (rules.suffixes.some((rule) => rule[0] === from && rule[1] === to)) continue;
    added = tryRule(rules.suffixes, [from, to, Number(factor), count], (id) => id.endsWith(`_${from}`)) || added;
  }
  for (const [[from, to], count] of candidates(wordVotes, 2)) {
    // (A rule per run of words: the first kept, made by the most measures.)
    if (rules.words.some(([other]) => other === from)) continue;
    added = tryRule(rules.words, [from, to, count], (id) => `_${id}_`.includes(`_${from}_`)) || added;
  }
  for (const [[from, to, factor], count] of candidates(measureVotes, 2)) {
    if (rules.measures.some(([other]) => other === from)) continue;
    added = tryRule(rules.measures, [from, to, Number(factor), count], (id) => `_${id}_`.includes(`_${from}_`)) || added;
  }
}

// A pair found by its values that the rules don't read: written out, if it says something of measures (see
// measuresOf), its names have something in common (a third of their words, or their spelling more than half: a
// transaction count's average and a pool's share agreed once, by 100), and it isn't only words dropped once
// (`transfer_volume_in_profit` read as `transfer_volume`, where a cohort's coins are all in profit: a coincidence,
// which a rule made by several would have told apart).
for (const pair of pairs) {
  const reading = now(pair.from);
  if (reading && reading.id === pair.to && sameFactor(reading.factor, pair.factor)) continue;
  const measures = measuresOf(pair);
  if (!measures || (likeness(pair.from, pair.to) < 1 / 3 && spelling(pair.from, pair.to) <= 0.5)) continue;
  let { before, now: after } = measures;
  // (Its unit read off both, as a suffix rule reads it.)
  const suffix = rulesIndex.suffixes.find(
    ([from, to, factor]) => before.endsWith(`_${from}`) && (!to || after.endsWith(`_${to}`)) && sameFactor(factor, pair.factor),
  );
  if (suffix) (before = before.slice(0, -suffix[0].length - 1)), (after = suffix[1] ? after.slice(0, -suffix[1].length - 1) : after);
  if (differences(tokens(before), tokens(after)).every((run) => !run.to.length)) continue;
  rules.ids.push([pair.from, pair.to, pair.factor]);
}
// (Each also alike in early 2015, where both were read there.)
console.log(`Reading the ${rules.ids.length} ids renamed one by one in early 2015:`);
const [oldDeep, newDeep] = await Promise.all([
  windows(OLD, oldSeries, rules.ids.map(([from]) => from), true),
  windows(NEW, newSeries, rules.ids.map(([, to]) => to), true),
]);
rules.ids = rules.ids.filter(([from, to, factor]) => {
  const before = oldDeep.get(from), after = newDeep.get(to);
  return !before || !after || before.index !== after.index || alike(before, after, factor);
});
rulesIndex = index(rules);

/** Each id gone that the next release serves under another, and whether its values are the same. */
/** @type {Map<string, { to: string, factor: number, values: "same" | "different" | "unknown" }>} */
const renamed = new Map();
/** @type {Set<string>} */
const removed = new Set();
for (const id of gone) {
  const reading = now(id);
  if (!reading) {
    removed.add(id);
    continue;
  }
  const before = oldWindows.get(id), after = newWindows.get(reading.id);
  const values = !before || !after || !strong(before) ? "unknown" : alike(before, after, reading.factor) ? "same" : "different";
  renamed.set(id, { to: reading.id, factor: reading.factor, values });
}
const tally = { same: 0, different: 0, unknown: 0 };
for (const { values } of renamed.values()) tally[values]++;
console.log(
  `${renamed.size} renamed (${tally.same} with the same values, ${tally.different} with others, ${tally.unknown} unknown), ` +
    `${removed.size} removed; rules: ${cohortRules.filter(([a, b]) => a !== b).length} cohorts, ${rules.measures.length} ` +
    `measures, ${rules.words.length} words, ${rules.suffixes.length} suffixes, ${rules.ids.length} ids`,
);

// ── What's gone for good, grouped ──

// By cohort, those all gone (`utxos_over_1y_old_`), then by measure, those gone from three cohorts or more, then the
// rest one by one. A cohort here is its words where the most measures follow and the most cohorts have the measure
// (the smaller of the two counts, at its largest, four at least).
/** @type {Map<string, number>} */
const prefixCount = new Map(), measureCount = new Map();
for (const id of oldIds) {
  const words = tokens(id);
  for (let k = 1; k < words.length; k++) bump(prefixCount, words.slice(0, k).join("_") + "_"), bump(measureCount, words.slice(k).join("_"));
}
/** @param {string} id @returns {[string, string]} */
function split(id) {
  const rule = cohortOf(id);
  if (rule) return [rule[0], id.slice(rule[0].length)];
  const words = tokens(id);
  /** @type {[string, string]} */
  let best = ["", id];
  for (let k = 1, score = 3; k < words.length; k++) {
    const prefix = words.slice(0, k).join("_") + "_", measure = words.slice(k).join("_");
    const count = Math.min(prefixCount.get(prefix) ?? 0, measureCount.get(measure) ?? 0);
    if (count > score) (score = count), (best = [prefix, measure]);
  }
  return best;
}
/** @type {Map<string, string[]>} */
const removedByCohort = new Map();
/** @type {Set<string>} */
const living = new Set();
for (const id of oldIds) {
  const [prefix] = split(id);
  if (!removed.has(id)) living.add(prefix);
  else if (prefix) removedByCohort.set(prefix, [...(removedByCohort.get(prefix) ?? []), id]);
}
/** @type {[string, number][]} */
const removedCohorts = [];
/** @type {Map<string, string[]>} */
const removedByMeasure = new Map();
/** @type {string[]} */
const removedIds = [];
for (const [prefix, ids] of removedByCohort) {
  if (!living.has(prefix) && ids.length >= 5) removedCohorts.push([prefix, ids.length]);
  else for (const id of ids) removedByMeasure.set(split(id)[1], [...(removedByMeasure.get(split(id)[1]) ?? []), id]);
}
for (const id of removed) if (!split(id)[0]) removedIds.push(id);
/** @type {[string, number][]} */
const removedMeasures = [];
for (const [measure, ids] of removedByMeasure) {
  if (ids.length >= 3) removedMeasures.push([measure, ids.length]);
  else removedIds.push(...ids);
}
removedCohorts.sort((a, b) => a[0].localeCompare(b[0]));
removedMeasures.sort((a, b) => a[0].localeCompare(b[0]));
removedIds.sort();

// ── Indexes ──

// Renamed: what an index of a series as it was became for the series it came to (its other indexes the same), most
// of the time and three times at least, or every time (an index of one series); gone, the rest no longer served. Every
// spelling but an index's own is gone.
const oldIndexNames = oldIndexes.map((/** @type {any} */ entry) => (typeof entry === "string" ? entry : entry.index));
const newIndexNames = new Set(newIndexes.map((/** @type {any} */ entry) => (typeof entry === "string" ? entry : entry.index)));
/** @type {Map<string, Map<string, number>>} */
const indexVotes = new Map();
for (const { from, to } of evidence) {
  const before = /** @type {Series} */ (oldSeries.get(from)).indexes, after = /** @type {Series} */ (newSeries.get(to)).indexes;
  const onlyBefore = before.filter((name) => !after.includes(name)), onlyAfter = after.filter((name) => !before.includes(name));
  if (onlyBefore.length !== onlyAfter.length) continue;
  onlyBefore.forEach((name, i) => {
    const votes = indexVotes.get(name) ?? new Map();
    indexVotes.set(name, bump(votes, onlyAfter[i]));
  });
}
/** @type {[string, string][]} */
const indexRenames = [];
/** @type {string[]} */
const indexesRemoved = [];
// (An index no series voted for, read as the series' words are: `p2a_addr_index` as `p2a_address_index`.)
const indexWords = index({ cohorts: [], measures: [], words: rules.words, suffixes: [], ids: [] });
for (const name of oldIndexNames) {
  if (newIndexNames.has(name)) continue;
  const votes = [...(indexVotes.get(name) ?? [])].sort((a, b) => b[1] - a[1]);
  const total = votes.reduce((sum, [, count]) => sum + count, 0);
  const byWords = read(name, indexWords).find((reading) => newIndexNames.has(reading.id))?.id;
  if (votes.length && ((votes[0][1] >= 3 && votes[0][1] >= total / 2) || votes.length === 1)) indexRenames.push([name, votes[0][0]]);
  else if (byWords) indexRenames.push([name, byWords]);
  else indexesRemoved.push(name);
}
const indexRename = new Map(indexRenames);
/** @type {[string, string][]} */
const aliases = oldIndexes
  .filter((/** @type {any} */ entry) => typeof entry === "object")
  .flatMap((/** @type {any} */ entry) =>
    entry.aliases.filter((/** @type {string} */ alias) => alias !== entry.index && !newIndexNames.has(alias)).map((/** @type {string} */ alias) => [alias, entry.index]),
  );

// Ids kept whose values changed under them: in another unit (multiplied by what's said), computed anew, or read as
// another JSON type; or that lost an index (its rename read as kept). Each: the id, the factor (1 if none), whether
// computed anew (1) or not (0), its JSON type as it was and as it is if they differ, and its indexes lost.
/** @type {[string, number, number, string, string[]][]} */
const changed = [];
for (const id of kept) {
  const before = /** @type {Series} */ (oldSeries.get(id)), after = /** @type {Series} */ (newSeries.get(id));
  const lost = before.indexes.map((name) => indexRename.get(name) ?? name).filter((name) => !after.indexes.includes(name));
  const a = oldWindows.get(id), b = newWindows.get(id);
  const factor = !a || !b || alike(a, b, 1) ? 1 : (FACTORS.find((f) => f !== 1 && strong(a) && alike(a, b, f)) ?? 0);
  const json = before.json === after.json ? "" : `${before.json} → ${after.json}`;
  if (factor !== 1 || json || lost.length) changed.push([id, factor || 1, factor ? 0 : 1, json, lost]);
}

// Value types renamed (`StoredU64` is `Count`): their values read alike, but typed clients name them anew. Each: as it
// was, as it is, and how many series of the same values went from one to the other.
/** @type {Map<string, number>} */
const kindRenames = new Map();
for (const { from, to, factor } of [...pairs, ...kept.map((id) => ({ from: id, to: id, factor: 1 }))]) {
  const before = /** @type {Series} */ (oldSeries.get(from)).kind, after = /** @type {Series} */ (newSeries.get(to)).kind;
  if (before && after && before !== after && factor === 1) bump(kindRenames, `${before}\t${after}`);
}
const kinds = [...kindRenames]
  .map(([key, count]) => /** @type {[string, string, number]} */ ([...key.split("\t"), count]))
  .sort((a, b) => a[0].localeCompare(b[0]) || b[2] - a[2]);

// ── Endpoints ──

// An endpoint is its method and path, the names of its path's parameters aside (a renamed one breaks no request).
// Changed: its path's parameters renamed; a parameter gone, required now, or with other bounds or values; its answer
// with a field gone, of another type, or with values it no longer takes.
const METHODS = ["get", "post", "put", "patch", "delete"];
/** @param {any} spec */
function operations(spec) {
  /** @type {Map<string, { method: string, path: string, operation: any }>} */
  const found = new Map();
  for (const [path, item] of Object.entries(spec.paths)) {
    for (const method of METHODS) {
      const operation = /** @type {any} */ (item)[method];
      if (operation) found.set(`${method} ${path.replace(/\{[^}]+\}/g, "{}")}`, { method: method.toUpperCase(), path, operation });
    }
  }
  return found;
}
/** A schema, its references followed. @param {any} spec @param {any} schema */
function resolve(spec, schema) {
  for (let hops = 0; schema?.$ref && hops < 20; hops++) schema = spec.components?.schemas?.[schema.$ref.split("/").pop()];
  return schema ?? {};
}
/** @param {any} schema */
const variantsOf = (schema) => (schema.oneOf ?? schema.anyOf)?.filter((/** @type {any} */ v) => v.type !== "null");
/** Whether a schema may be null. @param {any} schema */
const nullable = (schema) =>
  schema.nullable === true || (Array.isArray(schema.type) && schema.type.includes("null")) || (schema.oneOf ?? schema.anyOf ?? []).some((/** @type {any} */ v) => v.type === "null");
/** A schema, its references followed, and itself where it's one of itself or null. @param {any} spec @param {any} schema */
function unwrap(spec, schema) {
  for (let hops = 0; hops < 20; hops++) {
    schema = resolve(spec, schema);
    const variants = variantsOf(schema);
    if (variants?.length !== 1) return schema;
    schema = variants[0];
  }
  return schema;
}
/** What a schema is, null aside: its type, or the types it's one of. @param {any} schema */
function kindOf(schema) {
  if (Array.isArray(schema.type)) return schema.type.filter((/** @type {string} */ t) => t !== "null").join(" or ");
  if (schema.type) return schema.type;
  const variants = variantsOf(schema);
  if (variants) return variants.map((/** @type {any} */ v) => v.$ref?.split("/").pop() ?? kindOf(v)).sort().join(" or ");
  return schema.properties ? "object" : schema.items ? "array" : "any";
}
/**
 * What an answer's schema lost, from as it was (`a`) to as it is (`b`), at `path` (`answer.field[]`).
 * @param {any} a @param {any} b @param {string} path @param {string[]} notes @param {Set<string>} seen
 */
function compareAnswers(a, b, path, notes, seen, depth = 0) {
  const pairKey = `${a?.$ref}|${b?.$ref}|${path}`;
  if (depth > 8 || notes.length >= 12 || seen.has(pairKey)) return;
  seen.add(pairKey);
  const x = unwrap(oldSpec, a), y = unwrap(newSpec, b), where = `\`answer${path}\``;
  const [kindX, kindY] = [kindOf(x), kindOf(y)];
  if (kindX !== kindY && kindX !== "any" && kindY !== "any") return void notes.push(`${where}: ${kindX}, now ${kindY}`);
  if (!nullable(resolve(oldSpec, a)) && nullable(resolve(newSpec, b))) notes.push(`${where} may be null now`);
  if (x.enum && y.enum) {
    const dropped = x.enum.filter((/** @type {unknown} */ value) => !y.enum.includes(value));
    if (dropped.length) notes.push(`${where}: no longer ${some(dropped, (v) => JSON.stringify(v))}`);
  }
  for (const [name, schema] of Object.entries(x.properties ?? {})) {
    if (!y.properties?.[name]) notes.push(`\`answer${path}.${name}\` gone`);
    else compareAnswers(schema, y.properties[name], `${path}.${name}`, notes, seen, depth + 1);
  }
  if (x.items && y.items) compareAnswers(x.items, y.items, `${path}[]`, notes, seen, depth + 1);
}
/** Values said, six at most. @param {unknown[]} values @param {(value: unknown) => string} say */
const some = (values, say) =>
  values.slice(0, 6).map(say).join(", ") + (values.length > 6 ? ` and ${values.length - 6} more` : "");
/** @param {any} operation */
const answerOf = (operation) =>
  Object.entries(operation.responses ?? {}).find(([status]) => status.startsWith("2"))?.[1]?.content?.["application/json"]?.schema;
const BOUNDS = ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum", "minLength", "maxLength", "minItems", "maxItems"];

const oldOperations = operations(oldSpec), newOperations = operations(newSpec);
/** @type {[string, string, string][]} */
const endpointsRemoved = [];
/** @type {[string, string, string[]][]} */
const endpointsChanged = [];
for (const [key, { method, path, operation }] of oldOperations) {
  const now = newOperations.get(key);
  if (!now) {
    endpointsRemoved.push([method, path, operation.summary ?? ""]);
    continue;
  }
  /** @type {string[]} */
  const notes = [];
  const namesBefore = [...path.matchAll(/\{([^}]+)\}/g)].map((m) => m[1]), namesNow = [...now.path.matchAll(/\{([^}]+)\}/g)].map((m) => m[1]);
  namesBefore.forEach((name, i) => name !== namesNow[i] && notes.push(`\`{${name}}\` is \`{${namesNow[i]}}\` now`));
  const queryOf = (/** @type {any} */ op) => new Map((op.parameters ?? []).filter((/** @type {any} */ p) => p.in !== "path").map((/** @type {any} */ p) => [p.name, p]));
  const before = queryOf(operation), after = queryOf(now.operation);
  for (const [name, parameter] of before) {
    const other = after.get(name);
    if (!other) {
      notes.push(`\`${name}\` gone`);
      continue;
    }
    if (!parameter.required && other.required) notes.push(`\`${name}\` required`);
    const x = resolve(oldSpec, parameter.schema), y = resolve(newSpec, other.schema);
    for (const bound of BOUNDS) if (x[bound] !== y[bound]) notes.push(`\`${name}\` ${bound} ${x[bound] ?? "none"}, now ${y[bound] ?? "none"}`);
    const dropped = (x.enum ?? []).filter((/** @type {unknown} */ value) => y.enum && !y.enum.includes(value));
    if (dropped.length) notes.push(`\`${name}\` no longer takes ${some(dropped, (v) => `\`${v}\``)}`);
  }
  for (const [name, parameter] of after) if (parameter.required && !before.has(name)) notes.push(`\`${name}\` added, required`);
  const answerBefore = answerOf(operation), answerNow = answerOf(now.operation);
  if (answerBefore && answerNow) compareAnswers(answerBefore, answerNow, "", notes, new Set());
  if (notes.length) endpointsChanged.push([method, now.path, notes]);
}
console.log(`${endpointsRemoved.length} endpoints removed, ${endpointsChanged.length} changed; ${indexRenames.length} indexes renamed, ${indexesRemoved.length} removed`);

// ── Written, and listed on the page ──

/**
 * JSON with each row of a table on a line of its own, so the next release's changes read as a diff of this file.
 * @param {unknown} value
 * @returns {string}
 */
function format(value, indent = "") {
  const inner = indent + "  ";
  if (Array.isArray(value)) {
    if (!value.length) return "[]";
    if (value.every((item) => !item || typeof item !== "object" || Array.isArray(item))) {
      return `[\n${value.map((item) => inner + JSON.stringify(item)).join(",\n")}\n${indent}]`;
    }
    return `[\n${value.map((item) => inner + format(item, inner)).join(",\n")}\n${indent}]`;
  }
  if (value && typeof value === "object") {
    return `{\n${Object.entries(value).map(([key, item]) => `${inner}${JSON.stringify(key)}: ${format(item, inner)}`).join(",\n")}\n${indent}}`;
  }
  return JSON.stringify(value);
}
const changes = {
  from: FROM,
  to: TO,
  // (What was compared: the old release's server, and the new one's build, before it was released.)
  compared: [String(oldSpec.info.version), String(newSpec.info.version)],
  date: new Date().toISOString().slice(0, 10),
  series: { before: oldIds.length, now: newIds.length, kept: kept.length, renamed: renamed.size, ...tally, removed: removed.size },
  endpoints: { removed: endpointsRemoved, changed: endpointsChanged },
  indexes: { renamed: indexRenames, removed: indexesRemoved, aliases },
  kinds,
  rules: { ...rules, cohorts: cohortRules.sort((a, b) => a[0].localeCompare(b[0])) },
  // (Ids renamed whose values are others now: told apart in the lookup.)
  different: [...renamed].filter(([, { values }]) => values === "different").map(([id]) => id).sort(),
  removed: { cohorts: removedCohorts, measures: removedMeasures, ids: removedIds },
  changed,
};
const file = `${FROM}.json`;
writeFileSync(join(HERE, file), format(changes) + "\n");
const page = join(HERE, "index.html"), html = readFileSync(page, "utf8");
const listed = html.match(/const RELEASES = (\[.*\]);/);
if (!listed) throw new Error(`${page} has no RELEASES list`);
/** Each comparison as the page tabs it, newest first: from, to. @type {[string, string][]} */
const releases = JSON.parse(listed[1]).filter((/** @type {[string, string]} */ [from]) => from !== FROM);
writeFileSync(page, html.replace(listed[0], `const RELEASES = ${JSON.stringify([[FROM, TO], ...releases])};`));
console.log(`Wrote ${join(HERE, file)}`);
