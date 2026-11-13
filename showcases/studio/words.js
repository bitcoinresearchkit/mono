// @ts-check
// Studio's words, one module wherever they're said: the presets' tree, titles and legends (presets.js), and the page's
// heatmap and profile pickers. Windows, bands read from their keys, the band dimensions and the holder groups.

/** Words as a name or title starts them: their first letter in capital ("at least 1 BTC": "At least 1 BTC"). @param {string} text */
export const capitalized = (text) => text.charAt(0).toUpperCase() + text.slice(1);
/**
 * A name as it reads inside a sentence: its first letter lowered where it's a word's in sentence case and no later word
 * is capitalized as a name's ("Pool", not "SegWit"): "Short-term holders" "short-term holders", "At least 1 BTC" "at
 * least 1 BTC"; a name kept ("P2TR", "AntPool", "Open Assets"). @param {string} text
 */
export const inSentence = (text) => (/^[A-Z][a-z]+(?![A-Za-z])/.test(text) && !/[\s-][A-Z][a-z]+(?![A-Za-z])/.test(text) ? text.charAt(0).toLowerCase() + text.slice(1) : text);

/** Units as said, one and more, as keys spell them: a time's letter, else the key's unit. */
const UNIT_WORDS = { h: ["hour", "hours"], d: ["day", "days"], w: ["week", "weeks"], m: ["month", "months"], y: ["year", "years"], Block: ["block", "blocks"], Blocks: ["block", "blocks"], Bytes: ["byte", "bytes"], SatVb: ["sat/vB", "sat/vB"], Sats: ["sat", "sats"], Btc: ["BTC", "BTC"] };
/**
 * A span's or band's key, as the client spells it: a profit or loss band's side, its bound ("_" none, "over", "Over",
 * "under"), its edges (each a number, "0p1" 0.1, and a letter: with a unit after, a multiplier, "k" or "m"; without, a
 * unit of time, "m" a month) and its unit.
 */
const KEY = /^(profit|loss)?(_|over|Over|under)?(\d+(?:p\d+)?)([a-z]?)(?:To(\d+(?:p\d+)?)([a-z]?))?(Sats|Btc|Percent|SatVb|Bytes|Blocks?)?$/;
/** A key's number multipliers (amounts', fee rates', sizes'). */
const MULTIPLIERS = /** @type {Record<string, number>} */ ({ "": 1, k: 1e3, m: 1e6 });
/**
 * @typedef {{ value: number, written: string, unit: string }} KeyEdge an edge: its value, as the key writes it ("1K",
 * "0.1") and its unit (the key's, else its time's letter)
 * @typedef {{ side?: string, bound: string, low: KeyEdge, high?: KeyEdge }} Key
 */
/**
 * A span's or band's key, read, the one reader of them all (`_24h`, `over74Bytes`, `_0p1To0p25SatVb`, `_1yTo18m`,
 * `over100kBtc`, `profit10To20Percent`): its side, its bound ("_", "over", "under") and its edges. A key not so
 * written (a range with a bound, a letter its unit doesn't take): undefined.
 * @param {string} key @returns {Key | undefined}
 */
function readKey(key) {
  if (!READ.has(key)) READ.set(key, readOnce(key));
  return READ.get(key);
}
/** Each key's reading, once: a few hundred keys, read thousands of times as the presets are made. @type {Map<string, Key | undefined>} */
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
/** A value's unit's word, one or more (see UNIT_WORDS; a percent, none). @param {number} value @param {string} unit */
const unitWord = (value, unit) => UNIT_WORDS[/** @type {keyof typeof UNIT_WORDS} */ (unit)]?.[value === 1 ? 0 : 1] ?? "";
/**
 * A span's key as a place says it: a window's (`_24h` "24 hours", `_1w` "1 week", `_200d` "200 days"), a size's, an
 * exact one or a strict bound (`_70Bytes` "70 bytes", `_2016Blocks` "2,016 blocks", `over74Bytes` "Over 74 bytes": a
 * signature over 74 bytes is 75 and up), or a range of them in one unit (`_0p1To0p25SatVb` "0.1–0.25 sat/vB"); `after`
 * a noun, as it reads there ("of 70 bytes", "over 74 bytes"). A key it can't say, as it is. Shown, a window's said by
 * `windowSaid`, and a band that holds an edge by `band`.
 */
export function spanLabel(/** @type {string} */ key, after = false) {
  const read = readKey(key);
  if (!read || read.side || read.low.unit === "Percent" || (read.high && read.high.unit !== read.low.unit)) return key;
  const { bound, low, high } = read, last = high ?? low;
  const said = `${[low, high].flatMap((e) => (e ? [e.value.toLocaleString("en-US")] : [])).join("–")} ${unitWord(last.value, last.unit)}`;
  return bound !== "_" ? `${after ? bound : capitalized(bound)} ${said}` : after ? `of ${said}` : said;
}
/**
 * A band's edge as its places keep it, as its key writes it (`_1kTo10kSats` low "1K sats", `_1yTo18m` low "1 year",
 * `loss0To10Percent` high "10%"); none, undefined. @param {string} key @param {"low" | "high"} end
 */
