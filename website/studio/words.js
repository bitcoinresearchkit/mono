// @ts-check
// Studio's words, one module wherever they're said: the presets' tree, titles and legends (presets.js), and the page's
// heatmap and profile pickers. Windows, bands read from their keys, the band dimensions and the holder groups.

/** @param {string} text */
export const capitalized = (text) => text.charAt(0).toUpperCase() + text.slice(1);
/** @param {string} text */
export const inSentence = (text) => (/^[A-Z][a-z]+(?![A-Za-z])/.test(text) && !/[\s-][A-Z][a-z]+(?![A-Za-z])/.test(text) ? text.charAt(0).toLowerCase() + text.slice(1) : text);

const UNIT_WORDS = { h: ["hour", "hours"], d: ["day", "days"], w: ["week", "weeks"], m: ["month", "months"], y: ["year", "years"], Block: ["block", "blocks"], Blocks: ["block", "blocks"], Bytes: ["byte", "bytes"], SatVb: ["sat/vB", "sat/vB"], Sats: ["sat", "sats"], Btc: ["BTC", "BTC"] };
const KEY = /^(profit|loss)?(_|over|Over|under)?(\d+(?:p\d+)?)([a-z]?)(?:To(\d+(?:p\d+)?)([a-z]?))?(Sats|Btc|Percent|SatVb|Bytes|Blocks?)?$/;
const MULTIPLIERS = /** @type {Record<string, number>} */ ({ "": 1, k: 1e3, m: 1e6 });
/**
 * @typedef {{ value: number, written: string, unit: string }} KeyEdge an edge: its value, as the key writes it ("1K",
 * "0.1") and its unit (the key's, else its time's letter)
 * @typedef {{ side?: string, bound: string, low: KeyEdge, high?: KeyEdge }} Key
 */
/** @param {string} key @returns {Key | undefined} */
function readKey(key) {
  if (!READ.has(key)) READ.set(key, readOnce(key));
  return READ.get(key);
}
/**
 * Cached: read thousands of times as the presets are made.
 * @type {Map<string, Key | undefined>}
 */
const READ = new Map();
/** @param {string} key @returns {Key | undefined} */
function readOnce(key) {
  const [, side, bound = "_", a, aLetter = "", b, bLetter = "", unit = ""] = KEY.exec(key) ?? [];
  /** @returns {KeyEdge | undefined} */
  const edge = (/** @type {string} */ n, /** @type {string} */ letter) => {
    const number = Number(n.replace("p", ".")), written = `${n.replace("p", ".")}${letter.toUpperCase()}`;
    if (unit) return Object.hasOwn(MULTIPLIERS, letter) && !(unit === "Percent" && letter) ? { value: number * MULTIPLIERS[letter], written, unit } : undefined;
    return /^[hdwmy]$/.test(letter) ? { value: number, written: n, unit: letter } : undefined;
  };
  const low = a === undefined ? undefined : edge(a, aLetter), high = b === undefined ? undefined : edge(b, bLetter);
  if (!low || (b !== undefined && (!high || bound !== "_"))) return undefined;
  return { side, bound: bound.toLowerCase(), low, high };
}
/** @param {number} value @param {string} unit */
const unitWord = (value, unit) => UNIT_WORDS[/** @type {keyof typeof UNIT_WORDS} */ (unit)]?.[value === 1 ? 0 : 1] ?? "";
export function spanLabel(/** @type {string} */ key, after = false) {
  const read = readKey(key);
  if (!read || read.side || read.low.unit === "Percent" || (read.high && read.high.unit !== read.low.unit)) return key;
  const { bound, low, high } = read, last = high ?? low;
  const said = `${[low, high].flatMap((e) => (e ? [e.value.toLocaleString("en-US")] : [])).join("–")} ${unitWord(last.value, last.unit)}`;
  return bound !== "_" ? `${after ? bound : capitalized(bound)} ${said}` : after ? `of ${said}` : said;
}
/** @param {string} key @param {"low" | "high"} end */
export const placeEdge = (key, end) => {
  const e = readKey(key)?.[end];
  return e && (e.unit === "Percent" ? `${e.written}%` : `${e.written} ${unitWord(e.value, e.unit)}`);
};
/** @param {string} key */
export const isWindowKey = (key) => ((read) => Boolean(read && read.bound === "_" && !read.side && !read.high && "hdwmy".includes(read.low.unit)))(readKey(key));
// `by` is made on first read, not for every series a walk passes.
export const isLeaf = (node) => Boolean(node) && typeof node.name === "string" && typeof node.indexes === "function";
/**
 * A window counts fixed days, never calendar ones (lookback.rs): its months are said as 30 days each.
 * @param {string} key
 */
