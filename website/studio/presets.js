// @ts-check
/**
 * Studio's presets, made from a client's series tree (a path it doesn't have fails where it's written) and the chain's
 * facts (no charts of a pool that never found a block, or of what's still to come). Every other series Studio can chart
 * by date is meant to be in one.
 * Laid out alike everywhere: a family opens on its comparisons (every member in one chart), then each member with the
 * same charts, in sections by topic; a measure summed over trailing windows is one chart, its month shown; one in
 * bitcoin has its dollars in a pane below; the price is on top of every chart (see priced).
 * @typedef {Record<string, any>} Layer
 * @typedef {{ layers: Layer[], panes?: Record<string, any>[], index?: string, range?: string, halvings?: boolean }} Sketch
 * @typedef {{ p: string[], t: string, c: Sketch, s?: string, w?: Record<string, string> }} Preset `w`: its places from
 *   when it was one preset a window, each its window as a link says it (see addresses.js)
 * @typedef {{ presets: Preset[], labels: Record<string, string> }} Presets labels: each step's shown name by its place's key (see shownStep)
 * @typedef {string | { place: string, shown: string }} Name its place's step, and where that says less than its charts, how it's shown (see shownAs)
 * @typedef {{ coins: string, capital: string, place: { coins: string, capital: string }, cohort?: any, who?: string }} Whole
 *   whose coins and capital a cohort's shares divide, as said and as its charts' places keep them (see wholeOf,
 *   groupWhole); a holder group's, also its own series (`cohort`, see Naming's `beside`) and its name in titles
 * @typedef {{ name: Name, who: string, inWhole?: string, said?: string, of?: Whole, beside?: [string, string], short?: string }} Naming a family member's: `who` its
 *   charts are in titles, `inWhole` how its shares of its family's whole name it, `said` a band inside a sentence, `of`
 *   its own whole where each member has its own, `beside` its word and its whole's in a legend where its shares in profit and
 *   loss are shown beside its whole's, `short` its name where room is tight (see Layer)
 * @typedef {import("./words.js").Dimension & { inWhole?: (said: string) => string }} Bands a band dimension as its
 *   families and cutoffs read it (see Dimension), and `inWhole`, its members within their whole (see Naming)
 * @typedef {{ within?: string, of?: Whole, lookOf?: (key: string) => { color: string, opacity?: number } | undefined }} Kin
 *   what a band family's options and its cutoffs' share (see family)
 * @typedef {{ name: Name, title: string, c: Sketch, priceHidden?: boolean, was?: Record<string, string> }} ChartNode
 *   `was`: its places from when it was one chart a window, each that window's key (see Preset's `w`)
 * @typedef {(name: Name, title: string, layers: Layer[], options?: { index?: string, log?: number[], stack?: Record<number, string>, average?: Record<number, string>, size?: Record<number, number>, range?: string, halvings?: boolean, priceHidden?: boolean, window?: string, windows?: string[], was?: Record<string, string> }) => ChartNode} ChartBuilder
 * @typedef {{ name: Name, items: (ChartNode | SectionNode)[], workings?: boolean, icon?: string }} SectionNode `workings`: its charts are
 *   about the chain's workings, not the market (the price over them hidden: see priced)
 * @typedef {(name: Name, items: (ChartNode | SectionNode | null | undefined | false)[], options?: { workings?: boolean, icon?: string }) => SectionNode} SectionBuilder
 */

/** @import { Facts } from "./facts.js" */
import { HOLDER_COLORS, LANDSCAPE, SKY, holderColor, recencyColor, wheelColor } from "./palette.js";
import { AGES, ALL_TIME, BALANCES, EDGES, HOLDERS, UTXO_AMOUNTS, adjective, band, capitalized, dimensionBand, holderName, inSentence, isLeaf, isWindowKey, placeEdge, spanLabel, windowAdjective, windowSaid, windowShort } from "./words.js";
import { chainStatistics } from "./chain-statistics.js";
import { POWER_LAW } from "../modules/power-law/index.js";

/** @param {any} n */
const byDate = (n) => isLeaf(n) && n.indexes().includes("day1");
/** @param {any} n */
const entries = (n) => (n && typeof n === "object" && !isLeaf(n) ? Object.entries(n).filter(([, v]) => v && typeof v === "object") : []);

/** @param {string} title @param {string} words */
const appended = (title, words) => title.replace(/^([^:]*)(:|$)/, `$1, ${words}$2`);
function percentile(/** @type {string} */ key) {
  if (key === "median") return 50;
  const digits = /^p(\d+(?:_\d+)?)$/.exec(key)?.[1];
  return digits ? Number(digits.replace("_", ".")) : NaN;
}
function percentileLabel(/** @type {number} */ p) {
  if (p === 50) return "Median";
  const text = String(p), last = text.at(-1), teen = /(^|\D)1\d$/.test(text);
  return `${text}${teen ? "th" : last === "1" ? "st" : last === "2" ? "nd" : last === "3" ? "rd" : "th"}`;
}
/** @param {number} p @param {string} id */
function bandLabel(p, id) {
  return !id.includes("_interpolated_") ? percentileLabel(p) : p === 50 ? "Midpoint" : `Interpolated ${percentileLabel(p)}`;
}
const modeName = (/** @type {string} */ mode) => sentence(capitalized(mode));
function floorBandLabel(/** @type {string} */ key) {
  const [high, low] = (key.match(/p\d+(?:_\d+)?/gi) ?? []).map((p) => percentileLabel(percentile(p.toLowerCase())));
  return low ? `${high}–${low}` : `Below ${high}`;
}
function sentence(/** @type {string} */ text) {
  return text
    .split(" ")
    .map((word, i) => (i && /^[A-Z][a-z]+$/.test(word) ? word.toLowerCase() : word))
    .join(" ")
    .replace(/(\d|[KMB])-(\d)/g, "$1–$2");
}

function keyWords(/** @type {string} */ key) {
  return key.replace(/([a-z])([A-Z0-9])/g, "$1 $2").replace(/(\d)([a-z]{3,})/g, "$1 $2").toLowerCase().replace(/ (\d+[hdwmy])$/, ", $1");
}
/** @param {string} key */
const words = (key) => capitalized(keyWords(key));

const FEATURES = {
  fakePubkey: "Fake public keys", fakeScripthash: "Fake script hashes", segwit: "SegWit", annex: "Taproot annex", sighashAll: "SIGHASH_ALL",
  sighashNone: "SIGHASH_NONE", sighashSingle: "SIGHASH_SINGLE", sighashDefault: "SIGHASH_DEFAULT", sighashAnyoneCanPay: "SIGHASH_ANYONECANPAY",
  rbfSignaling: "RBF signaling", dustOutput: "Dust outputs", epobc: "EPOBC", oneInput: "One input", oneOutput: "One output", emptyScript: "Empty scripts", unknownScript: "Unknown scripts",
  oneInputOneOutput: "One input, one output", oneInputTwoOutput: "One input, two outputs", bip69: "BIP-69", bip69ExpectedByChance: "BIP-69, expected by chance",
  height: "Height-based absolute", time: "Time-based absolute", unenforced: "Unenforced", relativeHeight: "Height-based relative", relativeTime: "Time-based relative",
  onlySegwit: "Only SegWit", onlyLegacy: "Only legacy", segwitAndLegacy: "SegWit and legacy",
};
const FEATURE_PHRASES = {
  emptyScript: "empty scripts", unknownScript: "unknown scripts", fakePubkey: "fake public keys", fakeScripthash: "fake script hashes", annex: "a Taproot annex", rbfSignaling: "RBF signaling", dustOutput: "dust outputs", epobc: "EPOBC colored-coin tags", oneInput: "one input", oneOutput: "one output",
  oneInputOneOutput: "one input and one output", oneInputTwoOutput: "one input and two outputs", bip69: "BIP-69 ordering", bip69ExpectedByChance: "BIP-69 ordering expected by chance",
  height: "a height-based absolute locktime", time: "a time-based absolute locktime", unenforced: "an unenforced locktime", relativeHeight: "a height-based relative locktime", relativeTime: "a time-based relative locktime",
  onlySegwit: "only SegWit inputs", onlyLegacy: "only legacy inputs", segwitAndLegacy: "SegWit and legacy inputs",
};
const sighashName = (/** @type {string} */ key) => key.replace(/^(\w+?)(AnyoneCanPay)?$/, (_, type, anyone) => `SIGHASH_${type.toUpperCase()}${anyone ? "|SIGHASH_ANYONECANPAY" : ""}`);

/** Each price keeps its own color wherever it's drawn (a chart of its own, among others, a meter's reference). */
const PRICE_COLORS = /** @type {Readonly<Record<string, string>>} */ (Object.freeze({ realizedPrice: "orange", capitalizedPrice: "green", trueMarketMean: "sky", cointime: "violet", vaulted: "teal", active: "red", cvdd: "emerald", balanced: "cyan", delta: "purple" }));
const PRICE_NAMES = /** @type {Readonly<Record<string, string>>} */ (Object.freeze({ cvdd: "CVDD" }));
/** A count summed since the start: its month's color, not gray, which reads as off. */
const SUM_COLOR = "blue";
const PER_BLOCK_COLOR = "sky";
/** @param {number} far @param {boolean} good */
const polarLook = (far, good) => ((t) => ({ color: good ? wheelColor(t, 101, 120, 183) : wheelColor(t, 101, 85, 22) }))(Math.round(100 * Math.max(0, Math.min(1, far))));
/** @param {string} key */
function profitBandLook(key) {
  const m = /^(profit|loss)(?:Over)?(\d+)/.exec(key);
  if (!m) return undefined;
  const from = Number(m[2]);
  return polarLook(m[1] === "profit" ? Math.log10(1 + from / 10) / Math.log10(101) : from / 90, m[1] === "profit");
}
const WINDOW_COLORS = ["cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia", "pink", "rose", "red", "orange", "amber", "yellow"];
const ramps = new Map();
/** @returns {readonly string[]} */
function ramp(/** @type {number} */ count, from = 25, to = 320) {
  const key = `${count} ${from} ${to}`;
  if (!ramps.has(key)) ramps.set(key, Array.from({ length: count }, (_, i) => wheelColor(i, count, from, to)));
  return ramps.get(key);
}
function shades(/** @type {number} */ i, /** @type {number} */ count) {
  return { color: wheelColor(i, count, 131, 183), colorBelow: wheelColor(i, count, 44, 17) };
}

/** @param {any} t @param {Facts | undefined} facts @returns {Set<any>} */
function notYet(t, facts) {
  const nodes = new Set();
  if (!facts) return nodes;
  for (const [key, node] of entries(t.pools)) if (node.blockCount && !facts.pools.has(key.replace(/^_/, ""))) nodes.add(node);
  for (const [key, node] of entries(t.age.halvingEpochs)) if (Number(key.slice(1)) * 210_000 > facts.height) nodes.add(node);
  for (const [key, node] of entries(t.age.vintages)) if (Number(key.slice(1)) > facts.year) nodes.add(node);
  return nodes;
}

/**
 * What's wrong (a band key the client renamed) is `report`ed: its member, cutoff or legend is left out and every other
 * preset made as before; the gates fail on any.
 * @param {any} client @param {Facts} [facts] @param {(problem: string) => void} [report] @returns {Presets}
 */
