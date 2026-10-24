// @ts-check
/**
 * Studio's presets: its ready-made charts, in a tree of its own, made from a client's series tree (this repo's, so a
 * path it doesn't have fails where it's written), and the chain's facts (what has happened: no charts of a pool that
 * never found a block, or of what's still to come). Every other series Studio can chart (by date) is meant to be in one
 * of them; the page makes them as it loads.
 *
 * A preset: `p` its place (its sections, then its own name), `t` its title, `c` its chart as a link writes it (its
 * layers, its panes; the page reads it as it reads a link).
 *
 * How they're laid out, everywhere alike:
 * - a family (holder groups, age bands, pools…) opens on its comparisons, every member in one chart, then each member,
 *   with the same charts as the others, in sections by what they're about (supply, realized, cost basis…);
 * - a measure summed over trailing windows (24 hours to 1 year) is one chart, its month shown, the others in its legend;
 * - one in bitcoin has its value in dollars in a pane below it; one summed since the start is a chart of its own;
 * - a price shows over the price's candles;
 * - the price is on top of every one: its candles first, in the first pane, which holds only prices (a chart of anything
 *   else has them in a pane of their own above its own, shown short, or, about the chain's workings, not the market,
 *   hidden there: see priced).
 * @typedef {Record<string, any>} Layer
 * @typedef {{ layers: Layer[], panes?: Record<string, any>[], range?: string, halvings?: boolean }} Sketch
 * @typedef {{ p: string[], t: string, c: Sketch }} Preset
 * @typedef {{ name: string, title: string, c: Sketch, priceHidden?: boolean }} ChartNode
 * @typedef {{ name: string, items: (ChartNode | SectionNode)[], workings?: boolean }} SectionNode `workings`: its charts are
 * about the chain's workings, not the market (the price over them hidden: see priced)
 * @typedef {{ pools: Set<string>, height: number, year: number }} Facts the chain's, from its server: the pools that
 * have found a block, its height, the year
 */

/** @param {any} n */
const isLeaf = (n) => typeof n?.name === "string" && typeof n.indexes === "function";
/** Whether a series can be charted: by date (Studio charts by date; some are by block only). @param {any} n */
const byDate = (n) => isLeaf(n) && n.indexes().includes("day1");
/** A node's children that hold something, by key (the client's own keys, in its order). @param {any} n */
const entries = (n) => (n && typeof n === "object" && !isLeaf(n) ? Object.entries(n).filter(([, v]) => v && typeof v === "object") : []);

// ── Words ──────────────────────────────────────────────────────────────────

const UNIT_WORDS = { h: ["hour", "hours"], d: ["day", "days"], w: ["week", "weeks"], m: ["month", "months"], y: ["year", "years"] };
/** A window's key as said: `_24h` "24 hours", `_1w` "1 week", `_200d` "200 days", `cumulative` "All time". */
function windowLabel(/** @type {string} */ key) {
  if (key === "cumulative") return "All time";
  const match = /^_(\d+)([hdwmy])$/.exec(key);
  if (!match) return key;
  const [, n, unit] = match, words = UNIT_WORDS[/** @type {keyof typeof UNIT_WORDS} */ (unit)];
  return `${n} ${n === "1" ? words[0] : words[1]}`;
}
/** A window as an adjective: `_200d` "200-day", `_1w` "1-week". */
const windowAdjective = (/** @type {string} */ key) => windowLabel(key).replace(/^(\d+) (\w+?)s?$/, "$1-$2");
/** A window's key, short (axis-like): `_24h` "24h", `_200d` "200d", `_1m` "1m". */
const windowShort = (/** @type {string} */ key) => key.replace(/^_/, "");
/** A percentile's key as a number, its decimal point an underscore: `p5` 5, `p0_1` 0.1, `p99_5` 99.5, `median` 50. */
function percentile(/** @type {string} */ key) {
  if (key === "median") return 50;
  const digits = /^p(\d+(?:_\d+)?)$/.exec(key)?.[1];
  return digits ? Number(digits.replace("_", ".")) : NaN;
}
/** A percentile as said, its suffix by its last digit: 5 "5th", 0.1 "0.1st", 50 "Median", 99.5 "99.5th", 12 "12th". */
function percentileLabel(/** @type {number} */ p) {
  if (p === 50) return "Median";
  const text = String(p), last = text.at(-1), teen = /(^|\D)1\d$/.test(text);
  return `${text}${teen ? "th" : last === "1" ? "st" : last === "2" ? "nd" : last === "3" ? "rd" : "th"}`;
}
/** A Bedrock mode's name from its key: "awake" is "Awake". */
const modeName = (/** @type {string} */ mode) => sentence(mode[0].toUpperCase() + mode.slice(1));
/** A Bedrock floor band's label from its key: "p95ToP98" is "95th–98th", "belowP99_9" "Below 99.9th". */
function floorBandLabel(/** @type {string} */ key) {
  const [high, low] = (key.match(/p\d+(?:_\d+)?/gi) ?? []).map((p) => percentileLabel(percentile(p.toLowerCase())));
  return low ? `${high}–${low}` : `Below ${high}`;
}
/** A name as written in the client's tables ("1 to 2 Months Old", "100K-1M Sats") in sentence case, ranges with en dashes. */
function sentence(/** @type {string} */ text) {
  return text
    .split(" ")
    .map((word, i) => (i && /^[A-Z][a-z]+$/.test(word) ? word.toLowerCase() : word))
    .join(" ")
    .replace(/(\d|[KMB])-(\d)/g, "$1–$2");
}

/**
 * A client key's words ("supplyInLoss" "Supply in loss"), in sentence case, a trailing window set off
 * ("sellSideRiskRatio24h" "Sell side risk ratio, 24h").
 */
function words(/** @type {string} */ key) {
  const text = key.replace(/([a-z])([A-Z0-9])/g, "$1 $2").replace(/(\d)([a-z]{3,})/g, "$1 $2").toLowerCase();
  return (text[0].toUpperCase() + text.slice(1)).replace(/ (\d+[hdwmy])$/, ", $1");
}
/** A name as it reads inside a sentence: a plain capitalized word lowercased ("Unknown" "unknown"), a name kept ("P2TR"). */
const inSentence = (/** @type {string} */ name) => (/^[A-Z][a-z]+$/.test(name) ? name.toLowerCase() : name);

/** Transaction features as said (the client's keys spell them as words). */
const FEATURES = {
  fakePubkey: "Fake public keys", fakeScripthash: "Fake script hashes", segwit: "SegWit", annex: "Taproot annex", sighashAll: "SIGHASH_ALL",
  sighashNone: "SIGHASH_NONE", sighashSingle: "SIGHASH_SINGLE", sighashDefault: "SIGHASH_DEFAULT", sighashAnyoneCanPay: "SIGHASH_ANYONECANPAY",
  rbfSignaling: "RBF signaling", dustOutput: "Dust outputs", epobc: "EPOBC", oneInput: "One input", oneOutput: "One output", emptyScript: "Empty scripts", unknownScript: "Unknown scripts",
};
/** …and inside a sentence, where it reads otherwise ("Transactions with a Taproot annex"). */
const FEATURE_PHRASES = { emptyScript: "empty scripts", unknownScript: "unknown scripts", fakePubkey: "fake public keys", fakeScripthash: "fake script hashes", annex: "a Taproot annex", rbfSignaling: "RBF signaling", dustOutput: "dust outputs", epobc: "EPOBC colored-coin tags", oneInput: "one input", oneOutput: "one output" };

// ── Colors ─────────────────────────────────────────────────────────────────

/** Bedrock's landscape colors, top to bottom: the sky over the all-time high, then water, soil and bedrock, as the
 * Bedrock model first drew them. */
const SKY = "#58bbc4";
const LANDSCAPE = ["#3da8b4", "#25879d", "#176b8b", "#105272", "#093a5b", "#c9a54b", "#b68134", "#965a2f", "#714331", "#4b342e", "#83878a", "#666a6e", "#4b4f53", "#303438", "#181b1e"];
/** The windows' colors, short to long: cool to warm around the wheel, the usual four (24 hours to 1 year) in blues. */
const WINDOW_COLORS = ["cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia", "pink", "rose", "red", "orange", "amber", "yellow"];
/** A family's members' colors, in their order: hues around the wheel, evenly, as many as there are. */
function ramp(/** @type {number} */ count, from = 25, to = 320) {
  return Array.from({ length: count }, (_, i) => `oklch(0.72 0.15 ${Math.round(count > 1 ? from + ((to - from) * i) / (count - 1) : from)})`);
}
/**
 * A baseline's two colors, the `i`th of `count` alike (its windows, short to long): green above its base and red below,
 * as every baseline, from light and pale to deep and vivid, the green turning teal and the red crimson, so each reads
 * apart in either theme (the light one turns lightness over and narrows the middle: the hue and chroma still part
 * them) and all read the same way.
 */
function shades(/** @type {number} */ i, /** @type {number} */ count) {
  const k = count > 1 ? i / (count - 1) : 0.5, l = (0.82 - 0.26 * k).toFixed(2), c = (0.14 + 0.04 * k).toFixed(2);
  return { color: `oklch(${l} ${c} ${Math.round(135 + 40 * k)})`, colorBelow: `oklch(${l} ${c} ${Math.round(40 - 20 * k)})` };
}

/**
 * The chain's facts the presets go by, from its server (`origin`): the pools that have found a block, its height, the
 * year (see notYet).
 * @param {string} origin @returns {Promise<Facts>}
 */
export async function chainFacts(origin) {
  // (In a time: a stalled server holds up nothing; its facts are then missing, and the presets made without them.)
  const read = (/** @type {string} */ path) => fetch(`${origin}${path}`, { signal: AbortSignal.timeout(15_000) }).then((response) => (response.ok ? response : Promise.reject(new Error(`${path}: ${response.status}`))));
  const [pools, height] = await Promise.all([read("/api/v1/mining/pools/all").then((r) => r.json()), read("/api/blocks/tip/height").then((r) => r.text()).then(Number)]);
  if (!Number.isFinite(height)) throw new Error("The chain's height isn't a number");
  return { pools: new Set(pools.pools.filter((/** @type {any} */ pool) => pool.blockCount > 0).map((/** @type {any} */ pool) => pool.slug)), height, year: new Date().getUTCFullYear() };
}

/**
 * What never happened or hasn't yet, by the chain's facts: pools that never found a block, halving epochs not begun,
 * vintages to come. Their series are zeros or nothing: they get no presets, and the build doesn't miss them. Without
 * facts, nothing.
 * @param {any} t a client's series tree @param {Facts | undefined} facts @returns {Set<any>} their nodes
 */
function notYet(t, facts) {
  const nodes = new Set();
  if (!facts) return nodes;
  for (const [key, node] of entries(t.pools)) if (node.blockCount && !facts.pools.has(key.replace(/^_/, ""))) nodes.add(node);
  for (const [key, node] of entries(t.age.halvingEpochs)) if (Number(key.slice(1)) * 210_000 > facts.height) nodes.add(node);
  for (const [key, node] of entries(t.age.vintages)) if (Number(key.slice(1)) > facts.year) nodes.add(node);
  return nodes;
}

/**
 * Every preset, made from a client (its series tree and its tables of names) and the chain's facts (see notYet).
 * @param {any} client @param {Facts} [facts]
 * @returns {Preset[]}
 */
