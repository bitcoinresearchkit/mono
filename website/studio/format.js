// @ts-check
// Numbers, units, dates and words as Studio writes them: values, axes (their labels kept: each frame of a move asks
// again), durations, months, plurals, units' signs and names, the formats a layer can be read in.
import { clamp } from "/studio/math.js";
import { capitalized } from "/studio/words.js";

/** Each value type Bitview has, by the unit it's in (a type not here, or a bare number, a plain number). */
const TYPE_UNITS = {
  Dollars: "usd", OHLCDollars: "usd", Bitcoin: "btc", Sats: "sats", Percent: "percent", Multiple: "ratio",
  Count: "count", CountFract: "count", CountSigned: "count", Height: "count", Epoch: "count", Halving: "count",
  Days: "days", Seconds: "seconds", SecondsFract: "seconds", PerSecond: "persecond", PerDay: "perday",
  Bytes: "bytes", BytesFract: "bytes", VSize: "vbytes", Weight: "weight", WeightFract: "weight", SigOps: "sigops",
  Hashrate: "hashrate", Difficulty: "difficulty", FeeRate: "feerate", HashPrice: "hashprice", HashValue: "hashvalue",
  Work: "number", ExpectedCount: "count",
  CoinDays: "coindays", CoinBlocks: "coinblocks", DollarBlocks: "dollarblocks", DollarDaysPerCoin: "dollardays",
};
/** A series' unit, by its value type as Bitview names it (Dollars, Bitcoin, Percent, Bytes…). */
const unitOf = (/** @type {string} */ type) => TYPE_UNITS[/** @type {keyof typeof TYPE_UNITS} */ (type)] ?? "number";
/**
 * A thousand's powers' letters: a count's (K M B T), and SI's (k M G T) for the units said with a sign of their own
 * (1.587 MB, 976.1 EH/s), which `precise`, `axisTicks` and `format` take from the unit.
 */
const COUNT_LETTERS = ["", "K", "M", "B", "T", "P", "E", "Z", "Y"], SI_LETTERS = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];
const SI_UNITS = new Set(["bytes", "vbytes", "weight", "hashrate"]);
const lettersOf = (/** @type {string | undefined} */ unit) => (unit && SI_UNITS.has(unit) ? SI_LETTERS : COUNT_LETTERS);
/** Every letter either table has, for reading a label back: made from them, so it can't drift. */
const LETTERS = [...new Set([...COUNT_LETTERS, ...SI_LETTERS].join(""))].join("");
const EVEN_PARTS = new RegExp(`^(−?)(\\d+)(?:\\.(\\d+))?([${LETTERS}]?)(.*)$`), LABEL_PARTS = new RegExp(`^(−?)([\\d.]+(?:e[-+]?\\d+)?)([${LETTERS}]?)`);
/** A number with a letter from a thousand up, in at most `room` characters (the sign aside). */
function compact(value, digits = 3, room = 6, letters = COUNT_LETTERS) {
  const sign = value < 0 ? "−" : "", abs = Math.abs(value), small = abs < 1000 ? trim(abs, digits, room) : "";
  // (By the value as written: 999.6 rounds to a thousand, written 1K.)
  if (small && Number(small) < 1000) return sign + small;
  let tier = Math.min(letters.length - 1, Math.floor(Math.log10(abs) / 3));
  if (Number(trim(abs / 1000 ** tier, digits)) >= 1000 && tier < letters.length - 1) tier++;
  return `${sign}${trim(abs / 1000 ** tier, digits, room - 1)}${letters[tier]}`;
}
/**
 * Round 100 and up to integers; below, try `digits` significant digits in `room` characters (sign aside; powers
 * of ten if tight).
 */
