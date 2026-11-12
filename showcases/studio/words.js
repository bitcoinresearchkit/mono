// @ts-check
// Studio's words, one module wherever they're said: the presets' tree, titles and legends (presets.js), and the page's
// heatmap and profile pickers. Windows, bands read from their keys, the band dimensions and the holder groups.

/** Words as a name or title starts them: their first letter in capital ("at least 1 BTC": "At least 1 BTC"). @param {string} text */
export const capitalized = (text) => `${text[0].toUpperCase()}${text.slice(1)}`;

/** Units as said, one and more, as keys spell them. */
const UNIT_WORDS = { h: ["hour", "hours"], d: ["day", "days"], w: ["week", "weeks"], m: ["month", "months"], y: ["year", "years"], Block: ["block", "blocks"], Blocks: ["block", "blocks"], Bytes: ["byte", "bytes"], SatVb: ["sat/vB", "sat/vB"] };
/**
 * A span's key as said: a window's (`_24h` "24 hours", `_1w` "1 week", `_200d` "200 days"), a size's, an exact one or a
 * strict bound (`_70Bytes` "70 bytes", `_2016Blocks` "2,016 blocks", `over74Bytes` "Over 74 bytes": a signature over 74
 * bytes is 75 and up), or a range of them (`_0p1To0p25SatVb` "0.1–0.25 sat/vB"); `after` a noun, as it reads there ("of
 * 70 bytes", "over 74 bytes"). As a place's name, a window keeps this ("1 month"); shown, it's said by `windowSaid`, and
 * a band that holds an edge by `band`.
 */
export function spanLabel(/** @type {string} */ key, after = false) {
  const match = /^(?:_|(over|under))(\d+(?:p\d+)?k?)(?:To(\d+(?:p\d+)?k?))?([hdwmy]|Blocks?|Bytes|SatVb)$/.exec(key);
  if (!match) return key;
  const [, bound, from, to, unit] = match, words = UNIT_WORDS[/** @type {keyof typeof UNIT_WORDS} */ (unit)];
  const number = (/** @type {string} */ n) => Number(n.replace("p", ".").replace("k", "")) * (n.endsWith("k") ? 1000 : 1);
  const said = `${[from, to].filter(Boolean).map((n) => number(n).toLocaleString("en-US")).join("–")} ${number(to ?? from) === 1 ? words[0] : words[1]}`;
  return bound ? `${after ? bound : capitalized(bound)} ${said}` : after ? `of ${said}` : said;
}
/**
 * A trailing window as shown, in titles, legends and labels alike: a window counts fixed days, never calendar ones
 * (crates/bitview_plugin_mappings/src/lookback.rs), so its months are said as their days, 30 each (`_1m` "30 days",
 * `_6m` "180 days"); its other spans as `spanLabel` says them ("24 hours", "1 week", "1 year"). @param {string} key
 */
export const windowSaid = (key) => {
  const months = /^_(\d+)m$/.exec(key)?.[1];
  return months ? `${Number(months) * 30} days` : spanLabel(key);
};
/** A span's words as an adjective: "200 days" "200-day", "1 week" "1-week". @param {string} words */
export const adjective = (words) => words.replace(/^(\d+) (\w+?)s?$/, "$1-$2");
/** A window as an adjective, as shown (see windowSaid): `_200d` "200-day", `_1m` "30-day". @param {string} key */
export const windowAdjective = (key) => adjective(windowSaid(key));

/**
 * How a band family counts its edges, as the code that sorts coins and transactions into it does: `from`, a band holds
 * its low edge (ages, amounts, balances, losses, fee rates: "1 to under 10 sats"; from an edge up "at least" it, below
 * it "under" it); `through`, its high one (profits: "over 10% through 20%"; above an edge "over" it, below "at most"
 * it). `each` and `eachUnder`: a cutoff family's sides, in its folders' and comparisons' words ("at or above each age").
 * @typedef {{ over: string, each: string, under: string, eachUnder: string, band: (low: string, high: string) => string }} Edges
 * @type {Readonly<Record<"from" | "through", Edges>>}
 */
export const EDGES = Object.freeze({
  from: { over: "at least", each: "at or above", under: "under", eachUnder: "under", band: (low, high) => (parseFloat(low) ? `${low} to under ${high}` : `under ${high}`) },
  through: { over: "over", each: "over", under: "at most", eachUnder: "at most", band: (low, high) => `${parseFloat(low) ? "over " : ""}${low} through ${high}` },
});
/**
 * A band's key, as the client spells it: its side ("profit", "loss"), its bound ("over", "Over", "under"; "_" none), its
 * edges (each a number, "0p1" 0.1, and its letters: a multiplier, "k" or "m", or a unit of time, "y") and its family's unit.
 */