export function studioPresets(client, facts) {
  const t = client.series, absent = notYet(t, facts);
  /** A node's members that have been, by key. @param {any} n */
  const members = (n) => entries(n).filter(([, node]) => !absent.has(node));

  // ── Layers and charts ─────────────────────────────────────────────────────

  /** A series as a layer. @param {any} node @param {string} label @param {Layer} [extra] @returns {Layer} */
  const series = (node, label, extra = {}) => {
    if (!isLeaf(node)) throw new Error(`Not a series: ${label}`);
    // (A line unless said: a layer without a type would take the look other presets give its series.)
    return { series: node.name, label, type: "line", ...extra };
  };
  /** A chart's letter for its `i`th layer (fewer than 26). */
  const letter = (/** @type {number} */ i) => String.fromCharCode(65 + i);
  /** The price, as its candles. @returns {Layer} */
  const price = () => ({ series: t.price.ohlc.name, label: "Price", type: "candles", color: "ink", priceLine: true });
  /**
   * A chart: its layers, each with its pane (1 if none), and its panes' scales (a pane with the price, or `log` said,
   * logarithmic; else linear) and stacks.
   * @param {string} name @param {string} title @param {Layer[]} layers
   * `priceHidden`: the price on top starts hidden (a chart whose subject already moves with it, as market cap).
   * @param {{ log?: number[], linear?: number[], stack?: Record<number, string>, sizes?: number[], range?: string, halvings?: boolean, priceHidden?: boolean }} [options]
   * @returns {ChartNode}
   */
  const chart = (name, title, layers, options = {}) => {
    const count = Math.max(1, ...layers.map((layer) => layer.pane ?? 1));
    const panes = Array.from({ length: count }, (_, i) => {
      const n = i + 1, withPrice = layers.some((layer) => (layer.pane ?? 1) === n && layer.type === "candles");
      const log = options.log?.includes(n) || (withPrice && !options.linear?.includes(n));
      return { mode: log ? 1 : 0, ...(options.stack?.[n] ? { stack: options.stack[n] } : {}), ...(options.sizes?.[i] ? { size: options.sizes[i] } : {}) };
    });
    return { name, title, c: { layers, panes, ...(options.range ? { range: options.range } : {}), ...(options.halvings ? { halvings: true } : {}) }, ...(options.priceHidden ? { priceHidden: true } : {}) };
  };
  /**
   * @param {string} name @param {(ChartNode | SectionNode | null | undefined | false)[]} items
   * @param {{ workings?: boolean }} [options] @returns {SectionNode}
   */
  const section = (name, items, options = {}) => ({ name, items: /** @type {(ChartNode | SectionNode)[]} */ (items.filter(Boolean)), ...(options.workings ? { workings: true } : {}) });
  /**
   * A family's member: its folders, or, with one alone (0-sat outputs: their UTXOs only), that one's charts, not a
   * folder in a folder. @param {string} name @param {(ChartNode | SectionNode | null | undefined | false)[]} items @returns {SectionNode}
   */
  const member = (name, items) => ((kept) => section(name, kept.length === 1 && "items" in kept[0] ? kept[0].items : kept))(/** @type {(ChartNode | SectionNode)[]} */ (items.filter(Boolean)));

  /** A node's windows (`_24h`… and any other spans), in order. @param {any} node */
  const windowsOf = (node) => entries(node).filter(([key, value]) => /^_\d+[hdwmy]$/.test(key) && (isLeaf(value) || value.btc));
  /**
   * A measure over windows as layers: each window's series (`pick` takes it from the window's node), the month shown
   * (or those in `show`, none if it's empty), the others hidden; each labeled `what` and its window. Read `around` a
   * base (a ratio's 1, a change's 0): each a baseline there, in its shades.
   * @param {any} node @param {string} what @param {{ pick?: (n: any) => any, show?: string[], pane?: number, colors?: string[], extra?: Layer, around?: number }} [options]
   * @returns {Layer[]}
   */
  const windows = (node, what, options = {}) => {
    const list = windowsOf(node), pick = options.pick ?? ((n) => n);
    // (Said, as said, none at all too; else the month, or the longest where there's no month.)
    const shown = options.show ?? (list.some(([key]) => key === "_1m") ? ["_1m"] : [list.at(-1)?.[0]]);
    return list.map(([key, value], i) => series(pick(value), what ? `${what}, ${windowShort(key)}` : windowLabel(key), {
      color: (options.colors ?? WINDOW_COLORS)[i % (options.colors ?? WINDOW_COLORS).length],
      ...(shown.includes(key) ? {} : { hidden: true }),
      ...(options.pane ? { pane: options.pane } : {}),
      ...options.extra,
      ...(options.around === undefined ? {} : { ...around(options.around), ...shades(i, list.length) }),
    }));
  };
  /** A share or ratio over windows and since the start (`cumulative`), all its own windows. @param {any} node @param {string} what @param {Layer} [extra] */
  const sharesOf = (node, what, extra = {}) => [
    ...(byDate(node.cumulative) ? [series(node.cumulative, `${what}, all time`, { color: "gray", ...extra })] : []),
    ...windows(node, what, { extra }),
  ];
  /** A bitcoin amount's node, or the node itself if it's one series. @param {any} n */
  const btc = (n) => (isLeaf(n) ? n : n?.btc);
  /** Its dollars', if it has them. @param {any} n */
  const usd = (n) => (isLeaf(n) ? undefined : n?.usd);
  /** An amount as layers: its bitcoin, and its dollars' value below when it has them. @param {any} node @param {string} label */
  const amount = (node, label) => [series(btc(node), label, { color: "orange" }), ...(usd(node) ? [series(usd(node), "Value", { color: "green", pane: 2 })] : [])];
  /** A series' node, or its `value` (a level with its own series under `value`, beside others). @param {any} n */
  const own = (n) => (isLeaf(n) ? n : n?.value);
  /** A baseline at `base`: green above, red below (a ratio's base 1, a change's 0). @param {number} [base] @returns {Layer} */
  const around = (base = 0) => ({ type: "baseline", color: "green", colorBelow: "red", ...(base ? { base } : {}) });
  const baseline = around(0);

  /**
   * A flow (an amount per block: summed over windows, `sum`, and since the start, `cumulative`) as charts: its sums,
   * the bitcoin's dollars below; since the start, a chart of its own (as a count's is).
   * Since the start, its title says so after the measure, before whose it is ("Rewards, all time: Foundry USA").
   * @param {any} node @param {string} name @param {string} title
   * @returns {ChartNode[]}
   */
  const flow = (node, name, title) => {
    if (!node) return [];
    const titled = (/** @type {string} */ after) => title.replace(/^([^:]*)(:|$)/, `$1${after}$2`);
    const sums = node.sum ?? node;
    const pair = windowsOf(sums).some(([, value]) => value.btc);
    if (!pair) {
      const layers = windows(sums, "");
      return [...(layers.length ? [chart(name, titled(""), layers)] : []), ...(byDate(node.cumulative) ? [chart(`${name}, all time`, titled(", all time"), [series(node.cumulative, "All time", { color: "gray" })])] : [])];
    }
    const charts = [chart(name, titled(""), [...windows(sums, "Bitcoin", { pick: btc }), ...windows(sums, "Dollars", { pick: usd, pane: 2 })])];
    if (node.cumulative && byDate(btc(node.cumulative))) charts.push(chart(`${name}, all time`, titled(", all time"), [series(btc(node.cumulative), "Bitcoin", { color: "orange" }), ...(byDate(usd(node.cumulative)) ? [series(usd(node.cumulative), "Dollars", { color: "green", pane: 2 })] : [])]));
    return charts;
  };
  /**
   * A count (a kind of transaction's, a type of output's…): its windows, its share of the whole below (the `share`
   * given, else its own); since the start, a chart of its own (titled as a flow's).
   * @param {any} node @param {string} name @param {string} title @param {any} [share]
   * @returns {ChartNode[]}
   */
  const counted = (node, name, title, share = node.share) => [
    chart(name, title, [...windows(node.sum, ""), ...sharesOf(share, "Share", { pane: 2 })]),
    chart(`${name}, all time`, title.replace(/^([^:]*)(:|$)/, "$1, all time$2"), [series(node.cumulative, "All time", { color: "gray" })]),
  ];
  /**
   * A whole's parts compared: their amounts stacked, then their shares of it, as areas (`[name, title]` each chart's).
   * @param {[string, string]} amounts @param {[string, string]} shares @param {Layer[]} layers @returns {ChartNode[]}
   */
  const parts = ([name, title], [shareName, shareTitle], layers) => {
    const areas = layers.map((layer) => ({ ...layer, type: "area" }));
    return [chart(name, title, areas, { stack: { 1: "stack" } }), chart(shareName, shareTitle, areas, { stack: { 1: "percent" } })];
  };
  /**
   * Kinds of transactions, which overlap: their shares of all transactions compared, then each one's count.
   * @param {[string, any][]} list @param {(key: string) => string} name @param {(key: string) => string} title @param {string} compared
   */
  const txKinds = (list, name, title, compared) => [
    chart("Compare", `${compared}, 1 month`, list.map(([key, node], i) => series(node.share._1m, name(key), { color: ramp(list.length)[i] }))),
    ...list.flatMap(([key, node]) => counted(node, name(key), title(key))),
  ];
  /**
   * A change over windows (`absolute` and `rate`): the amount (its windows) and, below, the rate.
   * @param {any} delta @param {string} name @param {string} title
   */
  const change = (delta, name, title) =>
    delta?.absolute
      ? [chart(name, title, [...windows(delta.absolute, "", { extra: { type: "histogram", color: "green", colorBelow: "red" } }), ...windows(delta.percent, "Percent", { pane: 2, around: 0 })])]
      : [];

  // ── A cohort's charts (the holders, an age band, an amount, a type…) ────────

  /**
   * A cohort's charts in their sections, for whatever of them it has (every family's members have their own share of
   * the same parts). `who` its name in titles ("Short-term holders").
   * `whole`: the cohort is all of what its shares divide (a family's `all`), its shares a flat 100%, left out. `of`:
   * whose coins and capital its shares divide ("all", else a group's: a short-term holders' profit band's,
   * "short-term holders'").
   * @param {any} c @param {string} who @returns {(ChartNode | SectionNode)[]}
   */
  const cohort = (c, who, whole = false, of = "all") => {
    const titled = (/** @type {string} */ what) => `${what}: ${who}`;
    const supply = c.supply, capital = c.capital, utxos = c.utxos, activity = c.activity, realized = c.realized, unrealized = c.unrealized, cost = c.costBasis;
    const inProfit = supply?.inProfit, inLoss = supply?.inLoss;
    return [
      supply && section("Supply", [
        chart("Total", titled("Supply"), amount(supply.total, "Supply")),
        inProfit && chart("In profit and loss", titled("Supply in profit and loss"), [
          series(btc(inProfit), "In profit", { color: "green", type: "area" }), series(btc(inLoss), "In loss", { color: "red", type: "area" }),
          ...(usd(inProfit) ? [series(usd(inProfit), "In profit, value", { color: "green", type: "area", pane: 2 }), series(usd(inLoss), "In loss, value", { color: "red", type: "area", pane: 2 })] : []),
        ], { stack: { 1: "stack", 2: "stack" } }),
        inProfit?.share && chart("Share in profit", titled("Share of supply in profit"), [series(inProfit.share, "In profit", { color: "green", levels: [{ value: 50, color: "gray", dash: 3 }] }), series(inLoss.share, "In loss", { color: "red" })]),
        supply.share && !whole && chart(`Share of ${of} coins`, titled(`Share of ${of} coins and capital`), [series(supply.share, `Of ${of} coins`, { color: "orange" }), ...(capital?.share ? [series(capital.share, `Of ${of} capital`, { color: "green" })] : [])]),
        ...change(supply.delta, "Change", titled("Supply change")),
        ...flow(supply.agedOut, "Aged out", titled("Supply aging out")),
      ]),
      c.funded && section("Addresses", [
        chart("Count", titled("Address count"), [series(own(c.funded), "Addresses", { color: "sky" })], { log: [1] }),
        ...change(c.funded.delta, "Change", titled("Address count change")),
      ]),
      capital && section("Capital", [
        chart("Total", titled("Realized cap"), [
          series(capital.realizedCap, "Realized cap", { color: "orange" }),
          ...(capital.inProfit ? [series(own(capital.inProfit), "In profit", { color: "green", hidden: true }), series(own(capital.inLoss), "In loss", { color: "red", hidden: true })] : []),
          ...(capital.inProfit?.share ? [series(capital.inProfit.share, "Share in profit", { color: "green", pane: 2 }), series(capital.inLoss.share, "Share in loss", { color: "red", pane: 2 })] : []),
        ], { log: [1] }),
        ...change(capital.delta, "Change", titled("Capital change")),
      ]),
      utxos && section("UTXOs", [
        // (All coins' count is UTXOs › Count itself: its one home.)
        utxos.count && !whole && chart("Count", titled("UTXO count"), [series(own(utxos.count), "UTXOs", { color: "sky" })], { log: [1] }),
        ...change(utxos.count?.delta, "Change", titled("UTXO count change")),
        ...flow(utxos.spentCount, "Spent", titled("UTXOs spent")),
      ]),
      activity && section("Activity", [
        ...flow(activity.transferVolume, "Transfer volume", titled("Transfer volume")),
        ...(activity.transferVolume?.inProfit ? [
          chart("Volume in profit and loss", titled("Transfer volume in profit and loss"), [
            ...windows(activity.transferVolume.inProfit.sum, "In profit", { pick: btc, extra: { color: "green" } }), ...windows(activity.transferVolume.inLoss.sum, "In loss", { pick: btc, extra: { color: "red" } }),
            ...windows(activity.transferVolume.inProfit.sum, "In profit, value", { pick: usd, pane: 2, extra: { color: "green" } }), ...windows(activity.transferVolume.inLoss.sum, "In loss, value", { pick: usd, pane: 2, extra: { color: "red" } }),
          ]),
          chart("Volume in profit and loss, all time", titled("Transfer volume in profit and loss, all time"), [
            series(btc(activity.transferVolume.inProfit.cumulative), "In profit", { color: "green" }), series(btc(activity.transferVolume.inLoss.cumulative), "In loss", { color: "red" }),
            series(usd(activity.transferVolume.inProfit.cumulative), "In profit, value", { color: "green", pane: 2 }), series(usd(activity.transferVolume.inLoss.cumulative), "In loss, value", { color: "red", pane: 2 }),
          ]),
        ] : []),
        ...flow(activity.coindaysDestroyed, "Coin days destroyed", titled("Coin days destroyed")),
        ...flow(activity.coindaysCreated, "Coin days created", titled("Coin days created")),
        activity.dormancy && chart("Dormancy", titled("Dormancy"), windows(activity.dormancy, "")),
      ]),
      realized && section("Realized", [
        realized.profit && chart("Profit and loss", titled("Realized profit and loss"), [
          ...windows(realized.profit.sum, "Profit", { extra: { color: "green" } }), ...windows(realized.loss.sum, "Loss", { extra: { color: "red" } }),
          ...(realized.netPnl ? windows(realized.netPnl.sum, "Net", { pane: 2, around: 0 }) : []),
        ]),
        realized.profit && chart("Profit and loss, all time", titled("Realized profit and loss, all time"), [
          series(realized.profit.cumulative, "Profit", { color: "green" }), series(realized.loss.cumulative, "Loss", { color: "red" }),
          ...(realized.netPnl ? [series(realized.netPnl.cumulative, "Net", { pane: 2, ...baseline })] : []),
        ]),
        ...flow(realized.grossPnl, "Gross profit and loss", titled("Gross realized profit and loss")),
        realized.profitToLoss && chart("Profit to loss ratio", titled("Realized profit to loss ratio"), windows(realized.profitToLoss, "", { around: 1 }), { log: [1] }),
        ...flow(realized.valueDestroyed, "Value destroyed", titled("Value destroyed")),
        ...flow(realized.peakRegret, "Peak regret", titled("Realized peak regret")),
        realized.sopr && chart("SOPR", titled("Spent output profit ratio (SOPR)"), windows(realized.sopr, "", { show: ["_1w"], around: 1 })),
        realized.adjustedSopr && chart("Adjusted SOPR", titled("Adjusted SOPR (aSOPR)"), [
          ...windows(realized.adjustedSopr, "", { show: ["_1w"], around: 1 }),
          ...windows(realized.adjustedSopr.valueCreated.sum, "Value created", { pane: 2, extra: { color: "green" } }),
          ...windows(realized.adjustedSopr.valueDestroyed.sum, "Value destroyed", { pane: 2, extra: { color: "red" } }),
        ]),
        realized.adjustedSopr && chart("Adjusted value, all time", titled("Adjusted value created and destroyed, all time"), [
          series(realized.adjustedSopr.valueCreated.cumulative, "Created", { color: "green" }), series(realized.adjustedSopr.valueDestroyed.cumulative, "Destroyed", { color: "red" }),
        ]),
        realized.sellSideRiskRatio && chart("Sell-side risk ratio", titled("Sell-side risk ratio"), windows(realized.sellSideRiskRatio, ""), { log: [1] }),
      ]),
      unrealized && section("Unrealized", [
        chart("Profit and loss", titled("Unrealized profit and loss"), [
          series(unrealized.profit, "Profit", { color: "green" }), series(unrealized.loss, "Loss", { color: "red" }),
          ...(unrealized.grossPnl ? [series(unrealized.grossPnl, "Gross", { color: "gray", hidden: true })] : []),
          series(unrealized.netPnl, "Net", { pane: 2, ...baseline }),
        ]),
        unrealized.nupl && chart("NUPL", titled("Net unrealized profit and loss (NUPL)"), [series(unrealized.nupl, "NUPL", { ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
      ]),
      cost && section("Cost basis", [
        chart("Realized price", titled("Realized price"), [
          price(), series(cost.byCoin.realizedPrice, "Realized price", { color: "orange" }),
          ...(cost.byCoin.medianRealizedPrice ? [series(cost.byCoin.medianRealizedPrice, "Median realized price", { color: "yellow", hidden: true })] : []),
          ...(cost.min ? [series(cost.min, "Lowest", { color: "red", dash: 3, hidden: true }), series(cost.max, "Highest", { color: "green", dash: 3, hidden: true })] : []),
          ...(cost.byCoin.avg?.mvrv ? [series(cost.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1) })] : []),
        ], { log: [2] }),
        cost.byDollar && chart("Capitalized price", titled("Capitalized price"), [
          price(), series(cost.byDollar.capitalizedPrice, "Capitalized price", { color: "orange" }),
          ...(cost.byDollar.medianCapitalizedPrice ? [series(cost.byDollar.medianCapitalizedPrice, "Median capitalized price", { color: "yellow", hidden: true })] : []),
          series(cost.byDollar.avg.ratio, "Price ratio", { pane: 2, ...around(1) }),
        ], { log: [2] }),
        ...percentiles(cost.byCoin, "Percentiles by coin", titled("Cost basis percentiles, by coin")),
        ...(cost.byDollar ? percentiles(cost.byDollar, "Percentiles by dollar", titled("Cost basis percentiles, by dollar")) : []),
        cost.supplyDensity && chart("Supply density", titled("Share of supply with a cost basis within 5% of the price"), [
          series(cost.supplyDensity.total, "Supply", { color: "orange" }), series(cost.supplyDensity.inProfit, "In profit", { color: "green", hidden: true }), series(cost.supplyDensity.inLoss, "In loss", { color: "red", hidden: true }),
        ]),
        cost.capitalDensity && chart("Capital density", titled("Share of capital with a cost basis within 5% of the price"), [
          series(cost.capitalDensity.total, "Capital", { color: "sky" }), series(cost.capitalDensity.inProfit, "In profit", { color: "green", hidden: true }), series(cost.capitalDensity.inLoss, "In loss", { color: "red", hidden: true }),
        ]),
      ]),
    ].filter(Boolean).map((item) => lift(/** @type {SectionNode} */ (item)));
  };
  /**
   * Percentiles of a price (`p5`… keys) over the price, as a fan: each its own line, the median solid, colored from
   * low to high; with `more` layers after them. None, no chart.
   * @param {any} node @param {string} name @param {string} title @param {Layer[]} [more] @param {{ log?: number[] }} [options]
   */
  const percentiles = (node, name, title, more = [], options = {}) => {
    const list = entries(node).filter(([key, value]) => isLeaf(value) && Number.isFinite(percentile(key))).sort(([a], [b]) => percentile(a) - percentile(b));
    if (!list.length) return [];
    const colors = ramp(list.length, 25, 160);
    // (The median by its well-known name where it has one: a cohort's median realized or capitalized price.)
    const median = [node.medianRealizedPrice, node.medianCapitalizedPrice].find(isLeaf);
    return [chart(name, title, [price(), ...list.map(([key, value], i) => series(key === "median" && median ? median : value, percentileLabel(percentile(key)), { color: colors[i], width: key === "median" ? 2 : 1 })), ...more], options)];
  };
  /**
   * A rarity meter component's charts: its price bands over the price, with its reference price, and its ratio
   * percentiles, with the price's ratio to it (a median's its own; the others' where the cohort or Cointime has them:
   * `reference`). `who` its cohort in titles.
   * @param {any} node @param {string} label @param {string} who @param {[any, any] | undefined} reference
   */
  const component = (node, label, who, reference) => {
    const titled = (/** @type {string} */ what) => `${label} ${what}: ${who}`, [base, ratio] = own(node) ? [own(node), node.ratio] : reference ?? [];
    return section(label, [
      chart("Bands", titled("rarity bands"), [price(), ...(base ? [series(base, label, { color: "orange" })] : []), ...bandLines(node, (v) => v.band)]),
      chart("Ratios", titled("rarity ratios"), [...(ratio ? [series(ratio, "Price ratio", around(1))] : []), ...bandLines(node, (v) => v.ratio)], { log: [1] }),
    ]);
  };
  /**
   * A rarity component's reference price and the price's ratio to it, by the component's cohort (a holder group's, or
   * the awake or mobile coins') and key.
   * @param {string} cohort @param {string} key @returns {[any, any] | undefined}
   */
  const referenceOf = (cohort, key) => {
    const cost = (t.holders[cohort] ?? { awake: ct.cohorts.all.awake, mobile: cf.cohorts.all.mobile }[cohort])?.costBasis;
    return /** @type {Record<string, [any, any]>} */ ({
      realizedPrice: [cost?.byCoin.realizedPrice, cost?.byCoin.avg.mvrv],
      capitalizedPrice: [cost?.byDollar.capitalizedPrice, cost?.byDollar.avg.ratio],
      vaultedPrice: [ct.prices.vaulted.value, ct.prices.vaulted.ratio],
      trueMarketMean: [ct.prices.trueMarketMean.value, ct.prices.trueMarketMean.ratio],
    })[key];
  };
  /**
   * A node's rarity bands (its percentile children) as lines, rarest low to rarest high; `pick` takes a band's series,
   * `shown` which are shown (all if not said).
   * @param {any} node @param {(v: any) => any} pick @param {(p: number) => boolean} [shown]
   */
  const bandLines = (node, pick, shown = () => true) => {
    const bands = entries(node).filter(([k]) => Number.isFinite(percentile(k)));
    return bands.map(([k, v], i) => series(pick(v), percentileLabel(percentile(k)), { color: ramp(bands.length, 140, 360)[i], ...(shown(percentile(k)) ? {} : { hidden: true }) }));
  };
  /**
   * The rarity meter's extremes, by key: each one's name, and the event its 0.1% tail marks on the price (its color and
   * side); `low`: its rare side is low (the sell-side risk ratio's: sellers exhausted).
   * @type {Record<string, { label: string, event: string, color: string, place: string, low?: boolean }>}
   */
  const EXTREMES = {
    realizedProfitSum24h: { label: "Realized profit, 24h", event: "Profit taking", color: "green", place: "above" },
    realizedLossSum24h: { label: "Realized loss, 24h", event: "Capitulation", color: "red", place: "below" },
    realizedPeakRegretSum24h: { label: "Realized peak regret, 24h", event: "Peak regret", color: "pink", place: "below" },
    sellSideRiskRatio24h: { label: "Sell-side risk ratio, 24h", event: "Seller exhaustion", color: "yellow", place: "above", low: true },
    supplyInLoss: { label: "Supply in loss", event: "Supply in loss", color: "orange", place: "below" },
  };
  /**
   * A section of one chart, that chart in its place (no folder holding one thing), by its own name ("Realized price");
   * a name that's its section's part says its section's ("Total" in Capital, "Capital"; "Profit and loss" in
   * Unrealized, "Unrealized profit and loss"). @param {SectionNode | ChartNode} item
   */
  const lift = (item) => {
    if (!("items" in item) || item.items.length !== 1 || "items" in item.items[0]) return item;
    const only = item.items[0], name = only.name === "Total" ? item.name : only.name === "Profit and loss" ? `${item.name} profit and loss` : only.name;
    return { ...only, name };
  };

  // ── Families: comparisons first, then each member ───────────────────────────

  /**
   * A family of cohorts: its comparisons (each a chart of every member), then each member's charts.
   * @param {[string, any][]} members its members' keys and nodes
   * @param {(key: string) => { name: string, who: string }} naming each member's name (in the tree) and title (in charts)
   * @param {{ by: string, disjoint?: boolean, of?: string, known?: Record<string, string>, charts?: (key: string, c: any, who: string) => (ChartNode | SectionNode)[] }} options
   * `by`: what a comparison's title says it's by ("age band", "holder group, short-term"); `disjoint`: the members split
   * the whole (bands), so their amounts stack, and as shares; `of`: what the members' shares divide, if not all coins
   * (see cohort); `known`: the established name of a measure's shares, by the measure ("Supply": "HODL waves"), in their title;
   * `charts`: each member's, if not a cohort's
   * @returns {(ChartNode | SectionNode)[]}
   */
  const family = (members, naming, options) => {
    const colors = ramp(members.length);
    /** Every member's series (`pick` takes it from the member), as a chart's layers. @param {(c: any) => any} pick @param {Layer} [extra] */
    // (A family's `all`, the others' reference: gray and dashed.)
    const each = (pick, extra = {}) => members.flatMap(([key, c], i) => (byDate(pick(c)) ? [series(pick(c), naming(key).name, { color: colors[i], ...extra, ...(key === "all" ? { color: "gray", dash: 3 } : {}) })] : []));
    /**
     * A comparison: every member's `pick`, as lines; an amount (`additive`) of members that split the whole, stacked,
     * then as their shares of it ("Supply", "Supply share"). Its title says by what, its window last ("…by age band, 1
     * month"). @param {string} name @param {(c: any) => any} pick @param {{ price?: boolean, log?: boolean, additive?: boolean, level?: number }} [o]
     * @returns {ChartNode[]}
     */
    const compare = (name, pick, o = {}) => {
      // (Read against a reference, its level across: a ratio's 1, a net's 0; the members stay lines, each its color.)
      const layers = each(pick, o.level === undefined ? {} : { levels: [{ value: o.level, color: "gray", dash: 3 }] });
      if (!layers.length) return [];
      const [, measure, window = ""] = /** @type {RegExpExecArray} */ (/^(.*?)(, 1 \w+)?$/.exec(name)), title = `${measure} by ${options.by}${window}`;
      if (options.disjoint && o.additive) {
        const known = options.known?.[measure];
        return parts([name, title], [`${measure} share${window}`, `${measure} share by ${options.by}${window}${known ? ` (${known})` : ""}`], layers);
      }
      if (o.price) return [chart(name, title, [price(), ...layers])];
      return [chart(name, title, layers, o.log ? { log: [1] } : {})];
    };
    const comparisons = [
      ...compare("Supply", (c) => btc(c.supply?.total), { log: true, additive: true }),
      ...compare("Realized cap", (c) => c.capital?.realizedCap, { log: true, additive: true }),
      // (Overlapping members' shares as lines, all coins' 100% their reference; disjoint ones stack above.)
      ...(options.disjoint ? [] : [...compare("Share of all coins", (c) => c.supply?.share), ...compare("Share of all capital", (c) => c.capital?.share)]),
      ...compare("Realized price", (c) => c.costBasis?.byCoin?.realizedPrice, { price: true }),
      ...compare("MVRV", (c) => c.costBasis?.byCoin?.avg?.mvrv, { level: 1, log: true }),
      ...compare("Share in profit", (c) => c.supply?.inProfit?.share),
      ...compare("NUPL", (c) => c.unrealized?.nupl, { level: 0 }),
      ...compare("UTXO count", (c) => own(c.utxos?.count), { log: true, additive: true }),
      ...compare("Address count", (c) => own(c.funded), { log: true, additive: true }),
      ...compare("Transfer volume, 1 month", (c) => btc(c.activity?.transferVolume?.sum?._1m), { additive: true }),
      ...compare("Coin days destroyed, 1 month", (c) => c.activity?.coindaysDestroyed?.sum?._1m, { additive: true }),
      ...compare("Realized profit, 1 month", (c) => c.realized?.profit?.sum?._1m, { additive: true }),
      ...compare("Realized loss, 1 month", (c) => c.realized?.loss?.sum?._1m, { additive: true }),
      ...compare("Net realized profit and loss, 1 month", (c) => c.realized?.netPnl?.sum?._1m, { level: 0 }),
      ...compare("SOPR, 1 week", (c) => c.realized?.sopr?._1w, { level: 1 }),
    ];
    return [
      ...(comparisons.length ? [section("Compare", comparisons)] : []),
      ...members.map(([key, c]) => ((n) => member(n.name, options.charts ? options.charts(key, c, n.who) : cohort(c, n.who, key === "all", options.of)))(naming(key))),
    ];
  };
  /** The client's name for a member (its table's `long`), in sentence case. @param {Record<string, { long: string }>} table @param {string} key */
  const named = (table, key) => sentence(table[key]?.long ?? key);
  /** A holder group's (or weighted cohort's) name, title, and what it adds to some coins ("Awake coins under 4 months old"). */
  const HOLDERS = /** @type {Record<string, [string, string, string]>} */ ({
    all: ["All coins", "All coins", ""], sth: ["Short-term holders", "Short-term holders", " of short-term holders"], lth: ["Long-term holders", "Long-term holders", " of long-term holders"],
    under4m: ["Under 4 months old", "Coins under 4 months old", " under 4 months old"], under6m: ["Under 6 months old", "Coins under 6 months old", " under 6 months old"],
    over4m: ["Over 4 months old", "Coins over 4 months old", " over 4 months old"], over6m: ["Over 6 months old", "Coins over 6 months old", " over 6 months old"],
    awake: ["Awake coins", "Awake coins", ""], mobile: ["Mobile coins", "Mobile coins", ""],
  });
  /**
   * The holder groups' things (by their keys: their cohorts, profit bands…), in the sets they're read in, each set's
   * items from `build` (by what its comparisons are by): all coins and short- and long-term holders, then the other age
   * cutoffs, in pairs, if any. @param {[string, any][]} list @param {(members: [string, any][], by: string) => (ChartNode | SectionNode)[]} build
   */
  const holderSets = (list, build) => {
    const pick = (/** @type {string[]} */ keys) => keys.flatMap((key) => list.filter(([k]) => k === key));
    const cutoffs = pick(["under4m", "over4m", "under6m", "over6m"]);
    return [...build(pick(["all", "sth", "lth"]), "holder group"), ...(cutoffs.length ? [section("Age cutoffs", build(cutoffs, "age cutoff"))] : [])];
  };
  /** Where an entry cohort came in, by its key, against the capitalized price then. @type {Record<string, string>} */
  const ENTERED = { discount: "at or below", premium: "above" };
  const holderName = (/** @type {string} */ key) => ({ name: HOLDERS[key]?.[0] ?? key, who: HOLDERS[key]?.[1] ?? key, of: (/** @type {string} */ coins) => `${coins}${HOLDERS[key]?.[2] ?? `, ${key}`}` });

  // ── The tree ──────────────────────────────────────────────────────────────

  const m = t.market, sup = t.supply, mining = t.mining, blocks = t.blocks, tx = t.transactions, ct = t.cointime, cf = t.coinflow;

  /** Moving averages (`sma` or `ema`): each over the price with its ratio below, and all of them together. @param {any} node @param {string} kind */
  const averages = (node, kind) => {
    const list = entries(node), colors = ramp(list.length, 200, 330);
    return section(kind, [
      chart("Compare", `${kind}s of the price`, [price(), ...list.map(([key, value], i) => series(own(value), `${windowAdjective(key)} ${kind}`, { color: colors[i], ...(["_50d", "_200d", "_200w", "_1y"].includes(key) ? {} : { hidden: true }) }))]),
      ...list.map(([key, value]) => chart(windowLabel(key), `${windowAdjective(key)} ${kind}${value.mayerMultiple ? " and the Mayer multiple" : ""}`, [
        price(), series(own(value), `${windowAdjective(key)} ${kind}`, { color: "sky" }),
        // (The 200-day's ratio is the Mayer multiple: shown as it, with its bands.)
        value.mayerMultiple ? series(value.mayerMultiple, "Mayer multiple", { pane: 2, ...around(1), levels: [{ value: 2.4, color: "red", dash: 3 }, { value: 0.8, color: "green", dash: 3 }] }) : series(value.ratio, "Price ratio", { pane: 2, ...around(1) }),
      ], { log: [2] })),
    ]);
  };

  const tree = [
    section("Price", [
      chart("In dollars", "Bitcoin price", [price()]),
      chart("Sats per dollar", "Sats per dollar", [series(t.price.satsPerDollar, "Sats per dollar", { color: "orange" })], { log: [1] }),
      section("All-time high", [
        chart("Drawdown", "Price drawdown from its all-time high", [price(), series(m.ath.high, "All-time high", { color: "green" }), series(m.ath.drawdown, "Drawdown", { pane: 2, color: "red", type: "area" })]),
        chart("Days since", "Days since the price's all-time high", [series(m.ath.daysSince, "Days since", { color: "orange" }), series(m.ath.daysBetweenMax, "Longest between", { color: "gray" })]),
      ]),
      section("Returns", [
        chart("Short-term", "Price returns, up to a year", windowsOf(m.returns).filter(([key]) => !/y$/.test(key) || key === "_1y").map(([key, value], i, all) => series(value, windowLabel(key), { ...baseline, ...shades(i, all.length), ...(key === "_1m" ? {} : { hidden: true }) }))),
        // (Over years, returns run from −80% to millions of percent: as growth multiples, on a log scale, 1× no change.)
        chart("Long-term", "Price growth over years", windowsOf(m.returns).filter(([key]) => /y$/.test(key) && key !== "_1y").map(([key, value], i, all) => ({ formula: `1 + ${value.name} / 100`, label: windowLabel(key), format: "ratio", ...around(1), ...shades(i, all.length), ...(key === "_4y" ? {} : { hidden: true }) })), { log: [1] }),
        // (Growth a year, compounded: what years of it made, each year alike; 4 years, a halving cycle, shown.)
        chart("Compound annual growth", "Compound annual growth of the price", [price(), ...[["2y", "2 years"], ["4y", "4 years"], ["8y", "8 years"]].map(([span, label], i) => ({ formula: `cagr(A, ${span})`, label, pane: 2, ...around(0), ...shades(i, 3), ...(span === "4y" ? {} : { hidden: true }) })), { formula: "cagr(A)", label: "Since its start", pane: 2, color: "gray", hidden: true }]),
        ...windowsOf(m.returns).map(([key, value]) => chart(windowLabel(key), `${windowAdjective(key)} price return`, [price(), series(value, `${windowAdjective(key)} return`, { pane: 2, ...baseline })])),
      ]),
      section("Moving averages", [averages(m.sma, "SMA"), averages(m.ema, "EMA")]),
      section("Range", [
        ...entries(m.range.max).map(([key]) => chart(`${windowAdjective(key)} high and low`, `Price's ${windowAdjective(key)} high and low`, [price(), series(m.range.max[key], "High", { color: "green" }), series(m.range.min[key], "Low", { color: "red" }), { cloud: ["B", "C"], label: "Range", color: "sky", opacity: 0.1 }])),
      ]),
      section("Technical", [
        chart("RSI", "Relative strength index of the price", [price(), { formula: "rsi(A)", label: "RSI", pane: 2, color: "violet", levels: [{ value: 30, color: "green", dash: 3 }, { value: 70, color: "red", dash: 3 }] }]),
        chart("MACD", "MACD of the price", [price(), { formula: "macd(A)", label: "MACD", pane: 2, color: "sky" }, { formula: "macd_signal(A)", label: "Signal", pane: 2, color: "orange" }, { formula: "macd_hist(A)", label: "Histogram", pane: 2, type: "histogram", colors: 4, color: "green", colorBelow: "red" }]),
        chart("Volatility", "Annualized price volatility", [price(), { formula: "volatility(A, 30d)", label: "30 days", pane: 2, color: "sky" }, { formula: "volatility(A, 1y)", label: "1 year", pane: 2, color: "violet" }]),
        // (Marked where the 111-day crosses over twice the 350-day: the tops it called.)
        chart("Pi cycle", "Pi cycle top", [price(), { formula: "sma(A, 111d)", label: "111-day SMA", color: "green" }, { formula: "2 * sma(A, 350d)", label: "350-day SMA × 2", color: "red" }, { formula: "B > C", type: "signal", ref: "A", label: "Top signal", color: "red" }]),
        // (Up where the 50-day is over the 200-day, down where under: each run's result on the cross that ends it, their
        // total in its line.)
        chart("Golden and death crosses", "Golden and death crosses", [price(), { formula: "sma(A, 50d)", label: "50-day SMA", color: "amber" }, { formula: "sma(A, 200d)", label: "200-day SMA", color: "sky" }, { formula: "(B > C) - (B < C)", type: "signal", ref: "A", place: "below", results: true, label: "Cross", color: "green" }]),
        // (A stop on the last 20 days' low while the price holds over it; broken, on their high, till that breaks: the
        // turtles' breakout, as a trend.)
        chart("Trailing stop", "A 20-day trailing stop", [price(), { formula: "lag(highest(A, 20d), 1d)", label: "20-day high", color: "gray", dash: 2, hidden: true }, { formula: "lag(lowest(A, 20d), 1d)", label: "20-day low", color: "gray", dash: 2, hidden: true }, { trend: ["B", "C"], ref: "A", label: "Trailing stop", color: "red", colorBelow: "green", width: 2 }], { range: "1y" }),
        // (Each halving cycle mined from its start, at 1×: as ⌘K's "Compare halving cycles" makes them; a cycle's years,
        // its halving's, every four.)
        ((cycles) => chart("Halving cycles", "The price's halving cycles, each from its start", [price(), ...Array.from({ length: cycles }, (_, n) => ({ formula: `rebase(when(A, halving_epoch == ${n}))`, label: `${n ? 2008 + 4 * n : 2009}–${n + 1 < cycles ? 2012 + 4 * n : "now"}`, pane: 2, color: ["orange", "red", "blue", "violet", "green", "teal", "pink"][n % 7] }))], { log: [2], halvings: true, range: "max" }))(Math.floor((facts?.height ?? 840_000) / 210_000) + 1),
      ]),
    ]),
    section("Valuation", [
      chart("Market and realized cap", "Market cap and realized cap", [series(t.holders.all.supply.total.usd, "Market cap", { color: "green" }), series(t.holders.all.capital.realizedCap, "Realized cap", { color: "orange" })], { log: [1], priceHidden: true }),
      ...change(sup.marketCap.delta, "Market cap change", "Market cap change"),
      chart("MVRV", "Market value to realized value (MVRV)", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: "orange" }), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1), levels: [{ value: 3.5, color: "red", dash: 3 }] })], { log: [2] }),
      // (The price in its MVRV's zone, date by date: under 1, under its realized value; over 3.5, far over it.)
      chart("Price by MVRV zone", "The price by its MVRV zone", [{ ...price(), hidden: true }, series(t.price.close, "Price by zone", { colorFrom: "C", color: "gray", width: 2 }), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, color: "green", byLevels: true, underName: "Undervalued", levels: [{ value: 1, color: "gray", name: "Fair", dash: 3 }, { value: 3.5, color: "red", name: "Overvalued", dash: 3 }] })], { log: [1, 2] }),
      // (How far market cap is over realized cap, in market cap's deviations over all history: over 7, its tops.)
      chart("MVRV Z-score", "MVRV Z-score", [price(), { formula: `(${t.holders.all.supply.total.usd.name} - ${t.holders.all.capital.realizedCap.name}) / stdev(${t.holders.all.supply.total.usd.name})`, label: "MVRV Z-score", pane: 2, format: "number", ...baseline, levels: [{ value: 7, color: "red", dash: 3, name: "Top zone" }] }]),
      chart("NUPL", "Net unrealized profit and loss (NUPL)", [price(), series(t.holders.all.unrealized.nupl, "NUPL", { pane: 2, ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
      // (Market cap over the day's transfer volume in dollars: the network's value against what moves on it; high, valued
      // over its use. A day's volume swings: faint, its signal over it (the volume's 90-day average), as the hash rate's.)
      ((cap, volume) => chart("NVT", "Network value to transactions (NVT)", [price(), { formula: `${cap} / ${volume}`, label: "NVT", format: "ratio", pane: 2, color: "orange", width: 1, opacity: 0.4 }, { formula: `${cap} / sma(${volume}, 90d)`, label: "NVT signal", format: "ratio", pane: 2, color: "orange", width: 2 }], { log: [2] }))(sup.marketCap.value.name, t.holders.all.activity.transferVolume.sum._24h.usd.name),
      // (Market cap over thermo cap, every dollar ever paid to miners: how far the market's value runs over what securing it
      // has cost.)
      ((cap, thermo) => chart("Thermocap multiple", "Thermocap multiple", [price(), { formula: `${cap} / ${thermo}`, label: "Thermocap multiple", format: "ratio", pane: 2 }], { log: [2] }))(sup.marketCap.value.name, ct.caps.thermo.name),
      // (The newest coins' realized cap, a day to a week old, over the one-to-two-year-olds', weighed by the market's age in
      // days (more of it held as it ages): high, new money over held, as near the tops.)
      ((week, young, old) => chart("RHODL", "RHODL ratio", [price(), { formula: `${week} / (${young} + ${old}) * (date - 2009-01-03) / 86400`, label: "RHODL ratio", format: "number", pane: 2 }], { log: [2] }))(t.age.ranges._1dTo1w.capital.realizedCap.name, t.age.ranges._1yTo18m.capital.realizedCap.name, t.age.ranges._18mTo2y.capital.realizedCap.name),
      chart("On-chain prices", "Realized, true market mean, Cointime and capitalized prices", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: "orange" }), series(ct.prices.trueMarketMean.value, "True market mean", { color: "sky" }), series(ct.prices.cointime.value, "Cointime price", { color: "violet" }), series(t.holders.all.costBasis.byDollar.capitalizedPrice, "Capitalized price", { color: "green" })]),
      cloudsChart(),
      chart("Cointime caps", "Cointime capitalizations", [...[["thermo", "Thermo cap"], ["investor", "Investor cap"], ["active", "Active cap"], ["vaulted", "Vaulted cap"], ["cointime", "Cointime cap"]].map(([key, label], i) => series(ct.caps[key], label, { color: ["orange", "green", "red", "sky", "violet"][i] })), series(ct.caps.investorness, "Investorness", { pane: 2, color: "green" }), series(ct.caps.producerness, "Producerness", { pane: 2, color: "orange" })], { log: [1] }),
      chart("Reserve risk", "Reserve risk", [price(), series(ct.reserveRisk.value, "Reserve risk", { pane: 2, color: "orange" }), series(ct.reserveRisk.hodlBank, "HODL bank", { pane: 3, color: "sky" })], { log: [2, 3] }),
      // (By its short name, as MVRV and NUPL are, the long one in its title: long, it and its all-time twin read alike, cut short.)
      ...flow(ct.reserveRisk.vocdd, "VOCDD", "Value of coin days destroyed (VOCDD)"),
      section("Cointime prices", [
        chart("Compare", "Cointime prices", [price(), ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label], i) => series(ct.prices[key].value, label, { color: ["green", "red", "violet"][i] })), series(ct.prices.trueMarketMean.value, "True market mean", { color: "sky" })]),
        ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label]) => chart(label, label, [price(), series(ct.prices[key].value, label, { color: "sky" }), series(ct.prices[key].mvrv, "MVRV", { pane: 2, ...around(1) })], { log: [2] })),
        timeAboveChart(),
        chart("True market mean", "True market mean and AVIV", [price(), series(ct.prices.trueMarketMean.value, "True market mean", { color: "sky" }), series(ct.prices.trueMarketMean.aviv, "AVIV", { pane: 2, ...around(1) }), series(ct.prices.trueMarketMean.avivNupl, "AVIV NUPL", { pane: 3, ...baseline })], { log: [2] }),
      ]),
      section("Rarity meter", [
        ...entries(t.rarityMeter).filter(([key]) => ["full", "local", "cycle"].includes(key)).map(([key, node]) => section(`${key[0].toUpperCase()}${key.slice(1)}`, [
          chart("Bands", `Rarity meter bands, ${key}`, [price(), ...entries(node).filter(([k]) => Number.isFinite(percentile(k))).map(([k, value], i, all) => series(value, percentileLabel(percentile(k)), { color: ramp(all.length, 140, 360)[i], width: k === "median" ? 2 : 1 }))]),
          chart("Level and score", `Rarity meter level and score, ${key}`, [series(node.level, "Level", { ...baseline, type: "histogram" }), series(node.score, "Score", { pane: 2, ...baseline })]),
        ])),
        // (Each component by its cohort: a reference price's bands and ratios.)
        section("Components", entries(t.rarityMeter.components).map(([key, node]) => section(holderName(key).name, entries(node).map(([k, v]) => component(v, words(k), holderName(key).name, referenceOf(key, k)))))),
        // (Each extreme against its bands: the median and its rare side's tail shown, the others a click away.)
        section("Extremes", [eventsChart(), ...entries(t.rarityMeter.extremes).map(([key, node]) => {
          const label = EXTREMES[key]?.label ?? words(key), low = EXTREMES[key]?.low;
          return chart(label, `Rarity meter extremes: ${label}`, [series(own(node), label, { color: "orange", width: 2 }), ...bandLines(node, (v) => v, (p) => p === 50 || (low ? p <= 1 : p >= 99))], { log: [1] });
        })]),
      ]),
    ]),
    section("Supply", [
      chart("Circulating", "Circulating supply", [series(sup.circulating, "Supply", { color: "orange" }), series(sup.marketCap.value, "Market cap", { color: "green", pane: 2 })], { log: [2] }),
      chart("Inflation", "Inflation rate", [series(sup.inflationRate, "Inflation rate", { color: "orange" }), series(ct.adjusted.inflationRate, "Cointime-adjusted", { color: "violet" })], { log: [1] }),
      ...flow(sup.burned, "Burned", "Bitcoin burned"),
      chart("Velocity", "Velocity", [series(sup.velocity.btc, "In bitcoin", { color: "orange" }), series(sup.velocity.usd, "In dollars", { color: "green" }), series(ct.adjusted.velocity.btc, "Cointime-adjusted, in bitcoin", { color: "violet", hidden: true }), series(ct.adjusted.velocity.usd, "Cointime-adjusted, in dollars", { color: "sky", hidden: true })]),
    ]),
    section("Holders", [
      ...holderSets(entries(t.holders), (members, by) => family(members, holderName, { by })),
      // (The discount and premium coins: all coins split by the price they came in at, at or below or above the
      // capitalized price then.)
      section("Entry price", family(entries(t.entry), (key) => ({ name: named(client.ENTRY_NAMES, key), who: `${named(client.ENTRY_NAMES, key)}, entered ${ENTERED[key] ?? key} the capitalized price at the time` }), { by: "entry price", disjoint: true })),
    ]),
    section("Coin age", [
      lastMovedChart(),
      section("Age bands", family(entries(t.age.ranges), (key) => ({ name: named(client.AGE_RANGE_NAMES, key).replace(/ old$/, ""), who: `UTXOs ${named(client.AGE_RANGE_NAMES, key).toLowerCase().replace(/^utxos /, "")}` }), { by: "age band", disjoint: true, known: { Supply: "HODL waves", "Realized cap": "realized cap HODL waves" } })),
      section("Halving epochs", family(members(t.age.halvingEpochs), (key) => ({ name: `Epoch ${key.slice(1)}`, who: `Coins last moved in halving epoch ${key.slice(1)}` }), { by: "halving epoch last moved", disjoint: true })),
      section("Vintages", family(members(t.age.vintages), (key) => ({ name: key.slice(1), who: `Coins last moved in ${key.slice(1)}` }), { by: "vintage", disjoint: true })),
    ]),
    section("Profitability", holderSets(entries(t.profitability), (groups) => groups.map(([key, bands]) => section(holderName(key).name, family(entries(bands), (band) => ({ name: named(client.PROFITABILITY_RANGE_NAMES, band), who: `${holderName(key).who}, ${named(client.PROFITABILITY_RANGE_NAMES, band).toLowerCase()}` }), { by: `profit band, ${holderName(key).who.toLowerCase()}`, disjoint: true, ...(key === "all" ? {} : { of: `${holderName(key).who.toLowerCase()}'` }) }))))),
    section("UTXOs", [
      chart("Count", "UTXO count", [series(t.utxoSet.count, "UTXOs", { color: "sky" })], { log: [1] }),
      section("By amount", family(entries(t.utxos.amounts), (key) => ({ name: named(client.AMOUNT_RANGE_NAMES, key), who: `UTXOs of ${named(client.AMOUNT_RANGE_NAMES, key)}` }), { by: "UTXO amount", disjoint: true })),
      section("By type", family(entries(t.utxos.types), (key) => ({ name: client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, who: `${client.SPENDABLE_TYPE_NAMES[key]?.short ?? key} outputs` }), { by: "output type", disjoint: true })),
    ]),
    section("Addresses", [
      ...addressCharts(t.addresses, "All addresses"),
      section("By type", [
        section("Compare", [
          ...[["Funded", "Funded addresses by type", (/** @type {any} */ a) => own(a.funded)], ["Total", "Total addresses by type", (/** @type {any} */ a) => a.total], ["New, 1 month", "New addresses by type, 1 month", (/** @type {any} */ a) => a.new.sum._1m], ["Active per block, 1 month", "Addresses active per block by type, 1-month average", (/** @type {any} */ a) => a.activity.active.avg._1m]].map(([name, title, pick]) => chart(/** @type {string} */ (name), /** @type {string} */ (title), entries(t.addresses.types).map(([key, a], i, all) => series(/** @type {(a: any) => any} */ (pick)(a), client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, { color: ramp(all.length)[i] })), { log: [1] })),
        ]),
        ...entries(t.addresses.types).map(([key, a]) => section(client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, addressCharts(a, `${client.SPENDABLE_TYPE_NAMES[key]?.short ?? key} addresses`))),
      ]),
      section("By balance", family(entries(t.addresses.balances), (key) => ({ name: named(client.AMOUNT_RANGE_NAMES, key), who: `Addresses holding ${named(client.AMOUNT_RANGE_NAMES, key)}` }), { by: "address balance", disjoint: true, of: "all addresses'" })),
    ]),
    section("Mining", [
      section("Hashing", [
        // (The day's, noisy, under its average: thin and faint, the average the line to read.)
        chart("Hash rate", "Hash rate", [series(own(mining.hashrate), "Hash rate", { color: "orange", width: 1, opacity: 0.4 }), ...windows(mining.hashrate.sma, "Average", { show: ["_1m"], extra: { width: 2 } }), series(mining.hashrate.ath, "All-time high", { color: "green", dash: 3, hidden: true }), series(blocks.difficulty.hashrate, "From difficulty", { color: "gray", hidden: true }), series(mining.hashrate.drawdown, "Drawdown", { pane: 2, color: "red", type: "area" })], { log: [1] }),
        // (Its 1- and 2-month averages, the 30- and 60-day of the convention: capitulation while the first is under, shaded;
        // no marks where it crosses back, the averages crossing too often to read.)
        chart("Hash ribbons", "Hash ribbons", [
          price(),
          series(mining.hashrate.sma._1m, "1-month average", { pane: 2, color: "sky" }),
          series(mining.hashrate.sma._2m, "2-month average", { pane: 2, color: "violet" }),
          { formula: "B < C", type: "signal", ref: "B", place: "shade", label: "Capitulation", color: "red" },
        ], { log: [2] }),
        // (A day's revenue per PH/s: in dollars, its hash price, the one asked for, first; below in satoshis, its hash value;
        // each over its all-time low, then how far above it each is.)
        chart("Hash price and value", "Hash price and hash value", [
          series(own(mining.hashPrice), "Hash price", { color: "green" }), series(mining.hashPrice.atl, "Hash price's all-time low", { color: "red", dash: 3 }),
          series(own(mining.hashValue), "Hash value", { pane: 2, color: "orange" }), series(mining.hashValue.atl, "Hash value's all-time low", { pane: 2, color: "red", dash: 3 }),
        ], { log: [1, 2] }),
        chart("Rebound", "Hash price and hash value above their all-time lows", [series(mining.hashPrice.rebound, "Hash price", { color: "green" }), series(mining.hashValue.rebound, "Hash value", { color: "orange" })]),
      ]),
      section("Difficulty", [
        chart("History", "Difficulty", [series(blocks.difficulty.value, "Difficulty", { color: "orange", shape: "steps" }), series(blocks.difficulty.adjustment, "Adjustment", { pane: 2, type: "histogram", color: "green", colorBelow: "red" })], { log: [1] }),
        chart("Next adjustment", "Until the next difficulty adjustment", [series(blocks.difficulty.blocksToRetarget, "Blocks", { color: "sky" }), series(blocks.difficulty.daysToRetarget, "Days", { color: "orange", pane: 2 })]),
        chart("Epoch", "Difficulty epoch", [series(blocks.difficulty.epoch, "Epoch", { color: "gray" })]),
      ]),
      // (The subsidy's schedule, as Difficulty is the work's.)
      section("Halvings", [
        chart("Next halving", "Until the next halving", [series(blocks.halving.blocksToHalving, "Blocks", { color: "sky" }), series(blocks.halving.daysToHalving, "Days", { color: "orange", pane: 2 })]),
        chart("Epoch", "Halving epoch", [series(blocks.halving.epoch, "Halving epoch", { color: "gray" })]),
      ]),
      section("Rewards", [
        ...flow(mining.rewards.coinbase, "Coinbase", "Coinbase rewards"),
        ...flow(mining.rewards.subsidy, "Subsidy", "Block subsidy"),
        ...flow(mining.rewards.fees, "Fees", "Transaction fees"),
        ...flow(mining.rewards.unclaimed, "Unclaimed", "Unclaimed rewards"),
        // (The two make up the rewards: one chart, each in its color, since the start dashed.)
        chart("Subsidy and fee shares", "Subsidy's and fees' shares of rewards", /** @type {[any, string, string][]} */ ([[mining.rewards.subsidy.share, "Subsidy", "orange"], [mining.rewards.fees.share, "Fees", "green"]]).flatMap(([share, what, color]) => [
          series(share.cumulative, `${what}, all time`, { color, dash: 3 }),
          ...windows(share, what, { extra: { color } }),
        ])),
        // (A day's issuance in dollars against its year's average, the server's rolling sums (the same at every interval):
        // miners' income high or low for the time. From a year after the first price on: before it, issuance is worth 0,
        // the year's sum short; the guard a year back, 1 or 0 over 0, hides it.)
        ((day, year) => chart("Puell multiple", "Puell multiple", [price(), { formula: `365 * ${day} / ${year} * (lag(${day}, 365d) / lag(${day}, 365d))`, label: "Puell multiple", format: "ratio", pane: 2, ...around(1), levels: [{ value: 4, color: "red", dash: 3 }, { value: 0.5, color: "green", dash: 3 }] }], { log: [2] }))(mining.rewards.subsidy.sum._24h.usd.name, mining.rewards.subsidy.sum._1y.usd.name),
        ...["avg", "min", "max", "p10", "p25", "median", "p75", "p90"].filter((key) => mining.rewards.fees[key]).length
          ? [chart("Fees per block", "Fees per block", [...["avg", "min", "p10", "p25", "median", "p75", "p90", "max"].filter((key) => mining.rewards.fees[key]).flatMap((key, i) => windows(mining.rewards.fees[key], key === "avg" ? "Average" : key === "min" ? "Lowest" : key === "max" ? "Highest" : percentileLabel(percentile(key)), { pick: btc, show: key === "median" || key === "avg" ? ["_1m"] : [], extra: { color: ramp(8, 25, 300)[i] } }))], { log: [1] }),
            chart("Fees per block in dollars", "Fees per block, in dollars", [...["avg", "min", "p10", "p25", "median", "p75", "p90", "max"].filter((key) => mining.rewards.fees[key]).flatMap((key, i) => windows(mining.rewards.fees[key], key === "avg" ? "Average" : key === "min" ? "Lowest" : key === "max" ? "Highest" : percentileLabel(percentile(key)), { pick: usd, show: key === "median" || key === "avg" ? ["_1m"] : [], extra: { color: ramp(8, 25, 300)[i] } }))], { log: [1] })]
          : [],
      ]),
      section("Pools", poolCharts(), { workings: true }),
    ]),
    section("Network", [
      section("Blocks", [
        ...flow(blocks.count, "Count", "Blocks mined"),
        chart("Interval", "Time between blocks", windows(blocks.interval.avg, "", { extra: { levels: [{ value: 600, color: "gray", dash: 3 }] } })),
        // (Their sums, every block's together, "total"; each block's, its average and percentiles.)
        ...stats(blocks.size, "Size", "Total block size", "Block size"),
        ...stats(blocks.weight, "Weight", "Total block weight", "Block weight"),
      ]),
      section("Transactions", [
        ...stats(tx.count, "Count", "Transactions"),
        chart("Per second", "Transactions per second", windows(tx.perSecond, "")),
        ...flow(tx.transferVolume, "Volume", "Transaction volume"),
        chart("Vsize", "Transaction vsize", percentileStats(tx.vsize), { log: [1] }),
        chart("Fee", "Transaction fee", percentileStats(tx.fees.fee), { log: [1] }),
        chart("Fee rate", "Transaction fee rate", percentileStats(tx.fees.effectiveFeeRate), { log: [1] }),
        ...counted(tx.nonstandard, "Nonstandard", "Nonstandard transactions"),
        ...flow(tx.sigopCost, "Signature operations", "Signature operation cost"),
        section("Features", [
          ...txKinds(entries(tx.features).filter(([, node]) => node.share), featureName, featureTx, "Share of transactions by feature"),
          section("Scripts", txKinds(entries(tx.features.scripts), featureName, featureTx, "Share of transactions by script")),
          section("Signatures", txKinds(entries(tx.features.signatures), featureName, featureTx, "Share of transactions by signature feature")),
        ]),
        section("Versions", [
          section("Compare", parts(["Transactions, 1 month", "Transactions by version, 1 month"], ["Transaction share, 1 month", "Share of transactions by version, 1 month"], entries(tx.versions).map(([key, node], i, all) => series(node.sum._1m, versionName(key), { color: ramp(all.length)[i] })))),
          ...entries(tx.versions).flatMap(([key, node]) => counted(node, versionName(key), key === "other" ? "Transactions of other versions" : `${versionName(key)} transactions`)),
        ]),
        section("Patterns", txKinds(entries(tx.patterns), words, (key) => `${words(key)} transactions`, "Share of transactions by pattern")),
        section("CPFP", [...counted(tx.fees.cpfpParent, "Parents", "CPFP parent transactions"), ...counted(tx.fees.cpfpChild, "Children", "CPFP child transactions")]),
        section("Inscriptions", [...counted(tx.inscription, "Count", "Inscription transactions"), ...flow(tx.inscription.fees, "Transaction fees", "Inscription transactions' fees"), chart("Fee share", "Inscription transactions' share of all fees", sharesOf(tx.inscription.fees.chainShare, "Share"))]),
      ]),
      section("Inputs", [
        ...stats(t.inputs.count, "Count", "Inputs"),
        chart("Per second", "Inputs per second", windows(t.inputs.perSecond, "")),
        section("By type", byType(t.inputs.types, "inputs", "spending")),
      ]),
      section("Outputs", [
        ...stats(t.outputs.count, "Count", "Outputs"),
        chart("Per second", "Outputs per second", windows(t.outputs.perSecond, "")),
        ...flow(t.outputs.spendableCount, "Spendable", "Spendable outputs"),
        section("By type", byType(t.outputs.types, "outputs", "creating", ["opReturn"])),
      ]),
      section("OP_RETURN", [
        ...opReturnCharts(t.opReturn, (what) => `OP_RETURN ${what}`, t.outputs.types.opReturn.share, t.outputs.opReturnValue),
        section("Protocols", [
          section("Compare", [
            ...parts(["Outputs, 1 month", "OP_RETURN outputs by protocol, 1 month"], ["Output share, 1 month", "Share of OP_RETURN outputs by protocol, 1 month"], entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.outputCount.sum._1m, protocolName(key), { color: ramp(all.length)[i] }))),
            chart("Transaction share, 1 month", "Share of transactions by OP_RETURN protocol, 1 month", entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.txShare._1m, protocolName(key), { color: ramp(all.length)[i] }))),
          ]),
          ...entries(t.opReturn.protocols).map(([key, node]) => section(protocolName(key), opReturnCharts(node, (what) => `${protocolName(key)} OP_RETURN ${what}`))),
        ]),
        section("Policies", [
          // (They overlap, oversized and several outputs both beyond the standard before v30: lines.)
          section("Compare", [
            chart("Transaction share, 1 month", "Share of transactions by OP_RETURN policy, 1 month", entries(t.opReturn.policies).map(([key, node], i, all) => series(node.txShare._1m, policyName(key), { color: ramp(all.length)[i] }))),
            chart("Data, 1 month", "OP_RETURN data by policy, 1 month", entries(t.opReturn.policies).map(([key, node], i, all) => series(node.dataBytes.sum._1m, policyName(key), { color: ramp(all.length)[i] })), { log: [1] }),
          ]),
          ...entries(t.opReturn.policies).map(([key, node]) => section(policyName(key), opReturnCharts(node, (what) => `OP_RETURN ${what}: ${policyName(key)}`))),
        ]),
      ]),
    ], { workings: true }),
    section("Cointime", [
      // (HODLed or lost is the vaulted supply by another name: one line.)
      chart("Supply", "Vaulted and active supply", [...[["vaulted", "Vaulted"], ["active", "Active"]].flatMap(([key, label], i) => [series(ct.supply[key].btc, label, { color: ["green", "red"][i] }), series(ct.supply[key].usd, `${label}, value`, { color: ["green", "red"][i], pane: 2 })])]),
      section("Activity", [
        ...flow(ct.activity.coinblocksCreated, "Coin blocks created", "Coin blocks created"),
        ...flow(ct.activity.coinblocksDestroyed, "Coin blocks destroyed", "Coin blocks destroyed"),
        ...flow(ct.activity.coinblocksStored, "Coin blocks stored", "Coin blocks stored"),
        chart("Liveliness", "Liveliness and vaultedness", [series(ct.activity.liveliness, "Liveliness", { color: "red" }), series(ct.activity.vaultedness, "Vaultedness", { color: "green" }), series(ct.activity.livelinessToVaultedness, "Liveliness to vaultedness", { pane: 2, color: "orange" })]),
        chart("Concurrent liveliness", "Concurrent liveliness", windows(ct.activity.concurrentLiveliness, "")),
      ]),
      section("Value", entries(ct.value).flatMap(([key, node]) => flow(node, `Value ${key}`, `Cointime value ${key}`))),
      section("Age bands", cointimeBands(ct.ageRanges)),
      section(holderName("awake").name, weightedSets(ct.cohorts, "awake", "dormant", "Awake", "Dormant")),
    ]),
    section("Coinflow", [
      section("Age bands", coinflowBands(cf.ageRanges)),
      section(holderName("mobile").name, weightedSets(cf.cohorts, "mobile", "immobile", "Mobile", "Immobile")),
    ]),
    section("Bedrock", [
      // (Each floor, its modes' over the price: how far weighting moves it.)
      section("Compare", entries(entries(t.bedrock)[0][1].floor).map(([key]) => chart(`${percentileLabel(percentile(key))} floor`, `Bedrock ${percentileLabel(percentile(key))} floor by mode`, [price(), ...entries(t.bedrock).map(([mode, node], i, all) => series(node.floor[key], modeName(mode), { color: ramp(all.length)[i] }))]))),
      ...entries(t.bedrock).map(([mode, node]) => {
        // The Bedrock landscape, top to bottom: sky over the all-time high, water from it to the median cost basis above
        // the floor, soil down to the first floor, bedrock through the floors, the deepest down to the bottom.
        const edges = [
          [m.ath.high, "All-time high"],
          ...entries(node.costBasisAboveP95Floor).reverse().map(([key, value]) => [value, `${percentileLabel(percentile(key))} cost basis`]),
          ...entries(node.floor).map(([key, value]) => [value, `${percentileLabel(percentile(key))} floor`]),
        ];
        const bands = entries(node.entrySupply), colors = ramp(bands.length, 140, 200);
        return section(modeName(mode), [
          chart("Floors", `Bedrock floors, ${mode}`, [price(), ...entries(node.floor).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 140, 200)[i] }))]),
          // (The price first: in front.)
          chart("Floors, colored", `Bedrock landscape, ${mode}`, [
            price(),
            ...edges.map(([value, label], i) => series(value, label, { color: LANDSCAPE[i], width: 1 })),
            { cloud: [letter(1), "top"], label: "Above the all-time high", color: SKY, opacity: 1 },
            ...edges.map(([, label], i) => ({ cloud: [letter(i + 1), i + 1 < edges.length ? letter(i + 2) : "bottom"], label: i + 1 < edges.length ? `${label} to ${edges[i + 1][1].toLowerCase()}` : `Below the ${label.toLowerCase()}`, color: LANDSCAPE[i], opacity: 1 })),
          ]),
          chart("Supply between floors", `Supply created between Bedrock floors, ${mode}`, [
            ...bands.map(([key, value], i) => series(value.btc, floorBandLabel(key), { color: colors[i], type: "area" })),
            ...bands.map(([key, value], i) => series(value.usd, `${floorBandLabel(key)}, value`, { color: colors[i], type: "area", pane: 2 })),
          ], { stack: { 1: "stack", 2: "stack" } }),
          chart("Capital between floors", `Capital of the supply created between Bedrock floors, ${mode}`, entries(node.entryCapital).map(([key, value], i) => series(value, floorBandLabel(key), { color: colors[i], type: "area" })), { stack: { 1: "stack" } }),
          chart("Cost basis above the floor", `Cost basis above the Bedrock floor, ${mode}`, [price(), ...entries(node.costBasisAboveP95Floor).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 160)[i], width: key === "median" ? 2 : 1 }))]),
          chart("Supply-in-loss thresholds", `Bedrock's supply-in-loss thresholds, ${mode}`, entries(node.supplyInLossShareThreshold).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 100)[i] }))),
        ]);
      }),
    ]),
  ];

  // ── Pieces the tree above uses ────────────────────────────────────────────

  /**
   * A per-block measure's charts: its sums (with since the start) and its statistics over windows (average,
   * percentiles), titled `perBlock` ("Transactions per block", "Block size").
   * @param {any} node @param {string} name @param {string} title
   */
  function stats(node, name, title, perBlock = `${title} per block`) {
    const keys = ["avg", "min", "p10", "p25", "median", "p75", "p90", "max"].filter((key) => node[key]), colors = ramp(keys.length, 25, 300);
    return [
      ...flow(node, name, title),
      ...(keys.length ? [chart(`${name} per block`, `${perBlock}: average and percentiles`, keys.flatMap((key, i) => windows(node[key], statLabel(key), { show: key === "avg" ? ["_1m"] : [], extra: { color: colors[i] } })), { log: [1] })] : []),
    ];
  }
  /** Percentile statistics as layers (min, percentiles, max), the median shown. @param {any} node */
  function percentileStats(node) {
    const keys = ["min", "p10", "p25", "median", "p75", "p90", "max"].filter((key) => byDate(node[key])), colors = ramp(keys.length, 25, 300);
    return keys.map((key, i) => series(node[key], statLabel(key), { color: colors[i], ...(key === "median" ? {} : { hidden: true }) }));
  }
  function statLabel(/** @type {string} */ key) {
    return key === "avg" ? "Average" : key === "min" ? "Lowest" : key === "max" ? "Highest" : percentileLabel(percentile(key));
  }
  /** A transaction feature's name ("P2TR", "SIGHASH_ALL"): an output type's, else as said (FEATURES), else its key's words. @param {string} key */
  function featureName(key) {
    return client.SPENDABLE_TYPE_NAMES[key]?.short ?? FEATURES[/** @type {keyof typeof FEATURES} */ (key)] ?? words(key);
  }
  /** A feature's transactions ("Transactions with P2TR", "with unknown scripts", "with a Taproot annex"). @param {string} key */
  function featureTx(key) {
    const type = client.SPENDABLE_TYPE_NAMES[key]?.short;
    if (type) return `Transactions with ${inSentence(type)}${inSentence(type) === type ? "" : " scripts"}`;
    const feature = /** @type {keyof typeof FEATURE_PHRASES} */ (key);
    return `Transactions with ${FEATURE_PHRASES[feature] ?? FEATURES[/** @type {keyof typeof FEATURES} */ (key)] ?? words(key).toLowerCase()}`;
  }
  /** An OP_RETURN protocol's name ("Eternity Wall"), as the client names it. @param {string} key */
  function protocolName(key) {
    return client.OP_RETURN_KIND_NAMES[key];
  }
  /** A transaction version's name: `v2` "Version 2", `other` "Other". @param {string} key */
  function versionName(key) {
    return key === "other" ? "Other" : `Version ${key.slice(1)}`;
  }
  function policyName(/** @type {string} */ key) {
    return key === "preV30Standard" ? "Standard before v30" : key === "preV30Nonstandard" ? "Nonstandard before v30" : words(key);
  }
  /**
   * Inputs or outputs by type: each type's count and transactions, each with its share; a type with a home of its own
   * (`apart`: OP_RETURN's, in Network) compared here, its charts there. @param {any} types @param {string} what
   * @param {string} doing @param {string[]} [apart]
   */
  function byType(types, what, doing, apart = []) {
    const list = entries(types), colors = ramp(list.length), What = `${what[0].toUpperCase()}${what.slice(1)}`;
    const name = (/** @type {string} */ key) => client.SPENDABLE_TYPE_NAMES[key]?.short ?? (key === "opReturn" ? "OP_RETURN" : key);
    return [
      section("Compare", [
        ...parts([`${What}, 1 month`, `${What} by type, 1 month`], [`${What.slice(0, -1)} share, 1 month`, `Share of ${what} by type, 1 month`], list.map(([key, node], i) => series(node.count.sum._1m, name(key), { color: colors[i] }))),
        chart("Transaction share, 1 month", `Share of transactions ${doing} each ${what.slice(0, -1)} type, 1 month`, list.map(([key, node], i) => series(node.txShare._1m, name(key), { color: colors[i] }))),
      ]),
      ...list.filter(([key]) => !apart.includes(key)).map(([key, node]) => section(name(key), [
        ...counted(node.count, "Count", `${name(key)} ${what}`, node.share),
        ...counted(node.txCount, "Transactions", `Transactions ${doing} ${inSentence(name(key))} ${what}`, node.txShare),
      ])),
    ];
  }
  /**
   * Supply last moved over 1, 2, 5 and 10 years ago: the age bands from each on, summed (in bitcoin, and their shares of
   * all coins below).
   */
  function lastMovedChart() {
    const bands = entries(t.age.ranges), colors = ramp(4);
    // (Each from the band starting at its age: one the client renamed fails here, not quietly summed short.)
    const from = (/** @type {string} */ key) => {
      const at = bands.findIndex(([k]) => k === key);
      if (at < 0) throw new Error(`No age band ${key}`);
      return bands.slice(at).map(([, node]) => node.supply);
    };
    const over = /** @type {[string, string][]} */ ([["_1yTo18m", "1 year"], ["_2yTo3y", "2 years"], ["_5yTo6y", "5 years"], ["_10yTo12y", "10 years"]]);
    return chart("Last moved", "Supply last moved over 1, 2, 5 and 10 years ago", [
      ...over.map(([key, age], i) => ({ formula: from(key).map((supply) => supply.total.btc.name).join(" + "), label: `Over ${age}`, format: "btc", color: colors[i] })),
      ...over.map(([key, age], i) => ({ formula: from(key).map((supply) => supply.share.name).join(" + "), label: `Over ${age}, share`, format: "percent", color: colors[i], pane: 2 })),
    ]);
  }
  /** Bitcoin clouds: each holder group's awake and mobile realized and capitalized prices (its cohorts' together), a
   * cloud from the lowest to the highest. */
  function cloudsChart() {
    const groups = /** @type {[string, string[], string][]} */ ([["Short-term holders", ["under4m", "sth", "under6m"], "#e19b00"], ["All holders", ["all"], "#f2660f"], ["Long-term holders", ["over4m", "lth", "over6m"], "#d50ced"]]);
    return chart("Cost basis clouds", "Holder groups' cost basis ranges (Bitcoin clouds)", [
      price(),
      ...groups.flatMap(([label, keys, color]) => {
        const names = keys.flatMap((key) => [ct.cohorts[key].awake, cf.cohorts[key].mobile]).flatMap((cohort) => [cohort.costBasis.byCoin.realizedPrice.name, cohort.costBasis.byDollar.capitalizedPrice.name]);
        const fold = (/** @type {string} */ fn) => names.reduce((all, name) => `${fn}(${all}, ${name})`);
        return [{ formula: fold("min"), label: `${label}, lowest`, format: "usd", color, width: 1 }, { formula: fold("max"), label: `${label}, highest`, format: "usd", color, width: 1 }];
      }),
      ...groups.map(([label, , color], i) => ({ cloud: [letter(1 + 2 * i), letter(2 + 2 * i)], label, color, opacity: 0.3 })),
    ]);
  }
  /** Extreme events: the days each extreme's input crossed into its historical 0.1% tail (its 0.1st percentile for the
   * sell-side risk ratio, whose rare side is low), marked on the price. */
  function eventsChart() {
    return chart("Events", "Rarity meter extreme events", [
      price(),
      // (Each extreme named here, by its key: one the client renamed fails here, not quietly gone.)
      ...Object.entries(EXTREMES).map(([key, { event, color, place, low }]) => {
        const node = t.rarityMeter.extremes[key];
        if (!node) throw new Error(`No rarity meter extreme ${key}`);
        return { formula: `${own(node).name} ${low ? "<=" : ">="} ${node[low ? "p0_1" : "p99_9"].name}`, type: "signal", label: event, color, place };
      }),
    ]);
  }
  /** The price against the awake capitalized price and the true market mean, and the share of days it closed at or
   * above each. */
  function timeAboveChart() {
    const lines = /** @type {[any, string, string][]} */ ([[ct.cohorts.all.awake.costBasis.byDollar.capitalizedPrice, "Awake capitalized price", "orange"], [ct.prices.trueMarketMean.value, "True market mean", "sky"]]);
    return chart("Time above", "Time above the awake capitalized price and true market mean", [
      price(),
      ...lines.map(([node, label, color]) => series(node, label, { color })),
      ...lines.map(([node, label, color]) => ({ formula: `100 * sma(${t.price.close.name} >= ${node.name})`, label: `Share of time at or above the ${label.toLowerCase()}`, format: "percent", color, pane: 2 })),
    ]);
  }
  /**
   * OP_RETURN's charts (all of them, a protocol's, a policy's), each titled by `titled` from what it shows ("outputs"
   * as "Runes OP_RETURN outputs", "OP_RETURN outputs: Oversized"); all of them, with their share of all outputs
   * (`outputShare`, the outputs' by type), as that type's own chart is, and the value sent to them (`value`).
   * @param {any} node @param {(what: string) => string} titled @param {any} [outputShare] @param {any} [value]
   */
  function opReturnCharts(node, titled, outputShare, value) {
    return [
      ...(outputShare ? counted(node.outputCount, "Outputs", titled("outputs"), outputShare) : flow(node.outputCount, "Outputs", titled("outputs"))),
      ...(value ? flow(value, "Value", "Value sent to OP_RETURN outputs") : []),
      ...(node.txCount ? counted(node.txCount, "Transactions", titled("transactions"), node.txShare) : []),
      ...flow(node.dataBytes, "Data", titled("data")),
      ...(node.dataBytes?.chainShare ? [chart("Data share", titled("data's share of block bytes"), sharesOf(node.dataBytes.chainShare, "Share"))] : []),
      // (Their transactions' whole vsize, summed over each window: not the data's, nor a typical transaction's.)
      ...flow(node.txVsize, "Transaction vsize", titled("transactions' vsize")),
      ...flow(node.fees, "Transaction fees", titled("transactions' fees")),
      ...(node.fees?.chainShare ? [chart("Fee share", titled("transactions' share of all fees"), sharesOf(node.fees.chainShare, "Share"))] : []),
    ];
  }
  /** An address group's charts (every address, or one type's). @param {any} a @param {string} who */
  function addressCharts(a, who) {
    const of = (/** @type {string} */ what) => `${what}: ${who}`;
    // (Whose coins their shares divide: these addresses', all or one type's.)
    const whose = `${who.replace(/^All /, "all ")}'`;
    /**
     * Supply held by some of the addresses (`holders`, "Reused addresses"): its amount, then its share of all these
     * addresses' coins, a chart of its own. @param {any} supply @param {string} holders @returns {ChartNode[]}
     */
    const heldSupply = (supply, holders) => [
      chart("Supply", of(`Supply in ${holders.toLowerCase()}`), amount(supply, "Supply")),
      chart("Supply share", `Share of ${whose} coins in ${holders.toLowerCase()}`, [series(supply.share, "Share", { color: "sky" })]),
    ];
    return [
      // (Supply over funded addresses: empty ones aren't counted.)
      a.avgBalance && chart("Average funded balance", of("Average balance of funded addresses"), amount(a.avgBalance, "Average balance"), { log: [1, 2] }),
      section("Count", [
        chart("Funded, empty and total", of("Address count"), [series(own(a.funded), "Funded", { color: "orange" }), series(a.empty, "Empty", { color: "gray" }), series(a.total, "Total", { color: "sky" })], { log: [1] }),
        ...change(a.funded.delta, "Change", of("Funded address change")),
        ...flow(a.new, "New", of("New addresses")),
        // (Each block's distinct addresses, averaged over a window: not the distinct addresses of the window.)
        chart("Active per block", of("Addresses active per block"), entries(a.activity).flatMap(([key, node], i, all) => windows(node.avg, words(key), { show: key === "active" ? ["_1m"] : [], extra: { color: ramp(all.length)[i] } }))),
      ]),
      ...[["reused", "Reused"], ["respent", "Respent"]].filter(([key]) => a[key]).map(([key, label]) => section(label, [
        chart("Count", of(`${label} addresses`), [series(a[key].count.funded, "Funded", { color: "orange" }), series(a[key].count.total, "Total", { color: "sky" })], { log: [1] }),
        ...heldSupply(a[key].supply, `${label} addresses`),
        ...flow(a[key].events.outputs.count, "Outputs to them", of(`Outputs to ${label.toLowerCase()} addresses`)),
        chart("Output share", of(`Share of outputs to ${label.toLowerCase()} addresses`), sharesOf(a[key].events.outputs.share, "Share")),
        ...flow(a[key].events.inputs.count, "Inputs from them", of(`Inputs from ${label.toLowerCase()} addresses`)),
        chart("Input share", of(`Share of inputs from ${label.toLowerCase()} addresses`), sharesOf(a[key].events.inputs.share, "Share")),
        chart("Active per block", of(`${label} addresses active per block`), windows(a[key].events.active.count.avg, "")),
        chart("Active share", of(`Share of active addresses that are ${label.toLowerCase()}`), sharesOf(a[key].events.active.share, "Share")),
      ])),
      a.exposed && section("Exposed", [
        // (Their public key or spending script seen on chain: P2PK and P2TR once funded, the hashed types once spent.)
        chart("Count", of("Addresses with exposed keys or scripts"), [series(a.exposed.count.funded, "Funded", { color: "orange" }), series(a.exposed.count.total, "Total", { color: "sky" })], { log: [1] }),
        ...heldSupply(a.exposed.supply, "Addresses with exposed keys or scripts"),
      ]),
      ...(a.supply ? cohort({ supply: a.supply, capital: a.capital }, who) : []),
    ].filter(Boolean).map((item) => lift(/** @type {SectionNode} */ (item)));
  }
  /** Pools: their shares compared, then each pool, by name. */
  function poolCharts() {
    const list = members(t.pools).filter(([, node]) => node.blockCount).map(([key, node]) => /** @type {[string, any, string]} */ ([key, node, client.POOL_SLUG_TO_NAME[key.replace(/^_/, "")] ?? key])).sort((a, b) => a[2].localeCompare(b[2]));
    // (Those mining most of the blocks in recent years: compared, and first; the others after, by name.)
    const MAJOR = ["foundryusa", "antpool", "viabtc", "f2pool", "marapool", "spiderpool", "binancepool", "luxor", "secpool", "braiinspool", "ocean", "poolin", "btccom", "unknown"];
    const major = MAJOR.flatMap((key) => list.filter(([k]) => k.replace(/^_/, "") === key)), others = list.filter((pool) => !major.includes(pool)), colors = ramp(major.length);
    /** A pool's charts. @param {[string, any, string]} pool */
    const pool = ([, node, name]) => section(name, [
      ...counted(node.blockCount, "Blocks mined", `Blocks mined: ${name}`, node.blockShare),
      ...flow(node.coinbase, "Coinbase", `Coinbase rewards: ${name}`),
      chart("Fees per block", `Fees per block: ${name}`, [...windows(node.fees.avg, "Bitcoin", { pick: btc }), ...windows(node.fees.avg, "Dollars", { pick: usd, pane: 2 })], { log: [1, 2] }),
      chart("Hash rate", `Hash rate: ${name}`, windows(node.hashrate, ""), { log: [1] }),
    ]);
    return [
      section("Compare", [
        // (The others together, what the rest leaves: the whole accounted for.)
        chart("Share, 1 month", "Mining pools' share of blocks, 1 month", [
          ...major.map(([, node, name], i) => series(node.blockShare._1m, name, { color: colors[i], type: "area" })),
          { formula: `100 - (${major.map(([, node]) => node.blockShare._1m.name).join(" + ")})`, label: "Other pools", format: "percent", color: "gray", type: "area" },
        ], { stack: { 1: "stack" } }),
        chart("Hash rate, 1 month", "Mining pools' hash rate, 1 month", major.map(([, node, name], i) => series(node.hashrate._1m, name, { color: colors[i] })), { log: [1] }),
      ]),
      ...major.map(pool),
      section("Other pools", others.map(pool)),
    ];
  }
  /** Weighted coins (awake and dormant, mobile and immobile) for a holder group: supply, capital, cost basis. @param {any} on @param {any} off @param {string} who @param {string} onLabel @param {string} offLabel */
  function weighted(on, off, who, onLabel, offLabel) {
    return [
      chart("Supply", `Supply: ${who}`, [series(on.supply.total.btc, onLabel, { color: "orange", type: "area" }), series(btc(off.supply), offLabel, { color: "gray", type: "area" }), series(on.supply.total.usd, `${onLabel}, value`, { color: "green", type: "area", pane: 2 }), series(usd(off.supply), `${offLabel}, value`, { color: "gray", type: "area", pane: 2 })], { stack: { 1: "stack", 2: "stack" } }),
      chart("Share in loss", `Share in loss: ${who}`, [series(on.supply.inLoss.share, `${onLabel}, share in loss`, { color: "red" })]),
      chart("Realized cap", `Realized cap: ${who}`, [series(on.capital.realizedCap, "Realized cap", { color: "orange" })], { log: [1] }),
      ...cohort({ costBasis: on.costBasis }, who),
    ];
  }
  /**
   * Weighted coins (`on`, "awake"; the rest `off`, "dormant") of each holder group, in the holder groups' sets: compared,
   * then each group's. @param {any} cohorts @param {string} on @param {string} off @param {string} onLabel @param {string} offLabel
   */
  function weightedSets(cohorts, on, off, onLabel, offLabel) {
    const naming = (/** @type {string} */ key) => ({ name: holderName(key).name, who: holderName(key).of(holderName(on).name) });
    return holderSets(entries(cohorts).map(([key, c]) => [key, c[on]]), (members, by) => family(members, naming, { by: `${by}, ${holderName(on).name.toLowerCase()}`, charts: (key, c, who) => weighted(c, cohorts[key][off], who, onLabel, offLabel) }));
  }
  /** Cointime by age band: coin days created and consumed, wakefulness, awake and dormant supply. @param {any} ranges */
  function cointimeBands(ranges) {
    const list = entries(ranges), name = (/** @type {string} */ key) => named(client.AGE_RANGE_NAMES, key).replace(/ old$/, ""), colors = ramp(list.length);
    return [
      section("Compare", [
        chart("Wakefulness", "Wakefulness by age band", list.map(([key, node], i) => series(node.wakefulness, name(key), { color: colors[i] }))),
        ...parts(["Awake supply", "Awake supply by age band"], ["Awake supply share", "Awake supply share by age band"], list.map(([key, node], i) => series(node.awake.supply.btc, name(key), { color: colors[i] }))),
        ...parts(["Dormant supply", "Dormant supply by age band"], ["Dormant supply share", "Dormant supply share by age band"], list.map(([key, node], i) => series(node.dormant.supply.btc, name(key), { color: colors[i] }))),
      ]),
      ...list.map(([key, node]) => section(name(key), [
        // (The coin days spent outputs had built up while in this band, whatever their age when spent.)
        ...flow(node.coindaysConsumed, "Coin days consumed", `Coin days consumed from the ${name(key).toLowerCase()} band`),
        chart("Wakefulness", `Wakefulness: UTXOs ${name(key).toLowerCase()} old`, [series(node.wakefulness, "Wakefulness", { color: "orange" }), series(node.awakeToDormant, "Awake to dormant", { color: "sky", pane: 2 })]),
        chart("Awake and dormant", `Awake and dormant supply: UTXOs ${name(key).toLowerCase()} old`, [series(node.awake.supply.btc, "Awake", { color: "orange", type: "area" }), series(node.dormant.supply.btc, "Dormant", { color: "gray", type: "area" }), series(node.awake.supply.usd, "Awake, value", { color: "green", type: "area", pane: 2 }), series(node.dormant.supply.usd, "Dormant, value", { color: "gray", type: "area", pane: 2 })], { stack: { 1: "stack", 2: "stack" } }),
      ])),
    ];
  }
  /** Coinflow by age band: spending rate, exposure, mobility, mobile and immobile supply. @param {any} ranges */
  function coinflowBands(ranges) {
    const list = entries(ranges), name = (/** @type {string} */ key) => named(client.AGE_RANGE_NAMES, key).replace(/ old$/, ""), colors = ramp(list.length);
    return [
      section("Compare", [
        chart("Spending rate", "Spending rate by age band", list.map(([key, node], i) => series(node.spendingRate, name(key), { color: colors[i] })), { log: [1] }),
        chart("Spending exposure", "Spending exposure by age band", list.map(([key, node], i) => series(node.spendingExposure, name(key), { color: colors[i] }))),
        chart("Mobility", "Mobility by age band", list.map(([key, node], i) => series(node.mobility, name(key), { color: colors[i] }))),
        ...parts(["Mobile supply", "Mobile supply by age band"], ["Mobile supply share", "Mobile supply share by age band"], list.map(([key, node], i) => series(node.mobile.supply.btc, name(key), { color: colors[i] }))),
        ...parts(["Immobile supply", "Immobile supply by age band"], ["Immobile supply share", "Immobile supply share by age band"], list.map(([key, node], i) => series(node.immobile.supply.btc, name(key), { color: colors[i] }))),
      ]),
      ...list.map(([key, node]) => section(name(key), [
        chart("Spending rate", `Spending rate: UTXOs ${name(key).toLowerCase()} old`, [series(node.spendingRate, "Spending rate", { color: "orange" })]),
        chart("Spending exposure", `Spending exposure: UTXOs ${name(key).toLowerCase()} old`, [series(node.spendingExposure, "Spending exposure", { color: "sky" })]),
        chart("Mobility", `Mobility: UTXOs ${name(key).toLowerCase()} old`, [series(node.mobility, "Mobility", { color: "violet" })]),
        chart("Mobile and immobile", `Mobile and immobile supply: UTXOs ${name(key).toLowerCase()} old`, [series(node.mobile.supply.btc, "Mobile", { color: "orange", type: "area" }), series(node.immobile.supply.btc, "Immobile", { color: "gray", type: "area" }), series(node.mobile.supply.usd, "Mobile, value", { color: "green", type: "area", pane: 2 }), series(node.immobile.supply.usd, "Immobile, value", { color: "gray", type: "area", pane: 2 })], { stack: { 1: "stack", 2: "stack" } }),
      ])),
    ];
  }

  // ── Flattened: each chart with its place ──────────────────────────────────

  /**
   * The price on top of a chart: its candles first, in the first pane (a pane with the price holds only prices: a cap
   * isn't one). A chart without them has them in a pane of their own above its panes, each one down: shown, short, its
   * own first pane as tall as a first one is; about the chain's workings, or a chart's `priceHidden` (`hidden`), hidden
   * there (a strip of its legend's, a click away).
   * @param {Sketch} c @param {boolean} hidden @returns {Sketch}
   */
  const priced = (c, hidden) => {
    const at = c.layers.findIndex((layer) => layer.series === t.price.ohlc.name);
    if (at === 0 && (c.layers[0].pane ?? 1) === 1) return c;
    // (Layers are named by their place, A first: a chart that names one would be read wrong with another before it.)
    if (at > 0 || c.layers.some((layer) => layer.cloud || /\b[A-Z]{1,2}\b/.test(layer.formula ?? ""))) throw new Error(`A preset names its layers by place without the price first: ${c.layers.map((layer) => layer.label).join(", ")}`);
    const [first, ...rest] = c.panes ?? [{ mode: 0 }];
    return {
      ...c,
      layers: [{ ...price(), ...(hidden ? { hidden: true } : {}) }, ...c.layers.map((layer) => ({ ...layer, pane: (layer.pane ?? 1) + 1 }))],
      panes: [{ mode: 1, size: 1.4 }, { ...first, size: first.size ?? 3 }, ...rest],
    };
  };

  /**
   * Two panes shown as a preset opens (the price's and its subject's, else its first two): a pane past them, another
   * form of the same subject (its dollars, its share, its percent), starts folded, its layers hidden (its legend's
   * strip, a click away). More than three panes is more than one subject: each a preset of its own.
   * @param {Sketch} c @param {string[]} path @returns {Sketch}
   */
  const twoShown = (c, path) => {
    const panes = Math.max(...c.layers.map((layer) => layer.pane ?? 1));
    if (panes > 3) throw new Error(`A preset with ${panes} panes, three at most (one subject a preset): ${path.join(" / ")}`);
    const shown = [...new Set(c.layers.filter((layer) => !layer.hidden).map((layer) => layer.pane ?? 1))].sort((x, y) => x - y).slice(0, 2);
    return { ...c, layers: c.layers.map((layer) => (layer.hidden || shown.includes(layer.pane ?? 1) ? layer : { ...layer, hidden: true })) };
  };

  /** @type {Preset[]} */
  const presets = [];
  const walk = (/** @type {(ChartNode | SectionNode)[]} */ items, /** @type {string[]} */ path, workings = false) => {
    for (const item of items) {
      if ("items" in item) walk(item.items, [...path, item.name], workings || Boolean(item.workings));
      else presets.push({ p: [...path, item.name], t: item.title, c: twoShown(priced(item.c, workings || Boolean(item.priceHidden)), [...path, item.name]) });
    }
  };
  walk(tree, []);
  return presets;
}