export const placeEdge = (key, end) => {
  const e = readKey(key)?.[end];
  return e && (e.unit === "Percent" ? `${e.written}%` : `${e.written} ${unitWord(e.value, e.unit)}`);
};
/**
 * A trailing window as shown, in titles, legends and labels alike: a window counts fixed days, never calendar ones
 * (crates/bitview_plugin_mappings/src/lookback.rs), so its months are said as their days, 30 each (`_1m` "30 days",
 * `_6m` "180 days"); its other spans as `spanLabel` says them ("24 hours", "1 week", "1 year"). @param {string} key
 */
export const windowSaid = (key) => {
  const read = readKey(key);
  return read?.bound === "_" && read.low.unit === "m" && !read.high ? `${read.low.value * 30} days` : spanLabel(key);
};
/** A span's words as an adjective: "200 days" "200-day", "1 week" "1-week". @param {string} words */
export const adjective = (words) => words.replace(/^(\d+) (\w+?)s?$/, "$1-$2");
/** A window as an adjective, as shown (see windowSaid): `_200d` "200-day", `_1m` "30-day". @param {string} key */
export const windowAdjective = (key) => adjective(windowSaid(key));

/**
 * How a band family counts its edges, as the code that sorts coins and transactions into it does: `from`, a band holds
 * its low edge (ages, amounts, balances, losses, fee rates: "1 to under 10 sats"; from an edge up "at least" it, below
 * it "under" it); `through`, its high one (profits: "over 10% through 20%"; above an edge "over" it, below "at most"
 * it). `each`: a cutoff family's over side in its folders' and comparisons' words ("at or above each age"; under, as
 * `under` says it).
 * @typedef {{ over: string, each: string, under: string, band: (low: KeyEdge, lowSaid: string, highSaid: string) => string }} Edges
 * @type {Readonly<Record<"from" | "through", Edges>>}
 */
export const EDGES = Object.freeze({
  from: { over: "at least", each: "at or above", under: "under", band: (low, lowSaid, highSaid) => (low.value ? `${lowSaid} to under ${highSaid}` : `under ${highSaid}`) },
  through: { over: "over", each: "over", under: "at most", band: (low, lowSaid, highSaid) => `${low.value ? "over " : ""}${lowSaid} through ${highSaid}` },
});
/** A profit or loss band's side, as its key starts: how its edges count (see EDGES) and its words after. */
const BAND_SIDES = /** @type {Record<string, { counts: Edges, after: string }>} */ ({ profit: { counts: EDGES.through, after: " in profit" }, loss: { counts: EDGES.from, after: " in loss" } });
/** A count in short (1000 "1K", 100000 "100K", 0.1 "0.1"), as amounts are said. */
const SHORT = new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 2 });
/** Where amounts are said in bitcoin, not sats: from 0.01 BTC. */
const BTC_FROM = 1e6;
/**
 * A band, read from its key (see readKey): said with its edges as its family counts them (`said`, see EDGES: "1 to
 * under 10 sats", "at least 100K BTC", "1 year to under 18 months"; one value alone as it is, "0 sats", `zero` where
 * it's nothing but that; a profit or loss band by its side, see BAND_SIDES, "over 10% through 20% in profit"), and its
 * edges each alone (`low`, `high`: a cutoff's), each with its unit. An amount in sats under 0.01 BTC, else in BTC, by
 * its band's first edge. A key it can't read: undefined (its reader reports it).
 * @param {string} key @param {Edges} [family] how its family counts its edges, if its key doesn't say
 * @returns {{ said: string, low: string, high: string, zero: boolean } | undefined}
 */
