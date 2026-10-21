// @ts-check
/**
 * Studio's presets: its ready-made charts, in a tree of its own, made from a client's series tree (this repo's, so a
 * path it doesn't have fails where it's written), and the chain's facts (what has happened: no charts of a pool that
 * never found a block, or of what's still to come). Every other series Studio can chart (by date) is in one of them; the
 * build checks that (build.mjs) and puts them in the page; the dev page makes them as it loads.
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
 * @typedef {{ layers: Layer[], panes?: Record<string, any>[], range?: string }} Sketch
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
 * ("realizedProfit24h" "Realized profit, 24h").
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
   * @param {{ log?: number[], linear?: number[], stack?: Record<number, string>, sizes?: number[], range?: string, priceHidden?: boolean }} [options]
   * @returns {ChartNode}
   */
  const chart = (name, title, layers, options = {}) => {
    const count = Math.max(1, ...layers.map((layer) => layer.pane ?? 1));
    const panes = Array.from({ length: count }, (_, i) => {
      const n = i + 1, withPrice = layers.some((layer) => (layer.pane ?? 1) === n && layer.type === "candles");
      const log = options.log?.includes(n) || (withPrice && !options.linear?.includes(n));
      return { mode: log ? 1 : 0, ...(options.stack?.[n] ? { stack: options.stack[n] } : {}), ...(options.sizes?.[i] ? { size: options.sizes[i] } : {}) };
    });
    return { name, title, c: { layers, panes, ...(options.range ? { range: options.range } : {}) }, ...(options.priceHidden ? { priceHidden: true } : {}) };
  };
  /**
   * @param {string} name @param {(ChartNode | SectionNode | null | undefined | false)[]} items
   * @param {{ workings?: boolean }} [options] @returns {SectionNode}
   */
  const section = (name, items, options = {}) => ({ name, items: /** @type {(ChartNode | SectionNode)[]} */ (items.filter(Boolean)), ...(options.workings ? { workings: true } : {}) });

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
  /** A series' node, or its `block` (a node with its own series under `block`, beside others). @param {any} n */
  const own = (n) => (isLeaf(n) ? n : n?.block);
  /** A baseline at `base`: green above, red below (a ratio's base 1, a change's 0). @param {number} [base] @returns {Layer} */
  const around = (base = 0) => ({ type: "baseline", color: "green", colorBelow: "red", ...(base ? { base } : {}) });
  const baseline = around(0);

  /**
   * A flow (an amount per block: summed over windows, `sum`, and since the start, `cumulative`) as charts: its sums,
   * the bitcoin's dollars below; since the start, a chart of its own (in a pane below its sums, when it's a plain one).
   * Since the start, its title says so after the measure, before whose it is ("Rewards, all time: Foundry USA").
   * @param {any} node @param {string} name @param {string} title @param {{ extra?: Layer }} [options]
   * @returns {ChartNode[]}
   */
  const flow = (node, name, title, options = {}) => {
    if (!node) return [];
    const titled = (/** @type {string} */ after) => title.replace(/^([^:]*)(:|$)/, `$1${after}$2`);
    const sums = node.sum ?? node, extra = options.extra ?? {};
    const pair = windowsOf(sums).some(([, value]) => value.btc);
    if (!pair) {
      const layers = [...windows(sums, "", { extra }), ...(byDate(node.cumulative) ? [series(node.cumulative, "All time", { pane: 2, color: "gray" })] : [])];
      return layers.length ? [chart(name, titled(""), layers)] : [];
    }
    const charts = [chart(name, titled(""), [...windows(sums, "Bitcoin", { pick: btc, extra }), ...windows(sums, "Dollars", { pick: usd, pane: 2, extra })])];
    if (node.cumulative && byDate(btc(node.cumulative))) charts.push(chart(`${name}, all time`, titled(", all time"), [series(btc(node.cumulative), "Bitcoin", { color: "orange" }), ...(byDate(usd(node.cumulative)) ? [series(usd(node.cumulative), "Dollars", { color: "green", pane: 2 })] : [])]));
    return charts;
  };
  /**
   * A count (a kind of transaction's, a type of output's…): its windows, its share of the whole below (the `share`
   * given, else its own), and since the start at the bottom.
   * @param {any} node @param {string} name @param {string} title @param {any} [share]
   */
  const counted = (node, name, title, share = node.share) =>
    chart(name, title, [...windows(node.sum, ""), ...sharesOf(share, "Share", { pane: 2 }), series(node.cumulative, "All time", { pane: 3, color: "gray" })]);
  /**
   * Kinds of transactions, which overlap: their shares of all transactions compared, then each one's count.
   * @param {[string, any][]} list @param {(key: string) => string} name @param {(key: string) => string} title @param {string} compared
   */
  const txKinds = (list, name, title, compared) => [
    chart("Compare", `${compared}, 1 month`, list.map(([key, node], i) => series(node.share._1m, name(key), { color: ramp(list.length)[i] }))),
    ...list.map(([key, node]) => counted(node, name(key), title(key))),
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
   * `whole`: the cohort is all of what its shares divide (a family's `all`), its shares a flat 100%, left out.
   * @param {any} c @param {string} who @returns {(ChartNode | SectionNode)[]}
   */
  const cohort = (c, who, whole = false) => {
    const of = (/** @type {string} */ what) => `${what}: ${who}`;
    const supply = c.supply, capital = c.capital, utxos = c.utxos, activity = c.activity, realized = c.realized, unrealized = c.unrealized, cost = c.costBasis;
    const inProfit = supply?.inProfit, inLoss = supply?.inLoss;
    return [
      supply && section("Supply", [
        chart("Total", of("Supply"), amount(supply.total, "Supply")),
        inProfit && chart("In profit and loss", of("Supply in profit and loss"), [
          series(btc(inProfit), "In profit", { color: "green", type: "area" }), series(btc(inLoss), "In loss", { color: "red", type: "area" }),
          ...(usd(inProfit) ? [series(usd(inProfit), "In profit, value", { color: "green", type: "area", pane: 2 }), series(usd(inLoss), "In loss, value", { color: "red", type: "area", pane: 2 })] : []),
        ], { stack: { 1: "stack", 2: "stack" } }),
        inProfit?.share && chart("Share in profit", of("Share of supply in profit"), [series(inProfit.share, "In profit", { color: "green", levels: [{ value: 50, color: "gray", dash: 3 }] }), series(inLoss.share, "In loss", { color: "red" })]),
        supply.share && !whole && chart("Share of all coins", of("Share of all coins and capital"), [series(supply.share, "Of all coins", { color: "orange" }), ...(capital?.share ? [series(capital.share, "Of all capital", { color: "green" })] : [])]),
        ...change(supply.delta, "Change", of("Supply change")),
        ...flow(supply.matured, "Matured", of("Supply aging out")),
      ]),
      c.funded && section("Addresses", [
        chart("Count", of("Address count"), [series(own(c.funded), "Addresses", { color: "sky" })], { log: [1] }),
        ...change(c.funded.delta, "Change", of("Address count change")),
      ]),
      capital && section("Capital", [
        chart("Total", of("Realized cap"), [
          series(capital.realizedCap, "Realized cap", { color: "orange" }),
          ...(capital.inProfit ? [series(own(capital.inProfit), "In profit", { color: "green", hidden: true }), series(own(capital.inLoss), "In loss", { color: "red", hidden: true })] : []),
          ...(capital.inProfit?.share ? [series(capital.inProfit.share, "Share in profit", { color: "green", pane: 2 }), series(capital.inLoss.share, "Share in loss", { color: "red", pane: 2 })] : []),
        ], { log: [1] }),
        ...change(capital.delta, "Change", of("Capital change")),
      ]),
      utxos && section("UTXOs", [
        utxos.count && chart("Count", of("UTXO count"), [series(own(utxos.count), "UTXOs", { color: "sky" })], { log: [1] }),
        ...change(utxos.count?.delta, "Change", of("UTXO count change")),
        ...flow(utxos.spentCount, "Spent", of("UTXOs spent")),
      ]),
      activity && section("Activity", [
        ...flow(activity.transferVolume, "Transfer volume", of("Transfer volume")),
        ...(activity.transferVolume?.inProfit ? [
          chart("Volume in profit and loss", of("Transfer volume in profit and loss"), [
            ...windows(activity.transferVolume.inProfit.sum, "In profit", { pick: btc, extra: { color: "green" } }), ...windows(activity.transferVolume.inLoss.sum, "In loss", { pick: btc, extra: { color: "red" } }),
            ...windows(activity.transferVolume.inProfit.sum, "In profit, value", { pick: usd, pane: 2, extra: { color: "green" } }), ...windows(activity.transferVolume.inLoss.sum, "In loss, value", { pick: usd, pane: 2, extra: { color: "red" } }),
          ]),
          chart("Volume in profit and loss, all time", of("Transfer volume in profit and loss, all time"), [
            series(btc(activity.transferVolume.inProfit.cumulative), "In profit", { color: "green" }), series(btc(activity.transferVolume.inLoss.cumulative), "In loss", { color: "red" }),
            series(usd(activity.transferVolume.inProfit.cumulative), "In profit, value", { color: "green", pane: 2 }), series(usd(activity.transferVolume.inLoss.cumulative), "In loss, value", { color: "red", pane: 2 }),
          ]),
        ] : []),
        ...flow(activity.coindaysDestroyed, "Coin days destroyed", of("Coin days destroyed")),
        ...flow(activity.coindaysCreated, "Coin days created", of("Coin days created")),
        activity.dormancy && chart("Dormancy", of("Dormancy"), windows(activity.dormancy, "")),
      ]),
      realized && section("Realized", [
        realized.profit && chart("Profit and loss", of("Realized profit and loss"), [
          ...windows(realized.profit.sum, "Profit", { extra: { color: "green" } }), ...windows(realized.loss.sum, "Loss", { extra: { color: "red" } }),
          ...(realized.netPnl ? windows(realized.netPnl.sum, "Net", { pane: 2, around: 0 }) : []),
        ]),
        realized.profit && chart("Profit and loss, all time", of("Realized profit and loss, all time"), [
          series(realized.profit.cumulative, "Profit", { color: "green" }), series(realized.loss.cumulative, "Loss", { color: "red" }),
          ...(realized.netPnl ? [series(realized.netPnl.cumulative, "Net", { pane: 2, ...baseline })] : []),
        ]),
        ...flow(realized.grossPnl, "Gross profit and loss", of("Gross realized profit and loss")),
        realized.profitToLoss && chart("Profit to loss", of("Realized profit to loss"), windows(realized.profitToLoss, "", { around: 1 }), { log: [1] }),
        ...flow(realized.valueDestroyed, "Value destroyed", of("Value destroyed")),
        ...flow(realized.peakRegret, "Peak regret", of("Realized peak regret")),
        realized.sopr && chart("SOPR", of("SOPR"), windows(realized.sopr, "", { show: ["_1w"], around: 1 })),
        realized.adjustedSopr && chart("Adjusted SOPR", of("Adjusted SOPR"), [
          ...windows(realized.adjustedSopr, "", { show: ["_1w"], around: 1 }),
          ...windows(realized.adjustedSopr.valueCreated.sum, "Value created", { pane: 2, extra: { color: "green" } }),
          ...windows(realized.adjustedSopr.valueDestroyed.sum, "Value destroyed", { pane: 2, extra: { color: "red" } }),
        ]),
        realized.adjustedSopr && chart("Adjusted value, all time", of("Adjusted value created and destroyed, all time"), [
          series(realized.adjustedSopr.valueCreated.cumulative, "Created", { color: "green" }), series(realized.adjustedSopr.valueDestroyed.cumulative, "Destroyed", { color: "red" }),
        ]),
        realized.sellSideRiskRatio && chart("Sell-side risk", of("Sell-side risk ratio"), windows(realized.sellSideRiskRatio, ""), { log: [1] }),
      ]),
      unrealized && section("Unrealized", [
        chart("Profit and loss", of("Unrealized profit and loss"), [
          series(unrealized.profit, "Profit", { color: "green" }), series(unrealized.loss, "Loss", { color: "red" }),
          ...(unrealized.grossPnl ? [series(unrealized.grossPnl, "Gross", { color: "gray", hidden: true })] : []),
          series(unrealized.netPnl, "Net", { pane: 2, ...baseline }),
        ]),
        unrealized.nupl && chart("NUPL", of("Net unrealized profit and loss (NUPL)"), [series(unrealized.nupl, "NUPL", { ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
      ]),
      cost && section("Cost basis", [
        chart("Realized price", of("Realized price"), [
          price(), series(cost.byCoin.realizedPrice, "Realized price", { color: "orange" }),
          ...(cost.byCoin.medianRealizedPrice ? [series(cost.byCoin.medianRealizedPrice, "Median realized price", { color: "yellow", hidden: true })] : []),
          ...(cost.min ? [series(cost.min, "Lowest", { color: "red", dash: 3, hidden: true }), series(cost.max, "Highest", { color: "green", dash: 3, hidden: true })] : []),
          ...(cost.byCoin.avg?.mvrv ? [series(cost.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1) })] : []),
        ], { log: [2] }),
        cost.byDollar && chart("Capitalized price", of("Capitalized price"), [
          price(), series(cost.byDollar.capitalizedPrice, "Capitalized price", { color: "orange" }),
          ...(cost.byDollar.medianCapitalizedPrice ? [series(cost.byDollar.medianCapitalizedPrice, "Median capitalized price", { color: "yellow", hidden: true })] : []),
          series(cost.byDollar.avg.ratio, "Price ratio", { pane: 2, ...around(1) }),
        ], { log: [2] }),
        ...percentiles(cost.byCoin, "Distribution", of("Cost basis distribution, by coin")),
        ...(cost.byDollar ? percentiles(cost.byDollar, "Distribution by dollar", of("Cost basis distribution, by dollar")) : []),
        cost.supplyDensity && chart("Supply density", of("Supply near the price"), [
          series(cost.supplyDensity.total, "Supply", { color: "orange" }), series(cost.supplyDensity.inProfit, "In profit", { color: "green", hidden: true }), series(cost.supplyDensity.inLoss, "In loss", { color: "red", hidden: true }),
        ]),
        cost.capitalDensity && chart("Capital density", of("Capital near the price"), [
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
      vaultedPrice: [ct.prices.vaulted.block, ct.prices.vaulted.ratio],
      trueMarketMean: [ct.prices.trueMarketMean.block, ct.prices.trueMarketMean.ratio],
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
    realizedProfit24h: { label: "Realized profit, 24h", event: "Profit taking", color: "green", place: "above" },
    realizedLoss24h: { label: "Realized loss, 24h", event: "Capitulation", color: "red", place: "below" },
    realizedPeakRegret24h: { label: "Realized peak regret, 24h", event: "Peak regret", color: "pink", place: "below" },
    sellSideRiskRatio24h: { label: "Sell-side risk ratio, 24h", event: "Seller exhaustion", color: "yellow", place: "above", low: true },
    supplyInLoss: { label: "Supply in loss", event: "Supply in loss", color: "orange", place: "below" },
  };
  /**
   * A section of one chart, that chart in its place, by the section's name (no folder holding one thing: a capital's
   * only chart, "Total" in its section, is "Capital"). @param {SectionNode | ChartNode} item
   */
  const lift = (item) => ("items" in item && item.items.length === 1 && !("items" in item.items[0]) ? { ...item.items[0], name: item.name } : item);

  // ── Families: comparisons first, then each member ───────────────────────────

  /**
   * A family of cohorts: its comparisons (each a chart of every member), then each member's charts.
   * @param {[string, any][]} members its members' keys and nodes
   * @param {(key: string) => { name: string, who: string }} naming each member's name (in the tree) and title (in charts)
   * @param {{ by: string, disjoint?: boolean }} options `by`: what a comparison's title says it's by ("age band",
   * "holder group, short-term"); `disjoint`: the members split the whole (bands), so their amounts stack, as shares
   * @returns {(ChartNode | SectionNode)[]}
   */
  const family = (members, naming, options) => {
    const colors = ramp(members.length);
    /** Every member's series (`pick` takes it from the member), as a chart's layers. @param {(c: any) => any} pick @param {Layer} [extra] */
    // (A family's `all`, where it reads as the others' reference, gray and dashed.)
    const each = (pick, extra = {}, reference = false) => members.flatMap(([key, c], i) => (byDate(pick(c)) ? [series(pick(c), naming(key).name, { color: colors[i], ...extra, ...(reference && key === "all" ? { color: "gray", dash: 3 } : {}) })] : []));
    /**
     * A comparison: every member's `pick`, as lines; an amount (`additive`) of members that split the whole, as their
     * shares of it, stacked ("Supply share"). Its title says by what, its window last ("…by age band, 1 month").
     * @param {string} name @param {(c: any) => any} pick @param {{ price?: boolean, log?: boolean, additive?: boolean, level?: number, reference?: boolean }} [o]
     */
    const compare = (name, pick, o = {}) => {
      // (Read against a reference, its level across: a ratio's 1, a net's 0; the members stay lines, each its color.)
      const layers = each(pick, o.level === undefined ? {} : { levels: [{ value: o.level, color: "gray", dash: 3 }] }, o.reference);
      if (!layers.length) return null;
      const [, measure, window = ""] = /** @type {RegExpExecArray} */ (/^(.*?)(, 1 \w+)?$/.exec(name)), shares = Boolean(options.disjoint && o.additive);
      const named = shares ? `${measure} share${window}` : name, title = `${shares ? `${measure} share` : measure} by ${options.by}${window}`;
      if (o.price) return chart(named, title, [price(), ...layers]);
      if (shares) return chart(named, title, layers.map((layer) => ({ ...layer, type: "area" })), { stack: { 1: "percent" } });
      return chart(named, title, layers, o.log ? { log: [1] } : {});
    };
    const comparisons = [
      compare("Supply", (c) => btc(c.supply?.total), { log: true, additive: true }),
      compare("Realized cap", (c) => c.capital?.realizedCap, { log: true, additive: true }),
      // (Overlapping members' shares as lines, all coins' 100% their reference; disjoint ones stack above.)
      !options.disjoint && compare("Share of all coins", (c) => c.supply?.share, { reference: true }),
      !options.disjoint && compare("Share of all capital", (c) => c.capital?.share, { reference: true }),
      compare("Realized price", (c) => c.costBasis?.byCoin?.realizedPrice, { price: true }),
      compare("MVRV", (c) => c.costBasis?.byCoin?.avg?.mvrv, { level: 1, log: true }),
      compare("Share in profit", (c) => c.supply?.inProfit?.share),
      compare("NUPL", (c) => c.unrealized?.nupl, { level: 0 }),
      compare("UTXO count", (c) => own(c.utxos?.count), { log: true, additive: true }),
      compare("Address count", (c) => own(c.funded), { log: true, additive: true }),
      compare("Transfer volume, 1 month", (c) => btc(c.activity?.transferVolume?.sum?._1m), { additive: true }),
      compare("Coin days destroyed, 1 month", (c) => c.activity?.coindaysDestroyed?.sum?._1m, { additive: true }),
      compare("Realized profit, 1 month", (c) => c.realized?.profit?.sum?._1m, { additive: true }),
      compare("Realized loss, 1 month", (c) => c.realized?.loss?.sum?._1m, { additive: true }),
      compare("SOPR, 1 week", (c) => c.realized?.sopr?._1w, { level: 1 }),
    ].filter(Boolean);
    return [
      ...(comparisons.length ? [section("Compare", comparisons)] : []),
      ...members.map(([key, c]) => ((n) => section(n.name, cohort(c, n.who, key === "all")))(naming(key))),
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
  const holderName = (/** @type {string} */ key) => ({ name: HOLDERS[key]?.[0] ?? key, who: HOLDERS[key]?.[1] ?? key, of: (/** @type {string} */ coins) => `${coins}${HOLDERS[key]?.[2] ?? `, ${key}`}` });

  // ── The tree ──────────────────────────────────────────────────────────────

  const m = t.market, sup = t.supply, mining = t.mining, blocks = t.blocks, tx = t.transactions, ct = t.cointime, cf = t.coinflow;

  /** Moving averages (`sma` or `ema`): each over the price with its ratio below, and all of them together. @param {any} node @param {string} kind */
  const averages = (node, kind) => {
    const list = entries(node), colors = ramp(list.length, 200, 330);
    return section(kind, [
      chart("Compare", `${kind}s of the price`, [price(), ...list.map(([key, value], i) => series(own(value), `${windowAdjective(key)} ${kind}`, { color: colors[i], ...(["_50d", "_200d", "_200w", "_1y"].includes(key) ? {} : { hidden: true }) }))]),
      ...list.map(([key, value]) => chart(windowLabel(key), `${windowAdjective(key)} ${kind}`, [
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
        ...windowsOf(m.returns).map(([key, value]) => chart(windowLabel(key), `${windowAdjective(key)} price return`, [price(), series(value, `${windowAdjective(key)} return`, { pane: 2, ...baseline })])),
      ]),
      section("Moving averages", [averages(m.sma, "SMA"), averages(m.ema, "EMA")]),
      section("Range", [
        ...entries(m.range.max).map(([key]) => chart(`${windowAdjective(key)} high and low`, `Price's ${windowAdjective(key)} high and low`, [price(), series(m.range.max[key], "High", { color: "green" }), series(m.range.min[key], "Low", { color: "red" }), { cloud: ["B", "C"], label: "Range", color: "sky", opacity: 0.1 }])),
        chart("True range", "Price's daily true range", [series(m.range.trueRange, "True range", { color: "sky" })], { log: [1] }),
      ]),
      section("Technical", [
        chart("RSI", "Relative strength index of the price", [price(), { formula: "rsi(A)", label: "RSI", pane: 2, color: "violet", levels: [{ value: 30, color: "green", dash: 3 }, { value: 70, color: "red", dash: 3 }] }]),
        chart("MACD", "MACD of the price", [price(), { formula: "macd(A)", label: "MACD", pane: 2, type: "histogram", color: "green", colorBelow: "red" }]),
        chart("Volatility", "Price volatility", [price(), { formula: "volatility(A, 30d)", label: "30 days", pane: 2, color: "sky" }, { formula: "volatility(A, 1y)", label: "1 year", pane: 2, color: "violet" }]),
        chart("Pi cycle", "Pi cycle top", [price(), { formula: "sma(A, 111d)", label: "111-day SMA", color: "green" }, { formula: "2 * sma(A, 350d)", label: "350-day SMA × 2", color: "red" }]),
      ]),
    ]),
    section("Valuation", [
      chart("Market and realized cap", "Market cap and realized cap", [series(t.holders.all.supply.total.usd, "Market cap", { color: "green" }), series(t.holders.all.capital.realizedCap, "Realized cap", { color: "orange" })], { log: [1], priceHidden: true }),
      chart("MVRV", "Market value to realized value (MVRV)", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: "orange" }), series(t.holders.all.costBasis.byCoin.avg.mvrv, "MVRV", { pane: 2, ...around(1), levels: [{ value: 3.5, color: "red", dash: 3 }] })], { log: [2] }),
      chart("NUPL", "Net unrealized profit and loss (NUPL)", [price(), series(t.holders.all.unrealized.nupl, "NUPL", { pane: 2, ...baseline, levels: [0.25, 0.5, 0.75].map((value) => ({ value, color: "gray", dash: 3 })) })]),
      chart("Value anchors", "Price and its on-chain anchors", [price(), series(t.holders.all.costBasis.byCoin.realizedPrice, "Realized price", { color: "orange" }), series(ct.prices.trueMarketMean.block, "True market mean", { color: "sky" }), series(ct.prices.cointime.block, "Cointime price", { color: "violet" }), series(t.holders.all.costBasis.byDollar.capitalizedPrice, "Capitalized price", { color: "green" })]),
      cloudsChart(),
      section("Cointime prices", [
        chart("Compare", "Cointime prices", [price(), ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label], i) => series(ct.prices[key].block, label, { color: ["green", "red", "violet"][i] })), series(ct.prices.trueMarketMean.block, "True market mean", { color: "sky" })]),
        ...[["vaulted", "Vaulted price"], ["active", "Active price"], ["cointime", "Cointime price"]].map(([key, label]) => chart(label, label, [price(), series(ct.prices[key].block, label, { color: "sky" }), series(ct.prices[key].mvrv, "MVRV", { pane: 2, ...around(1) })], { log: [2] })),
        timeAboveChart(),
        chart("True market mean", "True market mean and AVIV", [price(), series(ct.prices.trueMarketMean.block, "True market mean", { color: "sky" }), series(ct.prices.trueMarketMean.aviv, "AVIV", { pane: 2, ...around(1) }), series(ct.prices.trueMarketMean.avivNupl, "AVIV NUPL", { pane: 3, ...baseline })], { log: [2] }),
      ]),
      chart("Cointime caps", "Cointime capitalizations", [...[["thermo", "Thermo cap"], ["investor", "Investor cap"], ["active", "Active cap"], ["vaulted", "Vaulted cap"], ["cointime", "Cointime cap"]].map(([key, label], i) => series(ct.caps[key], label, { color: ["orange", "green", "red", "sky", "violet"][i] })), series(ct.caps.investorness, "Investorness", { pane: 2, color: "green" }), series(ct.caps.producerness, "Producerness", { pane: 2, color: "orange" })], { log: [1] }),
      chart("Reserve risk", "Reserve risk", [price(), series(ct.reserveRisk.block, "Reserve risk", { pane: 2, color: "orange" }), series(ct.reserveRisk.hodlBank, "HODL bank", { pane: 3, color: "sky" })], { log: [2, 3] }),
      ...flow(ct.reserveRisk.vocdd, "Value of coin days destroyed", "Value of coin days destroyed (VOCDD)"),
      section("Bedrock", entries(t.bedrock).map(([mode, node]) => {
        // The Bedrock landscape, top to bottom: sky over the all-time high, water from it to the median cost basis above
        // the floor, soil down to the first floor, bedrock through the floors, the deepest down to the bottom.
        const edges = [
          [m.ath.high, "All-time high"],
          ...entries(node.costBasisAboveFloor).reverse().map(([key, value]) => [value, `${percentileLabel(percentile(key))} cost basis`]),
          ...entries(node.floor).map(([key, value]) => [value, `${percentileLabel(percentile(key))} floor`]),
        ];
        const bands = entries(node.floorSupply), colors = ramp(bands.length, 140, 200);
        return section(sentence(mode[0].toUpperCase() + mode.slice(1)), [
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
          chart("Capital between floors", `Capital of the supply created between Bedrock floors, ${mode}`, entries(node.floorCapital).map(([key, value], i) => series(value, floorBandLabel(key), { color: colors[i], type: "area" })), { stack: { 1: "stack" } }),
          chart("Cost basis above the floor", `Cost basis above the Bedrock floor, ${mode}`, [price(), ...entries(node.costBasisAboveFloor).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 160)[i], width: key === "median" ? 2 : 1 }))]),
          chart("Supply-in-loss thresholds", `Bedrock's supply-in-loss thresholds, ${mode}`, entries(node.supplyInLossShareThreshold).map(([key, value], i, all) => series(value, percentileLabel(percentile(key)), { color: ramp(all.length, 25, 100)[i] }))),
        ]);
      })),
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
      chart("Circulating", "Circulating supply", [series(sup.circulating, "Supply", { color: "orange" }), series(sup.marketCap.block, "Market cap", { color: "green", pane: 2 })], { log: [2] }),
      chart("Inflation", "Inflation rate", [series(sup.inflationRate, "Inflation rate", { color: "orange" }), series(ct.adjusted.inflationRate, "Cointime-adjusted", { color: "violet" })], { log: [1] }),
      ...flow(sup.burned, "Burned", "Bitcoin burned"),
      chart("Velocity", "Velocity", [series(sup.velocity.btc, "In bitcoin", { color: "orange" }), series(sup.velocity.usd, "In dollars", { color: "green" }), series(ct.adjusted.velocity.btc, "Cointime-adjusted, in bitcoin", { color: "violet", hidden: true }), series(ct.adjusted.velocity.usd, "Cointime-adjusted, in dollars", { color: "sky", hidden: true })]),
      ...change(sup.marketCap.delta, "Market cap change", "Market cap change"),
    ]),
    section("Holders", [
      // (The veteran and rookie coins with them: groups of holders too, by when they came in.)
      ...family([...entries(t.holders), ...entries(t.entry)], (key) => (HOLDERS[key] ? holderName(key) : { name: named(client.ENTRY_NAMES, key), who: named(client.ENTRY_NAMES, key) }), { by: "holder group" }),
    ]),
    section("Coin age", [
      section("Age bands", family(entries(t.age.ranges), (key) => ({ name: named(client.AGE_RANGE_NAMES, key).replace(/ old$/, ""), who: `UTXOs ${named(client.AGE_RANGE_NAMES, key).toLowerCase().replace(/^utxos /, "")}` }), { by: "age band", disjoint: true })),
      section("Halving epochs", family(members(t.age.halvingEpochs), (key) => ({ name: `Epoch ${key.slice(1)}`, who: `Coins last moved in halving epoch ${key.slice(1)}` }), { by: "halving epoch last moved", disjoint: true })),
      section("Vintages", family(members(t.age.vintages), (key) => ({ name: key.slice(1), who: `Coins last moved in ${key.slice(1)}` }), { by: "vintage", disjoint: true })),
    ]),
    section("Profitability", entries(t.profitability).map(([key, bands]) => section(holderName(key).name, family(entries(bands), (band) => ({ name: named(client.PROFITABILITY_RANGE_NAMES, band), who: `${holderName(key).who}, ${named(client.PROFITABILITY_RANGE_NAMES, band).toLowerCase()}` }), { by: `profit band, ${holderName(key).who.toLowerCase()}`, disjoint: true })))),
    section("UTXOs", [
      chart("Count", "UTXO count", [series(t.utxoSet.count, "UTXOs", { color: "sky" })], { log: [1] }),
      section("By amount", family(entries(t.utxos.amounts), (key) => ({ name: named(client.AMOUNT_RANGE_NAMES, key), who: `UTXOs of ${named(client.AMOUNT_RANGE_NAMES, key)}` }), { by: "UTXO amount", disjoint: true })),
      section("By type", family(entries(t.utxos.types), (key) => ({ name: client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, who: `${client.SPENDABLE_TYPE_NAMES[key]?.short ?? key} outputs` }), { by: "output type", disjoint: true })),
    ]),
    section("Addresses", [
      ...addressCharts(t.addresses, "All addresses"),
      section("By type", [
        section("Compare", [
          ...[["Funded", (/** @type {any} */ a) => own(a.funded)], ["Total", (/** @type {any} */ a) => a.total], ["New, 1 month", (/** @type {any} */ a) => a.new.sum._1m], ["Active, 1 month", (/** @type {any} */ a) => a.activity.active.avg._1m]].map(([name, pick]) => chart(/** @type {string} */ (name), String(name).replace(/^(\w+)(.*)$/, "$1 addresses by type$2"), entries(t.addresses.types).map(([key, a], i, all) => series(/** @type {(a: any) => any} */ (pick)(a), client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, { color: ramp(all.length)[i] })), { log: [1] })),
        ]),
        ...entries(t.addresses.types).map(([key, a]) => section(client.SPENDABLE_TYPE_NAMES[key]?.short ?? key, addressCharts(a, `${client.SPENDABLE_TYPE_NAMES[key]?.short ?? key} addresses`))),
      ]),
      section("By balance", family(entries(t.addresses.balances), (key) => ({ name: named(client.AMOUNT_RANGE_NAMES, key), who: `Addresses holding ${named(client.AMOUNT_RANGE_NAMES, key)}` }), { by: "address balance", disjoint: true })),
    ]),
    section("Mining", [
      section("Hashing", [
        chart("Hash rate", "Hash rate", [series(own(mining.hashrate.rate), "Hash rate", { color: "orange" }), ...windows(mining.hashrate.rate.sma, "Average", { show: ["_1m"] }), series(mining.hashrate.rate.ath, "All-time high", { color: "green", dash: 3, hidden: true }), series(blocks.difficulty.hashrate, "From difficulty", { color: "gray", hidden: true }), series(mining.hashrate.rate.drawdown, "Drawdown", { pane: 2, color: "red", type: "area" })], { log: [1] }),
        chart("Hash price", "Hash price", [series(own(mining.hashrate.price), "Hash price", { color: "green" }), series(mining.hashrate.price.atl, "All-time low", { color: "red", dash: 3 }), series(mining.hashrate.price.rebound, "Rebound", { pane: 2, color: "green" })], { log: [1] }),
        chart("Hash value", "Hash value", [series(own(mining.hashrate.value), "Hash value", { color: "orange" }), series(mining.hashrate.value.atl, "All-time low", { color: "red", dash: 3 }), series(mining.hashrate.value.rebound, "Rebound", { pane: 2, color: "orange" })], { log: [1] }),
      ]),
      section("Difficulty", [
        chart("History", "Difficulty", [series(blocks.difficulty.block, "Difficulty", { color: "orange" }), series(blocks.difficulty.adjustment, "Adjustment", { pane: 2, type: "histogram", color: "green", colorBelow: "red" })], { log: [1] }),
        chart("Next adjustment", "Until the next difficulty adjustment", [series(blocks.difficulty.blocksToRetarget, "Blocks", { color: "sky" }), series(blocks.difficulty.daysToRetarget, "Days", { color: "orange", pane: 2 }), series(blocks.difficulty.epoch, "Epoch", { color: "gray", pane: 3 })]),
        chart("Next halving", "Until the next halving", [series(blocks.halving.blocksToHalving, "Blocks", { color: "sky" }), series(blocks.halving.daysToHalving, "Days", { color: "orange", pane: 2 }), series(blocks.halving.epoch, "Halving epoch", { color: "gray", pane: 3 })]),
      ]),
      section("Rewards", [
        ...flow(mining.rewards.coinbase, "Coinbase", "Coinbase rewards"),
        ...flow(mining.rewards.subsidy, "Subsidy", "Block subsidy"),
        ...flow(mining.rewards.fees, "Fees", "Transaction fees"),
        ...flow(mining.rewards.unclaimed, "Unclaimed", "Unclaimed rewards"),
        chart("Fee share", "Fees' share of rewards", sharesOf(mining.rewards.fees.share, "Fees")),
        chart("Subsidy share", "Subsidy's share of rewards", sharesOf(mining.rewards.subsidy.share, "Subsidy")),
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
        chart("Height", "Block height", [series(t.mappings.day1.firstHeight, "Height", { color: "orange" })]),
        chart("Interval", "Time between blocks", windows(blocks.interval.avg, "", { extra: { levels: [{ value: 600, color: "gray", dash: 3 }] } })),
        // (Their sums, every block's together, "total"; each block's, its average and percentiles.)
        ...stats(blocks.size, "Size", "Total block size", "Block size"),
        ...stats(blocks.weight, "Weight", "Total block weight", "Block weight"),
      ]),
      section("Transactions", [
        ...stats(tx.count, "Count", "Transactions"),
        chart("Per second", "Transactions per second", windows(tx.perSecond, "")),
        ...flow(tx.volume, "Volume", "Transaction volume"),
        chart("Size", "Transaction size", percentileStats(tx.vsize), { log: [1] }),
        chart("Fee", "Transaction fee", percentileStats(tx.fees.fee), { log: [1] }),
        chart("Fee rate", "Transaction fee rate", percentileStats(tx.fees.effectiveFeeRate), { log: [1] }),
        section("Features", [
          section("Scripts", txKinds(entries(tx.features.scripts), featureName, featureTx, "Share of transactions by script")),
          section("Signatures", txKinds(entries(tx.features.signatures), featureName, featureTx, "Share of transactions by signature feature")),
          ...txKinds(entries(tx.features).filter(([, node]) => node.share), featureName, featureTx, "Share of transactions by feature"),
        ]),
        section("Versions", [
          chart("Compare", "Transactions by version, 1 month", entries(tx.versions).map(([key, node], i, all) => series(node.sum._1m, versionName(key), { color: ramp(all.length)[i], type: "area" })), { stack: { 1: "percent" } }),
          ...entries(tx.versions).map(([key, node]) => counted(node, versionName(key), key === "other" ? "Transactions of other versions" : `${versionName(key)} transactions`)),
        ]),
        section("Patterns", txKinds(entries(tx.patterns), words, (key) => `${words(key)} transactions`, "Share of transactions by pattern")),
        section("CPFP", [counted(tx.fees.cpfpParent, "Parents", "CPFP parent transactions"), counted(tx.fees.cpfpChild, "Children", "CPFP child transactions")]),
        section("Inscriptions", [counted(tx.inscription, "Count", "Inscription transactions"), ...flow(tx.inscription.fees, "Fees", "Inscription fees"), chart("Fee share", "Inscriptions' share of fees", sharesOf(tx.inscription.fees.chainShare, "Share"))]),
        counted(tx.nonstandard, "Nonstandard", "Nonstandard transactions"),
        ...flow(tx.sigopCost, "Signature operations", "Signature operation cost"),
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
        ...flow(t.outputs.opReturnValue, "OP_RETURN value", "Value sent to OP_RETURN outputs"),
        section("By type", byType(t.outputs.types, "outputs", "creating")),
      ]),
    ], { workings: true }),
    section("OP_RETURN", [
      ...opReturnCharts(t.opReturn, (what) => `OP_RETURN ${what}`, t.outputs.types.opReturn.share),
      section("Protocols", [
        section("Compare", [
          chart("Outputs, 1 month", "OP_RETURN outputs by protocol, 1 month", entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.outputCount.sum._1m, protocolName(key), { color: ramp(all.length)[i], type: "area" })), { stack: { 1: "percent" } }),
          chart("Transaction share, 1 month", "Share of transactions by OP_RETURN protocol, 1 month", entries(t.opReturn.protocols).map(([key, node], i, all) => series(node.txShare._1m, protocolName(key), { color: ramp(all.length)[i] }))),
        ]),
        ...entries(t.opReturn.protocols).map(([key, node]) => section(protocolName(key), opReturnCharts(node, (what) => `${protocolName(key)} OP_RETURN ${what}`))),
      ]),
      section("Policies", entries(t.opReturn.policies).map(([key, node]) => section(policyName(key), opReturnCharts(node, (what) => `OP_RETURN ${what}: ${policyName(key)}`)))),
    ], { workings: true }),
    section("Cointime", [
      section("Activity", [
        ...flow(ct.activity.coinblocksCreated, "Coin blocks created", "Coin blocks created"),
        ...flow(ct.activity.coinblocksDestroyed, "Coin blocks destroyed", "Coin blocks destroyed"),
        ...flow(ct.activity.coinblocksStored, "Coin blocks stored", "Coin blocks stored"),
        chart("Liveliness", "Liveliness and vaultedness", [series(ct.activity.liveliness, "Liveliness", { color: "red" }), series(ct.activity.vaultedness, "Vaultedness", { color: "green" }), series(ct.activity.livelinessToVaultedness, "Liveliness to vaultedness", { pane: 2, color: "orange" })]),
        chart("Concurrent liveliness", "Concurrent liveliness", windows(ct.activity.concurrentLiveliness, "")),
      ]),
      chart("Supply", "Vaulted, active and lost supply", [...[["vaulted", "Vaulted"], ["active", "Active"], ["hodledOrLost", "HODLed or lost"]].flatMap(([key, label], i) => [series(ct.supply[key].btc, label, { color: ["green", "red", "gray"][i] }), series(ct.supply[key].usd, `${label}, value`, { color: ["green", "red", "gray"][i], pane: 2 })])]),
      section("Value", entries(ct.value).flatMap(([key, node]) => flow(node, `Value ${key}`, `Cointime value ${key}`))),
      section("Age bands", cointimeBands(ct.ageRanges)),
      section(holderName("awake").name, entries(ct.cohorts).map(([key, c]) => section(holderName(key).name, weighted(c.awake, c.dormant, holderName(key).of(holderName("awake").name), "Awake", "Dormant")))),
    ]),
    section("Coinflow", [
      section("Age bands", coinflowBands(cf.ageRanges)),
      section(holderName("mobile").name, entries(cf.cohorts).map(([key, c]) => section(holderName(key).name, weighted(c.mobile, c.immobile, holderName(key).of(holderName("mobile").name), "Mobile", "Immobile")))),
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
  /** Inputs or outputs by type: each type's count and transactions, each with its share. @param {any} types @param {string} what @param {string} doing */
  function byType(types, what, doing) {
    const list = entries(types), colors = ramp(list.length);
    const name = (/** @type {string} */ key) => client.SPENDABLE_TYPE_NAMES[key]?.short ?? (key === "opReturn" ? "OP_RETURN" : key);
    return [
      section("Compare", [
        chart("Count, 1 month", `${what[0].toUpperCase()}${what.slice(1)} by type, 1 month`, list.map(([key, node], i) => series(node.count.sum._1m, name(key), { color: colors[i], type: "area" })), { stack: { 1: "percent" } }),
        chart("Transaction share, 1 month", `Share of transactions ${doing} each ${what.slice(0, -1)} type, 1 month`, list.map(([key, node], i) => series(node.txShare._1m, name(key), { color: colors[i] }))),
      ]),
      ...list.map(([key, node]) => section(name(key), [
        counted(node.count, "Count", `${name(key)} ${what}`, node.share),
        counted(node.txCount, "Transactions", `Transactions ${doing} ${inSentence(name(key))} ${what}`, node.txShare),
      ])),
    ];
  }
  /** Bitcoin clouds: each holder group's awake and mobile realized and capitalized prices (its cohorts' together), a
   * cloud from the lowest to the highest. */
  function cloudsChart() {
    const groups = /** @type {[string, string[], string][]} */ ([["Short-term holders", ["under4m", "sth", "under6m"], "#e19b00"], ["All holders", ["all"], "#f2660f"], ["Long-term holders", ["over4m", "lth", "over6m"], "#d50ced"]]);
    return chart("Clouds", "Bitcoin clouds", [
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
    const lines = /** @type {[any, string, string][]} */ ([[ct.cohorts.all.awake.costBasis.byDollar.capitalizedPrice, "Awake capitalized price", "orange"], [ct.prices.trueMarketMean.block, "True market mean", "sky"]]);
    return chart("Time above", "Time above the awake capitalized price and true market mean", [
      price(),
      ...lines.map(([node, label, color]) => series(node, label, { color })),
      ...lines.map(([node, label, color]) => ({ formula: `100 * sma(${t.price.close.name} >= ${node.name})`, label: `Share of time at or above the ${label.toLowerCase()}`, format: "percent", color, pane: 2 })),
    ]);
  }
  /**
   * OP_RETURN's charts (all of them, a protocol's, a policy's), each titled by `titled` from what it shows ("outputs"
   * as "Runes OP_RETURN outputs", "OP_RETURN outputs: Oversized"); all of them, with their share of all outputs
   * (`outputShare`, the outputs' by type), as that type's own chart is.
   * @param {any} node @param {(what: string) => string} titled @param {any} [outputShare]
   */
  function opReturnCharts(node, titled, outputShare) {
    return [
      ...(outputShare ? [counted(node.outputCount, "Outputs", titled("outputs"), outputShare)] : flow(node.outputCount, "Outputs", titled("outputs"))),
      ...(node.txCount ? [counted(node.txCount, "Transactions", titled("transactions"), node.txShare)] : []),
      ...flow(node.dataBytes, "Data", titled("data")),
      ...(node.dataBytes?.chainShare ? [chart("Data share", titled("share of block space"), sharesOf(node.dataBytes.chainShare, "Share"))] : []),
      ...flow(node.txVsize, "Transaction size", titled("transaction size")),
      ...flow(node.fees, "Fees", titled("fees")),
      ...(node.fees?.chainShare ? [chart("Fee share", titled("share of all fees"), sharesOf(node.fees.chainShare, "Share"))] : []),
    ];
  }
  /** An address group's charts (every address, or one type's). @param {any} a @param {string} who */
  function addressCharts(a, who) {
    const of = (/** @type {string} */ what) => `${what}: ${who}`;
    /** Supply held by some of the addresses: its amount, then its share of all coins. @param {any} supply @param {string} title */
    const heldSupply = (supply, title) => chart("Supply", title, [...amount(supply, "Supply"), series(supply.share, "Share of all coins", { color: "sky", pane: 3 })]);
    return [
      section("Count", [
        chart("Funded, empty and total", of("Address count"), [series(own(a.funded), "Funded", { color: "orange" }), series(a.empty, "Empty", { color: "gray" }), series(a.total, "Total", { color: "sky" })], { log: [1] }),
        ...change(a.funded.delta, "Change", of("Funded address change")),
        ...flow(a.new, "New", of("New addresses")),
        chart("Active", of("Active addresses"), entries(a.activity).flatMap(([key, node], i, all) => windows(node.avg, words(key), { show: key === "active" ? ["_1m"] : [], extra: { color: ramp(all.length)[i] } }))),
      ]),
      a.avgBalance && chart("Average balance", of("Average address balance"), amount(a.avgBalance, "Average balance"), { log: [1, 2] }),
      ...[["reused", "Reused"], ["respent", "Respent"]].filter(([key]) => a[key]).map(([key, label]) => section(label, [
        chart("Count", of(`${label} addresses`), [series(a[key].count.funded, "Funded", { color: "orange" }), series(a[key].count.total, "Total", { color: "sky" })], { log: [1] }),
        heldSupply(a[key].supply, of(`Supply in ${label.toLowerCase()} addresses`)),
        ...flow(a[key].events.outputs.count, "Outputs to them", of(`Outputs to ${label.toLowerCase()} addresses`)),
        chart("Output share", of(`Share of outputs to ${label.toLowerCase()} addresses`), sharesOf(a[key].events.outputs.share, "Share")),
        ...flow(a[key].events.inputs.count, "Inputs from them", of(`Inputs from ${label.toLowerCase()} addresses`)),
        chart("Input share", of(`Share of inputs from ${label.toLowerCase()} addresses`), sharesOf(a[key].events.inputs.share, "Share")),
        chart("Active", of(`Active ${label.toLowerCase()} addresses`), windows(a[key].events.active.count.avg, "")),
        chart("Active share", of(`Share of active addresses that are ${label.toLowerCase()}`), sharesOf(a[key].events.active.share, "Share")),
      ])),
      a.exposed && section("Exposed", [
        chart("Count", of("Addresses with exposed public keys"), [series(a.exposed.count.funded, "Funded", { color: "orange" }), series(a.exposed.count.total, "Total", { color: "sky" })], { log: [1] }),
        heldSupply(a.exposed.supply, of("Supply in addresses with exposed public keys")),
      ]),
      ...(a.supply ? cohort({ supply: a.supply, capital: a.capital }, who) : []),
    ].filter(Boolean).map((item) => lift(/** @type {SectionNode} */ (item)));
  }
  /** Pools: their shares compared, then each pool, by name. */
  function poolCharts() {
    const list = members(t.pools).filter(([, node]) => node.blockCount).map(([key, node]) => /** @type {[string, any, string]} */ ([key, node, client.POOL_ID_TO_POOL_NAME[key.replace(/^_/, "")] ?? key])).sort((a, b) => a[2].localeCompare(b[2]));
    // (Those mining most of the blocks in recent years: compared, and first; the others after, by name.)
    const MAJOR = ["foundryusa", "antpool", "viabtc", "f2pool", "marapool", "spiderpool", "binancepool", "luxor", "secpool", "braiinspool", "ocean", "poolin", "btccom", "unknown"];
    const major = MAJOR.flatMap((key) => list.filter(([k]) => k.replace(/^_/, "") === key)), others = list.filter((pool) => !major.includes(pool)), colors = ramp(major.length);
    /** A pool's charts. @param {[string, any, string]} pool */
    const pool = ([, node, name]) => section(name, [
      counted(node.blockCount, "Blocks mined", `Blocks mined: ${name}`, node.blockShare),
      ...flow(node.coinbase, "Coinbase", `Coinbase rewards: ${name}`),
      chart("Fees per block", `Fees per block: ${name}`, [...windows(node.fees.avg, "Bitcoin", { pick: btc }), ...windows(node.fees.avg, "Dollars", { pick: usd, pane: 2 })], { log: [1, 2] }),
      chart("Hash rate", `Hash rate: ${name}`, windows(node.hashrate, ""), { log: [1] }),
    ]);
    return [
      section("Compare", [
        chart("Share, 1 month", "Mining pools' share of blocks, 1 month", major.map(([, node, name], i) => series(node.blockShare._1m, name, { color: colors[i], type: "area" })), { stack: { 1: "stack" } }),
        chart("Hash rate, 1 month", "Mining pools' hash rate, 1 month", major.map(([, node, name], i) => series(node.hashrate._1m, name, { color: colors[i] })), { log: [1] }),
      ]),
      ...major.map(pool),
      section("Other pools", others.map(pool)),
    ];
  }
  /** Weighted coins (awake and dormant, mobile and immobile) for a holder group: supply, capital, cost basis. @param {any} on @param {any} off @param {string} who @param {string} onLabel @param {string} offLabel */
  function weighted(on, off, who, onLabel, offLabel) {
    return [
      chart("Supply", `Supply: ${who}`, [series(on.supply.total.btc, onLabel, { color: "orange", type: "area" }), series(btc(off.supply), offLabel, { color: "gray", type: "area" }), series(on.supply.total.usd, `${onLabel}, value`, { color: "green", type: "area", pane: 2 }), series(usd(off.supply), `${offLabel}, value`, { color: "gray", type: "area", pane: 2 }), series(on.supply.inLoss.share, `${onLabel}, share in loss`, { color: "red", pane: 3 })], { stack: { 1: "stack", 2: "stack" } }),
      chart("Realized cap", `Realized cap: ${who}`, [series(on.capital.realizedCap, "Realized cap", { color: "orange" })], { log: [1] }),
      ...cohort({ costBasis: on.costBasis }, who),
    ];
  }
  /** Cointime by age band: coin days created and consumed, wakefulness, awake and dormant supply. @param {any} ranges */
  function cointimeBands(ranges) {
    const list = entries(ranges), name = (/** @type {string} */ key) => named(client.AGE_RANGE_NAMES, key).replace(/ old$/, ""), colors = ramp(list.length);
    return [
      section("Compare", [
        chart("Wakefulness", "Wakefulness by age band", list.map(([key, node], i) => series(node.wakefulness, name(key), { color: colors[i] }))),
        chart("Awake supply", "Awake supply by age band", list.map(([key, node], i) => series(node.awake.supply.btc, name(key), { color: colors[i], type: "area" })), { stack: { 1: "percent" } }),
        chart("Dormant supply", "Dormant supply by age band", list.map(([key, node], i) => series(node.dormant.supply.btc, name(key), { color: colors[i], type: "area" })), { stack: { 1: "percent" } }),
      ]),
      ...list.map(([key, node]) => section(name(key), [
        ...flow(node.coindaysConsumed, "Coin days consumed", `Coin days consumed: UTXOs ${name(key).toLowerCase()} old`),
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
        chart("Mobility", "Mobility by age band", list.map(([key, node], i) => series(node.mobility, name(key), { color: colors[i] }))),
        chart("Mobile supply", "Mobile supply by age band", list.map(([key, node], i) => series(node.mobile.supply.btc, name(key), { color: colors[i], type: "area" })), { stack: { 1: "percent" } }),
        chart("Immobile supply", "Immobile supply by age band", list.map(([key, node], i) => series(node.immobile.supply.btc, name(key), { color: colors[i], type: "area" })), { stack: { 1: "percent" } }),
      ]),
      ...list.map(([key, node]) => section(name(key), [
        chart("Spending", `Spending rate, exposure and mobility: UTXOs ${name(key).toLowerCase()} old`, [series(node.spendingRate, "Spending rate", { color: "orange" }), series(node.spendingExposure, "Spending exposure", { color: "sky", pane: 2 }), series(node.mobility, "Mobility", { color: "violet", pane: 3 })]),
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

  /** @type {Preset[]} */
  const presets = [];
  const walk = (/** @type {(ChartNode | SectionNode)[]} */ items, /** @type {string[]} */ path, workings = false) => {
    for (const item of items) {
      if ("items" in item) walk(item.items, [...path, item.name], workings || Boolean(item.workings));
      else presets.push({ p: [...path, item.name], t: item.title, c: priced(item.c, workings || Boolean(item.priceHidden)) });
    }
  };
  walk(tree, []);
  return presets;
}

/**
 * The server's aliases: two keys of one node naming the same series under two ids, the well-known name first (a cohort's
 * `realizedPrice` is its average cost basis by coin, `avg`; `mvrv` is that average's `ratio`). The presets chart the
 * well-known one.
 */
const TWINS = [["realizedCap", "total"], ["realizedPrice", "avg"], ["capitalizedPrice", "avg"], ["medianRealizedPrice", "median"], ["medianCapitalizedPrice", "median"], ["mvrv", "ratio"], ["mayerMultiple", "ratio"], ["aviv", "ratio"]];
/** Each alias' twin, both ways, by name, in a client's tree. @param {any} tree @returns {Map<string, string>} */
function twins(tree) {
  const map = new Map(), stack = [tree];
  while (stack.length) {
    const node = stack.pop();
    if (!node || typeof node !== "object" || isLeaf(node)) continue;
    for (const [known, generic] of TWINS) {
      const a = node[known], b = isLeaf(node[generic]) ? node[generic] : node[generic]?.block;
      if (isLeaf(a) && isLeaf(b)) map.set(a.name, b.name).set(b.name, a.name);
    }
    for (const value of Object.values(node)) stack.push(value);
  }
  return map;
}

/**
 * The series Studio can chart (by date) in a client's tree that no preset shows (as a layer's series), by name: the
 * build stops at any. (Not the indexes' own, `indexer` and `mappings`: a block's fields, an interval's first height,
 * date and start, the plumbing between indexes.) The price's candles count for its parts, and an alias' twin for it.
 * What never happened or hasn't yet (notYet, by `facts`) has no presets: its series aren't missed.
 * @param {any} client @param {Preset[]} presets @param {Facts} [facts] @returns {string[]}
 */
export function leftOut(client, presets, facts) {
  const shown = new Set(presets.flatMap((preset) => preset.c.layers.map((layer) => layer.series).filter(Boolean)));
  const absent = notYet(client.series, facts);
  for (const [name, twin] of twins(client.series)) if (shown.has(twin)) shown.add(name);
  const missing = new Set();
  // (Nor the price's own parts, open, high, low, close and spot: its candles, in every preset with the price, show them.)
  const { indexer: _, mappings: __, price: { ohlc: ___, open, high, low, close, spot, ...priced }, ...rest } = client.series;
  const data = { ...rest, price: priced };
  const stack = [data];
  while (stack.length) {
    const node = stack.pop();
    if (absent.has(node)) continue;
    if (isLeaf(node)) {
      if (byDate(node) && !shown.has(node.name)) missing.add(node.name);
    } else if (node && typeof node === "object") for (const value of Object.values(node)) stack.push(value);
  }
  return [...missing].sort();
}