export const windowSaid = (key) => {
  const read = readKey(key);
  return read?.bound === "_" && read.low.unit === "m" && !read.high ? `${read.low.value * 30} days` : spanLabel(key);
};
/** @param {string} key */
export const windowShort = (key) => windowSaid(key).replace(/^(\d+) (\w)\w*$/, "$1$2");
/** @param {string} words @param {string[]} keys */
export const windowNamed = (words, keys) => ((said) => keys.find((key) => windowShort(key).toLowerCase() === said))(words.trim().toLowerCase());
/** @param {string} words */
export const adjective = (words) => words.replace(/^(\d+) (\w+?)s?$/, "$1-$2");
/** @param {string} key */
export const windowAdjective = (key) => adjective(windowSaid(key));

/**
 * `from`: a band holds its low edge (ages, amounts, balances, losses, fee rates); `through`: its high one (profits).
 * @typedef {{ over: string, each: string, under: string, band: (low: KeyEdge, lowSaid: string, highSaid: string) => string }} Edges
 * @type {Readonly<Record<"from" | "through", Edges>>}
 */
export const EDGES = Object.freeze({
  from: { over: "at least", each: "at or above", under: "under", band: (low, lowSaid, highSaid) => (low.value ? `${lowSaid} to under ${highSaid}` : `under ${highSaid}`) },
  through: { over: "over", each: "over", under: "at most", band: (low, lowSaid, highSaid) => `${low.value ? "over " : ""}${lowSaid} through ${highSaid}` },
});
const BAND_SIDES = /** @type {Record<string, { counts: Edges, after: string }>} */ ({ profit: { counts: EDGES.through, after: " in profit" }, loss: { counts: EDGES.from, after: " in loss" } });
const SHORT = new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 2 });
const BTC_FROM = 1e6;
/** @param {string} key @param {Edges} [family] @returns {{ said: string, low: string, high: string, zero: boolean } | undefined} */
export function band(key, family = EDGES.from) {
  const read = readKey(key);
  if (!read) return undefined;
  const side = read.side ? BAND_SIDES[read.side] : undefined, counts = side?.counts ?? family, { bound, low, high } = read;
  const sats = (/** @type {KeyEdge} */ e) => e.value * (e.unit === "Btc" ? 1e8 : 1), inBtc = sats(low) >= BTC_FROM;
  /** @returns {[string, string]} */
  const edge = (/** @type {KeyEdge} */ e) => {
    if (e.unit === "Sats" || e.unit === "Btc") {
      const amount = sats(e) / (inBtc ? 1e8 : 1), said = SHORT.format(amount);
      return [said, `${said} ${inBtc ? "BTC" : unitWord(amount, "Sats")}`];
    }
    // A time's edges each say their unit: a band may run from one to another ("1 year to under 18 months").
    const said = `${e.value.toLocaleString("en-US")}${e.unit === "Percent" ? "%" : ""}`, whole = e.unit === "Percent" ? said : `${said} ${unitWord(e.value, e.unit)}`;
    return [/^[hdwmy]$/.test(e.unit) ? whole : said, whole];
  };
  const [lowBare, lowSaid] = edge(low), highSaid = high && edge(high)[1];
  const said = highSaid ? counts.band(low, lowBare, highSaid) : bound === "under" ? `under ${lowSaid}` : bound === "_" ? lowSaid : `${counts.over} ${lowSaid}`;
  return { said: `${said}${side?.after ?? ""}`, low: lowSaid, high: highSaid ?? lowSaid, zero: !high && bound === "_" && low.value === 0 };
}