export function band(key, family = EDGES.from) {
  const read = readKey(key);
  if (!read) return undefined;
  const side = read.side ? BAND_SIDES[read.side] : undefined, counts = side?.counts ?? family, { bound, low, high } = read;
  const sats = (/** @type {KeyEdge} */ e) => e.value * (e.unit === "Btc" ? 1e8 : 1), inBtc = sats(low) >= BTC_FROM;
  /** An edge, bare (its band's unit said once, after its last; a time's, its own) and with its unit. @returns {[string, string]} */
  const edge = (/** @type {KeyEdge} */ e) => {
    if (e.unit === "Sats" || e.unit === "Btc") {
      const amount = sats(e) / (inBtc ? 1e8 : 1), said = SHORT.format(amount);
      return [said, `${said} ${inBtc ? "BTC" : unitWord(amount, "Sats")}`];
    }
    // (A time's edges each say their unit: a band may run from one to another, "1 year to under 18 months".)
    const said = `${e.value.toLocaleString("en-US")}${e.unit === "Percent" ? "%" : ""}`, whole = e.unit === "Percent" ? said : `${said} ${unitWord(e.value, e.unit)}`;
    return [/^[hdwmy]$/.test(e.unit) ? whole : said, whole];
  };
  const [lowBare, lowSaid] = edge(low), highSaid = high && edge(high)[1];
  const said = highSaid ? counts.band(low, lowBare, highSaid) : bound === "under" ? `under ${lowSaid}` : bound === "_" ? lowSaid : `${counts.over} ${lowSaid}`;
  return { said: `${said}${side?.after ?? ""}`, low: lowSaid, high: highSaid ?? lowSaid, zero: !high && bound === "_" && low.value === 0 };
}

/**
 * A band dimension, its band families' and cutoffs' words alike: its noun ("age"), its places' article ("an age"), how
 * its bands count their edges (`family`, see EDGES), which edge of a band a cutoff at it names (`end`), the words its
 * bands and cutoffs add when said (`after`, " old"), how titles name what its band holds (`who`), and, for a band that's
 * exactly nothing (`zero`, see band), what it holds instead ("Zero-value UTXOs").
 * @typedef {{ noun: string, article: string, family: Edges, end: "low" | "high", after: string, who: (said: string) => string, zero?: string }} Dimension
 */
/** @type {Dimension} */
export const AGES = { noun: "age", article: "an age", family: EDGES.from, end: "low", after: " old", who: (said) => `Coins ${said}` };
/** @type {Dimension} */
export const UTXO_AMOUNTS = { noun: "UTXO amount", article: "an amount", family: EDGES.from, end: "low", after: "", who: (said) => `UTXOs of ${said}`, zero: "Zero-value UTXOs" };
/** @type {Dimension} */
export const BALANCES = { noun: "address balance", article: "a balance", family: EDGES.from, end: "low", after: "", who: (said) => `Addresses holding ${said}`, zero: "Funded addresses holding only zero-value outputs" };
/**
 * A band of a dimension, as said and shown (`said` "1 day to under 1 week old", shown capitalized) and as titles name
 * what it holds (`who` "Coins 1 day to under 1 week old"); a key `band` can't read, undefined.
 * @param {string} key @param {Dimension} d
 */
export const dimensionBand = (key, d) => {
  const read = band(key, d.family);
  if (!read) return undefined;
  const said = `${read.said}${d.after}`;
  return { said, who: read.zero && d.zero ? d.zero : d.who(said) };
};

/**
 * The holder groups (and weighted cohorts), by their keys, in the order the page lists them (all coins, then each
 * complementary pair: the terms, under 4 months and at least 4, under 6 and at least 6). Each one phrase, kept as its
 * parts: its head noun (`coins`, a set of coins; `holders`, the holders of them) and its qualifier, before it or after;
 * `place`, where its places keep an older name (the groups at least 4 and 6 months old, from when they said "over");
 * `urpd`, its URPD cohort, where it has one (the weighted ones don't). See holderName.
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
/**
 * A holder group's names, all from its phrase (see HOLDERS): `who` in titles ("Short-term holders", "Coins under 4
 * months old"); `shown` in the tree and legends (its qualifier where it follows its noun: "Under 4 months old"); `brief`
 * in a list that says what they are (holders by their term, "Short-term"; coins as shown, "All coins"); `coins`, its
 * coins ("short-term holders' coins"; all coins narrowed, "coins"); `of(kind)`, its coins of a kind ("short-term
 * holders' awake coins"); `place`, its places' name. A key it doesn't know: by its key, its place the key itself.
 * @param {string} key
 */
export function holderName(key) {
  const h = HOLDERS[key] ?? { noun: "coins", after: key, place: key };
  const phrase = [h.before, h.noun, h.after].filter(Boolean).join(" "), who = capitalized(phrase);
  const shown = h.after && !h.before ? capitalized(h.after) : who;
  const of = (/** @type {string} */ kind) => (h.noun === "holders" ? `${phrase}' ${kind}${kind ? " " : ""}coins` : [key === "all" ? "" : h.before, kind, "coins", h.after].filter(Boolean).join(" "));
  return { who, shown, brief: h.noun === "holders" && h.before ? capitalized(h.before) : shown, coins: of(""), of, place: h.place ?? shown, urpd: h.urpd };
}

/** A place's last step as shown: its label, where it has one (see studioPresets). @param {Record<string, string>} labels @param {string[]} path */
export const shownStep = (labels, path) => labels[JSON.stringify(path)] ?? /** @type {string} */ (path.at(-1));
