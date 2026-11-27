// Shared by the page's lookup and scripts/api-changes.mjs.

/**
 * @typedef {[string, string, number]} Rename
 * @typedef {[string, string, number, number]} MeasureRename
 * @typedef {MeasureRename} SuffixRename
 * @typedef {[string, string, number]} IdRename
 * @typedef {{ cohorts: Rename[], measures: MeasureRename[], words: Rename[], suffixes: SuffixRename[], ids: IdRename[] }} Rules
 * @typedef {{ id: string, factor: number, cohort?: Rename, measure?: MeasureRename, words?: Rename[],
 *   suffix?: SuffixRename, exact?: true }} Reading
 */

/**
 * Longest first, so `utxos_2y_to_3y_old_` is tried before `utxos_`.
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
 * Likeliest first: the first the next release serves is the one.
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
  if (id.startsWith("tx_count_")) add({ id: `${id.slice(9)}_tx_count`, factor: 1, words: [["tx_count_X", "X_tx_count", 0]] });
  return readings;
}