/** @typedef {{ noun: string, article: string, family: Edges, end: "low" | "high", after: string, who: (said: string) => string, zero?: string }} Dimension */
/** @type {Dimension} */
export const AGES = { noun: "age", article: "an age", family: EDGES.from, end: "low", after: " old", who: (said) => `Coins ${said}` };
/** @type {Dimension} */
export const UTXO_AMOUNTS = { noun: "UTXO amount", article: "an amount", family: EDGES.from, end: "low", after: "", who: (said) => `UTXOs of ${said}`, zero: "Zero-value UTXOs" };
/** @type {Dimension} */
export const BALANCES = { noun: "address balance", article: "a balance", family: EDGES.from, end: "low", after: "", who: (said) => `Addresses holding ${said}`, zero: "Funded addresses holding only zero-value outputs" };
/** @param {string} key @param {Dimension} d */
export const dimensionBand = (key, d) => {
  const read = band(key, d.family);
  if (!read) return undefined;
  const said = `${read.said}${d.after}`;
  return { said, who: read.zero && d.zero ? d.zero : d.who(said) };
};

/**
 * `place`: where its places keep an older name (the groups at least 4 and 6 months old, from when they said "over");
 * `urpd`: its URPD cohort, none for the weighted ones.
 * @typedef {{ before?: string, noun: "coins" | "holders", after?: string, place?: string, urpd?: string }} Holder
 * @type {Readonly<Record<string, Holder>>}
 */
export const HOLDERS = Object.freeze({
  all: { before: "all", noun: "coins", urpd: "all" },
  sth: { before: "short-term", noun: "holders", urpd: "sth" },
  lth: { before: "long-term", noun: "holders", urpd: "lth" },
  under4m: { noun: "coins", after: "under 4 months old", urpd: "age_under_4m" },
  over4m: { noun: "coins", after: "at least 4 months old", place: "Over 4 months old", urpd: "age_over_4m" },
  under6m: { noun: "coins", after: "under 6 months old", urpd: "age_under_6m" },
  over6m: { noun: "coins", after: "at least 6 months old", place: "Over 6 months old", urpd: "age_over_6m" },
  awake: { before: "awake", noun: "coins" },
  mobile: { before: "mobile", noun: "coins" },
});
/** @param {string} key */
export function holderName(key) {
  const h = HOLDERS[key] ?? { noun: "coins", after: key, place: key };
  const phrase = [h.before, h.noun, h.after].filter(Boolean).join(" "), who = capitalized(phrase);
  const shown = h.after && !h.before ? capitalized(h.after) : who;
  const of = (/** @type {string} */ kind) => (h.noun === "holders" ? `${phrase}' ${kind}${kind ? " " : ""}coins` : [key === "all" ? "" : h.before, kind, "coins", h.after].filter(Boolean).join(" "));
  const whole = h.noun === "holders"
    ? { coins: `${phrase}' coins`, capital: `${phrase}' capital` }
    : { coins: phrase, capital: h.before ? `${h.before} capital` : `the capital of ${phrase}` };
  return { who, shown, brief: h.noun === "holders" && h.before ? capitalized(h.before) : shown, coins: of(""), of, whole, place: h.place ?? shown, urpd: h.urpd };
}

/** @param {Record<string, string>} labels @param {string[]} path */
export const shownStep = (labels, path) => labels[JSON.stringify(path)] ?? /** @type {string} */ (path.at(-1));