const BAND_KEY = /^(profit|loss)?(_|over|Over|under)?(\d+(?:p\d+)?)([a-z]*?)(?:To(\d+(?:p\d+)?)([a-z]*?))?(Sats|Btc|Percent|SatVb)?$/;
/** A profit or loss band's side, as its key starts: how its edges count (see EDGES) and its words after. */
const BAND_SIDES = /** @type {Record<string, { counts: Edges, after: string }>} */ ({ profit: { counts: EDGES.through, after: " in profit" }, loss: { counts: EDGES.from, after: " in loss" } });
/** A count in short (1000 "1K", 100000 "100K", 0.1 "0.1"), as amounts are said. */
const SHORT = new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 2 });
/** A band key's number multipliers (amounts', fee rates'). */
const MULTIPLIERS = /** @type {Record<string, number>} */ ({ "": 1, k: 1e3, m: 1e6 });
/** Where amounts are said in bitcoin, not sats: from 0.01 BTC. */
const BTC_FROM = 1e6;
/**
 * A band, read from its key (`_1To10Sats`, `over100kBtc`, `_1yTo18m`, `profit10To20Percent`, `_0p1To0p25SatVb`): said
 * with its edges as its family counts them (`said`, see EDGES: "1 to under 10 sats", "at least 100K BTC", "1 year to
 * under 18 months"; one value alone as it is, "0 sats", `zero` where it's nothing but that; a profit or loss band by its
 * side, see BAND_SIDES, "over 10% through 20% in profit"), and its edges each alone (`low`, `high`: a cutoff's), each
 * with its unit. An amount in sats under 0.01 BTC, else in BTC, by its band's first edge. A key it can't read:
 * undefined (its reader reports it).
 * @param {string} key @param {Edges} [family] how its family counts its edges, if its key doesn't say
 * @returns {{ said: string, low: string, high: string, zero: boolean } | undefined}
 */
export function band(key, family = EDGES.from) {
  const [, sideKey, boundKey = "_", a, aLetters = "", b, bLetters = "", unit = ""] = BAND_KEY.exec(key) ?? [];
  const side = sideKey ? BAND_SIDES[sideKey] : undefined, counts = side?.counts ?? family, bound = boundKey.toLowerCase(), after = side?.after ?? "";
  const number = (/** @type {string} */ n, /** @type {string} */ letters) => Number(n.replace("p", ".")) * (MULTIPLIERS[letters] ?? NaN);
  const satsOf = (/** @type {string} */ n, /** @type {string} */ letters) => number(n, letters) * (unit === "Btc" ? 1e8 : 1);
  const inBtc = a !== undefined && satsOf(a, aLetters) >= BTC_FROM;
  /** An edge, bare (its band's unit said once, after its last) and with its unit; none, unreadable. @returns {[string, string] | undefined} */
  const edge = (/** @type {string} */ n, /** @type {string} */ letters) => {
    if (unit === "Sats" || unit === "Btc") {
      const amount = satsOf(n, letters) / (inBtc ? 1e8 : 1), said = SHORT.format(amount);
      return Number.isFinite(amount) ? [said, `${said} ${inBtc ? "BTC" : amount === 1 ? "sat" : "sats"}`] : undefined;
    }
    if (unit === "Percent") return letters ? undefined : [`${number(n, "").toLocaleString("en-US")}%`, `${number(n, "").toLocaleString("en-US")}%`];
    if (unit === "SatVb") {
      const value = number(n, letters);
      return Number.isFinite(value) ? [value.toLocaleString("en-US"), `${value.toLocaleString("en-US")} sat/vB`] : undefined;
    }
    const words = /^[hdwmy]$/.test(letters) ? UNIT_WORDS[/** @type {keyof typeof UNIT_WORDS} */ (letters)] : undefined, value = number(n, "");
    return words ? [`${value} ${words[value === 1 ? 0 : 1]}`, `${value} ${words[value === 1 ? 0 : 1]}`] : undefined;
  };
  const low = a === undefined ? undefined : edge(a, aLetters), high = b === undefined ? undefined : edge(b, bLetters);
  if (!low || (b !== undefined && (!high || bound !== "_"))) return undefined;
  const said = high ? counts.band(low[0], high[1]) : bound === "under" ? `under ${low[1]}` : bound === "_" ? low[1] : `${counts.over} ${low[1]}`;
  return { said: `${said}${after}`, low: low[1], high: (high ?? low)[1], zero: !high && bound === "_" && number(a, aLetters) === 0 };
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
 * `short`, its abbreviation; `urpd`, its URPD cohort, where it has one (the weighted ones don't). See holderName.
 * @typedef {{ before?: string, noun: "coins" | "holders", after?: string, place?: string, short?: string, urpd?: string }} Holder
 * @type {Readonly<Record<string, Holder>>}
 */
export const HOLDERS = Object.freeze({
  all: { before: "all", noun: "coins", urpd: "all" },
  sth: { before: "short-term", noun: "holders", short: "STH", urpd: "sth" },
  lth: { before: "long-term", noun: "holders", short: "LTH", urpd: "lth" },
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
 * in a list that says what they are ("Short-term"); `coins`, its coins ("short-term holders' coins"; all coins narrowed,
 * "coins"); `of(kind)`, its coins of a kind ("short-term holders' awake coins"); `place`, its places' name. A key it
 * doesn't know: by its key.
 * @param {string} key
 */
export function holderName(key) {
  const h = HOLDERS[key] ?? { noun: "coins", after: key };
  const phrase = [h.before, h.noun, h.after].filter(Boolean).join(" "), who = capitalized(phrase);
  const shown = h.after && !h.before ? capitalized(h.after) : who;
  const of = (/** @type {string} */ kind) => (h.noun === "holders" ? `${phrase}' ${kind}${kind ? " " : ""}coins` : [key === "all" ? "" : h.before, kind, "coins", h.after].filter(Boolean).join(" "));
  return { who, shown, brief: capitalized(h.before ?? h.after ?? key), coins: of(""), of, place: h.place ?? shown, short: h.short, urpd: h.urpd };
}

/** A place's last step as shown: its label, where it has one (see studioPresets). @param {Record<string, string>} labels @param {string[]} path */
export const shownStep = (labels, path) => labels[JSON.stringify(path)] ?? /** @type {string} */ (path.at(-1));
