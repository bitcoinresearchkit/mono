// How a series id of one release reads in the next, shared by the page's lookup and diff.mjs, which finds the rules
// from the two servers and keeps only those this reads back exactly. An id is a cohort's prefix, then a measure, maybe
// ending in a suffix (its unit): a prefix is renamed whole (`utxos_2y_to_3y_old_` to `age_2y_to_3y_`), a measure whole
// or word by word (`addrs` to `address_count`), a suffix whole, its values maybe multiplied (`cents` to `usd`, by
// 0.01); an id none of them explains is written out.

/**
 * @typedef {[string, string, number]} Rename What was renamed, as it was, as it is, and how many ids it renamed.
 * @typedef {[string, string, number, number]} MeasureRename A measure or a suffix, as it was, as it is (a suffix empty:
 *   dropped), what its values were multiplied by, and how many ids it renamed. A suffix may read two ways (`cents` as
 *   `usd` where the measure counts bitcoin, dropped where it counts dollars): both are kept, the one renaming more first.
 * @typedef {MeasureRename} SuffixRename
 * @typedef {[string, string, number]} IdRename An id, as it was, as it is, and what its values were multiplied by.
 * @typedef {{ cohorts: Rename[], measures: MeasureRename[], words: Rename[], suffixes: SuffixRename[], ids: IdRename[] }} Rules
 * @typedef {{ id: string, factor: number, cohort?: Rename, measure?: MeasureRename, words?: Rename[],
 *   suffix?: SuffixRename, exact?: true }} Reading
 */

/**
 * The rules, ready to read with: cohorts longest first (`utxos_2y_to_3y_old_` before `utxos_`); suffixes longest first,
 * then those read as another before those dropped (`cents` as `usd` if the next release serves that, else dropped: a
 * measure counting bitcoin has its dollars apart, one counting dollars has nothing to drop them for), then the likeliest;
 * words by their first word.
 * @param {Rules} rules
 */
export function index(rules) {
  /** @type {Map<string, Rename[]>} */
  const words = new Map();
  for (const rule of [...rules.words].sort((a, b) => b[0].length - a[0].length)) {
    const first = rule[0].split("_")[0];
    words.set(first, [...(words.get(first) ?? []), rule]);
  }
  return {
    cohorts: [...rules.cohorts].sort((a, b) => b[0].length - a[0].length),
    measures: new Map(rules.measures.map((rule) => [rule[0], rule])),
    words,
    suffixes: [...rules.suffixes].sort((a, b) => b[0].length - a[0].length || Number(!a[1]) - Number(!b[1]) || b[3] - a[3]),
    ids: new Map(rules.ids.map((rule) => [rule[0], rule])),
  };
}

/**
 * `measure` with its words renamed, each rule where its words stand in it, the longest first; and the rules used.
 * @param {string} measure
 * @param {Map<string, Rename[]>} words
 */
function renameWords(measure, words) {
  const tokens = measure.split("_"), out = [];
  /** @type {Rename[]} */
  const used = [];
  for (let i = 0; i < tokens.length; ) {
    const rule = words.get(tokens[i])?.find(([from]) => from.split("_").every((word, j) => tokens[i + j] === word));
    if (rule) {
      if (rule[1]) out.push(rule[1]);
      used.push(rule);
      i += rule[0].split("_").length;
    } else out.push(tokens[i++]);
  }
  return { measure: out.join("_"), used };
}

/**
 * What `id` may read as in the next release, likeliest first: written out if it's an exception; else its cohort's
 * prefix renamed, then its measure renamed whole, or with each reading of its suffix (and last none) its words renamed,
 * then as they were. The first that the next release serves is the one (an id no rule touches reads as itself).
 * @param {string} id
 * @param {ReturnType<typeof index>} rules
 * @returns {Reading[]}
 */
export function read(id, rules) {
  const exact = rules.ids.get(id);
  if (exact) return [{ id: exact[1], factor: exact[2], exact: true }];
  const cohort = rules.cohorts.find(([from]) => id.length > from.length && id.startsWith(from));
  const prefix = cohort ? cohort[1] : "", rest = cohort ? id.slice(cohort[0].length) : id;
  const renamedCohort = cohort && cohort[0] !== cohort[1] ? { cohort } : {};
  /** @type {Reading[]} */
  const readings = [];
  /** @param {Reading} reading */
  const add = (reading) => readings.some((other) => other.id === reading.id) || readings.push(reading);
  const measure = rules.measures.get(rest);
  if (measure) add({ id: prefix + measure[1], factor: measure[2], ...renamedCohort, measure });
  const suffixes = rules.suffixes.filter(([from]) => rest.length > from.length + 1 && rest.endsWith(`_${from}`));
  for (const suffix of [...suffixes, undefined]) {
    const base = suffix ? rest.slice(0, -suffix[0].length - 1) : rest, tail = suffix?.[1] ? `_${suffix[1]}` : "";
    const factor = suffix ? suffix[2] : 1, withSuffix = suffix ? { suffix } : {};
    const whole = rules.measures.get(base);
    if (whole) add({ id: prefix + whole[1] + tail, factor: factor * whole[2], ...renamedCohort, measure: whole, ...withSuffix });
    const { measure: renamed, used } = renameWords(base, rules.words);
    if (used.length) add({ id: prefix + renamed + tail, factor, ...renamedCohort, words: used, ...withSuffix });
    add({ id: prefix + base + tail, factor, ...renamedCohort, ...withSuffix });
  }
  return readings;
}