function trim(value, digits, room = 6) {
  const abs = Math.abs(value), sign = value < 0 ? "−" : "";
  if (abs === 0) return "0";
  if (abs >= 100) return sign + abs.toFixed(0);
  const bare = (text) => (text.includes(".") ? text.replace(/\.?0+$/, "") : text);
  for (let p = digits; p >= 1; p--) {
    // Past a hundred places (beyond toFixed, and any room), only as a power of ten.
    const places = Math.max(0, p - 1 - Math.floor(Math.log10(abs)));
    const plain = places <= 100 ? bare(abs.toFixed(places)) : "";
    if (plain && plain.length <= room) return sign + plain;
    const power = abs.toExponential(p - 1).replace(/\.?0+e/, "e");
    if (power.length <= room) return sign + power;
  }
  return sign + abs.toExponential(0);
}
const grouped = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });
/** Dollars to the cent, as a live price is read: $97,123.45. */
const toTheCent = new Intl.NumberFormat("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const dollarsAndCents = (/** @type {number} */ value) => `$${toTheCent.format(value)}`;
/**
 * A value in its unit, as the legend and the panel write it: its figure as everywhere (precise, by the value, not
 * the unit), the unit's sign around it.
 */
function format(value, unit) {
  if (value === undefined || value === null || !Number.isFinite(value)) return "—";
  // (A signal's state: above 0 up, below down, at 0 off.)
  if (unit === "signal") return value > 0 ? "Up" : value < 0 ? "Down" : "Off";
  // (A true minus, as the ruler writes it, outside the unit's sign: −$1,234.)
  const text = precise(Math.abs(value), "", unit), sign = value < 0 && text !== "0" ? "−" : "";
  // (A unit said with an SI prefix (H/s, B, vB, WU) takes the figure's letter onto it: 976.1 EH/s, 1.587 MB,
  // 3.927 MWU, a thousand's k, a billion's G.)
  const prefixed = (/** @type {string} */ symbol) => (SI_LETTERS.indexOf(text.slice(-1)) > 0 ? `${sign}${text.slice(0, -1)} ${text.slice(-1)}${symbol}` : `${sign}${text} ${symbol}`);
  switch (unit) {
    case "usd":
      return `${sign}$${text}`;
    case "btc":
      return `${sign}₿${text}`;
    case "sats":
      return `${sign}${text} sat`;
    case "percent":
      return `${sign}${text}%`;
    case "ratio":
      return `${sign}${text}×`;
    case "days":
      return `${sign}${text} ${text === "1" ? "day" : "days"}`;
    case "seconds":
      return `${sign}${text} s`;
    case "minutes":
      return `${sign}${text} min`;
    case "persecond":
      return `${sign}${text}/s`;
    case "perday":
      return `${sign}${text}/day`;
    case "bytes":
      return prefixed("B");
    case "vbytes":
      return prefixed("vB");
    case "weight":
      return prefixed("WU");
    case "sigops":
      return `${sign}${text} sigops`;
    case "hashrate":
      return prefixed("H/s");
    case "feerate":
      return `${sign}${text} sat/vB`;
    case "hashprice":
      return `${sign}$${text}/PH/s/day`;
    case "hashvalue":
      return `${sign}${text} sat/PH/s/day`;
    case "coindays":
      return `${sign}${text} coin days`;
    case "coinblocks":
      return `${sign}${text} coin blocks`;
    case "dollarblocks":
      return `${sign}$${text} × blocks`;
    case "dollardays":
      return `${sign}$${text} × days/₿`;
    default:
      return `${sign}${text}`;
  }
}
/** An axis label is at most this many characters, its sign aside; the price scales at most that wide, and a sign. */
const AXIS_CHARS = 6;
/** The axes' labels, by value and suffix: the chart asks for the same ticks every frame of a move. */
const axisLabels = new Map();
/**
 * Axes omit units because a scale can mix them (the legend gives each; a percent-change scale's % belongs to
 * all).
 */
function axisLabel(value, suffix = "", unit = "") {
  const letters = lettersOf(unit), key = `${value}${suffix}${letters[1]}`, known = axisLabels.get(key);
  if (known !== undefined) return known;
  if (axisLabels.size > 4000) axisLabels.clear();
  // The library steps its ticks by subtraction: the zero one can come out as 6e-17. Below the scales' least move,
  // it's 0.
  // (Its sign aside, as a precise figure's: −0.0002 as 0.0002, not 2e-4 beside it.)
  const shown = Math.abs(value) < 5e-10 ? 0 : value, sign = shown < 0 ? "−" : "";
  const label = `${sign}${compact(Math.abs(shown), 3, AXIS_CHARS - suffix.length, letters)}${suffix}`;
  axisLabels.set(key, label);
  return label;
}
/**
 * An axis's tick labels, axisLabel's, each read as its tick: where three digits write one off (zoomed in close,
 * 100.05 and 100.1 both "100"; 1.605T "1.61T"), the fewest digits that write each within a hundredth of the
 * ticks' least gap, letters as axisLabel's, a letter's decimals alike (100.00, 100.05; 99.8K, 100.0K), each in its
 * own unit (900 under 1.0K, 250K under 1.00M: not 0.9K, 0.25M), while they fit the scale's width; past it, a tick
 * written as the one before is left blank.
 */
function axisTicks(/** @type {number[]} */ values, suffix = "", unit = "") {
  const letters = lettersOf(unit), labels = values.map((value) => axisLabel(value, suffix, unit));
  let gap = Infinity, most = 0;
  for (let i = 1; i < values.length; i++) (gap = Math.min(gap, Math.abs(values[i] - values[i - 1]))), (most = Math.max(most, Math.abs(values[i] - values[i - 1])));
  if (!gap) return labels;
  const near = (/** @type {number[]} */ read) => read.every((value, i) => Math.abs(value - values[i]) <= gap / 100);
  const fit = (/** @type {string[]} */ texts) => texts.every((text) => text.replace(/^−/, "").length <= AXIS_CHARS && !text.includes("e"));
  // (Ticks evenly apart (a linear scale's), a letter's decimals alike: 2.50M beside 2.75M, not 2.5M; 1.0K beside
  // 1.1K. Each one's own letter: 900 under 1.0K, not 0.9K. A log scale's decades each as they are: 300, 30, 0.3.)
  const evened = (/** @type {string[]} */ texts) => {
    const parts = texts.map((text) => EVEN_PARTS.exec(text)), places = /** @type {Map<string, number>} */ (new Map());
    if (parts.some((part) => !part)) return texts;
    for (const part of /** @type {RegExpExecArray[]} */ (parts)) places.set(part[4], Math.max(places.get(part[4]) ?? 0, (part[3] ?? "").length));
    return /** @type {RegExpExecArray[]} */ (parts).map(([, sign, whole, decimals = "", letter, rest]) => `${sign}${whole}${places.get(letter) ? `.${decimals.padEnd(/** @type {number} */ (places.get(letter)), "0")}` : ""}${letter}${rest}`);
  };
  if (near(labels.map((label) => readLabel(label, letters)))) return most - gap <= gap / 100 ? ((even) => (fit(even) ? even : labels))(evened(labels)) : labels;
  for (let digits = 4; digits <= 9; digits++) {
    // (With letters, as axisLabel's; where those are too long (12.345K), in plain digits (12345).)
    for (const lettered of [true, false]) {
      const parts = values.map((value) => {
        const abs = Math.abs(value) < 5e-10 ? 0 : Math.abs(value), sign = abs && value < 0 ? "−" : "";
        let tier = abs && lettered ? clamp(Math.floor(Math.log10(abs) / 3), 0, letters.length - 1) : 0;
        if (lettered && Number((abs / 1000 ** tier).toPrecision(digits)) >= 1000 && tier < letters.length - 1) tier++;
        return { sign, tier, mantissa: Number((abs / 1000 ** tier).toPrecision(digits)) };
      });
      if (!near(parts.map(({ sign, tier, mantissa }) => (sign ? -1 : 1) * mantissa * 1000 ** tier))) break;
      const places = /** @type {Map<number, number>} */ (new Map());
      for (const { tier, mantissa } of parts) places.set(tier, Math.max(places.get(tier) ?? 0, (String(mantissa).split(".")[1] ?? "").length));
      const even = parts.map(({ sign, tier, mantissa }) => `${sign}${mantissa.toFixed(places.get(tier))}${letters[tier]}${suffix}`);
      if (fit(even)) return even;
      const bare = parts.map(({ sign, tier, mantissa }) => `${sign}${mantissa}${letters[tier]}${suffix}`);
      if (fit(bare)) return bare;
    }
  }
  // (Too close to part in the scale's width (100.001, 100.002): a label only where it reads true for its tick (one
  // rounded onto its neighbor's value, blank), each the same as the one before left blank.)
  // (None reads true: each the same as the one before left blank, as before.)
  const kept = labels.map((label, i) => Math.abs(readLabel(label, letters) - values[i]) <= gap / 2);
  if (!kept.some(Boolean)) return labels.map((label, i) => (i && label === labels[i - 1] ? "" : label));
  let said = "";
  return labels.map((label, i) => (kept[i] && label !== said ? (said = label) : ""));
}
/** A label's value as written (its sign, digits and letter; a suffix after). */
function readLabel(/** @type {string} */ label, /** @type {string[]} */ letters) {
  const [, sign, digits, letter] = LABEL_PARTS.exec(label) ?? [];
  return digits === undefined ? NaN : (sign ? -1 : 1) * Number(digits) * 1000 ** Math.max(0, letters.indexOf(letter));
}
/** Four significant digits, as written (decimal rounding: 823.05 is 823.1), no grouping: precise's figures under a thousand. */
const figures = new Intl.NumberFormat("en-US", { maximumSignificantDigits: 4, useGrouping: false });
/** Four digits down to one, with a power of ten, rounded as figures are (see scientific). */
const SCIENTIFIC = [4, 3, 2, 1].map((digits) => new Intl.NumberFormat("en-US", { notation: "scientific", maximumSignificantDigits: digits }));
/**
 * A value with a power of ten, rounded as figures are (0.00012345 is 1.235e-4), in as many of four digits as PRECISE
 * holds (1.23e-10): too small, or too large, for figures there. @param {number} abs
 */
function scientific(abs) {
  let text = "";
  for (const format of SCIENTIFIC) if ((text = format.format(abs).replace("E", "e")).length <= PRECISE.length) break;
  return text;
}
/** The widest a precise figure is, its sign and suffix aside (0.001234, 1.234e-5, 123,456). */
const PRECISE = "0.000000";
/**
 * A value at the widest each of precise's forms writes (its figures read as zeros, see widest): a fraction's
 * (0.001234: a smaller one as wide, 1.234e-5), thousands grouped, a letter's. What a reading's room is measured by,
 * whatever its value.
 */
const WIDEST_VALUES = [0.001234, 888_888, 888.8e6];
/**
 * One value, the same text everywhere (legend, panel, axis, crosshair and drawings), with its unit affixed by
 * format: four significant digits, thousands whole and grouped, millions and up lettered, never wider than
 * PRECISE (its sign and suffix aside): a smaller fraction said shorter, its four digits kept (1.234e-5).
 * @param {number} value @param {string} [suffix] @param {string} [unit] its letters (see lettersOf)
 */
function precise(value, suffix = "", unit = "") {
  const letters = lettersOf(unit), key = `p${value}|${suffix}|${letters[1]}`, known = axisLabels.get(key);
  if (known !== undefined) return known;
  if (axisLabels.size > 4000) axisLabels.clear();
  const abs = Math.abs(value), sign = value < 0 && abs ? "−" : "";
  // (A thousand and up in four digits and a letter: 123.5M, 1.235B.)
  const lettered = (/** @type {number} */ n) => {
    let tier = Math.min(letters.length - 1, Math.floor(Math.log10(n) / 3)), written = figures.format(n / 1000 ** tier);
    if (Number(written) >= 1000 && tier < letters.length - 1) written = figures.format(n / 1000 ** ++tier);
    return `${written}${letters[tier]}`;
  };
  // (Each threshold by the value as written: 999,999.6 is a million.)
  const text = Math.round(abs) >= 1e6 ? lettered(abs) : abs >= 999.95 ? grouped.format(abs) : figures.format(abs);
  // (Too wide (a small fraction, past the last letter's thousands), with a power of ten: 1.235e-4, 1.235e30.)
  const fit = text.length <= PRECISE.length ? text : abs >= 999.95 ? lettered(abs) : "";
  const label = `${sign}${fit && fit.length <= PRECISE.length ? fit : scientific(abs)}${suffix}`;
  axisLabels.set(key, label);
  return label;
}
function duration(seconds) {
  const days = Math.abs(seconds) / 86400;
  if (days * 24 < 1) return plural(Math.max(1, Math.round(days * 1440)), "minute");
  if (days < 2) return plural(Math.round(days * 24), "hour");
  if (days < 90) return `${Math.round(days)} days`;
  if (days < 730) return `${Math.round(days / 30.44)} months`;
  return `${(days / 365.25).toFixed(1)} years`;
}
/** Relative change between two values, unsigned: "12.34%", or "17.25×" once it has grown tenfold; null when it has no meaning. */
function growth(from, to) {
  if (!(from > 0) || !(to >= 0)) return null;
  const ratio = to / from;
  return { up: ratio >= 1, text: ratio >= 10 ? `${precise(ratio)}×` : `${precise(Math.abs(ratio - 1) * 100)}%` };
}
/** An unsigned difference in a unit: percentages differ by points, ratios by plain numbers. */
const amount = (delta, unit) => (unit === "percent" ? `${precise(Math.abs(delta))} pt` : format(Math.abs(delta), unit === "ratio" ? "number" : unit));
/** Words of series names and catalog folders, written the way people write them. */
const ACRONYMS = new Set(["sth", "lth", "mvrv", "sopr", "nupl", "usd", "btc", "ath", "sma", "ema", "utxo", "ohlc", "op", "vocdd", "hodl", "rbf", "cpfp", "tx", "pnl"]);
const WORDS = { addrs: "addresses", addr: "address", txs: "transactions", utxos: "UTXOs" };
/** A percentile as said, its decimal point an underscore or a dot: "p5" 5th percentile, "p0_1" 0.1st, "p99_9" 99.9th. */
const percentileWord = (text) => ((p) => p && `${p}${/(^|\D)1\d$/.test(p) ? "th" : p.at(-1) === "1" ? "st" : p.at(-1) === "2" ? "nd" : p.at(-1) === "3" ? "rd" : "th"} percentile`)(/^p(\d+(?:[._]\d+)?)$/.exec(text)?.[1].replace("_", "."));
const word = (text) => WORDS[text] ?? percentileWord(text) ?? (ACRONYMS.has(text) || /^p2[a-z]+\d*$/.test(text) ? text.toUpperCase() : text);
const plural = (n, word) => `${grouped.format(n)} ${word}${n === 1 ? "" : "s"}`;
/** A series id as words, a percentile's decimal point kept: "supply_in_loss_rarity_p99_9" …rarity 99.9th percentile. */
const humanize = (name) => capitalized(name.replace(/(^|_)p(\d+)_(\d+)(?=_|$)/g, "$1p$2.$3").split("_").filter(Boolean).map(word).join(" "));
/** A catalog key: "utxoSet" is UTXO set, "minute10" is Minute 10, "p2pkh" stays whole, "p99_5ToP99_9" 99.5th percentile to 99.9th percentile. */
const humanizeKey = (key) => capitalized(key.replace(/^_/, "").replace(/([a-z\d])([A-Z])/g, "$1 $2").toLowerCase().split(" ").flatMap((part) => (/^p2/.test(part) ? [part] : part.replace(/([a-z]{2})(\d)/g, "$1 $2").split(" "))).map(word).join(" "));
function ago(time) {
  const minutes = Math.round((Date.now() - time) / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 48) return `${hours}h ago`;
  return `${Math.round(hours / 24)}d ago`;
}
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/** Whether a window over time is told with its times: at a fine interval, two days or less. */
const showsTimes = (from, to, fine) => fine && to - from < 2 * 86400;
/** A date as the page writes it: "25 Sep 2022" (and "14:30", for fine intervals), in UTC. */
const dateLabel = (/** @type {number} */ time, /** @type {boolean} */ fine) => {
  const at = new Date(time * 1000), day = `${at.getUTCDate()} ${MONTHS[at.getUTCMonth()]} ${at.getUTCFullYear()}`;
  return fine ? `${day}, ${String(at.getUTCHours()).padStart(2, "0")}:${String(at.getUTCMinutes()).padStart(2, "0")}` : day;
};

/**
 * The formats a layer can be read in, by hand (Automatic: its unit's), in groups: each group's first starts it.
 * @type {[string, string][]}
 */
const FORMATS = [
  ["auto", "Automatic"],
  ["usd", "US dollars"], ["btc", "Bitcoin"], ["sats", "Satoshis"],
  ["percent", "Percent"], ["ratio", "Multiple ×"], ["count", "Count"], ["number", "Number"],
  ["days", "Days"], ["seconds", "Seconds"], ["minutes", "Minutes"], ["persecond", "Per second"], ["perday", "Per day"],
  ["bytes", "Bytes"], ["vbytes", "Virtual bytes"], ["weight", "Weight units"], ["sigops", "Signature operations"],
  ["hashrate", "Hash rate"], ["difficulty", "Difficulty"], ["feerate", "Fee rate"], ["hashprice", "Hash price"], ["hashvalue", "Hash value"],
  ["coindays", "Coin days"], ["coinblocks", "Coin blocks"], ["dollarblocks", "Dollar blocks"], ["dollardays", "Dollar days per coin"],
];
const UNIT_SIGNS = {
  usd: "USD", btc: "BTC", sats: "sats", percent: "%", ratio: "×", days: "days", seconds: "s", minutes: "min", persecond: "/s", perday: "/day", bytes: "B", vbytes: "vB",
  weight: "WU", sigops: "sigops", hashrate: "H/s", feerate: "sat/vB", hashprice: "$/PH/s/day", hashvalue: "sat/PH/s/day", coindays: "coin days",
  coinblocks: "coin blocks", dollarblocks: "$ × blocks", dollardays: "$ × days/₿",
};
/**
 * A unit after a field's number: a ratio's left out (a × there reads as a button that clears it), and one longer
 * than the field keeps room for ("sat/vB": "$/PH/s/day" would run under the number).
 */
const fieldUnit = (/** @type {string} */ unit) => ((sign) => (unit === "ratio" || sign.length > 6 ? "" : sign))(UNIT_SIGNS[/** @type {keyof typeof UNIT_SIGNS} */ (unit)] ?? "");
/** Each unit's name (its format's, where it has one, as the menu says it). */
const UNIT_NAMES = { ...Object.fromEntries(FORMATS.slice(1)), ratio: "Multiple" };
const MONTH_NAMES = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
/** A month as the page writes it: "Sep 2022" (`whole`: "September 2022"), in UTC. */
const monthYear = (/** @type {number} */ time, whole = false) => {
  const at = new Date(time * 1000);
  return `${whole ? capitalized(MONTH_NAMES[at.getUTCMonth()]) : MONTHS[at.getUTCMonth()]} ${at.getUTCFullYear()}`;
};

export {
  ago, amount, AXIS_CHARS, axisLabel, axisTicks, dateLabel, dollarsAndCents, duration, fieldUnit, format, FORMATS,
  grouped, growth, humanize, humanizeKey, MONTH_NAMES, MONTHS, monthYear, plural, PRECISE, precise, showsTimes,
  SI_UNITS, toTheCent, UNIT_NAMES, unitOf, WIDEST_VALUES,
};