export function studioPresets(client, facts, report = (problem) => console.error(`Studio presets: ${problem}`)) {
  const t = client.series, absent = notYet(t, facts);
  /** @param {any} n */
  const members = (n) => entries(n).filter(([, node]) => !absent.has(node));
  /** @template T @param {string} key @param {T | undefined} read @returns {T | undefined} */
  const reported = (key, read) => {
    if (!read) report(`band ${key} has no edges it can read; left out`);
    return read;
  };

  /** @param {string} place @param {string} [shown] @returns {Name} */
  const shownAs = (place, shown = place) => (shown === place ? place : { place, shown });
  /** @param {Name} name */
  const placeOf = (name) => (typeof name === "string" ? name : name.place);
  /** @param {Name} name */
  const shownOf = (name) => (typeof name === "string" ? name : name.shown);
  /** @param {Name} name @param {string} words @param {string} [said] */
  const extended = (name, words, said = words) => shownAs(placeOf(name) + words, shownOf(name) + said);
  /** A chart lifted out of a folder of two, named after it ("Activity, transfer volume"). @param {string} folder @param {Name} name */
  const liftedName = (folder, name) => ((said) => shownAs(said(placeOf(name)), said(shownOf(name))))((/** @type {string} */ text) => `${folder}, ${/^[A-Z][a-z]/.test(text) ? text[0].toLowerCase() + text.slice(1) : text}`);

  /** @param {any} node @param {string} label @param {Layer} [extra] @returns {Layer} */
  const series = (node, label, extra = {}) => {
    if (!isLeaf(node)) throw new Error(`Not a series: ${label}`);
    return { ...(node.formula ? { formula: node.formula, ...(node.format ? { format: node.format } : {}) } : { series: node.name }), label, type: "line", ...extra };
  };
  const letter = (/** @type {number} */ i) => (i < 26 ? String.fromCharCode(65 + i) : String.fromCharCode(64 + Math.floor(i / 26)) + String.fromCharCode(65 + (i % 26)));
  /** @returns {Layer} */
  const price = () => ({ series: t.price.ohlc.name, label: "Price", type: "candles", color: "ink", colors: 1, priceLine: true });
  /**
   * `window`: said in the title, never in the place: a window is how a chart is looked at (see chartWindow). `was`: its
   * places from when it was one chart a window. Over a heatmap or a cloud the price is outlined, to read on what's under it.
   * @type {ChartBuilder}
   */
  const chart = (name, title, layers, options = {}) => {
    const was = options.was ?? ((keys) => (keys.length ? Object.fromEntries(keys.map((key) => [`${placeOf(name)}, ${spanLabel(key)}`, key])) : undefined))(options.windows ?? (options.window ? [options.window] : []));
    if (options.window) title = appended(title, windowSaid(options.window));
    if (layers.some((layer) => layer.heatmap || layer.cloud)) layers = layers.map((layer) => (layer.series === t.price.ohlc.name ? { ...layer, outline: true } : layer));
    const count = Math.max(1, ...layers.map((layer) => layer.pane ?? 1));
    const panes = Array.from({ length: count }, (_, i) => {
      const n = i + 1, withPrice = layers.some((layer) => (layer.pane ?? 1) === n && layer.type === "candles");
      const log = options.log?.includes(n) || withPrice;
      return { mode: log ? 1 : 0, ...(options.stack?.[n] ? { stack: options.stack[n] } : {}), ...(options.average?.[n] ? { average: options.average[n] } : {}), ...(options.size?.[n] ? { size: options.size[n] } : {}) };
    });
    return { name, title, c: { layers, panes, ...(options.index ? { index: options.index } : {}), ...(options.range ? { range: options.range } : {}), ...(options.halvings ? { halvings: true } : {}) }, ...(options.priceHidden ? { priceHidden: true } : {}), ...(was ? { was } : {}) };
  };
  /** @param {Name} name @param {string | undefined} who @param {Layer[]} layers @param {Parameters<ChartBuilder>[3]} [options] */
  const measure = (name, who, layers, options = {}) => chart(name, who ? `${shownOf(name)}: ${who}` : shownOf(name), layers, options);
  /** @type {SectionBuilder} */
  const section = (name, items, options = {}) => ({ name, items: /** @type {(ChartNode | SectionNode)[]} */ (items.filter(Boolean)), ...(options.workings ? { workings: true } : {}), ...(options.icon ? { icon: options.icon } : {}) });
  /** @param {Name} name @param {(ChartNode | SectionNode | null | undefined | false)[]} items @returns {SectionNode} */
  const member = (name, items) => {
    const kept = /** @type {(ChartNode | SectionNode)[]} */ (items.filter(Boolean));
    return section(name, kept.length === 1 && "items" in kept[0] ? kept[0].items : kept);
  };

  /** @param {any} node */
  const windowsOf = (node) => entries(node).filter(([key, value]) => isWindowKey(key) && (isLeaf(value) || value.btc));
  /**
   * A measure over trailing windows, one layer at its window (the month's, else its longest, unless `show` says): the
   * page offers the others, and a sum's running total (see chartWindow), so none is drawn hidden.
   * @param {any} node
   * @param {string} what
   * @param {{ pick?: (n: any) => any, show?: string, pane?: number, extra?: Layer, around?: number, color?: string }} [options]
   * @returns {Layer[]}
   */
  const windows = (node, what, options = {}) => {
    const list = windowsOf(node), pick = options.pick ?? ((n) => n);
    const i = list.findIndex(([key]) => key === (options.show ?? (list.some(([key]) => key === "_1m") ? "_1m" : list.at(-1)?.[0])));
    if (i < 0) return [];
    const [key, value] = list[i];
    return [series(pick(value), what ? `${what}, ${windowSaid(key)}` : windowSaid(key), {
      color: options.color || WINDOW_COLORS[i % WINDOW_COLORS.length],
      ...(options.pane ? { pane: options.pane } : {}),
      ...options.extra,
      ...(options.around === undefined ? {} : { ...around(options.around), color: "green", colorBelow: "red" }),
    })];
  };
  /** @param {any} node @param {string} what @param {Layer} [extra] */
  const sharesOf = (node, what, extra = {}) => windows(node.sum ?? node, what, { extra });
  /** The place its all-time twin had: it opens this chart at All time now. @param {Name} name */
  const allTimeWas = (name) => ({ [`${placeOf(name)}, all time`]: ALL_TIME });
  /** @param {any} n */
  const btc = (n) => (isLeaf(n) ? n : n?.btc);
  /** @param {any} n */
  const usd = (n) => (isLeaf(n) ? undefined : n?.usd);
  /** @param {any} node @param {string} label */
  const amount = (node, label, color = "orange") => [series(btc(node), label, { color })];
  /** @param {any} n */
  const own = (n) => (isLeaf(n) ? n : n?.value);
  /** @param {number} [base] @returns {Layer} */
  const around = (base = 0) => ({ type: "baseline", color: "green", colorBelow: "red", ...(base ? { base } : {}) });
  const baseline = around(0);

  /**
   * Dollars on a log scale: they move with the price over decades, and on a linear one the early years lie flat.
   * @param {any} node
   * @param {Name} name
   * @param {string} title
   * @param {{ color?: string, dollars?: boolean }} [options]
   * @returns {ChartNode[]}
   */
  const flow = (node, name, title, { color, dollars = false } = {}) => {
    if (!node) return [];
    const sums = node.sum ?? node, pair = windowsOf(sums).some(([, value]) => value.btc), scale = pair ? { log: [2] } : dollars ? { log: [1] } : {};
    const layers = pair ? [...windows(sums, "Bitcoin", { pick: btc, color: color ?? "orange" }), ...windows(sums, "Dollars", { pick: usd, pane: 2, color: "green" })] : windows(sums, "", { color });
    if (layers.length) return [chart(name, title, layers, { ...scale, ...(byDate(btc(node.cumulative)) ? { was: allTimeWas(name) } : {}) })];
    return byDate(node.cumulative) ? [chart(extended(name, ", all time"), appended(title, "all time"), [series(node.cumulative, "All time", { color: color ?? SUM_COLOR })], scale)] : [];
  };
  /** A flow's dollars alone, where the price takes the room beside its bitcoin (see priced). @param {any} node @param {Name} name @param {string} title @returns {ChartNode[]} */
  const inDollars = (node, name, title) => [chart(name, title, windows(node.sum ?? node, "", { pick: usd, color: "green" }), { log: [1] })];
  /** @param {any} node @param {Name} name @param {string} things @param {any} [share] @returns {ChartNode[]} */
  const counted = (node, name, things, share = node.share) => [
    chart(name, numberOf(things), [...windows(node.sum, ""), ...sharesOf(share, "Share", { pane: 2 })], { was: allTimeWas(name) }),
  ];
  /**
   * @param {[Name, string]} amounts
   * @param {[Name, string]} shares
   * @param {Layer[]} layers
   * @param {string} [window]
   * @param {string[]} [windows]
   * @param {boolean} [log]
   * @returns {ChartNode[]}
   */
  const parts = ([name, title], [shareName, shareTitle], layers, window = undefined, windows = undefined, log = false) => {
    const areas = layers.map((layer) => ({ ...layer, type: "area" }));
    return [chart(name, title, areas, { stack: { 1: "stack" }, ...(log ? { log: [1] } : {}), window, windows }), chart(shareName, shareTitle, areas, { stack: { 1: "percent" }, window, windows })];
  };
  /** @param {string} things */
  const numberOf = (things) => `Number of ${things}`;
  /** @param {any} value @param {string} things @returns {ChartNode[]} */
  const valued = (value, things) => [
    ...flow(value, "Value", `Value moved by ${things}`),
    chart("Value share", `Share of transfer volume moved by ${things}`, sharesOf(value.share, "Share")),
  ];
  /** @param {any} node @param {string} things */
  const kindCharts = (node, things) => [...counted(node, "Count", things), ...valued(node.value, things)];
  /**
   * @param {[string, any][]} list
   * @param {(key: string) => Name} name
   * @param {(key: string) => string} things
   * @param {string} by
   * @param {{ what?: string, disjoint?: boolean, count?: boolean, more?: ChartNode[] }} [options]
   */
  const kinds = (list, name, things, by, { what = "transactions", disjoint = false, count = true, more = [] } = {}) => {
    const lines = (/** @type {[string, any][]} */ some, /** @type {(node: any) => any} */ pick) => some.map(([key, node], i) => series(pick(node), shownOf(name(key)), { color: ramp(some.length)[i] }));
    const shared = list.filter(([, node]) => node.share), sorted = by ? ` by ${by}` : "", measured = (/** @type {string} */ some) => (count ? numberOf(some) : capitalized(some));
    return [
      ...(shared.length
        ? [chart("Compare", `Share of ${what}${sorted}`, lines(shared, (node) => node.share._1m), { window: "_1m" }), chart("Compare value", `Share of transfer volume${sorted}`, lines(shared, (node) => node.value.share._1m), { window: "_1m" })]
        : disjoint
          ? [section("Compare", parts([capitalized(what), measured(`${what}${sorted}`)], ["Share", `Share of ${what}${sorted}`], lines(list, (node) => node.sum._1m), "_1m"))]
          : [chart("Compare", measured(`${what}${sorted}`), lines(list, (node) => node.sum._1m), { log: [1], window: "_1m" })]),
      ...more,
      ...list.flatMap(([key, node]) => /** @type {(ChartNode | SectionNode)[]} */ (node.share ? [section(name(key), kindCharts(node, things(key)))] : flow(node, name(key), measured(things(key))))),
    ];
  };
  /**
   * @param {any} node
   * @param {(said: string) => string} things
   * @param {string} by
   * @param {Parameters<typeof kinds>[4]} options
   * @param {any} [zero]
   */
  const feeRates = (node, things, by, options, zero = undefined) => {
    const said = (/** @type {string} */ key) => /** @type {string} */ (band(key)?.said);
    const list = entries(node).filter(([key]) => reported(key, band(key)));
    return kinds([.../** @type {[string, any][]} */ (zero ? [["zero", zero]] : []), ...list], (key) => (key === "zero" ? "Zero fee" : shownAs(spanLabel(key), capitalized(said(key)))), (key) => (key === "zero" ? "zero-fee transactions" : things(said(key))), by, options);
  };
  /** @param {any} delta @param {string} name @param {string} title */
  const change = (delta, name, title) =>
    delta?.absolute
      ? [chart(name, title, [...windows(delta.absolute, "", { extra: { type: "histogram", color: "green", colorBelow: "red" } })])]
      : [];

  /** The backend's fixed-day flow windows, not calendar months or years. */
  const FLOW_WINDOWS = /** @type {[string, number][]} */ ([["_24h", 1], ["_1w", 7], ["_1m", 30], ["_1y", 365]]);
  /** @param {string} formula @param {any} like @param {string} [format] */
  const formulaNode = (formula, like, format = "percent") => ({ name: `(${formula})`, formula, format, indexes: () => like.indexes() });
  /** @param {any} sums @param {string} key */
  const flowWindow = (sums, key) => byDate(sums?.[key]) ? sums[key] : undefined;
  /** @param {any} sums @param {string} key */
  const rollingWindow = (sums, key) => {
    const node = flowWindow(sums, key);
    if (node) return node;
    const daily = sums?._24h, days = FLOW_WINDOWS.find(([window]) => window === key)?.[1];
    return byDate(daily) && days ? formulaNode(`sum(${daily.name}, ${days}d)`, daily, daily.format ?? "number") : undefined;
  };
  /** @param {any} c @param {string} key */
  const realizedTotal = (c, key) => {
    const p = rollingWindow(c.realized?.profit?.sum, key), l = rollingWindow(c.realized?.loss?.sum, key);
    return p && l ? formulaNode(`${p.name} + ${l.name}`, p, "usd") : undefined;
  };
  /** @param {any} c @param {string} key */
  const grossWindow = (c, key) => byDate(c.realized?.grossPnl?.sum?.[key]) ? c.realized.grossPnl.sum[key] : realizedTotal(c, key);
  /** @type {{ flow: string, places: [string, string], log: boolean, section: string, pick: (c: any, key: string) => any }[]} */
  const FLOW_SHARES = [
    { flow: "transfer volume", places: ["Transfer volume share", "Transfer volume share"], log: false, section: "Activity", pick: (/** @type {any} */ c, /** @type {string} */ key) => rollingWindow(Object.fromEntries(FLOW_WINDOWS.map(([k]) => [k, btc(c.activity?.transferVolume?.sum?.[k])])), key) },
    { flow: "gross realized profit and loss", places: ["Gross realized share", "Gross realized share"], log: true, section: "Realized", pick: grossWindow },
    { flow: "realized profit", places: ["Share of realized profit", "Realized profit share"], log: true, section: "Realized", pick: (/** @type {any} */ c, /** @type {string} */ key) => rollingWindow(c.realized?.profit?.sum, key) },
    { flow: "realized loss", places: ["Share of realized loss", "Realized loss share"], log: true, section: "Realized", pick: (/** @type {any} */ c, /** @type {string} */ key) => rollingWindow(c.realized?.loss?.sum, key) },
  ];
  /** @param {any} share @param {any} total @param {string} format */
  const shareAmount = (share, total, format) => formulaNode(`${share.name} * ${total.name} / 100`, total, format);
  /** @param {any} part @param {any} total */
  const flowShare = (part, total) => byDate(part) && byDate(total) ? formulaNode(`100 * ${part.name} / ${total.name}`, part) : undefined;
  /** @param {any} c */
  const utxoShare = (c) => flowShare(own(c.utxos?.count), t.utxoSet.count);
  /** @param {any} c @param {string} selected @param {string} who @param {string} [color] */
  const flowShares = (c, selected, who, color = "violet") => FLOW_SHARES.flatMap(({ flow, places: [place], pick }) => {
    if (place !== selected || pick(c, "_24h")?.name === pick(t.holders.all, "_24h")?.name) return [];
    const sums = Object.fromEntries(FLOW_WINDOWS.map(([key]) => [key, flowShare(pick(c, key), pick(t.holders.all, key))]).filter(([, node]) => node));
    return Object.keys(sums).length ? [measure(shownAs(place, `Share of all ${flow}`), who, windows(sums, "", { color }))] : [];
  });

  /** @param {string} name @returns {Whole} */
  const wholeOf = (name, place = name) => ({ coins: `${name} coins`, capital: `${name} capital`, place: { coins: `${place} coins`, capital: `${place} capital` } });
  /** @param {string} key @returns {Whole} */
  const groupWhole = (key) => {
    const { whole, who } = holderName(key);
    return { ...whole, place: whole, cohort: t.holders[key], who };
  };
  const ALL = groupWhole("all");
  const MARKET_CAP = t.holders.all.supply.total.marketCap.name;
  /** The coins in addresses of the tracked types, not all outputs': never "all addresses'". */
  const ADDRESSED = wholeOf("addresses'", "all addresses'");
  /**
   * `whole`: the cohort is all of what its shares divide (a family's `all`): its shares would be a flat 100%, so left
   * out. `of`: whose coins and capital its shares divide. `urpd`: its URPD cohort, where it has one. `color`: its single
   * lines; series with a meaning of their own keep theirs. `weight`: its URPD's weighting. `beside`: its shares in
   * loss shown beside its whole's, in place of profit and loss stacked. `activity`: charts of its activity measured
   * elsewhere (an age band's Cointime and Coinflow), after its own.
   * @param {any} c
   * @param {string} who
   * @param {{ whole?: boolean, of?: Whole, urpd?: string, color?: string, weight?: string, inWhole?: string, beside?: [string, string], activity?: ChartNode[] }} [options]
   * @returns {(ChartNode | SectionNode)[]}
   */
  const cohort = (c, who, { whole = false, of = ALL, urpd = undefined, color = undefined, weight = undefined, inWhole = who, beside = undefined, activity: activityMore = [] } = {}) => {
    const titled = (/** @type {string} */ what) => `${what}: ${who}`;
    /**
     * Beside its whole's the share in loss is two unfilled lines: a fill would tint the plot under them.
     * @param {string} what @param {any} own @param {any} theirs
     */
    const shares = (what, own, theirs) => {
      if (!beside) return measure(shownAs("Share in profit", `Share of ${what} in profit and in loss`), who, [series(own.inProfit.share, "In profit", { color: "green", type: "area" }), series(own.inLoss.share, "In loss", { color: "red", type: "area" })], { stack: { 1: "stack" } });
      if (!theirs?.inLoss?.share) throw new Error(`${who}: ${of.who} has no share of ${what} in loss to show it beside`);
      return measure(shownAs("Share in loss", `Share of ${what} in loss`), of.who, [series(own.inLoss.share, beside[0], { color: "red", type: "line" }), series(theirs.inLoss.share, beside[1], { color: "soft", type: "line", width: 1 })]);
    };
    const supply = c.supply, capital = c.capital, utxos = c.utxos, activity = c.activity, realized = c.realized, unrealized = c.unrealized;
    const inProfit = supply?.inProfit, inLoss = supply?.inLoss;
    // A member's realized price first, at its root: the one cost basis chart most look for. (The whole market's stays
    // in its Cost basis, taken apart at the top of Valuation.)
    const cost = c.costBasis ? /** @type {ChartNode[]} */ (costCharts(c, who, urpd, color, weight).filter(Boolean)) : [], realizedPrice = whole ? undefined : cost.find((item) => item.name === "Realized price");
    // (The whole market's folders are taken apart at the top of their subjects: none lifted in pairs.)
    return lift([
      realizedPrice,
      supply && section("Supply", [
        chart("Total", titled("Supply"), amount(supply.total, "Supply", color)),
        inProfit && chart("In profit and loss", titled("Supply in profit and loss"), [
          series(btc(inProfit), "In profit", { color: "green", type: "area" }), series(btc(inLoss), "In loss", { color: "red", type: "area" }),
        ], { stack: { 1: "stack" } }),
        inProfit?.share && shares("supply", supply, of.cohort?.supply),
        supply.share && !whole && measure(shownAs(`Share of ${of.place.coins}`, `Share of ${of.coins}`), inWhole, [series(supply.share, `Of ${of.coins}`, { color: color ?? "orange" })]),
        ...change(supply.delta, "Change", titled("Supply change")),
        ...flow(supply.agedOut, "Aged out", titled("Supply aging out"), { color }),
      ]),
      c.funded && section("Addresses", [
        chart("Count", titled("Funded address count"), [series(own(c.funded), "Addresses", { color: color ?? "sky" })], { log: [1] }),
        ...change(c.funded.delta, "Change", titled("Funded address count change")),
      ]),
      capital && section("Capital", [
        chart("Total", titled("Realized cap"), [series(capital.realizedCap, "Realized cap", { color: color ?? "orange" })], { log: [1] }),
        capital.inProfit && chart("In profit and loss", titled("Capital in profit and loss"), [
          series(own(capital.inProfit), "In profit", { color: "green", type: "area" }), series(own(capital.inLoss), "In loss", { color: "red", type: "area" }),
        ], { stack: { 1: "stack" } }),
        capital.inProfit?.share && shares("capital", capital, of.cohort?.capital),
        capital.share && !whole && measure(shownAs(`Share of ${of.place.capital}`, `Share of ${of.capital}`), inWhole, [series(capital.share, `Of ${of.capital}`, { color: color ?? "orange" })]),
        ...change(capital.delta, "Change", titled("Capital change")),
      ]),
      utxos && section("UTXOs", [
        // All coins' count is UTXOs › Count itself: its one home.
        utxos.count && !whole && chart("Count", titled("UTXO count"), [series(own(utxos.count), "UTXOs", { color: color ?? "sky" })], { log: [1] }),
        utxos.count && !whole && chart("Share of all UTXOs", titled("Share of all UTXOs"), [series(utxoShare(c), "Of all UTXOs", { color: color ?? "sky" })]),
        ...change(utxos.count?.delta, "Change", titled("UTXO count change")),
        ...flow(utxos.spentCount, "Spent", titled(numberOf("UTXOs spent")), { color }),
      ]),
      activity && section("Activity", [
        ...flow(activity.transferVolume, "Transfer volume", titled("Transfer volume"), { color }),
        ...flowShares(c, "Transfer volume share", who, color),
        ...(activity.transferVolume?.inProfit ? [
          chart("Volume in profit and loss", titled("Transfer volume in profit and loss"), [
            ...windows(activity.transferVolume.inProfit.sum, "In profit", { pick: btc, extra: { color: "green" } }), ...windows(activity.transferVolume.inLoss.sum, "In loss", { pick: btc, extra: { color: "red" } }),
          ], { was: allTimeWas("Volume in profit and loss") }),
        ] : []),
        ...flow(activity.coindaysDestroyed, "Coin days destroyed", titled("Coin days destroyed"), { color }),
        ...flow(activity.coindaysCreated, "Coin days created", titled("Coin days created"), { color }),
        activity.dormancy && chart("Dormancy", titled("Dormancy"), windows(activity.dormancy, "", { color })),
        ...activityMore,
      ]),
      realized && section("Realized", [
        realized.profit && chart("Profit and loss", titled("Realized profit and loss"), [
          ...windows(realized.profit.sum, "Profit", { extra: { color: "green" } }), ...windows(realized.loss.sum, "Loss", { extra: { color: "red" } }),
        ], { log: [1], was: allTimeWas("Profit and loss") }),
        realized.netPnl && chart("Net profit and loss", titled("Net realized profit and loss"), windows(realized.netPnl.sum, "Net", { around: 0 })),
        ...flowShares(c, "Share of realized profit", who, color),
        ...flowShares(c, "Share of realized loss", who, color),
        realized.profit?.sum?._24h && realized.loss?.sum?._24h && measure(shownAs("Profit share", "Realized profit as a share of profit plus loss"), who, windows(Object.fromEntries(FLOW_WINDOWS.map(([key]) => [key, flowShare(rollingWindow(realized.profit.sum, key), realizedTotal(c, key))])), "", { color: color ?? "violet", extra: { levels: [{ value: 50, color: "gray", dash: 3 }] } })),
        realized.netPnl?.sum?._24h && chart("Profit and loss bubbles", titled("Net realized profit and loss bubbles"), [price(), series(realized.netPnl.sum._24h, "Net", { type: "bubbles", ref: "A", color: "green", colorBelow: "red" })]),
        urpd && realized.profit?.sum?._24h && chart("Profit and loss bubbles on the heatmap", titled("Realized and unrealized profit and loss"), [
          price(), series(realized.profit.sum._24h, "Profit", { type: "bubbles", ref: "A", color: "green", colors: 1 }), series(realized.loss.sum._24h, "Loss", { type: "bubbles", ref: "A", color: "red", colors: 1 }),
          { heatmap: urpd, measure: "pnl" },
        ]),
        ...flow(realized.grossPnl ?? { sum: Object.fromEntries(FLOW_WINDOWS.map(([key]) => [key, grossWindow(c, key)])) }, "Gross profit and loss", titled("Gross realized profit and loss"), { color, dollars: true }),
        ...flowShares(c, "Gross realized share", who, color),
        realized.profitToLoss && chart("Profit to loss ratio", titled("Realized profit to loss ratio"), windows(realized.profitToLoss, "", { around: 1 }), { log: [1] }),
        ...flow(realized.valueDestroyed, "Value destroyed", titled("Value destroyed"), { color, dollars: true }),
        ...flow(realized.peakRegret, "Peak regret", titled("Realized peak regret"), { color, dollars: true }),
        // Ratios around 1 on a log scale, as every ratio: 0.5× is as far under as 2× over, and an old group's spikes to
                // 20× don't flatten the rest.
        realized.sopr && chart("SOPR", titled("Spent output profit ratio (SOPR)"), windows(realized.sopr, "", { show: "_1w", around: 1 }), { log: [1] }),
        realized.adjustedSopr && chart("Adjusted SOPR", titled("Adjusted SOPR (aSOPR)"), [
          ...windows(realized.adjustedSopr, "", { show: "_1w", around: 1 }),
        ], { log: [1] }),
        realized.adjustedSopr && chart("Adjusted value", titled("Adjusted value created and destroyed"), [
          series(realized.adjustedSopr.valueCreated.cumulative, "Created, all time", { color: "green" }), series(realized.adjustedSopr.valueDestroyed.cumulative, "Destroyed, all time", { color: "red" }),
        ], { log: [1], was: allTimeWas("Adjusted value") }),
        realized.sellSideRiskRatio && chart("Sell-side risk ratio", titled("Sell-side risk ratio"), windows(realized.sellSideRiskRatio, "", { color }), { log: [1] }),
      ]),
      unrealized && section("Unrealized", [
        chart("Profit and loss", titled("Unrealized profit and loss"), [
          series(unrealized.profit, "Profit", { color: "green" }), series(unrealized.loss, "Loss", { color: "red" }),
          ...(unrealized.grossPnl ? [series(unrealized.grossPnl, "Gross", { color: "gray", hidden: true })] : []),
        ]),
        chart("Net profit and loss", titled("Net unrealized profit and loss"), [series(unrealized.netPnl, "Net", baseline)]),
        unrealized.nupl && chart("NUPL", titled("Net unrealized profit and loss (NUPL)"), [series(unrealized.nupl, "NUPL", { ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
        // Against the market's value, not the cohort's own: one scale, so cohorts compare.
        ...((relative) => [
          chart("Relative to market cap", titled("Unrealized profit and loss relative to market cap"), [relative(unrealized.profit, "Profit", { color: "green" }), relative(unrealized.loss, "Loss", { color: "red" })]),
          chart("Net, relative to market cap", titled("Net unrealized profit and loss relative to market cap"), [relative(unrealized.netPnl, "Net", baseline)]),
        ])((/** @type {any} */ node, /** @type {string} */ label, /** @type {Partial<Layer>} */ look) => ({ formula: `100 * ${node.name} / ${MARKET_CAP}`, label, format: "percent", ...look })),
      ]),
      cost.length > (realizedPrice ? 1 : 0) && section("Cost basis", cost.filter((item) => item !== realizedPrice)),
    ].filter(Boolean), !whole);
  };
  /** @param {string} realized @param {string} ratio @param {(what: string) => string} titled */
  const deviationCharts = (realized, ratio, titled) => [
    chart("Realized price bands", titled("Realized price deviation bands"), [
      price(),
      ...[3, 2, 1, 0, -1].map((k) => ({ formula: `${realized} * (sma(${ratio}, 4y) ${k < 0 ? "-" : "+"} ${Math.abs(k)} * stdev(${ratio}, 4y))`, label: k ? `${k > 0 ? "+" : "−"}${Math.abs(k)}σ` : "Mean", format: "usd", width: 1, ...(k ? polarLook(Math.abs(k) / 3, k < 0) : { color: "gray", dash: 3 }) })),
    ]),
    chart("Realized price z-score", titled("Realized price z-score"), [
      price(),
      ...["_1y", "_2y", "_4y", ""].map((key, i) => ({ formula: key ? `zscore(${ratio}, ${key.slice(1)})` : `zscore(${ratio})`, label: key ? windowSaid(key) : "All time", pane: 2, format: "number", ...around(0), ...shades(i, 4), ...(key === "_4y" ? {} : { hidden: true }) })),
    ]),
  ];
  /**
   * @param {any} c
   * @param {string} who
   * @param {string} [urpd]
   * @param {string} [color]
   * @param {string} [weight]
   * @returns {(ChartNode | false | undefined)[]}
   */
  function costCharts(c, who, urpd, color, weight) {
    const cost = c.costBasis, titled = (/** @type {string} */ what) => `${what}: ${who}`, weighed = weight ? { weight } : {};
    const oldest = !weight && OLDEST.has(urpd ?? "");
    return [
      urpd && chart("Heatmap", titled("Supply by cost basis"), [price(), { heatmap: urpd, ...weighed }]),
      // The median solid and wider: a dash on a daily series breaks up.
      urpd && cost.byCoin.median && chart("Heatmap and percentiles", titled("Supply by cost basis, with its percentiles"), [
        price(), { heatmap: urpd, ...weighed },
        ...[["p25", "25th"], ["median", "Median"], ["p75", "75th"]].filter(([key]) => cost.byCoin[key]).map(([key, label]) => series(key === "median" && cost.byCoin.medianRealizedPrice ? cost.byCoin.medianRealizedPrice : cost.byCoin[key], label, { color: "gray", width: key === "median" ? 1.5 : 1, ...(key === "p25" && oldest ? { fit: false } : {}) })),
        series(cost.byCoin.realizedPrice, "Realized price", { color: color ?? PRICE_COLORS.realizedPrice, width: 2 }),
      ]),
      urpd && chart("Profile", titled("Supply profile"), [price(), { profile: [urpd], ...weighed }]),
      urpd && chart("Heatmap and profile", titled("Supply by cost basis, history and today"), [price(), { heatmap: urpd, ...weighed }, { profile: [urpd], ...weighed }]),
      urpd && chart("Cumulative profile", titled("Supply profile, cumulative from the price"), [price(), { profile: [urpd], cumulative: true, ...weighed }]),
      chart("Realized price", titled("Realized price"), [
        price(), series(cost.byCoin.realizedPrice, "Realized price", { color: color ?? "orange" }),
        ...(cost.byCoin.medianRealizedPrice ? [series(cost.byCoin.medianRealizedPrice, "Median realized price", { color: "yellow", hidden: true })] : []),
        ...(cost.min ? [series(cost.min, "Lowest", { color: "red", dash: 3, hidden: true }), series(cost.max, "Highest", { color: "green", dash: 3, hidden: true })] : []),
        ...(cost.byCoin.avg?.mvrv ? [series(cost.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1) })] : []),
      ], { log: [2] }),
      ...(cost.byCoin.avg?.mvrv && !cost.byCoin.avg.mvrv.formula ? deviationCharts(cost.byCoin.realizedPrice.name, cost.byCoin.avg.mvrv.name, titled) : []),
      cost.byDollar && chart("Capitalized price", titled("Capitalized price"), [
        price(), series(cost.byDollar.capitalizedPrice, "Capitalized price", { color: color ?? "green" }),
        ...(cost.byDollar.medianCapitalizedPrice ? [series(cost.byDollar.medianCapitalizedPrice, "Median capitalized price", { color: "yellow", hidden: true })] : []),
        series(cost.byDollar.avg.ratio, "Price ratio", { pane: 2, ...around(1) }),
      ], { log: [2] }),
      ...percentiles(cost.byCoin, "Percentiles by coin", titled("Cost basis percentiles, by coin"), PRICE_COLORS.realizedPrice, !oldest),
      ...(cost.byDollar ? percentiles(cost.byDollar, "Percentiles by dollar", titled("Cost basis percentiles, by dollar"), PRICE_COLORS.capitalizedPrice) : []),
      ...[[cost.supplyDensity, "Supply", btc(c.supply?.total), "btc", color ?? "orange"], [cost.capitalDensity, "Capital", c.capital?.realizedCap, "usd", color ?? "sky"]].flatMap(([node, label, total, format, hue]) => {
        if (!node) return [];
        const parts = [["total", label, hue], ["inProfit", "In profit", "green"], ["inLoss", "In loss", "red"]];
        return [chart(`${label} density`, titled(`Share of ${label.toLowerCase()} with a cost basis within 5% of the price`),
          parts.map(([key, name, color]) => series(node[key], name, { color, ...(key === "total" ? {} : { hidden: true }) })))];
      }),
    ];
  }
  /**
   * `outer` false (the oldest holder groups by coin, see OLDEST): their low-tail coins were mined for next to nothing
   * (5th $0, 10th cents) and would stretch the scale down to them, so only the inner band shows and every percentile
   * under the median is out of the axis' fit.
   * @param {any} node @param {string} name @param {string} title @param {string} color @param {boolean} [outer]
   */
  const percentiles = (node, name, title, color, outer = true) => {
    const list = entries(node).filter(([key, value]) => isLeaf(value) && Number.isFinite(percentile(key))).sort(([a], [b]) => percentile(a) - percentile(b));
    if (!list.length) return [];
    const median = [node.medianRealizedPrice, node.medianCapitalizedPrice].find(isLeaf);
    // The low tail and the band its edge bounds are out of the axis' fit: drawn past the plot's foot where they're far
        // under the price, the axis framed on the price and the core.
    const cut = outer ? 25 : 50, edges = outer ? [5, 25, 75, 95] : [25, 75], tail = (/** @type {string} */ key) => (percentile(key) < cut ? { fit: false } : {});
    const lines = list.map(([key, value]) => series(key === "median" && median ? median : value, percentileLabel(percentile(key)), { color, ...tail(key), ...(key === "median" ? { width: 2 } : { width: 1, opacity: 0.5, ...(edges.includes(percentile(key)) ? {} : { hidden: true }) }) }));
    const at = (/** @type {number} */ p) => ((i) => (i >= 0 ? letter(1 + i) : undefined))(list.findIndex(([key]) => percentile(key) === p));
    const band = (/** @type {number} */ low, /** @type {number} */ high, /** @type {number} */ opacity) => ((a, b) => (a && b ? [{ cloud: [a, b], label: `${percentileLabel(low)} to ${percentileLabel(high)}`, color, opacity, ...(low < cut ? { fit: false } : {}) }] : []))(at(low), at(high));
    const ramped = ramp(list.length, 25, 160), each = list.map(([key, value], i) => series(key === "median" && median ? median : value, percentileLabel(percentile(key)), { color: ramped[i], width: key === "median" ? 2 : 1, ...tail(key), ...(!outer && percentile(key) < 25 ? { hidden: true } : {}) }));
    return [chart(name, title, [price(), ...lines, ...(outer ? band(5, 95, 0.15) : []), ...band(25, 75, 0.3)]), chart(`${name.replace(/^Percentiles/, "Percentile lines")}`, title.replace(/^Cost basis percentiles/, "Cost basis percentile lines"), [price(), ...each])];
  };


  /** @param {any} node @param {string} label @param {string} who @param {[any, any] | undefined} reference @param {string} [color] */
  const component = (node, label, who, reference, color = "orange") => {
    const titled = (/** @type {string} */ what) => `${label} ${what}: ${who}`, [base, ratio] = own(node) ? [own(node), node.ratio] : reference ?? [];
    return [
      chart(`${label}, bands`, titled("rarity bands"), bandLayers([price(), ...(base ? [series(base, label, { color })] : [])], node, (v) => v.band)),
      chart(`${label}, ratios`, titled("rarity ratios"), [...(ratio ? [series(ratio, "Price ratio", around(1))] : []), ...bandLines(node, (v) => v.ratio)], { log: [1] }),
    ];
  };
  /** @param {string} cohort @param {any} node @param {string} [kind] @returns {(ChartNode | SectionNode)[]} */
  const rarityComponents = (cohort, node, kind = "") => {
    const list = entries(node), weighted = (/** @type {string} */ k) => k === "awake" || k === "mobile";
    const who = kind ? capitalized(holderName(cohort).of(kind)) : holderName(cohort).who;
    return [
      ...list.filter(([k]) => !weighted(k)).flatMap(([k, v]) => component(v, PRICE_NAMES[k] ?? words(k), who, referenceOf(cohort, kind, k), PRICE_COLORS[k] ?? PRICE_COLORS[k.replace(/Price$/, "")])),
      ...list.filter(([k]) => weighted(k)).map(([k, v]) => section(capitalized(k), rarityComponents(cohort, v, k))),
    ];
  };
  /** @param {string} cohort @param {string} kind @param {string} key @returns {[any, any] | undefined} */
  const referenceOf = (cohort, kind, key) => {
    const cost = ({ "": t.holders[cohort], awake: ct.cohorts[cohort]?.awake, mobile: cf.cohorts[cohort]?.mobile }[kind])?.costBasis;
    return /** @type {Record<string, [any, any]>} */ ({
      realizedPrice: [cost?.byCoin.realizedPrice, cost?.byCoin.avg.mvrv],
      capitalizedPrice: [cost?.byDollar.capitalizedPrice, cost?.byDollar.avg.ratio],
      vaultedPrice: [ct.prices.vaulted.value, ct.prices.vaulted.ratio],
      trueMarketMean: [ct.prices.trueMarketMean.value, ct.prices.trueMarketMean.ratio],
      cointimePrice: [ct.prices.cointime.value, ct.prices.cointime.ratio],
      activePrice: [ct.prices.active.value, ct.prices.active.ratio],
    })[key];
  };
  /** @param {any} node @param {(v: any) => any} pick @param {(p: number) => boolean} [shown] */
  const bandLines = (node, pick, shown = (/** @type {number} */ p) => p <= 20 || p >= 80 || p === 50) => {
    const bands = entries(node).filter(([k]) => Number.isFinite(percentile(k))), ps = bands.map(([k]) => percentile(k));
    return bands.map(([k, v], i) => series(pick(v), bandLabel(percentile(k), pick(v).name), { ...rarityLook(ps[i] < 50 ? ps[i - 1] ?? ps[i] : ps[i], ps[i] === 50), ...(shown(percentile(k)) ? {} : { hidden: true }) }));
  };
  /** @param {number} p @param {boolean} [median] */
  const rarityLook = (p, median = p === 50) => {
    if (median) return { color: "gray", width: 2 };
    return { ...polarLook(Math.log10(50 / Math.min(p, 100 - p)) / Math.log10(500), p < 50), width: 1, opacity: 0.5 };
  };
  /** @param {Layer[]} head @param {any} node @param {(v: any) => any} pick */
  const bandLayers = (head, node, pick) => {
    const ps = bandPercentiles(node);
    return [...head, ...bandLines(node, pick), ...rarityTails((p) => letter(head.length + ps.indexOf(p)), ps)];
  };
  /** @param {any} node */
  const bandPercentiles = (node) => entries(node).map(([k]) => percentile(k)).filter(Number.isFinite);
  /** @param {(p: number) => string | undefined} at @param {number[]} ps */
  const rarityTails = (at, ps) => {
    const side = (/** @type {(p: number) => boolean} */ on) => ps.filter(on).map(at).filter((letter) => letter !== undefined);
    const cheap = side((p) => p <= 20), dear = side((p) => p >= 80);
    return [
      ...(cheap.length > 1 ? [{ cloud: cheap, label: "Cheap side", color: "green", byLayers: true, opacity: 0.5 }] : []),
      ...(dear.length > 1 ? [{ cloud: dear, label: "Dear side", color: "red", byLayers: true, opacity: 0.5 }] : []),
    ];
  };
  /** @type {Record<string, { what: string, window?: string, event: string, color: string, place: string, low?: boolean }>} */
  const EXTREMES = {
    realizedProfitSum24h: { what: "realized profit", window: "_24h", event: "Profit taking", color: "green", place: "above" },
    realizedLossSum24h: { what: "realized loss", window: "_24h", event: "Capitulation", color: "red", place: "below" },
    realizedPeakRegretSum24h: { what: "realized peak regret", window: "_24h", event: "Peak regret", color: "pink", place: "below" },
    sellSideRiskRatio24h: { what: "sell-side risk ratio", window: "_24h", event: "Seller exhaustion", color: "yellow", place: "above", low: true },
    supplyInLoss: { what: "supply in loss", event: "Supply in loss", color: "orange", place: "below" },
  };
  /** A folder of one or two charts opened up into its parent: one as itself, two named after it (D's thin folders). */
  const lift = (/** @type {(ChartNode | SectionNode)[]} */ items, pairs = true) => {
    const names = (/** @type {SectionNode} */ folder) => folder.items.map((inner) => placeOf(inner.name).toLowerCase());
    // (A pair stays a folder beside a bigger one holding its names too: Addresses' Count and Change beside UTXOs'.)
    const parallel = (/** @type {SectionNode} */ folder) => items.some((other) => other !== folder && "items" in other && other.items.length > 2 && names(folder).every((name) => names(other).includes(name)));
    return items.flatMap((item) => {
      if (!("items" in item) || item.items.length > (pairs ? 2 : 1) || item.items.some((inner) => "items" in inner) || (item.items.length === 2 && parallel(item))) return [item];
      if (item.items.length === 2) return item.items.map((inner) => ({ ...inner, name: liftedName(placeOf(item.name), inner.name) }));
      const only = item.items[0];
      return [{ ...only, name: only.name === "Total" ? item.name : only.name === "Profit and loss" ? extended(item.name, " profit and loss") : only.name }];
    });
  };

  /**
   * `disjoint`: the members split the whole (bands), so their amounts stack, and show as shares. `profile`: its members
   * (all coins aside) split the whole: their supply profiles stack in one chart. `inTime`: members are times (halving
   * epochs, years), oldest first, colored as halving cycles and years are (recencyColor). `wholeAbove`: all coins compared,
   * their own charts placed elsewhere (see market). `more`: comparisons and members' activity measured elsewhere (age
   * bands' Cointime and Coinflow), after the family's own.
   * @param {[string, any][]} all
   * @param {(key: string) => Naming | undefined} naming
   * @param {{ each: string, more?: { compare: ChartNode[], activity: Map<string, ChartNode[]> }, within?: string, disjoint?: boolean, profile?: boolean, wholeAbove?: boolean, inTime?: boolean, of?: Whole, known?: Record<string, string>, weight?: string, lookOf?: (key: string) => { color: string, opacity?: number } | undefined, urpd?: (key: string) => string | undefined }} options
   * @returns {(ChartNode | SectionNode)[]}
   */
  const family = (all, naming, options) => {
    const names = /** @type {Map<string, Naming>} */ (new Map());
    for (const [key] of all) {
      const n = naming(key);
      if (n) names.set(key, n);
    }
    const members = all.filter(([key]) => names.has(key));
    /** @param {string} key */
    const nameOf = (key) => /** @type {Naming} */ (names.get(key));
    /** @param {string} key */
    const shortOf = (key) => ((short) => (short ? { short } : {}))(nameOf(key).short);
    /** @param {string} measure @param {{ after?: string, whole?: boolean }} [o] */
    const compared = (measure, { after = "", whole = false } = {}) => `${measure} ${options.each}${after}${options.within && !whole ? `: ${options.within}` : ""}`;
    const weighed = options.weight ? { weight: options.weight } : {};
    const looks = members.map(([key]) => options.lookOf?.(key));
    const colors = ((wheel) => members.map(([key], i) => looks[i]?.color ?? holderColor(key) ?? wheel[i]))(options.inTime ? members.map((_, i) => recencyColor(i, members.length)) : ramp(members.length));
    /** @param {(c: any) => any} pick @param {Layer} [extra] */
    const each = (pick, extra = {}) => members.flatMap(([key, c], i) => (byDate(pick(c)) ? [series(pick(c), shownOf(nameOf(key).name), { ...shortOf(key), color: colors[i], ...(looks[i]?.opacity === undefined ? {} : { opacity: looks[i]?.opacity }), ...extra })] : []));
    /**
     * @param {Name} name
     * @param {(c: any) => any} pick
     * @param {{ price?: boolean, log?: boolean, additive?: boolean, shares?: boolean, level?: number, whole?: boolean, window?: string, windows?: string[] }} [o]
     * @returns {ChartNode[]}
     */
    const compare = (name, pick, o = {}) => {
      const layers = each(pick, o.level === undefined ? {} : { levels: [{ value: o.level, color: "gray", dash: 3 }] });
      if (!layers.length) return [];
      const measure = shownOf(name), title = compared(measure, { whole: o.whole }), { window, windows } = o;
      if (options.disjoint && o.additive) {
        const known = options.known?.[placeOf(name)];
        return parts([name, title], [extended(name, " share"), compared(`${measure} share`, { after: known ? ` (${known})` : "" })], layers, window, windows, o.log).slice(0, o.shares === false ? 1 : 2);
      }
      if (o.price) return [chart(name, title, [price(), ...layers], { window, windows })];
      return [chart(name, title, layers, { window, windows, ...(o.log ? { log: [1] } : {}) })];
    };
    const whole = options.urpd ? HOLDERS.all.urpd : undefined;
    const groups = members.flatMap(([key]) => (key === "all" ? [] : [options.urpd?.(key)]).filter((id) => typeof id === "string"));
    const splits = Boolean(options.disjoint || options.profile);
    /**
     * Only partitions get one: overlapping cutoffs would normalize to a false 100%.
     * @param {string} where
     */
    const shareComparisons = (where) => !splits ? [] : FLOW_SHARES.flatMap(({ flow, places: [, place], log, section: at, pick }) => {
      const window = "_1m", total = pick(t.holders.all, window), parts = members.filter(([key, c]) => key !== "all" && byDate(pick(c, window)));
      if (at !== where || !byDate(total) || parts.length < 2) return [];
      const layers = parts.map(([key, c]) => series(flowShare(pick(c, window), total), shownOf(nameOf(key).name), { ...shortOf(key), color: colors[members.findIndex(([k]) => k === key)], type: "area" }));
      const said = `Share of all ${flow}`;
      const windows = FLOW_WINDOWS.map(([key]) => key);
      return [...compare(capitalized(flow), (c) => pick(c, window), { additive: true, shares: false, log, window, windows }), chart(shownAs(place, said), compared(said), layers, { stack: { 1: "stack" }, window, windows })];
    });
    const ownWholes = members.some(([key]) => nameOf(key).of);
    /** @param {"coins" | "capital"} what @param {(c: any) => any} pick */
    const shares = (what, pick) => ownWholes ? compare(`Share of their group's ${what}`, pick) : compare(options.of ? shownAs(`Share of all ${what}`, `Share of ${options.of[what]}`) : `Share of all ${what}`, pick, { whole: Boolean(options.of) });
    const comparisons = [
      ...compare("Supply", (c) => btc(c.supply?.total), { log: true, additive: true }),
      ...compare("Realized cap", (c) => c.capital?.realizedCap, { log: true, additive: true }),
      ...(options.disjoint ? [] : [...shares("coins", (c) => c.supply?.share), ...shares("capital", (c) => c.capital?.share), ...compare("Share of all UTXOs", utxoShare)]),
      ...compare("Realized price", (c) => c.costBasis?.byCoin?.realizedPrice, { price: true }),
      ...(options.profile && groups.length > 1 ? [
        chart("Profile", compared("Supply profile"), [price(), { profile: groups, ...weighed }]),
        ...(whole ? [chart("Profile on the heatmap", compared("Supply profile", { after: ", on the supply by cost basis" }), [price(), { heatmap: whole, ...weighed }, { profile: groups, ...weighed }])] : []),
        chart("Cumulative profile", compared("Supply profile", { after: ", cumulative from the price" }), [price(), { profile: groups, cumulative: true, ...weighed }]),
      ] : []),
      ...(whole && members.length > 1 && members.some(([key]) => key === "all") && members.every(([, c]) => c.costBasis?.byCoin?.realizedPrice) ? [chart("Realized prices on the heatmap", compared("Realized price", { after: ", on the supply by cost basis" }), [price(), { heatmap: whole, ...weighed }, ...each((c) => c.costBasis.byCoin.realizedPrice, { width: 2 })])] : []),
      ...compare("MVRV", (c) => c.costBasis?.byCoin?.avg?.mvrv, { level: 1, log: true }),
      ...compare(shownAs("Share in profit", "Share of supply in profit"), (c) => c.supply?.inProfit?.share),
      ...compare(shownAs("Capital share in profit", "Share of capital in profit"), (c) => c.capital?.inProfit?.share),
      ...compare("NUPL", (c) => c.unrealized?.nupl, { level: 0 }),
      ...compare("UTXO count", (c) => own(c.utxos?.count), { log: true, additive: true }),
      ...compare(shownAs("Address count", "Funded address count"), (c) => own(c.funded), { log: true, additive: true }),
      ...(splits ? [] : compare("Transfer volume", (c) => btc(c.activity?.transferVolume?.sum?._1m), { additive: true, shares: false, window: "_1m" })),
      ...shareComparisons("Activity"),
      ...compare("Coin days destroyed", (c) => c.activity?.coindaysDestroyed?.sum?._1m, { additive: true, window: "_1m" }),
      ...(splits ? [] : compare("Realized profit", (c) => c.realized?.profit?.sum?._1m, { additive: true, log: true, shares: false, window: "_1m" })),
      ...(splits ? [] : compare("Realized loss", (c) => c.realized?.loss?.sum?._1m, { additive: true, log: true, shares: false, window: "_1m" })),
      ...shareComparisons("Realized"),
      ...compare("Net realized profit and loss", (c) => c.realized?.netPnl?.sum?._1m, { level: 0, window: "_1m" }),
      ...compare("SOPR", (c) => c.realized?.sopr?._1w, { level: 1, log: true, window: "_1w" }),
    ];
    comparisons.push(...(options.more?.compare ?? []));
    return [
      ...(comparisons.length ? [section("Compare", comparisons)] : []),
      ...members.flatMap(([key, c], i) => {
        if (options.wholeAbove && key === "all") return [];
        const n = nameOf(key), color = key === "all" ? undefined : colors[i];
        return member(n.name, cohort(c, n.who, { whole: key === "all" && !n.of, of: n.of ?? options.of, urpd: options.urpd?.(key), color, weight: options.weight, inWhole: n.inWhole, beside: n.beside, activity: options.more?.activity.get(key) }));
      }),
    ];
  };
  /** @param {Record<string, { long: string }>} table @param {string} key */
  const named = (table, key) => sentence(table[key]?.long ?? key);
  /**
   * @param {Record<string, { long: string, short?: string }>} table
   * @param {Bands} d
   * @param {RegExp} [trim]
   * @returns {(key: string) => Naming | undefined}
   */
  const bandMember = (table, d, trim = undefined) => (key) => {
    const read = reported(key, dimensionBand(key, d));
    return read && { name: shownAs(trim ? named(table, key).replace(trim, "") : named(table, key), capitalized(read.said)), who: read.who, inWhole: d.inWhole?.(read.said), said: read.said, short: table[key]?.short };
  };
  const ageBand = bandMember(client.AGE_RANGE_NAMES, AGES, / old$/);
  /** @param {any} ranges @returns {[string, any, Naming][]} */
  const ageBands = (ranges) => entries(ranges).flatMap(([key, node]) => {
    const n = ageBand(key);
    return n ? [/** @type {[string, any, Naming]} */ ([key, node, n])] : [];
  });
  /** Holder groups with no upper age: they hold the chain's oldest coins, unmoved since they were mined. */
  const OLDEST = new Set(["all", "lth", "age_over_4m", "age_over_6m"]);
  /**
   * @param {[string, any][]} list
   * @param {(members: [string, any][], each: string, profile: boolean) => (ChartNode | SectionNode)[]} build
   */
  const holderSets = (list, build) => {
    const pick = (/** @type {string[]} */ keys) => keys.flatMap((key) => list.filter(([k]) => k === key));
    const cutoffs = pick(["under4m", "over4m", "under6m", "over6m"]);
    return [...build(pick(["all", "sth", "lth"]), "by holder group", true), ...(cutoffs.length ? [section("Age cutoffs", build(cutoffs, "by age cutoff", false))] : [])];
  };
  /** @param {string} key @returns {Naming} */
  const holder = (key) => {
    const h = holderName(key);
    return { name: shownAs(h.place, h.shown), who: h.who };
  };
  /** @param {string} key @param {(ChartNode | SectionNode | null | undefined | false)[]} items @param {{ icon?: string }} [options] */
  const holderSection = (key, items, options = {}) => section(holder(key).name, items, options);

  /**
   * Only what its bands sum to exactly (checked against the stored cutoffs over every day): no percentiles, prices by
   * dollar or heatmaps, which no sum makes. Prices and ratios come from the sums, never from the bands' own.
   * @param {any[]} bands @param {number} at @param {boolean} over
   */
  const cutoffCohort = (bands, at, over) => {
    const edge = bands[at];
    /** @param {string} formula @param {any} like @param {string} [format] */
    const made = (formula, like, format) => ({ name: formula.includes(" ") ? `(${formula})` : formula, indexes: () => like.indexes(), formula, ...(format ? { format } : {}) });
    /** @param {string[]} path */
    const sum = (...path) => ((band) => (isLeaf(band) ? made(`${over ? "over" : "under"}(${band.name})`, band) : undefined))(path.reduce((node, key) => node?.[key], edge));
    const between = (/** @type {any} */ a, /** @type {string} */ op, /** @type {any} */ b, /** @type {string} */ format = "") => (a && b ? made(`${a.name} ${op} ${b.name}`, a, format) : undefined);
    /** @param {string[]} path */
    const pair = (...path) => (sum(...path, "btc") ? { btc: sum(...path, "btc"), usd: sum(...path, "usd") } : undefined);
    /** @param {(...path: string[]) => any} read @param {string[]} path */
    const flowOf = (read, ...path) => (read(...path, "sum", "_1m") ? { sum: Object.fromEntries(FLOW_WINDOWS.map(([key]) => [key, read(...path, "sum", key)])), cumulative: read(...path, "cumulative") } : undefined);
    /** @param {any} a @param {any} b @param {string} op */
    const flowsOf = (a, b, op) => (a && b ? { sum: Object.fromEntries(FLOW_WINDOWS.map(([key]) => [key, between(a.sum[key], op, b.sum[key])])), cumulative: between(a.cumulative, op, b.cumulative) } : undefined);
    /** @param {any} value @param {string[]} path */
    const delta = (value, ...path) => {
      const change = sum(...path, "delta", "absolute", "_1m");
      return change && value ? { absolute: { _1m: change }, percent: { _1m: made(`100 * ${change.name} / (${value.name} - ${change.name})`, change, "percent") } } : undefined;
    };
    /** @param {any} high @param {any} low @param {(side: any) => any} [wrap] */
    const sides = (high, low, wrap = (side) => side) => {
      if (!high || !low) return [];
      const share = made(`100 * ${high.name} / (${high.name} + ${low.name})`, high, "percent");
      return [{ ...wrap(high), share }, { ...wrap(low), share: made(`100 - ${share.name}`, low, "percent") }];
    };

    const funded = sum("funded", "value");

    const coins = pair("supply", "total");
    const [coinsInProfit, coinsInLoss] = sides(sum("supply", "inProfit", "btc"), sum("supply", "inLoss", "btc"), (btc) => ({ btc }));
    const agedOut = over ? undefined : bands[at - 1].supply?.agedOut;

    const cap = sum("capital", "realizedCap");
    const [capitalInProfit, capitalInLoss] = sides(sum("capital", "inProfit"), sum("capital", "inLoss"));

    const count = sum("utxos", "count", "value");

    const coindays = sum("activity", "coindaysDestroyed", "sum", "_1m"), moved = sum("activity", "transferVolume", "sum", "_1m", "btc");

    const profit = flowOf(sum, "realized", "profit"), loss = flowOf(sum, "realized", "loss");
    const destroyed = sum("realized", "valueDestroyed", "sum", "_1w"), net = sum("realized", "netPnl", "sum", "_1w");

    const paperProfit = sum("unrealized", "profit"), paperLoss = sum("unrealized", "loss");

    const realizedPrice = between(cap, "/", coins?.btc, "usd");

    return {
      funded: edge.funded && { value: funded, delta: delta(funded, "funded") },
      supply: edge.supply && { total: coins, share: sum("supply", "share"), inProfit: coinsInProfit, inLoss: coinsInLoss, delta: delta(coins?.btc, "supply"), agedOut },
      capital: edge.capital && { realizedCap: cap, share: sum("capital", "share"), inProfit: capitalInProfit, inLoss: capitalInLoss, delta: delta(cap, "capital") },
      utxos: edge.utxos && { count: { value: count, delta: delta(count, "utxos", "count") }, spentCount: flowOf(sum, "utxos", "spentCount") },
      activity: edge.activity && {
        transferVolume: flowOf(pair, "activity", "transferVolume"),
        coindaysDestroyed: flowOf(sum, "activity", "coindaysDestroyed"),
        dormancy: coindays && moved ? { _1m: between(coindays, "/", moved, "number") } : undefined,
      },
      realized: edge.realized && {
        profit, loss, netPnl: flowsOf(profit, loss, "-"), grossPnl: flowsOf(profit, loss, "+"),
        valueDestroyed: flowOf(sum, "realized", "valueDestroyed"),
        // (What came out of the coins spent is what went in and the net: SOPR with no value created stored.)
        sopr: destroyed && net ? { _1w: made(`(${destroyed.name} + ${net.name}) / ${destroyed.name}`, destroyed, "ratio") } : undefined,
      },
      unrealized: edge.unrealized && { profit: paperProfit, loss: paperLoss, netPnl: between(paperProfit, "-", paperLoss), nupl: between(sum("unrealized", "netPnl"), "/", coins?.usd, "ratio") },
      costBasis: edge.costBasis && realizedPrice && { byCoin: { realizedPrice, avg: { mvrv: made(`${t.price.close.name} / ${realizedPrice.name}`, realizedPrice, "ratio") } } },
    };
  };
  /**
   * @param {[string, any][]} list
   * @param {string[]} edges
   * @param {Side} side
   * @param {Bands} d
   * @param {Kin} kin
   * @param {Stored} stored
   * @returns {SectionNode}
   */
  const cutoffFolder = (list, edges, { over, up }, d, kin, stored) => {
    const nodes = list.map(([, node]) => node), at = (/** @type {string} */ key) => list.findIndex(([k]) => k === key);
    const [side, each, place] = over ? [d.family.over, d.family.each, "Over"] : [d.family.under, d.family.under, "Under"];
    const members = edges.flatMap((key) => {
      const low = reported(key, band(key, d.family))?.[d.end];
      if (low === undefined) return [];
      const said = `${side} ${low}${d.after}`;
      const held = stored[key]?.[over ? "over" : "under"];
      if (held) return [{ key: held, c: t.holders[held], naming: { name: shownAs(`${place} ${placeEdge(key, d.end)}${d.after}`, capitalized(said)), who: holder(held).who } }];
      return [{ key, c: cutoffCohort(nodes, at(key), up), naming: { name: shownAs(`${place} ${placeEdge(key, d.end)}${d.after}`, capitalized(said)), who: d.who(said), inWhole: d.inWhole?.(said) } }];
    });
    const naming = new Map(members.map((m) => [m.key, m.naming]));
    return section(shownAs(`${place} ${d.article}`, capitalized(`${each} each ${d.noun}`)), family(members.map((m) => [m.key, m.c]), (key) => naming.get(key), { each: `${each} each ${d.noun}`, ...kin, urpd: (key) => (Object.hasOwn(t.holders, key) ? holderName(key).urpd : undefined) }));
  };
  /**
   * @param {[string, any][]} list
   * @param {string[]} edges
   * @param {Bands} d
   * @param {Kin} [kin]
   * @param {Side[]} [sides]
   * @param {Stored} [stored]
   * @returns {SectionNode[]}
   */
  const cutoffs = (list, edges, d, kin = {}, sides = [OVER, UNDER], stored = {}) => {
    const at = (/** @type {string} */ key) => list.findIndex(([k]) => k === key);
    const known = edges.filter((key) => {
      if (at(key) < 0) report(`no band ${key} to cut at; its cutoffs left out`);
      return at(key) >= 0;
    });
    return sides.map((side) => cutoffFolder(list, known.filter((key) => (side.up ? list.length - at(key) : at(key)) > 1), side, d, kin, stored));
  };
  /** @typedef {{ over: boolean, up: boolean }} Side */
  /**
   * The holder groups stored whole at a cutoff's edge, taken in place of the bands' sums there: the heatmaps and
   * percentiles no sum makes come with them.
   * @typedef {Record<string, { over: string, under: string }>} Stored
   */
  const OVER = { over: true, up: true }, UNDER = { over: false, up: false };
  /**
   * A profit band holds its high edge, a loss band its low one (see BAND_SIDES): its bands run from the deepest loss
   * up, so a loss cutoff takes the bands under its edge (see profitCutoffs).
   * @param {string} key @returns {{ bands: Bands, profits: Bands, losses: Bands, kin: Kin }}
   */
  const profitsOf = (key) => {
    const g = holderName(key);
    /** @type {Bands} */
    const profits = { noun: "profit level", article: "a profit", family: EDGES.through, end: "low", after: " in profit", who: (said) => capitalized(`${g.coins} ${said}`), inWhole: (said) => capitalized(`coins ${said}`) };
    return {
      bands: { ...profits, after: "" }, profits, losses: { ...profits, noun: "loss level", article: "a loss", family: EDGES.from, end: "high", after: " in loss" },
      kin: { lookOf: profitBandLook, ...(key === "all" ? {} : { within: g.who, of: wholeOf(`${g.who.toLowerCase()}'`) }) },
    };
  };
  /** @param {any} bands @param {ReturnType<typeof profitsOf>} sides @returns {SectionNode[]} */
  const profitCutoffs = (bands, { profits, losses, kin }) => {
    const list = entries(bands).reverse();
    return [...cutoffs(list, PROFIT_EDGES, profits, kin, [OVER]), ...cutoffs(list, LOSS_EDGES, losses, kin, [{ over: true, up: false }])];
  };
  /** @type {Stored} */
  const AGE_STORED = { _4mTo5m: { over: "over4m", under: "under4m" }, _6mTo9m: { over: "over6m", under: "under6m" } };
  /** @type {string[]} */
  const AGE_EDGES = ["_1dTo1w", "_1wTo1m", "_1mTo2m", "_2mTo3m", "_3mTo4m", "_4mTo5m", "_5mTo6m", "_6mTo9m", "_9mTo1y", "_1yTo18m", "_18mTo2y", "_2yTo3y", "_3yTo4y", "_4yTo5y", "_5yTo6y", "_6yTo7y", "_7yTo8y", "_8yTo10y", "_10yTo12y", "_12yTo15y", "over15y"];
  /** @type {string[]} */
  const PROFIT_EDGES = ["profit10To20Percent", "profit20To30Percent", "profit30To40Percent", "profit40To50Percent", "profit50To60Percent", "profit60To70Percent", "profit70To80Percent", "profit80To90Percent", "profit90To100Percent", "profit100To200Percent", "profit200To300Percent", "profit300To500Percent", "profit500To1000Percent"];
  /**
   * Each by the band just short of it: those under that one are at least so far in loss.
   * @type {string[]}
   */
  const LOSS_EDGES = ["loss0To10Percent", "loss10To20Percent", "loss20To30Percent", "loss30To40Percent", "loss40To50Percent", "loss50To60Percent", "loss60To70Percent", "loss70To80Percent"];
  /** @type {string[]} */
  const AMOUNT_EDGES = ["_1To10Sats", "_10To100Sats", "_100To1kSats", "_1kTo10kSats", "_10kTo100kSats", "_100kTo1mSats", "_1mTo10mSats", "_10mTo100mSats", "_1To10Btc", "_10To100Btc", "_100To1kBtc", "_1kTo10kBtc", "_10kTo100kBtc", "over100kBtc"];

  const m = t.market, sup = t.supply, mining = t.mining, blocks = t.blocks, tx = t.transactions, ct = t.cointime, cf = t.coinflow;
  const step = (/** @type {any} */ total) => `diff(${total.cumulative.name}, 1)`, txSteps = `${step(tx.count)} - ${step(blocks.count)}`;
  const underTenthSatVb = [tx.fees.zeroFee.txCount, tx.fees.txCountByFeeRate.under0p1SatVb];
  const underOneSatVb = [...underTenthSatVb, ...(/** @type {const} */ (["_0p1To0p25SatVb", "_0p25To0p5SatVb", "_0p5To1SatVb"])).map((key) => tx.fees.txCountByFeeRate[key])];

  /** @param {any} node @param {string} kind */
  const averages = (node, kind) => {
    const list = entries(node), colors = list.map((_, i) => recencyColor(list.length - 1 - i, list.length)), long = node._200d;
    if (!long?.ratio) throw new Error(`Moving averages: no 200-day ${kind} with its ratio`);
    const over = (/** @type {string} */ name, /** @type {Layer} */ ratio, /** @type {string} */ title, /** @type {boolean} */ mayer) => chart(name, title, [price(), series(own(long), `200-day ${kind}`, { color: colors[list.findIndex(([key]) => key === "_200d")] }), { ...ratio, pane: 2, ...around(1) }], { log: [2], was: Object.fromEntries(list.filter(([, value]) => Boolean(value.mayerMultiple) === mayer).map(([key]) => [spanLabel(key), key])) });
    return section(`Moving averages, ${kind}`, [
      chart("Compare", `${kind}s of the price`, [price(), ...list.map(([key, value], i) => series(own(value), `${windowAdjective(key)} ${kind}`, { color: colors[i], ...(["_50d", "_200d", "_200w"].includes(key) ? {} : { hidden: true }) }))]),
      over("Price ratio", series(long.ratio, "Price ratio"), `200-day ${kind}`, false),
      long.mayerMultiple && over("Mayer multiple", series(long.mayerMultiple, "Mayer multiple", { levels: [{ value: 2.4, color: "red", dash: 3 }, { value: 0.8, color: "green", dash: 3 }] }), `200-day ${kind} and the Mayer multiple`, true),
    ]);
  };

  /** @param {string} key */
  const overYears = (key) => /y$/.test(key) && key !== "_1y";
  /** @param {boolean} years */
  const returnsWere = (years) => Object.fromEntries(windowsOf(m.returns).filter(([key]) => overYears(key) === years).map(([key]) => [spanLabel(key), key]));

  // The whole market's charts at the top of their subjects, by its sections' names, each renamed there or left out
  // (null): its totals and NUPL are those subjects' own charts already.
  const marketSections = new Map(cohort(t.holders.all, holder("all").who, { whole: true, urpd: holderName("all").urpd }).map((item) => [placeOf(item.name), "items" in item ? item.items : [item]]));
  /** @param {string} name @param {Record<string, string | null>} [renamed] */
  const market = (name, renamed = {}) => (marketSections.get(name) ?? []).flatMap((item) => ((to) => (to === null ? [] : [to === undefined ? item : { ...item, name: to }]))(renamed[placeOf(item.name)]));

  const epoch = Math.floor((facts?.height ?? 840_000) / 210_000), year = facts?.year ?? new Date().getUTCFullYear();
  const derived = chainStatistics({ t, chart, ramp, flowWindow, underOneSatVb });
  const tree = [
    section("Price", [
      chart("In dollars", "Bitcoin price", [price()]),
      chart("Sats per dollar", "Sats per dollar", [series(t.price.satsPerDollar, "Sats per dollar", { color: "orange" })], { log: [1] }),
      chart("All-time high, drawdown", "Price drawdown from its all-time high", [price(), series(m.ath.high, "All-time high", { color: "green" }), series(m.ath.drawdown, "Drawdown", { pane: 2, color: "red", type: "area" })]),
      chart("All-time high, days since", "Days since the price's all-time high", [series(m.ath.daysSince, "Days since", { color: "orange" }), series(m.ath.daysBetweenMax, "Longest between", { color: "gray" })]),
      section("Returns", [
        chart("Short-term", "Price returns, up to a year", windowsOf(m.returns).filter(([key]) => !overYears(key)).map(([key, value], i, all) => series(value, windowSaid(key), { ...baseline, ...shades(i, all.length), ...(key === "_1m" ? {} : { hidden: true }) }))),
        // Over years, returns run from −80% to millions of percent: growth multiples on a log scale, 1× no change.
        chart("Long-term", "Price growth over years", windowsOf(m.returns).filter(([key]) => overYears(key)).map(([key, value], i, all) => ({ formula: `1 + ${value.name} / 100`, label: windowSaid(key), format: "ratio", ...around(1), ...shades(i, all.length), ...(key === "_4y" ? {} : { hidden: true }) })), { log: [1] }),
        chart("Compound annual growth", "Compound annual growth of the price", [price(), ...[["2y", "2 years"], ["4y", "4 years"], ["8y", "8 years"]].map(([span, label], i) => ({ formula: `cagr(A, ${span})`, label, pane: 2, ...around(0), ...shades(i, 3), ...(span === "4y" ? {} : { hidden: true }) })), { formula: "cagr(A)", label: "Since its start", pane: 2, color: "gray", hidden: true }]),
        chart("Return", "30-day price return", [price(), series(m.returns._1m, "30-day return", { pane: 2, ...baseline })], { was: returnsWere(false) }),
        chart("Growth", "4-year price growth", [price(), { formula: `1 + ${m.returns._4y.name} / 100`, label: "4-year growth", format: "ratio", pane: 2, ...around(1) }], { log: [2], was: returnsWere(true) }),
      ]),
      averages(m.sma, "SMA"),
      averages(m.ema, "EMA"),
      chart("Range", "Price's 30-day high and low", [price(), series(m.range.max._1m, "High", { color: "green" }), series(m.range.min._1m, "Low", { color: "red" }), { cloud: ["B", "C"], label: "Range", color: "sky", opacity: 0.1 }], { was: Object.fromEntries(entries(m.range.max).map(([key]) => [`${adjective(spanLabel(key))} high and low`, key])) }),
      section("Technical", [
        chart("RSI", "Relative strength index of the price", [price(), { formula: "rsi(A)", label: "RSI", pane: 2, color: "violet", levels: [{ value: 30, color: "green", dash: 3 }, { value: 70, color: "red", dash: 3 }] }]),
        chart("Stochastic RSI", "Stochastic RSI of the price (14, 14, 3, 3)", [price(), { formula: "stochrsi_k(A)", label: "%K", pane: 2, color: "sky", levels: [{ value: 20, color: "green", dash: 3 }, { value: 80, color: "red", dash: 3 }] }, { formula: "stochrsi_d(A)", label: "%D", pane: 2, color: "orange" }]),
        chart("MACD", "MACD of the price", [price(), { formula: "macd(A)", label: "MACD", pane: 2, color: "sky" }, { formula: "macd_signal(A)", label: "Signal", pane: 2, color: "orange" }, { formula: "macd_hist(A)", label: "Histogram", pane: 2, type: "histogram", colors: 4, color: "green", colorBelow: "red" }], { range: "1y" }),
        chart("Volatility", "Annualized price volatility", [price(), { formula: "volatility(A, 30d)", label: "30 days", pane: 2, color: "sky" }, { formula: "volatility(A, 1y)", label: "1 year", pane: 2, color: "violet" }]),
        // Marked where the 111-day crosses over twice the 350-day.
        chart("Pi cycle", "Pi cycle top", [price(), { formula: "sma(A, 111d)", label: "111-day SMA", color: "green" }, { formula: "2 * sma(A, 350d)", label: "350-day SMA × 2", color: "red" }, { formula: "B > C", type: "signal", ref: "A", label: "Top signal", color: "red" }]),
        // Trader Dončić's 2021 twin of the top. 0.745, not the 0.475 of one TradingView text, which crosses at
        // neither the 2015 nor the 2018 low.
        chart("Pi cycle bottom", "Pi cycle bottom", [price(), { formula: "ema(A, 150d)", label: "150-day EMA", color: "green" }, { formula: "0.745 * sma(A, 471d)", label: "471-day SMA × 0.745", color: "red" }, { formula: "B < C", type: "signal", ref: "A", label: "Bottom signal", color: "green" }]),
        chart("Golden and death crosses", "Golden and death crosses", [price(), { formula: "sma(A, 50d)", label: "50-day SMA", color: "amber" }, { formula: "sma(A, 200d)", label: "200-day SMA", color: "sky" }, { formula: "(B > C) - (B < C)", type: "signal", ref: "A", place: "below", results: true, label: "Cross", color: "green" }]),
        chart("Trailing stop", "A 20-day trailing stop", [price(), { formula: "shift(highest(A, 20d), 1d)", label: "20-day high", color: "gray", dash: 2, hidden: true }, { formula: "shift(lowest(A, 20d), 1d)", label: "20-day low", color: "gray", dash: 2, hidden: true }, { trend: ["B", "C"], ref: "A", label: "Trailing stop", color: "red", colorBelow: "green", width: 2 }], { range: "1y" }),
        ((cycles) => chart("Halving cycles", "The price's halving cycles, each from its start", [price(), ...Array.from({ length: cycles }, (_, n) => ({ formula: `halving(A, ${n})`, label: `${n ? 2008 + 4 * n : 2009}–${n + 1 < cycles ? 2012 + 4 * n : "now"}`, pane: 2, color: recencyColor(n, cycles) }))], { log: [2], halvings: true, range: "max" }))(epoch + 1),
        chart("Years", "The price's years, each from its start", [price(), ...Array.from({ length: 8 }, (_, k) => ({ formula: `year(A, ${year - 7 + k})`, label: String(year - 7 + k), pane: 2, color: recencyColor(k, 8) }))], { log: [2], range: "8y" }),
        .../** @type {[string, string, string, number, string, number, { halvings?: boolean, range: string }][]} */ ([["Halving cycles overlaid", "The price's cycle since the last halving, over the ones before it", "halving", epoch, "cycle", Math.min(3, epoch - 1), { halvings: true, range: "4y" }], ["Years overlaid", "The price's year so far, over the ones before it", "year", year, "year", 3, { range: "1y" }]]).map(([name, title, f, now, what, before, options]) =>
          chart(name, title, [price(), ...Array.from({ length: before + 1 }, (_, k) => ({ formula: k ? `${f}(shift(A, ${k}${f === "halving" ? "halving" : "y"}), ${now})` : `${f}(A, ${now})`, label: k ? `${k} ${what}${k > 1 ? "s" : ""} before` : `This ${what}`, pane: 2, color: recencyColor(3 - k, 4), ...(k ? {} : { width: 2 }) }))], { log: [2], ...options })),
      ]),
    ], { icon: "candles" }),
    section("Valuation", [
      chart("Market and realized cap", "Market cap and realized cap", [series(t.holders.all.supply.total.marketCap, "Market cap", { color: "green" }), series(t.holders.all.capital.realizedCap, "Realized cap", { color: "orange" })], { log: [1], priceHidden: true }),
      ...change(sup.marketCap.delta, "Market cap change", "Market cap change"),
      chart("MVRV", "Market value to realized value (MVRV)", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: "orange" }), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1), levels: [{ value: 3.5, color: "red", dash: 3 }] })], { log: [2] }),
      // Gradient stops are places in MVRV's range at a day (0.40 to 9.0): 0.66 at 3%, 1 at 7%, 2 at 19%.
      chart("MVRV by value", "MVRV colored by its value", [price(), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, type: "line", width: 2, byGradient: true, gradient: [[3, "red"], [7, "gray"], [19, "green"]], levels: [{ value: 1, color: "gray", dash: 3 }] })], { log: [2] }),
      chart("Price by MVRV zone", "The price by its MVRV zone", [{ ...price(), hidden: true }, series(t.price.close, "Price by zone", { colorFrom: "C", color: "gray", width: 2 }), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, color: "green", byLevels: true, underName: "Undervalued", levels: [{ value: 1, color: "gray", name: "Fair", dash: 3 }, { value: 3.5, color: "red", name: "Overvalued", dash: 3 }] })], { log: [1, 2] }),
      chart("MVRV Z-score", "MVRV Z-score", [price(), { formula: `(${t.holders.all.supply.total.marketCap.name} - ${t.holders.all.capital.realizedCap.name}) / stdev(${t.holders.all.supply.total.marketCap.name})`, label: "MVRV Z-score", pane: 2, format: "number", ...baseline, levels: [{ value: 7, color: "red", dash: 3, name: "Top zone" }] }]),
      // The volatility is the 30 days' own, not annualized (× √(30/365)).
      chart("Seller exhaustion", "Seller exhaustion constant", [price(), { formula: `${t.holders.all.supply.inProfit.share.name} / 100 * volatility(A, 30d) / 100 * ${Math.sqrt(30 / 365).toFixed(4)}`, label: "Seller exhaustion constant", format: "ratio", pane: 2, color: "orange" }], { log: [2] }),
      chart("NUPL", "Net unrealized profit and loss (NUPL)", [price(), series(t.holders.all.unrealized.nupl, "NUPL", { pane: 2, ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
      // A day's volume swings: faint, its 90-day average the signal, as the hash rate's.
      ((cap, volume) => chart("NVT", "Network value to transactions (NVT)", [price(), series(sup.nvt, "NVT", { pane: 2, color: "orange", width: 1, opacity: 0.4 }), { formula: `${cap} / sma(${volume}, 90d)`, label: "NVT signal", format: "ratio", pane: 2, color: "orange", width: 2 }], { log: [2] }))(sup.marketCap.value.name, tx.transferVolume.sum._24h.usd.name),
      ((cap, thermo) => chart("Thermocap multiple", "Thermocap multiple", [price(), { formula: `${cap} / ${thermo}`, label: "Thermocap multiple", format: "ratio", pane: 2 }], { log: [2] }))(sup.marketCap.value.name, ct.caps.thermo.name),
      // The subsidy as 52,560 blocks a year (the schedule, not the blocks found); the model price is PlanB's 2019 fit
            // of market cap to it (e^14.6 × S2F^3.3).
      ((supply, epoch) => ((s2f) => chart("Stock-to-flow", "Stock-to-flow", [price(), { formula: `exp(14.6) * (${s2f}) ^ 3.3 / ${supply}`, label: "Model price", format: "usd", color: "violet", dash: 3 }, { formula: s2f, label: "Stock-to-flow", format: "number", pane: 2, color: "orange" }], { log: [2] }))(`${supply} / (50 * 0.5 ^ ${epoch} * 52560)`))(t.holders.all.supply.total.btc.name, blocks.halving.epoch.name),
      (({ intercept, slope, support }, days) => chart("Power law", "Power-law fit and support line", [price(), { formula: `10 ^ (${intercept} + ${slope} * log(${days}))`, label: "Power-law fit", format: "usd", color: "violet", dash: 3 }, { formula: `10 ^ (${intercept} - ${support} + ${slope} * log(${days}))`, label: "Power-law floor", format: "usd", color: "green" }]))(POWER_LAW, "(date - 2009-01-03) / 86400"),
      chart("RHODL", "RHODL ratio", [price(), series(t.age.rhodlRatio, "RHODL ratio", { pane: 2 })], { log: [2] }),
      section("On-chain prices", [
        chart("Compare", "Realized, true market mean, Cointime and capitalized prices", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: PRICE_COLORS.realizedPrice }), series(ct.prices.trueMarketMean.value, "True market mean", { color: PRICE_COLORS.trueMarketMean }), series(ct.prices.cointime.value, "Cointime price", { color: PRICE_COLORS.cointime }), series(t.holders.all.costBasis.byDollar.capitalizedPrice, "Capitalized price", { color: PRICE_COLORS.capitalizedPrice })]),
        // Terminal marks the tops; the other three sit under the price at the lows.
        chart("Cumulative destruction", "CVDD, balanced, transferred and terminal prices", [price(), series(ct.prices.cvdd, PRICE_NAMES.cvdd, { color: PRICE_COLORS.cvdd }), series(ct.prices.balanced, "Balanced price", { color: PRICE_COLORS.balanced }), series(ct.prices.transferred, "Transferred price", { color: "gray" }), series(ct.prices.terminal, "Terminal price", { color: "red" })]),
        chart("Delta price", "Delta price and realized price", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: PRICE_COLORS.realizedPrice }), series(ct.prices.delta, "Delta price", { color: PRICE_COLORS.delta })]),
        cloudsChart(),
        section("Compass", compassCharts()),
        chart("Cointime caps", "Cointime capitalizations", [...[["thermo", "Thermo cap"], ["investor", "Investor cap"], ["active", "Active cap"], ["vaulted", "Vaulted cap"], ["cointime", "Cointime cap"]].map(([key, label], i) => series(ct.caps[key], label, { color: ["orange", "green", PRICE_COLORS.active, PRICE_COLORS.vaulted, PRICE_COLORS.cointime][i] }))], { log: [1] }),
        section("Cointime prices", [
          chart("Compare", "Cointime prices", [price(), ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label]) => series(ct.prices[key].value, label, { color: PRICE_COLORS[key] })), series(ct.prices.trueMarketMean.value, "True market mean", { color: PRICE_COLORS.trueMarketMean })]),
          ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label]) => chart(label, label, [price(), series(ct.prices[key].value, label, { color: PRICE_COLORS[key] }), series(ct.prices[key].mvrv, "MVRV", { pane: 2, ...around(1) })], { log: [2] })),
          timeAboveChart(),
          chart("Active less investor cap", "Active cap minus investor cap", [{ formula: `${ct.caps.active.name} - ${ct.caps.investor.name}`, label: "Active less investor cap", format: "usd", ...baseline }]),
          chart("True market mean", "True market mean and AVIV", [price(), series(ct.prices.trueMarketMean.value, "True market mean", { color: "sky" }), series(ct.prices.trueMarketMean.aviv, "AVIV", { pane: 2, ...around(1) })], { log: [2] }),
          chart("AVIV NUPL", "AVIV NUPL, against the true market mean", [price(), series(ct.prices.trueMarketMean.avivNupl, "AVIV NUPL", { pane: 2, ...baseline })]),
        ]),
      ]),
      chart("Reserve risk", "Reserve risk", [price(), series(ct.reserveRisk.value, "Reserve risk", { pane: 2, color: "orange" })], { log: [2] }),
      chart("HODL bank", "HODL bank, reserve risk's denominator", [price(), series(ct.reserveRisk.hodlBank, "HODL bank", { pane: 2, color: "sky" })], { log: [2] }),
      // Short name, the long one in the title: long, it and its all-time twin read alike when cut short.
      ...flow(ct.reserveRisk.vocdd, "VOCDD", "Value of coin days destroyed (VOCDD)", { dollars: true }),
      section("Rarity meter", [
        ...entries(t.rarityMeter).filter(([key]) => ["full", "local", "cycle"].includes(key)).flatMap(([key, node]) => [
          chart(`${capitalized(key)}, bands`, `Rarity meter bands, ${key}`, bandLayers([price()], node, (v) => v)),
          chart(`${capitalized(key)}, level`, `Rarity meter level, ${key}`, [series(node.level, "Level", { ...baseline, type: "histogram" })]),
          chart(`${capitalized(key)}, score`, `Rarity meter score, ${key}`, [series(node.score, "Score", baseline)]),
        ]),
        section("Components", entries(t.rarityMeter.components).map(([key, node]) => holderSection(key, rarityComponents(key, node)))),
        section("Extremes", [eventsChart(), ...entries(t.rarityMeter.extremes).flatMap(([key, node]) => {
          if (!EXTREMES[key]) {
            report(`no words for the rarity meter extreme ${key}; left out`);
            return [];
          }
          const { what, window, low } = EXTREMES[key], measure = capitalized(what), said = window ? `${measure}, ${windowSaid(window)}` : measure;
          return [chart(window ? shownAs(`${measure}, ${window.slice(1)}`, said) : measure, `Rarity bands of ${what}`, [series(own(node), said, { color: "orange", width: 2 }), ...bandLines(node, (v) => v, (p) => p === 50 || (low ? p <= 1 : p >= 99))], { log: [1], window })];
        })]),
      ]),
      section("Realized cap", [...market("Capital", { Total: null }), chart("Change, percent", "Realized cap change, in percent", windows(t.holders.all.capital.delta.percent, "", { around: 0 }))]),
      section("Cost basis", market("Cost basis")),
      section("Bedrock", [
        section("Compare", entries(entries(t.bedrock)[0][1].floor).map(([key]) => chart(`${percentileLabel(percentile(key))} floor`, `Bedrock ${percentileLabel(percentile(key))} floor by mode`, [price(), ...entries(t.bedrock).map(([mode, node], i, all) => series(node.floor[key], modeName(mode), { color: ramp(all.length)[i] }))]))),
        ...entries(t.bedrock).map(([mode, node]) => {
          const edges = [
            [m.ath.high, "All-time high"],
            ...entries(node.costBasisAboveP95Floor).reverse().map(([key, value]) => [value, `${percentileLabel(percentile(key))} cost basis`]),
            ...entries(node.floor).map(([key, value]) => [value, `${percentileLabel(percentile(key))} floor`]),
          ];
          const bands = entries(node.entrySupply), colors = ramp(bands.length, 140, 200);
          const supply = { awake: ct.cohorts.all.awake.supply.total.btc, mobile: cf.cohorts.all.mobile.supply.total.btc, unweighted: t.holders.all.supply.total.btc }[mode];
          return section(modeName(mode), [
            chart("Floors", `Bedrock floors, ${mode}`, [price(), ...entries(node.floor).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 140, 200)[i] }))]),
            chart("Floors, colored", `Bedrock landscape, ${mode}`, [
              price(),
              ...edges.map(([value, label], i) => series(value, label, { color: LANDSCAPE[i], width: 1 })),
              { cloud: [letter(1), "top"], label: "Above the all-time high", color: SKY, opacity: 1 },
              { cloud: [...edges.map((_, i) => letter(i + 1)), "bottom"], label: "Water, sand and rock", color: LANDSCAPE[0], byLayers: true, opacity: 1 },
            ]),
            chart("Supply between floors", `Supply created between Bedrock floors, ${mode}`, [
              ...bands.map(([key, value], i) => series(value.btc, floorBandLabel(key), { color: colors[i], type: "area" })),
            ], { stack: { 1: "stack" } }),
            chart("Capital between floors", `Capital of the supply created between Bedrock floors, ${mode}`, entries(node.entryCapital).map(([key, value], i) => series(value, floorBandLabel(key), { color: colors[i], type: "area" })), { stack: { 1: "stack" } }),
            chart("Cost basis above the floor", `Cost basis above the Bedrock floor, ${mode}`, [price(), ...entries(node.costBasisAboveP95Floor).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 160)[i], width: key === "median" ? 2 : 1 }))]),
            chart("Supply-in-loss thresholds", `Bedrock's supply-in-loss thresholds, ${mode}`, entries(node.supplyInLossShareThreshold).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 100)[i] }))),
          ]);
        }),
      ]),
    ], { icon: "scale" }),
    section("Profit and loss", [
      ...market("Realized", { "Profit and loss": "Realized profit and loss", "Net profit and loss": "Net realized profit and loss" }),
      ...market("Unrealized", { "Profit and loss": "Unrealized profit and loss", "Net profit and loss": "Net unrealized profit and loss", NUPL: null, "Net, relative to market cap": null }),
    ], { icon: "percent" }),
    section("Supply", [
      chart("Circulating", "Circulating supply", [series(sup.circulating, "Supply", { color: "orange" })]),
      chart("Inflation", "Inflation rate", [series(sup.inflationRate, "Inflation rate", { color: "orange" }), series(ct.adjusted.inflationRate, "Cointime-adjusted", { color: "violet" })], { log: [1] }),
      chart("Annual issuance", "New coins a year, at the inflation rate", [
        series(shareAmount(sup.inflationRate, btc(sup.circulating), "btc"), "Annual issuance", { color: "orange" }),
        series(shareAmount(ct.adjusted.inflationRate, btc(sup.circulating), "btc"), "Cointime-adjusted", { color: "violet" }),
      ], { log: [1] }),
      ...flow(sup.burned, "Burned", "Bitcoin burned"),
      ...market("Supply", { Total: null }),
      ...lastMovedCharts(),
      // HODLed or lost is the vaulted supply by another name: one line.
      chart("Vaulted and active", "Vaulted and active supply", [["vaulted", "Vaulted"], ["active", "Active"]].map(([key, label]) => series(ct.supply[key].btc, label, { color: PRICE_COLORS[key], type: "area" })), { stack: { 1: "stack" } }),
    ], { icon: "coins" }),
    section("Activity", [
      chart("Velocity", "Velocity", [series(sup.velocity.btc, "In bitcoin", { color: "orange" }), series(sup.velocity.usd, "In dollars", { color: "green" }), series(ct.adjusted.velocity.btc, "Cointime-adjusted, in bitcoin", { color: "violet", hidden: true }), series(ct.adjusted.velocity.usd, "Cointime-adjusted, in dollars", { color: "sky", hidden: true })]),
      ...market("UTXOs", { Change: "UTXO change", Spent: "UTXOs spent" }),
      ...market("Activity"),
      ...inDollars(t.holders.all.activity.transferVolume, "Transfer volume, in dollars", "Transfer volume, in dollars"),
      section("Cointime", [
        ...flow(ct.activity.coinblocksCreated, "Coin blocks created", "Coin blocks created"),
        ...flow(ct.activity.coinblocksDestroyed, "Coin blocks destroyed", "Coin blocks destroyed"),
        ...flow(ct.activity.coinblocksStored, "Coin blocks stored", "Coin blocks stored"),
        chart("Liveliness", "Liveliness and vaultedness", [series(ct.activity.vaultedness, "Vaultedness", { color: PRICE_COLORS.vaulted, type: "area" }), series(ct.activity.liveliness, "Liveliness", { color: PRICE_COLORS.active, type: "area" })], { stack: { 1: "stack" } }),
        chart("Investorness and producerness", "Investorness and producerness, Cointime's cap ratios", [series(ct.caps.investorness, "Investorness", { color: "green" }), series(ct.caps.producerness, "Producerness", { color: "orange" })]),
        chart("Concurrent liveliness", "Concurrent liveliness", windows(ct.activity.concurrentLiveliness, "")),
        ...entries(ct.value).flatMap(([key, node]) => flow(node, `Cointime value ${key}`, `Cointime value ${key}`)),
      ]),
    ], { icon: "swap" }),
    section("Holders", [
      ...family(entries(t.holders).filter(([key]) => key === "all" || key === "sth" || key === "lth"), holder, { each: "by holder group", urpd: (key) => holderName(key).urpd, profile: true, wholeAbove: true }),
      ...cutoffs(entries(t.age.ranges), AGE_EDGES, AGES, {}, [UNDER, OVER], AGE_STORED),
      // The oldest band has no upper edge: nothing ages out of it, its aged-out series are zeros by construction, so no
      // such charts.
      ((awake, mobile) => section("Age bands", family(entries(t.age.ranges).map(([key, c], i, all) => /** @type {[string, any]} */ (i === all.length - 1 ? [key, { ...c, supply: { ...c.supply, agedOut: undefined } }] : [key, c])), ageBand, { each: "by age band", disjoint: true, profile: true, urpd: (key) => `age_${client.AGE_RANGE_NAMES[key]?.id ?? key}`, known: { Supply: "HODL waves", "Realized cap": "realized cap HODL waves" }, more: { compare: [...awake.compare, ...mobile.compare], activity: new Map([...awake.activity].map(([key, charts]) => [key, [...charts, ...(mobile.activity.get(key) ?? [])]])) } })))(cointimeBands(ct.ageRanges), coinflowBands(cf.ageRanges)),
      section("Vintages", family(members(t.age.vintages), (key) => ({ name: key.slice(1), who: `Coins last moved in ${key.slice(1)}` }), { each: "by vintage", disjoint: true, inTime: true })),
      section("Halving epochs", family(members(t.age.halvingEpochs), (key) => ({ name: `Epoch ${key.slice(1)}`, who: `Coins last moved in halving epoch ${key.slice(1)}` }), { each: "by halving epoch last moved", disjoint: true, inTime: true })),
      section("Entry price", family(entries(t.entry), (key) => ({ name: named(client.ENTRY_NAMES, key), who: named(client.ENTRY_NAMES, key) }), { each: "by entry price", disjoint: true, lookOf: (key) => (key === "discount" ? { color: "green" } : key === "premium" ? { color: "red" } : undefined) })),
      section("Profit bands", holderSets(entries(t.profitability), (groups) => groups.map(([key, bands]) => {
        const sides = profitsOf(key), d = sides.bands;
        return holderSection(key, [...profitCutoffs(bands, sides), ...family(entries(bands), bandMember(client.PROFITABILITY_RANGE_NAMES, d), { each: "by profit band", disjoint: true, ...sides.kin })]);
      }))),
      holderSection("awake", weightedSets(ct.cohorts, "awake", "wakefulness")),
      holderSection("mobile", weightedSets(cf.cohorts, "mobile", "mobility")),
    ], { icon: "people" }),
    section("Mining", [
      section("Hashing", [
        chart("Hash rate", "Hash rate", [series(own(mining.hashrate), "Hash rate", { color: "orange", width: 1, opacity: 0.4 }), ...windows(mining.hashrate.sma, "Average", { extra: { width: 2 } }), series(mining.hashrate.ath, "All-time high", { color: "green", dash: 3, hidden: true }), series(blocks.difficulty.hashrate, "From difficulty", { color: "gray", hidden: true })], { log: [1] }),
        chart("Hash rate drawdown", "Hash rate drawdown from its all-time high", [series(mining.hashrate.drawdown, "Drawdown", { color: "red", type: "area" })]),
        // On a log scale: it has grown by decades with the hash rate.
        chart("Block work", "Block work, the hashes expected to find the blocks", windows(blocks.work.sum, ""), { log: [1], was: allTimeWas("Block work") }),
        // No marks where it crosses back: the averages cross too often to read.
        chart("Hash ribbons", "Hash ribbons", [
          price(),
          series(mining.hashrate.sma._1m, `${windowAdjective("_1m")} average`, { pane: 2, color: "sky" }),
          series(mining.hashrate.sma._2m, `${windowAdjective("_2m")} average`, { pane: 2, color: "violet" }),
          { formula: "B < C", type: "signal", ref: "B", place: "shade", label: "Capitulation", color: "red" },
        ], { log: [2] }),
        chart("Hash price", "Hash price", [series(own(mining.hashPrice), "Hash price", { color: "green" }), series(mining.hashPrice.atl, "Hash price's all-time low", { color: "red", dash: 3 })], { log: [1] }),
        chart("Hash value", "Hash value", [series(own(mining.hashValue), "Hash value", { color: "orange" }), series(mining.hashValue.atl, "Hash value's all-time low", { color: "red", dash: 3 })], { log: [1] }),
        chart("Rebound", "Hash price and hash value above their all-time lows", [series(mining.hashPrice.rebound, "Hash price", { color: "green" }), series(mining.hashValue.rebound, "Hash value", { color: "orange" })]),
      ]),
      section("Difficulty", [
        chart("History", "Difficulty", [series(blocks.difficulty.value, "Difficulty", { color: "orange", shape: "steps" })], { log: [1] }),
        chart("Adjustments", "Difficulty adjustments", [series(blocks.difficulty.adjustment, "Adjustment", { type: "histogram", color: "green", colorBelow: "red" })]),
        chart("Next adjustment", "Until the next difficulty adjustment", [series(blocks.difficulty.blocksToRetarget, "Blocks", { color: "sky" })], { range: "3m" }),
        chart("Next adjustment, in days", "Days until the next difficulty adjustment", [series(blocks.difficulty.daysToRetarget, "Days", { color: "orange" })], { range: "3m" }),
        chart("Epoch", "Difficulty epoch", [series(blocks.difficulty.epoch, "Epoch", { color: SUM_COLOR, shape: "steps" })]),
      ]),
      section("Halvings", [
        chart("Next halving", "Until the next halving", [series(blocks.halving.blocksToHalving, "Blocks", { color: "sky" })]),
        chart("Next halving, in days", "Days until the next halving", [series(blocks.halving.daysToHalving, "Days", { color: "orange" })]),
        chart("Epoch", "Halving epoch", [series(blocks.halving.epoch, "Halving epoch", { color: SUM_COLOR, shape: "steps" })]),
      ]),
      section("Rewards", [
        ...flow(mining.rewards.coinbase, "Coinbase", "Coinbase rewards"),
        ...inDollars(mining.rewards.coinbase, "Coinbase, in dollars", "Coinbase rewards, in dollars"),
        ...flow(mining.rewards.subsidy, "Subsidy", "Block subsidy"),
        ...flow(mining.rewards.unclaimed, "Unclaimed", "Unclaimed rewards"),
        // The month's alone: two windows in one stack would add up to two wholes.
        ((parts) => chart("Subsidy and fee shares", "Subsidy's and fees' shares of rewards", [
          ...parts.map(([share, what, color]) => series(share._1m, `${what}, ${windowSaid("_1m")}`, { color, type: "area" })),
        ], { stack: { 1: "stack" } }))(/** @type {[any, string, string][]} */ ([[mining.rewards.fees.share, "Fees", "green"], [mining.rewards.subsidy.share, "Subsidy", "orange"]])),
        chart("Puell multiple", "Puell multiple", [price(), series(mining.rewards.puellMultiple, "Puell multiple", { pane: 2, ...around(1), levels: [{ value: 4, color: "red", dash: 3 }, { value: 0.5, color: "green", dash: 3 }] })], { log: [2] }),
      ]),
      section("Pools", poolCharts(), { workings: true }),
    ], { icon: "pickaxe" }),
    section("Fees", [
      ...flow(mining.rewards.fees, "Fees paid", "Transaction fees"),
      ...perBlockFan(mining.rewards.fees, "Fees per block", "Fees per block", true, btc),
      ...perBlockFan(mining.rewards.fees, "Fees per block, in dollars", "Fees per block, in dollars", true, usd),
      ...spread(tx.fees.effectiveFeeRate, "Fee rate", "transactions' effective fee rates", "orange", true),
      ...(isLeaf(tx.fees.effectiveFeeRate?.median) && blocks.fullness?.avg?._1m ? [chart("Fee rate and fullness", "Transaction fee rates and block fullness", [
        { ...price(), hidden: true },
        { formula: tx.fees.effectiveFeeRate.p25.name, label: "25th", pane: 2, color: "orange", width: 1, opacity: 0.5, hidden: true },
        { formula: tx.fees.effectiveFeeRate.median.name, label: "Median fee rate", pane: 2, color: "orange", width: 2 },
        { formula: tx.fees.effectiveFeeRate.p75.name, label: "75th", pane: 2, color: "orange", width: 1, opacity: 0.5, hidden: true },
        { cloud: ["B", "D"], label: "25th to 75th", color: "orange", opacity: 0.25, pane: 2 },
        series(blocks.fullness.avg._1m, `Block fullness, ${windowAdjective("_1m")} average`, { pane: 3, color: "sky", levels: [{ value: 100, color: "gray", dash: 3 }] }),
      ], { log: [2], average: { 2: "7d" }, size: { 1: 1, 2: 2, 3: 1.4 } })] : []),
      section("By fee rate", feeRates(tx.fees.txCountByFeeRate, (said) => `transactions paying ${said}`, "fee rate", { disjoint: true, more: [
        chart("Under 1 sat/vB, by month", "Transactions paying under 1 sat/vB, by month", [
          { formula: `100 * (${underOneSatVb.map(step).join(" + ")}) / (${txSteps})`, label: "Share of transactions", type: "histogram", color: "orange", format: "percent", maxInterval: "month1" },
          { formula: `${step(blocks.weight)} / ${step(blocks.count)} / 40000`, label: "Block fullness", type: "line", color: "sky", width: 2, format: "percent", pane: 2, maxInterval: "month1", levels: [{ value: 100, color: "gray", dash: 3 }] },
        ], { index: "month1", size: { 2: 1.5 } }),
        chart("Under 0.1 sat/vB, by day", "Transactions paying under 0.1 sat/vB, by day", [
          { formula: underTenthSatVb.map(step).join(" + "), label: "Transactions, zero-fee ones too", type: "histogram", color: "orange", format: "number", maxInterval: "day1" },
          { formula: `100 * (${underTenthSatVb.map(step).join(" + ")}) / (${txSteps})`, label: "Share of transactions", type: "line", color: "orange", format: "percent", pane: 2, maxInterval: "day1" },
        ], { size: { 2: 1.5 } }),
      ] }, tx.fees.zeroFee.txCount)),
      section("Block space by effective fee rate", feeRates(tx.fees.vsizeByEffectiveFeeRate, (said) => `block space (vsize) at effective fee rates ${said}`, "effective fee rate", { what: "block space (vsize)", disjoint: true, count: false })),
      ...spread(tx.fees.fee, "Fee per transaction", "transactions' fees", "amber"),
    ], { workings: true, icon: "bolt" }),
    section("Network", [
      section("Blocks", [
        ...flow(blocks.count, "Count", numberOf("blocks mined")),
        chart("Interval", "Time between blocks", windows(blocks.interval.avg, "", { extra: { levels: [{ value: 600, color: "gray", dash: 3 }] } })),
        ...stats(blocks.size, "Size", "Total block size", "Block size"),
        ...stats(blocks.weight, "Weight", "Total block weight", "Block weight"),
        ...perBlockFan(blocks.fullness, "Fullness", "Block fullness", false),
        ...flow(blocks.sigopCost, "Signature operations", "Signature operation cost"),
        ...flow(blocks.emptyCount, "Empty", numberOf("empty blocks")),
        ...entries(blocks.versionBits).flatMap(([key, node]) => flow(node, `Version ${keyWords(key)}`, numberOf(`blocks signaling version ${keyWords(key)}`))),
        section("Coinbase locktime", [
          ...flow(blocks.coinbaseLocktime.nonzero, "Nonzero", numberOf("blocks with a nonzero coinbase locktime")),
          ...flow(blocks.coinbaseLocktime.bip54, "BIP-54", numberOf("blocks whose coinbase locktime matches BIP-54's rule")),
        ]),
      ]),
      section("Transactions", [
        ...stats(tx.count, "Count", numberOf("transactions")),
        chart("Per second", "Transactions per second", windows(tx.perSecond, "")),
        ...spread(tx.vsize, "Size", "transactions' sizes (vsize)", "sky"),
        ...flow(tx.vsize, "Total size", "Total transaction size (vsize)"),
        section("Payments", [
          ...flow(tx.payments.count, "Count", numberOf("payments, estimated from transaction outputs")),
          ...flow(tx.payments.segwitSpending, "SegWit inputs", numberOf("payments in transactions with SegWit inputs")),
          ...flow(tx.payments.rbfSignaling, "RBF signaling", numberOf("payments in transactions signaling RBF")),
        ]),
        section("Nonstandard", kindCharts(tx.nonstandard, "nonstandard transactions")),
        section("Features", [
          ...kinds(entries(tx.features).filter(([, node]) => isLeaf(node.cumulative)), featureName, featureTx, "feature"),
          ...flow(tx.ephemeralDust, "Spending ephemeral dust", numberOf("transactions spending ephemeral dust")),
          ...flow(tx.sameBlockSpending, "Spending same-block outputs", numberOf("transactions spending outputs from the same block")),
          section("Scripts", kinds(entries(tx.features.scripts), featureName, featureTx, "script")),
          section("Signatures", kinds(entries(tx.features.signatures), featureName, featureTx, "signature feature")),
        ]),
        section("Timelocks", kinds(entries(tx.timelocks), featureName, featureTx, "timelock")),
        section("Spending", kinds(entries(tx.spending), featureName, featureTx, "inputs spent", { disjoint: true })),
        section("Versions", [
          section("Compare", [
            ...parts(["Transactions", "Number of transactions by version"], ["Transaction share", "Share of transactions by version"], entries(tx.versions).map(([key, node], i, all) => series(node.sum._1m, versionName(key), { color: ramp(all.length)[i] })), "_1m"),
            ...parts(["Value", "Value moved by version"], ["Value share", "Share of transfer volume by version"], entries(tx.versions).map(([key, node], i, all) => series(node.value.sum._1m.btc, versionName(key), { color: ramp(all.length)[i] })), "_1m"),
          ]),
          ...entries(tx.versions).map(([key, node]) => section(versionName(key), kindCharts(node, key === "other" ? "transactions of other versions" : `version ${key.slice(1)} transactions`))),
        ]),
        section("Patterns", kinds(entries(tx.patterns), words, (key) => `${keyWords(key)} transactions`, "pattern")),
        section("CPFP, parents", kindCharts(tx.fees.cpfpParent, "CPFP parent transactions")),
        section("CPFP, children", kindCharts(tx.fees.cpfpChild, "CPFP child transactions")),
        section("Inscriptions", [...kindCharts(tx.inscription, "inscription transactions"), ...flow(tx.inscription.fees, "Transaction fees", "Inscription transactions' fees"), chart("Fee share", "Inscription transactions' share of all fees", sharesOf(tx.inscription.fees.chainShare, "Share"))]),
      ]),
      section("Inputs", [
        ...stats(t.inputs.count, "Count", numberOf("inputs")),
        chart("Per second", "Inputs per second", windows(t.inputs.perSecond, "")),
        section("By type", byType(t.inputs.types, "inputs", "spending")),
        section("Signatures", signatureCharts(t.inputs.signatures)),
        section("Public keys", publicKeys(t.inputs.pubkeys, "inputs")),
        section("Opcodes", kinds(entries(t.inputs.opcodes), opcodeName, (key) => `occurrences of ${opcodeName(key)} in revealed scripts`, "", { what: "opcode occurrences in revealed scripts" })),
        section("Nested SegWit", kinds(entries(t.inputs.nestedSegwit), (key) => `P2SH-${typeName(key)}`, (key) => `P2SH-${typeName(key)} inputs`, "type", { what: "nested SegWit inputs", disjoint: true })),
        // Bare multisig (P2MS) is its own type in By type: these wrap it, so lines, not parts of all.
        section("Multisig", kinds(entries(t.inputs.multisig), typeName, (key) => `multisig ${typeName(key)} inputs`, "type", { what: "multisig inputs" })),
        ...flow(t.inputs.p2trScriptPath, "Taproot script path", numberOf("Taproot script-path inputs")),
        ...flow(t.inputs.inscriptions, "Inscriptions", numberOf("inputs revealing an inscription")),
        ...flow(t.inputs.coinbaseWitness, "Coinbase witness", numberOf("coinbase inputs with a witness")),
        section("Spent within", kinds([["sameBlock", t.inputs.sameBlock], ...entries(t.inputs.spentWithin)], (key) => (key === "sameBlock" ? "Same block" : `Within ${spanLabel(key)}`), (key) => (key === "sameBlock" ? "inputs spending outputs from the same block" : `inputs spending outputs at most ${spanLabel(key)} old`), "age limit", { what: "inputs spending recent outputs" })),
      ]),
      section("Outputs", [
        ...stats(t.outputs.count, "Count", numberOf("outputs")),
        chart("Per second", "Outputs per second", windows(t.outputs.perSecond, "")),
        ...flow(t.outputs.nonOpReturnCount, "Non-OP_RETURN", numberOf("outputs created other than OP_RETURN")),
        section("By type", byType(t.outputs.types, "outputs", "creating", ["opReturn"])),
        ...flow(t.outputs.coinbase.count, "Coinbase, count", numberOf("coinbase outputs")),
        section("Coinbase, by type", kinds(entries(t.outputs.coinbase.types), typeName, (key) => `coinbase ${inSentence(typeName(key))} outputs`, "type", { what: "coinbase outputs", disjoint: true })),
        section("Public keys", publicKeys(t.outputs.pubkeys, "outputs")),
      ]),
      section("OP_RETURN", [
        ...opReturnCharts(t.opReturn, (what) => `OP_RETURN ${what}`, t.outputs.types.opReturn.share, t.outputs.opReturnValue),
        section("Protocols", [
          section("Compare", [
            ...parts(["Outputs", "Number of OP_RETURN outputs by protocol"], ["Output share", "Share of OP_RETURN outputs by protocol"], entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.outputCount.sum._1m, protocolName(key), { color: ramp(all.length)[i] })), "_1m"),
            chart("Transaction share", "Share of transactions by OP_RETURN protocol", entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.txShare._1m, protocolName(key), { color: ramp(all.length)[i] })), { window: "_1m" }),
          ]),
          ...entries(t.opReturn.protocols).map(([key, node]) => section(protocolName(key), opReturnCharts(node, (what) => `OP_RETURN ${what}: ${protocolName(key)}`))),
        ]),
        section("Policies", [
          section("Compare", [
            chart("Transaction share", "Share of transactions by OP_RETURN policy", entries(t.opReturn.policies).map(([key, node], i, all) => series(node.txShare._1m, policyName(key), { color: ramp(all.length)[i] })), { window: "_1m" }),
            chart("Data", "OP_RETURN data by policy", entries(t.opReturn.policies).map(([key, node], i, all) => series(node.dataBytes.sum._1m, policyName(key), { color: ramp(all.length)[i] })), { log: [1], window: "_1m" }),
          ]),
          ...entries(t.opReturn.policies).map(([key, node]) => section(policyName(key), opReturnCharts(node, (what) => `OP_RETURN ${what}: ${policyName(key)}`))),
        ]),
        section("Sizes", [
          chart("Compare", "Number of OP_RETURN outputs over each data size", entries(t.opReturn.sizes).map(([key, node], i, all) => series(node.outputCount.sum._1m, spanLabel(key), { color: ramp(all.length)[i] })), { log: [1], window: "_1m" }),
          ...entries(t.opReturn.sizes).map(([key, node]) => section(spanLabel(key), opReturnCharts(node, (what) => `OP_RETURN ${what} with data ${spanLabel(key, true)}`))),
        ]),
      ]),
      // Chain statistics' alone (chain-statistics.js).
      section("Inputs and outputs", []),
    ], { workings: true, icon: "nodes" }),
    section("UTXOs", [
      chart("Count", "UTXO count", [series(t.utxoSet.count, "UTXOs", { color: "sky" })], { log: [1] }),
      section("By amount", family(entries(t.utxos.amounts), bandMember(client.AMOUNT_RANGE_NAMES, UTXO_AMOUNTS), { each: "by UTXO amount", disjoint: true })),
      ...cutoffs(entries(t.utxos.amounts), AMOUNT_EDGES, UTXO_AMOUNTS),
      // Read off the amount bands' Lorenz curve; bands, not each UTXO, so a floor under the true figure. The 0-sat band
            // has UTXOs and no coins.
      ((bands) => ((count, coins) => chart("Gini coefficient", "Gini coefficient of UTXO amounts", [price(), { formula: `1 - (${coins.map((s, i) => `${count[i + 1]} * (${i ? `2 * under(${s}) + ` : ""}${s})`).join(" + ")}) / (over(${count[0]}) * over(${coins[0]}))`, label: "Gini coefficient", format: "number", pane: 2, color: "violet" }]))(bands.map(([, c]) => own(c.utxos.count).name), bands.slice(1).map(([, c]) => c.supply.total.btc.name)))(entries(t.utxos.amounts)),
      section("By type", family(entries(t.utxos.types), (key) => ({ name: client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, who: `${client.SPENDABLE_TYPE_NAMES[key]?.short ?? key} outputs` }), { each: "by output type", disjoint: true })),
    ], { icon: "cube" }),
    section("Addresses", [
      ...addressCharts(t.addresses),
      section("By type", [
        section("Compare", [
          .../** @type {[string, string, (a: any) => any, string?][]} */ ([["Funded", "Funded address count by type", (a) => own(a.funded)], ["Total", "Total address count by type", (a) => a.total], ["New", "Number of new addresses by type", (a) => a.new.sum._1m, "_1m"], ["Active per block", "Average number of active addresses per block by type", (a) => a.activity.active.avg._1m, "_1m"]]).map(([name, title, pick, window]) => chart(name, title, entries(t.addresses.types).map(([key, a], i, all) => series(pick(a), client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, { color: ramp(all.length)[i] })), { log: [1], window })),
        ]),
        ...entries(t.addresses.types).map(([key, a]) => section(client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, addressCharts(a, client.SPENDABLE_TYPE_NAMES[key]?.short ?? key))),
      ]),
      section("By balance", family(entries(t.addresses.balances), bandMember(client.AMOUNT_RANGE_NAMES, BALANCES), { each: "by address balance", disjoint: true, of: ADDRESSED })),
      ...cutoffs(entries(t.addresses.balances), AMOUNT_EDGES, BALANCES, { of: ADDRESSED }),
    ], { icon: "key" }),
  ];

  /** @param {any} node @param {string} name @param {string} title */
  function stats(node, name, title, perBlock = `${title} per block`) {
    return [...flow(node, name, title), ...perBlockFan(node, `${name} per block`, perBlock)];
  }
  /** @param {any} node @param {string} name @param {string} title */
  function perBlockFan(node, name, title, log = true, pick = (/** @type {any} */ n) => n) {
    const keys = ["min", "p10", "p25", "median", "p75", "p90", "max", "avg"].filter((key) => isLeaf(pick(node?.[key]?._1m)));
    return fan(name, `${title}, median and percentiles`, keys.map((key) => [key, series(pick(node[key]._1m), `${statLabel(key)}, ${windowSaid("_1m")}`, {})]), PER_BLOCK_COLOR, { log, window: "_1m" });
  }
  /** @param {any} node @param {string} name @param {string} what @param {string} color @param {boolean} [estimated] */
  function spread(node, name, what, color, estimated = false) {
    const keys = ["min", "p10", "p25", "median", "p75", "p90", "max"].filter((key) => isLeaf(node?.[key]));
    return fan(name, `Median and percentiles of ${what} in each block, ${adjective(windowSaid("_1w"))} average`, keys.map((key) => [key, series(node[key], statLabel(key, estimated), { pane: 2 })]), color, { average: "7d" });
  }
  /**
   * On a log scale (they span orders of magnitude; `log` false, a share's, linear), where tails decades past the middle
   * (the empty blocks' months, a fee's spikes) are out of the axis' fit, drawn past its edges.
   * @param {string} name
   * @param {string} title
   * @param {[string, Layer][]} lines
   * @param {string} color
   * @param {{ log?: boolean, average?: string, window?: string }} [options]
   */
  function fan(name, title, lines, color, { log = true, average = "", window = undefined } = {}) {
    const keys = lines.map(([key]) => key);
    if (!keys.includes("median")) return [];
    const at = (/** @type {string} */ key) => letter(1 + keys.indexOf(key));
    const tail = (/** @type {string} */ key) => (log && ["min", "p10", "p90", "max"].includes(key) ? { fit: false } : {});
    const layers = lines.map(([key, layer]) => ({ ...layer, pane: 2, color, ...tail(key), ...(key === "median" ? { width: 2 } : key === "avg" ? { width: 1 } : { width: 1, opacity: 0.5, hidden: true }) }));
    const band = (/** @type {string} */ low, /** @type {string} */ high, /** @type {string} */ label, /** @type {number} */ opacity) => (keys.includes(low) && keys.includes(high) ? [{ cloud: [at(low), at(high)], label, color, opacity, pane: 2, ...tail(low) }] : []);
    return [chart(name, title, [{ ...price(), hidden: true }, ...layers, ...band("p10", "p90", "10th to 90th", 0.15), ...band("p25", "p75", "25th to 75th", 0.25)], { log: log ? [2] : [], ...(average ? { average: { 2: average } } : {}), size: { 1: 1.4, 2: 3 }, window })];
  }
  function statLabel(/** @type {string} */ key, estimated = false) {
    return key === "avg" ? "Average" : key === "min" ? (estimated ? "Low estimate" : "Lowest") : key === "max" ? (estimated ? "High estimate" : "Highest") : percentileLabel(percentile(key));
  }
  /** @param {string} key */
  function featureName(key) {
    return client.SPENDABLE_TYPE_NAMES[key]?.short ?? FEATURES[/** @type {keyof typeof FEATURES} */ (key)] ?? words(key);
  }
  /** @param {string} key */
  function featureTx(key) {
    const type = client.SPENDABLE_TYPE_NAMES[key]?.short;
    if (type) return `transactions with ${inSentence(type)}${inSentence(type) === type ? "" : " scripts"}`;
    const feature = /** @type {keyof typeof FEATURE_PHRASES} */ (key);
    return `transactions with ${FEATURE_PHRASES[feature] ?? FEATURES[/** @type {keyof typeof FEATURES} */ (key)] ?? keyWords(key)}`;
  }
  /** @param {string} key */
  function protocolName(key) {
    return client.OP_RETURN_KIND_NAMES[key];
  }
  /** @param {string} key */
  function versionName(key) {
    return key === "other" ? "Other" : `Version ${key.slice(1)}`;
  }
  function policyName(/** @type {string} */ key) {
    return key === "withinPreV30RelayLimits" ? "Within pre-v30 relay limits" : key === "beyondPreV30RelayLimits" ? "Beyond pre-v30 relay limits" : words(key);
  }
  /** @param {string} key */
  function typeName(key) {
    return client.SPENDABLE_TYPE_NAMES[key]?.short ?? (key === "opReturn" ? "OP_RETURN" : key.split(/(?=[A-Z])/).map((part) => client.SPENDABLE_TYPE_NAMES[part.toLowerCase()]?.short ?? part).join("-"));
  }
  /** @param {string} key */
  function opcodeName(key) {
    return `OP_${key.toUpperCase()}`;
  }
  /** @param {any} types @param {string} what @param {string} doing @param {string[]} [apart] */
  function byType(types, what, doing, apart = []) {
    const list = entries(types), colors = ramp(list.length), What = capitalized(what);
    const values = list.flatMap(([key, node], i) => (node.value ? [series(node.value.sum._1m.btc, typeName(key), { color: colors[i] })] : []));
    return [
      section("Compare", [
        ...parts([What, `Number of ${what} by type`], [`${What.slice(0, -1)} share`, `Share of ${what} by type`], list.map(([key, node], i) => series(node.count.sum._1m, typeName(key), { color: colors[i] })), "_1m"),
        chart("Transaction share", `Share of transactions ${doing} each ${what.slice(0, -1)} type`, list.map(([key, node], i) => series(node.txShare._1m, typeName(key), { color: colors[i] })), { window: "_1m" }),
        ...(values.length ? parts(["Value", `Value sent to ${what} by type`], ["Value share", `Share of value sent to ${what} by type`], values, "_1m") : []),
      ]),
      ...list.filter(([key]) => !apart.includes(key)).map(([key, node]) => section(typeName(key), [
        ...counted(node.count, "Count", `${inSentence(typeName(key))} ${what}`, node.share),
        ...counted(node.txCount, "Transactions", `transactions ${doing} ${inSentence(typeName(key))} ${what}`, node.txShare),
        ...(node.value ? flow(node.value, "Value", `Value sent to ${inSentence(typeName(key))} ${what}`) : []),
      ])),
    ];
  }
  /** @param {any} signatures */
  function signatureCharts(signatures) {
    const list = entries(signatures), named = (/** @type {string} */ key) => (key === "ecdsa" ? "ECDSA" : words(key));
    const each = (/** @type {(node: any) => any} */ pick) => list.map(([key, node], i) => series(pick(node), named(key), { color: ramp(list.length)[i] }));
    const signed = (/** @type {any} */ node, /** @type {string} */ who) => [...flow(node.count, "Count", numberOf(`${who} signatures`)), ...flow(node.bytes, "Bytes", `${who} signature bytes`)];
    const { ecdsa, schnorr } = signatures;
    return [
      section("Compare", [
        ...parts(["Signatures", "Number of signatures by type"], ["Share", "Share of signatures by type"], each((node) => node.count.sum._1m), "_1m"),
        ...parts(["Bytes", "Signature bytes by type"], ["Byte share", "Share of signature bytes by type"], each((node) => node.bytes.sum._1m), "_1m"),
      ]),
      section("ECDSA", [
        ...signed(ecdsa, "ECDSA"),
        ...flow(ecdsa.strictDer, "Strict DER", numberOf("ECDSA signatures in strict DER")),
        ...flow(ecdsa.lowR, "Low R", numberOf("ECDSA signatures with a low R")),
        ...flow(ecdsa.lowS, "Low S", numberOf("ECDSA signatures with a low S")),
        ...flow(ecdsa.lowRLowS, "Low R and low S", numberOf("ECDSA signatures with a low R and a low S")),
        // Lengths from 70 bytes, those under uncounted: lines, not parts of all.
        section("By length", kinds(entries(ecdsa.length), spanLabel, (key) => `ECDSA signatures ${spanLabel(key, true)}`, "length, sighash byte included", { what: "ECDSA signatures" })),
        section("By sighash type", kinds(entries(ecdsa.sighash), sighashName, (key) => `ECDSA signatures with ${sighashName(key)}`, "sighash type", { what: "ECDSA signatures" })),
      ]),
      section("Schnorr", signed(schnorr, "Schnorr")),
    ];
  }
  /** @param {any} node @param {string} where */
  function publicKeys(node, where) {
    return kinds(entries(node), words, (key) => `${keyWords(key)} public keys in ${where}`, "compression", { what: `public keys in ${where}`, disjoint: true });
  }
  function lastMovedCharts() {
    const bands = entries(t.age.ranges), ages = ramp(bands.length), colorOf = (/** @type {string} */ key) => ages[bands.findIndex(([k]) => k === key)];
    const atLeast = ["_1yTo18m", "_2yTo3y", "_5yTo6y", "_10yTo12y"].flatMap((key) => {
      const low = reported(key, band(key, AGES.family))?.low;
      return low ? [[key, `${capitalized(AGES.family.over)} ${low} ago`]] : [];
    });
    return [
      chart("Last moved", "Supply last moved at least 1, 2, 5 and 10 years ago", atLeast.map(([key, said]) => ({ formula: `over(${t.age.ranges[key].supply.total.btc.name})`, label: said, format: "btc", color: colorOf(key) }))),
      chart("Last moved, share of supply", "Share of the supply last moved at least 1, 2, 5 and 10 years ago", atLeast.map(([key, said]) => ({ formula: `over(${t.age.ranges[key].supply.share.name})`, label: said, format: "percent", color: colorOf(key) }))),
    ];
  }
  function cloudsChart() {
    /**
     * min and max take two, so pairs of pairs.
     * @param {"min" | "max"} f @param {string[]} names @returns {string}
     */
    const across = (f, names) => (names.length === 1 ? names[0] : `${f}(${across(f, names.slice(0, names.length >> 1))}, ${across(f, names.slice(names.length >> 1))})`);
    const groups = /** @type {[keyof typeof HOLDER_COLORS, string[]][]} */ ([["sth", ["under4m", "sth", "under6m"]], ["all", ["all"]], ["lth", ["over4m", "lth", "over6m"]]]).map(([group, keys]) => /** @type {[string, string[], string]} */ ([holderName(group).shown, keys, HOLDER_COLORS[group]]));
    return chart("Cost basis clouds", "Holder groups' cost basis ranges (Bitcoin clouds)", [
      price(),
      ...groups.flatMap(([label, keys, color]) => {
        const names = keys.flatMap((key) => [ct.cohorts[key].awake, cf.cohorts[key].mobile]).flatMap((cohort) => [cohort.costBasis.byCoin.realizedPrice.name, cohort.costBasis.byDollar.capitalizedPrice.name]);
        return [{ formula: across("min", names), label: `${label}, lowest`, format: "usd", color, width: 1 }, { formula: across("max", names), label: `${label}, highest`, format: "usd", color, width: 1 }];
      }),
      ...groups.map(([label, , color], i) => ({ cloud: [letter(1 + 2 * i), letter(2 + 2 * i)], label, color, opacity: 0.3 })),
    ]);
  }
  function compassCharts() {
    /** @param {(keyof typeof HOLDER_COLORS)[]} keys @returns {[string, string, string][]} */
    const groups = (keys) => keys.map((key) => [key, holderName(key).shown, HOLDER_COLORS[key]]);
    const short = groups(["under4m", "sth", "under6m"]), long = groups(["over4m", "lth", "over6m"]);
    const stat = (/** @type {string} */ key, /** @type {string} */ which) => {
      const by = cf.cohorts[key].mobile.costBasis.byDollar;
      return which === "avg" ? by.avg.value : by[which];
    };
    /** @param {string} said */
    const title = (said) => capitalized(`${said} cost basis by dollar of each holder group's mobile coins, with the trends of the three youngest groups`);
    // Over all its coins, not its mobile ones alone.
    const edges = short.flatMap(([key, label]) => [series(t.holders[key].costBasis.min, `${label}, lowest cost basis of all its coins`, { color: "gray", hidden: true }), series(t.holders[key].costBasis.max, `${label}, highest cost basis of all its coins`, { color: "gray", hidden: true })]);
    const trends = (/** @type {number} */ from) => short.map(([, label, color], i) => ({ trend: [letter(from + 2 * i), letter(from + 1 + 2 * i)], ref: "A", label: `${label} trend`, color, dash: 2 }));
    /**
         * Layer letters: the price A, the short-term lines B to D, the long-term E to G, all holders' H, the clouds I and
         * J, the trends' edges K to P.
         */
    const ofOne = (/** @type {string} */ name, /** @type {string} */ which, /** @type {string} */ said) => {
      const line = (/** @type {[string, string, string]} */ [key, label, color]) => series(stat(key, which), label, { color, opacity: 0.5 });
      return chart(name, title(said), [
        price(),
        ...short.map(line),
        ...long.map(line),
        series(stat("all", which), holderName("all").shown, { color: HOLDER_COLORS.all, shape: "curved" }),
        { cloud: ["D", "C", "B"], label: "Short-term cloud", color: "red", colorBelow: "orange", byLayers: true, opacity: 0.5 },
        { cloud: ["F", "E", "G"], label: "Long-term cloud", color: "blue", colorBelow: "cyan", byLayers: true, opacity: 0.5 },
        ...edges,
        ...trends(10),
      ]);
    };
    /**
         * Same letters. A cloud's bands join its neighbor lines in its list, so they're listed as they usually lie, lowest
         * to highest (a band painted over another darkens): by statistic, each one's three groups back and forth. Measured
         * over its history: a third to a quarter of the overlap of listing by group.
         */
    const ofMany = (/** @type {string} */ name, /** @type {string[]} */ which, /** @type {string} */ said) => {
      const STAT = /** @type {Record<string, string>} */ ({ p40: "40th percentile", p60: "60th percentile", median: "median", avg: "average" });
      const lines = (/** @type {[string, string, string][]} */ groups) => groups.flatMap(([key, label, color]) => which.map((w) => series(stat(key, w), `${label}, ${STAT[w]}`, { color, opacity: 0.5 })));
      const RANK = /** @type {Record<string, number>} */ ({ p40: 0, avg: 1, median: 2, p60: 3 }), n = 3 * which.length;
      const at = (/** @type {number} */ from) => [...which.keys()].sort((a, b) => RANK[which[a]] - RANK[which[b]]).flatMap((stat, k) => (k % 2 ? [0, 1, 2] : [2, 1, 0]).map((g) => letter(from + g * which.length + stat)));
      return chart(name, title(said), [
        price(),
        ...lines(short),
        ...lines(long),
        series(stat("all", "median"), holderName("all").shown, { color: HOLDER_COLORS.all, shape: "curved" }),
        { cloud: at(1), label: "Short-term cloud", color: "red", colorBelow: "orange", byLayers: true, opacity: 0.5 },
        { cloud: at(1 + n), label: "Long-term cloud", color: "blue", colorBelow: "cyan", byLayers: true, opacity: 0.5 },
        ...edges,
        ...trends(2 * n + 4),
      ]);
    };
    return [
      ofMany("Percentiles and average", ["p40", "p60", "avg"], "40th–60th percentiles and average"),
      ofMany("Percentiles", ["p40", "p60"], "40th–60th percentiles"),
      ofOne("Medians", "median", "median"),
      ofOne("Averages", "avg", "average"),
      ofMany("Medians and averages", ["median", "avg"], "median and average"),
    ];
  }
  function eventsChart() {
    return chart("Events", "Rarity meter extreme events", [
      price(),
      // Named by key: one the client renamed fails here, not quietly gone.
      ...Object.entries(EXTREMES).map(([key, { event, color, place, low }]) => {
        const node = t.rarityMeter.extremes[key];
        if (!node) throw new Error(`No rarity meter extreme ${key}`);
        return { formula: `${own(node).name} ${low ? "<=" : ">="} ${node[low ? "p0_1" : "p99_9"].name}`, type: "signal", label: event, color, place };
      }),
    ]);
  }
  function timeAboveChart() {
    const lines = /** @type {[any, string, string][]} */ ([[ct.cohorts.all.awake.costBasis.byDollar.capitalizedPrice, "Awake capitalized price", "orange"], [ct.prices.trueMarketMean.value, "True market mean", "sky"]]);
    // The place keeps "above"; shown, it says the edge its closes count.
    const said = "Time at or above";
    return chart(shownAs("Time above", said), `${said} the awake capitalized price and true market mean`, [
      price(),
      ...lines.map(([node, label, color]) => series(node, label, { color })),
      ...lines.map(([node, label, color]) => ({ formula: `100 * sma(${t.price.close.name} >= ${node.name})`, label: `Share of closes at or above the ${label.toLowerCase()}`, format: "percent", color, pane: 2 })),
    ]);
  }
  /** @param {any} node @param {(what: string) => string} titled @param {any} [outputShare] @param {any} [value] */
  function opReturnCharts(node, titled, outputShare, value) {
    return [
      ...(outputShare ? counted(node.outputCount, "Outputs", titled("outputs"), outputShare) : flow(node.outputCount, "Outputs", numberOf(titled("outputs")))),
      ...(value ? flow(value, "Value", "Value sent to OP_RETURN outputs") : []),
      ...(node.txCount ? counted(node.txCount, "Transactions", titled("transactions"), node.txShare) : []),
      ...flow(node.dataBytes, "Data", capitalized(titled("data"))),
      ...(node.dataBytes?.chainShare ? [chart("Data share", capitalized(titled("data's share of block bytes")), sharesOf(node.dataBytes.chainShare, "Share"))] : []),
      // Their transactions' whole vsize, not the data's nor a typical transaction's.
      ...flow(node.txVsize, shownAs("Transaction vsize", "Transactions' total vsize"), capitalized(titled("transactions' total vsize"))),
      ...flow(node.fees, "Transaction fees", capitalized(titled("transactions' fees"))),
      ...(node.fees?.chainShare ? [chart("Fee share", capitalized(titled("transactions' share of all fees")), sharesOf(node.fees.chainShare, "Share"))] : []),
    ];
  }
  /** @param {any} a @param {string} [type] */
  function addressCharts(a, type) {
    const these = type ? `${type} addresses` : "addresses";
    const of = (/** @type {string} */ what) => (type ? `${what}: ${capitalized(these)}` : what);
    const addressKinds = [
      { key: "reused", name: "Reused", adjective: "reused" },
      { key: "respent", name: "Respent", having: "more than one spent output" },
      { key: "exposed", name: "Exposed", having: "exposed keys or scripts" },
    ].filter(({ key }) => a[key]).map((kind) => ({ ...kind, set: kind.adjective ? `${kind.adjective} ${these}` : `${these} with ${kind.having}` }));
    return lift([
      // Over funded addresses: empty ones aren't counted.
      a.avgBalance && chart("Average funded balance", of("Average balance of funded addresses"), amount(a.avgBalance, "Average balance"), { log: [1, 2] }),
      section("Count", [
        chart("Funded, empty and total", of("Address count"), [series(own(a.funded), "Funded", { color: "orange" }), series(a.empty, "Empty", { color: "gray" }), series(a.total, "Total", { color: "sky" })], { log: [1] }),
        ...change(a.funded.delta, "Change", of("Funded address count change")),
        ...flow(a.new, "New", of(numberOf("new addresses"))),
        // Each block's distinct addresses, averaged over a window: not the distinct addresses of the window.
        chart("Active per block", of("Average number of active addresses per block"), entries(a.activity).flatMap(([key, node], i, all) => windows(node.avg, words(key), { extra: { color: ramp(all.length)[i], ...(key === "active" ? {} : { hidden: true }) } }))),
      ]),
      ...addressKinds.map((kind) => {
        const k = a[kind.key], who = capitalized(kind.set);
        return section(shownAs(kind.name, who), [
          chart("Count", numberOf(kind.set), [series(k.count.funded, "Funded", { color: "orange" }), series(k.count.total, "Total", { color: "sky" })], { log: [1] }),
          measure("Supply", who, amount(k.supply, "Supply")),
          measure(shownAs("Supply share", `Share of ${these}' coins`), who, [series(k.supply.share, "Share", { color: "sky" })]),
          ...(k.events ? [
            ...flow(k.events.outputs.count, "Outputs to them", numberOf(`outputs to ${kind.set}`)),
            measure(shownAs("Output share", `Share of outputs to ${these}`), who, sharesOf(k.events.outputs.share, "Share")),
            ...(k.direct ? flow(k.direct.outputCount, "Direct reuse", of(numberOf("outputs paying an address the same transaction spends from"))) : []),
            ...flow(k.events.inputs.count, "Inputs from them", numberOf(`inputs from ${kind.set}`)),
            measure(shownAs("Input share", `Share of inputs from ${these}`), who, sharesOf(k.events.inputs.share, "Share")),
            measure(shownAs("Active per block", `Average number of active ${these} per block`), who, windows(k.events.active.count.avg, "")),
            measure(shownAs("Active share", `Share of active ${these}, counted per block`), who, sharesOf(k.events.active.share, "Share")),
          ] : []),
        ]);
      }),
      ...(a.supply ? cohort({ supply: a.supply, capital: a.capital }, `Coins in ${these}`) : []),
    ].filter(Boolean));
  }
  /**
     * A pool that hasn't found a block in 3 years (by the chain's facts) opens on its whole history: in the default 8
     * years most would show none.
     */
  function poolCharts() {
    const list = members(t.pools).filter(([, node]) => node.blockCount).map(([key, node]) => /** @type {[string, any, string]} */ ([key, node, client.POOL_SLUG_TO_NAME[key.replace(/^_/, "")] ?? key])).sort((a, b) => a[2].localeCompare(b[2]));
    const MAJOR = ["foundryusa", "antpool", "viabtc", "f2pool", "marapool", "spiderpool", "binancepool", "luxor", "secpool", "braiinspool", "ocean", "poolin", "cloverpool", "unknown"];
    const major = MAJOR.flatMap((key) => list.filter(([k]) => k.replace(/^_/, "") === key)), others = list.filter((pool) => !major.includes(pool)), colors = ramp(major.length);
    const groups = entries(t.pools.groups).map(([key, group]) => /** @type {[string, any, string]} */ ([key, group, `${client.POOL_SLUG_TO_NAME[key] ?? key} group`]));
    const grouped = `${groups.map(([, , name]) => name).join(" and ")} as one`;
    /** @param {any} node */
    const tops = (node) => entries(node).map(([key, value], i, all) => series(value, words(key), { color: ramp(all.length)[i] }));
    /** @param {[string, any, string]} pool */
    const pool = ([key, node, name]) => section(name, [
      ...counted(node.blockCount, "Blocks mined", `blocks mined: ${name}`, node.blockShare),
      ...flow(node.coinbase, "Coinbase", `Coinbase rewards: ${name}`),
      chart("Fees per block", `Fees per block: ${name}`, [...windows(node.fees.avg, "Bitcoin", { pick: btc, color: "orange" }), ...windows(node.fees.avg, "Dollars", { pick: usd, pane: 2, color: "green" })], { log: [1, 2] }),
      chart("Hash rate", `Hash rate: ${name}`, windows(node.hashrate, ""), { log: [1] }),
    ].map((item) => (facts && !facts.recent.has(key.replace(/^_/, "")) ? { ...item, c: { ...item.c, range: "max" } } : item)));
    return [
      section("Compare", [
        chart("Share", "Mining pools' share of blocks", [
          ...major.map(([, node, name], i) => series(node.blockShare._1m, name, { color: colors[i], type: "area" })),
          { formula: `100 - (${major.map(([, node]) => node.blockShare._1m.name).join(" + ")})`, label: "Other pools", format: "percent", color: "gray", type: "area" },
        ], { stack: { 1: "stack" }, window: "_1m" }),
        chart("Hash rate", "Mining pools' hash rate", major.map(([, node, name], i) => series(node.hashrate._1m, name, { color: colors[i] })), { log: [1], window: "_1m" }),
      ]),
      section("Top pools", [
        chart("Compare", numberOf("blocks mined by the day's top pools combined"), tops(t.pools.concentration)),
        chart(`Compare, ${grouped}`, numberOf(`blocks mined by the day's top pools combined, ${grouped}`), tops(t.pools.concentrationGrouped)),
        ...entries(t.pools.concentration).map(([key, node]) => chart(words(key), numberOf(`blocks mined by the day's ${keyWords(key)} pools combined`), [series(node, "Pools counted separately", { color: "orange" }), series(t.pools.concentrationGrouped[key], grouped, { color: "sky" })])),
      ]),
      ...groups.map(([, group, name]) => section(name, flow(group.blockCount, "Blocks mined", numberOf(`blocks mined: ${name}`)))),
      ...major.map(pool),
      section("Other pools", others.map(pool)),
    ];
  }
  /** @param {any} cohorts @param {string} on @param {string} weight */
  function weightedSets(cohorts, on, weight) {
    const beside = /** @type {[string, string]} */ ([capitalized(/** @type {string} */ (HOLDERS[on].before)), capitalized(/** @type {string} */ (HOLDERS.all.before))]);
    const naming = (/** @type {string} */ key) => ({ name: holder(key).name, who: capitalized(holderName(key).of(on)), of: groupWhole(key), beside });
    return holderSets(entries(cohorts).map(([key, c]) => [key, c[on]]), (members, each, profile) => family(members, naming, { each, within: holderName(on).who, urpd: (key) => holderName(key).urpd, weight, profile }));
  }
  /** Its charts by age band, to go with the age bands' own. @param {any} ranges @returns {{ compare: ChartNode[], activity: Map<string, ChartNode[]> }} */
  function cointimeBands(ranges) {
    const list = ageBands(ranges), colors = ramp(list.length);
    return {
      compare: [
        chart("Wakefulness", "Wakefulness by age band", list.map(([, node, n], i) => series(node.wakefulness, shownOf(n.name), { color: colors[i] }))),
        ...parts(["Awake supply", "Awake supply by age band"], ["Awake supply share", "Awake supply share by age band"], list.map(([, node, n], i) => series(node.awake.supply.btc, shownOf(n.name), { color: colors[i] }))),
        ...parts(["Dormant supply", "Dormant supply by age band"], ["Dormant supply share", "Dormant supply share by age band"], list.map(([, node, n], i) => series(node.dormant.supply.btc, shownOf(n.name), { color: colors[i] }))),
      ],
      activity: new Map(list.map(([key, node, n]) => [key, [
        ...flow(node.coindaysConsumed, "Coin days consumed", `Coin days consumed that built up while coins were ${n.said}`),
        chart("Wakefulness", `Wakefulness: ${n.who}`, [series(node.wakefulness, "Wakefulness", { color: "orange" })]),
        chart("Awake and dormant", `Awake and dormant supply: ${n.who}`, [series(node.awake.supply.btc, "Awake", { color: "orange", type: "area" }), series(node.dormant.supply.btc, "Dormant", { color: "gray", type: "area" })], { stack: { 1: "stack" } }),
      ]])),
    };
  }
  /** Its charts by age band, to go with the age bands' own. @param {any} ranges @returns {{ compare: ChartNode[], activity: Map<string, ChartNode[]> }} */
  function coinflowBands(ranges) {
    const list = ageBands(ranges), colors = ramp(list.length);
    return {
      compare: [
        chart("Spending rate", "Spending rate by age band", list.map(([, node, n], i) => series(node.spendingRate, shownOf(n.name), { color: colors[i] })), { log: [1] }),
        chart("Spending exposure", "Spending exposure by age band", list.map(([, node, n], i) => series(node.spendingExposure, shownOf(n.name), { color: colors[i] }))),
        chart("Mobility", "Mobility by age band", list.map(([, node, n], i) => series(node.mobility, shownOf(n.name), { color: colors[i] }))),
        ...parts(["Mobile supply", "Mobile supply by age band"], ["Mobile supply share", "Mobile supply share by age band"], list.map(([, node, n], i) => series(node.mobile.supply.btc, shownOf(n.name), { color: colors[i] }))),
        ...parts(["Immobile supply", "Immobile supply by age band"], ["Immobile supply share", "Immobile supply share by age band"], list.map(([, node, n], i) => series(node.immobile.supply.btc, shownOf(n.name), { color: colors[i] }))),
      ],
      activity: new Map(list.map(([key, node, n]) => [key, [
        chart("Spending rate", `Spending rate: ${n.who}`, [series(node.spendingRate, "Spending rate", { color: "orange" })]),
        chart("Spending exposure", `Spending exposure: ${n.who}`, [series(node.spendingExposure, "Spending exposure", { color: "sky" })]),
        chart("Mobility", `Mobility: ${n.who}`, [series(node.mobility, "Mobility", { color: "violet" })]),
        chart("Mobile and immobile", `Mobile and immobile supply: ${n.who}`, [series(node.mobile.supply.btc, "Mobile", { color: "orange", type: "area" }), series(node.immobile.supply.btc, "Immobile", { color: "gray", type: "area" })], { stack: { 1: "stack" } }),
      ]])),
    };
  }

  /**
   * The price over a chart, in a pane above. Shown, it leaves room for one pane of the chart's own: a second (a flow's
   * dollars beside its bitcoin) shows only over the chain's workings, where the price is hidden.
   * @param {Sketch} c @param {boolean} hidden @returns {Sketch}
   */
  const priced = (c, hidden) => {
    const at = c.layers.findIndex((layer) => layer.series === t.price.ohlc.name);
    if (at === 0 && (c.layers[0].pane ?? 1) === 1) return c;
    // Layers are named by their place, A first: a chart that names one would be read wrong with another before it.
    if (at > 0 || c.layers.some((layer) => layer.cloud || /\b[A-Z]{1,2}\b/.test(layer.formula ?? ""))) throw new Error(`A preset names its layers by place without the price first: ${c.layers.map((layer) => layer.label).join(", ")}`);
    const panes = c.panes ?? [{ mode: 0 }], first = panes[0], own = hidden ? c.layers : c.layers.filter((layer) => (layer.pane ?? 1) === 1);
    if (globalThis.TRIMMED && own.length < c.layers.length) globalThis.TRIMMED.push(c.layers.filter((layer) => !own.includes(layer)).map((layer) => layer.label));
    return {
      ...c,
      layers: [{ ...price(), ...(hidden ? { hidden: true } : {}) }, ...own.map((layer) => ({ ...layer, pane: (layer.pane ?? 1) + 1 }))],
      panes: [{ mode: 1, size: 1.4 }, { ...first, size: first.size ?? 3 }, ...(hidden ? panes.slice(1) : [])],
    };
  };

  // One subject a preset, all of it in view: at most two panes, the price's (hidden over the chain's workings) aside, and
  // none of them hidden, so nothing is drawn that a reader can't see.
  const hiddenPanes = /** @type {string[]} */ ([]);
  /** @param {Sketch} c @param {string[]} path */
  const allShown = (c, path) => {
    const shown = new Set(c.layers.filter((layer) => !layer.hidden).map((layer) => layer.pane ?? 1)), panes = Math.max(...c.layers.map((layer) => layer.pane ?? 1));
    const data = Array.from({ length: panes }, (_, i) => i + 1).filter((pane) => !c.layers.some((layer) => (layer.pane ?? 1) === pane && layer.series === t.price.ohlc.name));
    if (data.some((pane) => !shown.has(pane)) || shown.size > 2) hiddenPanes.push(path.join(" › "));
    return c;
  };

  /** @type {Preset[]} */
  const presets = [];
  /** @type {Record<string, string>} */
  const labels = {};
  let icon = "";
  // By index, each place made once: with thousands of presets, an engine without a JIT would step an iterator for each.
  const walk = (/** @type {(ChartNode | SectionNode)[]} */ items, /** @type {string[]} */ path, workings = false) => {
    for (let k = 0; k < items.length; k++) {
      const item = items[k], place = path.concat(placeOf(item.name));
      if (typeof item.name !== "string") labels[JSON.stringify(place)] = item.name.shown;
      if ("items" in item) {
        if (!path.length) icon = item.icon ?? "";
        // (Chain statistics at the end of their sections.)
        const more = derived.at(place.join(" › "));
        walk(more.length ? item.items.concat(more) : item.items, place, workings || Boolean(item.workings));
        continue;
      }
      presets.push({ p: place, t: item.title, c: allShown(priced(item.c, workings || Boolean(item.priceHidden)), place), ...(icon ? { s: icon } : {}), ...(item.was ? { w: Object.fromEntries(Object.entries(item.was).map(([name, key]) => [name, windowShort(key)])) } : {}) });
      icon = "";
    }
  };
  walk(tree, []);
  if (hiddenPanes.length) throw new Error(`Studio presets: ${hiddenPanes.length} with a pane hidden or past two, ${hiddenPanes.slice(0, 40).join("; ")}`);
  if (derived.left().length) throw new Error(`Studio presets: chain statistics placed in no section, ${derived.left().join("; ")}`);
  return { presets, labels };
}

