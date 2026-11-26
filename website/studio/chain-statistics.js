// @ts-check

/** @import { SeriesTree, SeriesPattern, Concentration } from "../../modules/bitview-client/index.js" */
/** @import { ChartBuilder, ChartNode } from "./presets.js" */
/**
 * @typedef {Pick<SeriesPattern<unknown>, "name">} NamedSeries
 * @typedef {{ cumulative: NamedSeries, sum?: Record<string, NamedSeries> }} Cumulative
 */

/**
 * Chain statistics no series holds as such: ratios, shares and per-block averages, each a formula over the day's
 * counts. Each is placed in the section it belongs to (`at`, by that section's place); `left` names any section never
 * asked for, so none is lost.
 * @param {{ t: SeriesTree, chart: ChartBuilder, ramp: (count: number) => readonly string[], flowWindow: (sums: Cumulative["sum"], key: "_1m") => NamedSeries | undefined, underOneSatVb: Cumulative[] }} parts
 */
export function chainStatistics({ t, chart, ramp, flowWindow, underOneSatVb }) {
  /** @param {NamedSeries} series */
  const perDay = (series) => `perday(${series.name})`;
  /** @param {Cumulative} total */
  const daily = (total) => perDay(total.cumulative);
  /** @param {string} part @param {string} whole */
  const percent = (part, whole) => `100 * (${part}) / (${whole})`;
  /** @param {string} all @param {string[]} parts */
  const rest = (all, parts) => `${all} - ${parts.map((part) => `(${part})`).join(" - ")}`;
  /** @param {string} name @param {string} title @param {string[][]} lines @param {number} days @param {{ format?: string, log?: number[] }} [options] */
  const line = (name, title, lines, days, { format, log } = {}) => chart(name, title, lines.map(([label, formula], i) => ({ formula, label, color: ramp(lines.length)[i], ...(format ? { format } : {}) })), { ...(log ? { log } : {}), ...(days > 1 ? { average: { 1: `${days}d` } } : {}) });
  /** @param {string} name @param {string} title @param {string[][]} lines @param {number} days */
  const percentLine = (name, title, lines, days) => line(name, title, lines, days, { format: "percent" });
  /** @param {string} name @param {string} title @param {string[][]} parts @param {number} days */
  const shareStack = (name, title, parts, days) => chart(name, title, parts.map(([label, formula], i) => ({ formula, label, type: "area", color: ramp(parts.length)[i] })), { stack: { 1: "percent" }, ...(days > 1 ? { average: { 1: `${days}d` } } : {}) });

  const day = { blocks: daily(t.blocks.count), txs: daily(t.transactions.count), inputs: daily(t.inputs.count), outputs: daily(t.outputs.count) };
  /** @param {string} formula */
  const perBlock = (formula) => `(${formula}) / ${day.blocks}`;
  /** @param {string} blocks */
  const ofBlocks = (blocks) => percent(blocks, day.blocks);

  const inputs = t.inputs.types, outputs = t.outputs.types, nested = t.inputs.nestedSegwit, multisig = t.inputs.multisig;
  const scriptPath = daily(t.inputs.p2trScriptPath);
  const keyPath = `${daily(inputs.p2tr.count)} - ${scriptPath}`;
  const plainP2sh = `${daily(inputs.p2sh.count)} - ${daily(nested.p2wpkh)} - ${daily(nested.p2wsh)}`;
  const { ecdsa, schnorr } = t.inputs.signatures;
  const work = t.blocks.work.cumulative, hashrate = t.mining.hashrate.value, difficulty = t.blocks.difficulty.value;
  const workPerBlock = perBlock(daily(t.blocks.work));
  /** @param {string} name @param {string} title @param {Concentration} tops */
  const centralization = (name, title, tops) => percentLine(name, title, (/** @type {const} */ ([2, 3, 4, 5, 6])).map((n) => [`Top ${n}`, ofBlocks(tops[`top${n}`].name)]), 30);
  const txs = t.transactions, features = txs.features, timelocks = txs.timelocks, payments = txs.payments, fees = txs.fees;
  /** @param {string} name @param {string} title @param {Cumulative} total @param {number} days */
  const ofTxs = (name, title, total, days) => percentLine(name, title, [["Share", percent(daily(total), day.txs)]], days);
  const within = t.inputs.spentWithin;
  /** @type {[string, Cumulative][]} */
  const spendAge = [["Same block", t.inputs.sameBlock], ["Within 1 block", within._1Block], ["Within 6 blocks", within._6Blocks], ["Within 144 blocks", within._144Blocks], ["Within 2,016 blocks", within._2016Blocks]];
  // One whose lines aren't all published is left out whole: a comparison missing a line would read as that line at
  // zero.
  const inAndOut = (/** @type {string} */ name, /** @type {keyof typeof inputs} */ key) => {
    const lines = /** @type {const} */ ([["Inputs", inputs[key].count], ["Outputs", outputs[key].count]]).flatMap(([label, total]) => ((node) => (node ? [[`${label}, 30 days`, node.name]] : []))(flowWindow(total.sum, "_1m")));
    return lines.length === 2 ? [line(`${name}, created and spent`, `${name} inputs and outputs, 30 days`, lines, 1)] : [];
  };

  /** @type {[string, (ChartNode | ChartNode[])[]][]} */
  const placed = [
    ["Mining › Hashing", [
      line("Chainwork, log2", "Chainwork in log2", [["log2(chainwork)", `ln(${work.name}) / ln(2)`]], 1),
      line("Work per block, log2", "Work per block in log2", [["log2(work)", `ln(${workPerBlock}) / ln(2)`]], 1),
      line("Chainwork, in days at each day's hash rate", "Chainwork in days at the hash rate of each day", [["Days", `${work.name} / ${hashrate.name} / 86400`]], 7),
      line("Work per block", "Average work per block", [["Work", workPerBlock]], 1, { log: [1] }),
    ]],
    ["Mining › Difficulty", [chart("Hash rate and difficulty", "Hash rate and difficulty", [{ formula: hashrate.name, label: "Hash rate", color: "orange" }, { formula: difficulty.name, label: "Difficulty", color: "sky", pane: 2 }], { log: [1, 2], average: { 1: "7d" } })]],
    ["Mining › Rewards", [chart("Reward per block, subsidy and fees", "Average reward per block, subsidy and fees", (/** @type {const} */ ([["Subsidy", t.mining.rewards.subsidy], ["Fees", t.mining.rewards.fees]])).map(([label, node], i) => ({ formula: perBlock(perDay(node.cumulative.btc)), label, format: "btc", type: "area", color: ramp(2)[i] })), { stack: { 1: "stack" }, average: { 1: "7d" } })]],
    ["Mining › Pools", [
      centralization("Centralization, top pools' share", "Share of blocks mined by the top pools", t.pools.concentration),
      centralization("Centralization, AntPool group as one", "Share of blocks mined by the top pools, AntPool's group as one", t.pools.concentrationGrouped),
    ]],
    ["Fees › By fee rate", [percentLine("Under 1 sat/vB, share of transactions", "Share of transactions paying under 1 sat/vB", [["Share", percent(underOneSatVb.map(daily).join(" + "), day.txs)]], 7)]],
    ["Fees", [line("Median fee rate, in dollars", "Median fee rate in dollars per vB", [["Median fee rate", `${fees.effectiveFeeRate.median.name} * ${t.price.spot.name} / 100000000`]], 7, { format: "usd" })]],
    ["Network › Blocks › Coinbase locktime", [
      percentLine("Nonzero, share", "Share of blocks with a nonzero coinbase locktime", [["Share", ofBlocks(daily(t.blocks.coinbaseLocktime.nonzero))]], 30),
      percentLine("BIP-54, share", "Share of blocks whose coinbase locktime matches BIP-54's rule", [["Share", ofBlocks(daily(t.blocks.coinbaseLocktime.bip54))]], 7),
    ]],
    ["Network › Blocks", [
      line("Signature operations per block", "Average signature operation cost per block", [["Cost", perBlock(daily(t.blocks.sigopCost))]], 30),
      line("Size per block, average", "Average block size", [["Size", perBlock(daily(t.blocks.size))]], 7),
      line("Weight per block, average", "Average block weight", [["Weight", perBlock(daily(t.blocks.weight))]], 7),
    ]],
    ["Network › Transactions › Payments", [
      line("Per transaction", "Payments per transaction", [["Payments", `${daily(payments.count)} / ${day.txs}`]], 7),
      percentLine("RBF signaling, share", "Share of payments signaling RBF", [["Share", percent(daily(payments.rbfSignaling), daily(payments.count))]], 7),
      percentLine("SegWit inputs, share", "Share of payments in transactions with SegWit inputs", [["Share", percent(daily(payments.segwitSpending), daily(payments.count))]], 7),
      chart("Against block size", "Payments and average block size", [{ formula: daily(payments.count), label: "Payments", color: "orange" }, { formula: perBlock(daily(t.blocks.size)), label: "Block size", format: "bytes", color: "sky", pane: 2 }], { average: { 1: "7d", 2: "7d" } }),
    ]],
    ["Network › Transactions › Features › RBF signaling", [ofTxs("Share", "Share of transactions signaling RBF", features.rbfSignaling, 7)]],
    ["Network › Transactions › Features", [
      ofTxs("BIP-69, share", "Share of transactions with BIP-69 ordering", features.bip69, 7),
      ofTxs("Spending same-block outputs, share", "Share of transactions spending outputs from the same block", txs.sameBlockSpending, 7),
      percentLine("BIP-69, share against chance", "Share of transactions with BIP-69 ordering, and of those expected by chance", [["BIP-69 ordering", percent(daily(features.bip69), day.txs)], ["Expected by chance", percent(daily(features.bip69ExpectedByChance), day.txs)]], 7),
    ]],
    ["Network › Transactions › Timelocks", [
      ofTxs("Height-based absolute, share", "Share of transactions with a height-based absolute locktime", timelocks.height, 7),
      ofTxs("Unenforced, share", "Share of transactions with an unenforced locktime", timelocks.unenforced, 7),
      ofTxs("Time-based absolute, share", "Share of transactions with a time-based absolute locktime", timelocks.time, 1),
      percentLine("Relative, share", "Share of transactions with a relative locktime", [["Height-based", percent(daily(timelocks.relativeHeight), day.txs)], ["Time-based", percent(daily(timelocks.relativeTime), day.txs)]], 7),
    ]],
    ["Network › Transactions › Inscriptions", [ofTxs("Share of transactions", "Share of transactions revealing an inscription", txs.inscription, 7)]],
    ["Network › Transactions", [
      line("Size, average", "Average transaction size", [["vsize", `${daily(txs.vsize)} / ${day.txs}`], ["Size", `${daily(t.blocks.size)} / ${day.txs}`]], 7),
      shareStack("By inputs and outputs", "Transactions by their number of inputs and outputs", [["1 in, 1 out", daily(features.oneInputOneOutput)], ["1 in, 2 out", daily(features.oneInputTwoOutput)], ["1 in, N out", `${daily(features.oneInput)} - ${daily(features.oneInputOneOutput)} - ${daily(features.oneInputTwoOutput)}`], ["N in, 1 out", `${daily(features.oneOutput)} - ${daily(features.oneInputOneOutput)}`], ["Others", `${day.txs} - ${daily(features.oneInput)} - ${daily(features.oneOutput)} + ${daily(features.oneInputOneOutput)}`]], 7),
    ]],
    ["Network › Inputs › Signatures", [
      shareStack("Public keys, compressed and uncompressed", "Public keys revealed in inputs and created in outputs, compressed and uncompressed", [["Uncompressed, inputs", daily(t.inputs.pubkeys.uncompressed)], ["Compressed, inputs", daily(t.inputs.pubkeys.compressed)], ["Compressed, outputs", daily(t.outputs.pubkeys.compressed)], ["Uncompressed, outputs", daily(t.outputs.pubkeys.uncompressed)]], 30),
      line("Verifications per block", "Average signature verifications per block", [["ECDSA and Schnorr", perBlock(`${daily(ecdsa.count)} + ${daily(schnorr.count)}`)]], 7),
    ]],
    ["Network › Inputs › Spent within", [
      percentLine("Same block, share of inputs", "Share of inputs spending outputs from the same block", [["Share", percent(daily(t.inputs.sameBlock), day.inputs)]], 7),
      percentLine("Share of inputs", "Share of inputs spending recent outputs", spendAge.map(([name, total]) => [name, percent(daily(total), day.inputs)]), 30),
    ]],
    ["Network › Inputs", [
      percentLine("Coinbase witness, share of blocks", "Share of blocks whose coinbase has a witness", [["Share", ofBlocks(daily(t.inputs.coinbaseWitness))]], 7),
      shareStack("Multisig, by type", "Multisig inputs by type", [["P2MS", daily(inputs.p2ms.count)], ["P2SH", daily(multisig.p2sh)], ["Nested P2WSH", daily(multisig.p2shP2wsh)], ["P2WSH", daily(multisig.p2wsh)]], 30),
      shareStack("P2SH, plain and nested SegWit", "P2SH inputs, plain and nested SegWit", [["P2SH", plainP2sh], ["Nested P2WPKH", daily(nested.p2wpkh)], ["Nested P2WSH", daily(nested.p2wsh)]], 30),
      line("Taproot key path", "Number of Taproot key-path inputs", [["Inputs", keyPath]], 7),
      shareStack("Taproot key and script path", "Taproot inputs by spending path", [["Key path", keyPath], ["Script path", scriptPath]], 30),
      line("Per transaction", "Inputs per transaction", [["Inputs", `${day.inputs} / ${day.txs}`]], 7),
      shareStack("SegWit, by type", "SegWit inputs by type", [["Nested P2WPKH", daily(nested.p2wpkh)], ["Nested P2WSH", daily(nested.p2wsh)], ["P2WPKH", daily(inputs.p2wpkh.count)], ["P2WSH", daily(inputs.p2wsh.count)], ["P2TR key path", keyPath], ["P2TR script path", scriptPath]], 30),
      percentLine("Multisig, share of inputs", "Share of inputs spending multisig, Taproot aside", [["Share", percent([daily(inputs.p2ms.count), daily(multisig.p2sh), daily(multisig.p2shP2wsh), daily(multisig.p2wsh)].join(" + "), day.inputs)]], 7),
    ]],
    ["Network › Outputs", [
      line("Coinbase outputs per block", "Coinbase outputs per block", [["Outputs", perBlock(daily(t.outputs.coinbase.count))]], 7),
      line("Per transaction", "Outputs per transaction", [["Outputs", `${day.outputs} / ${day.txs}`]], 7),
    ]],
    ["Network › Inputs and outputs", [
      line("Native SegWit v0, created and spent", "Native SegWit v0 inputs and outputs", [["Inputs", `${daily(inputs.p2wpkh.count)} + ${daily(inputs.p2wsh.count)}`], ["Outputs", `${daily(outputs.p2wpkh.count)} + ${daily(outputs.p2wsh.count)}`]], 7),
      inAndOut("P2A", "p2a"),
      inAndOut("P2MS", "p2ms"),
      inAndOut("P2PK", "p2pk"),
      inAndOut("P2PKH", "p2pkh"),
      line("P2SH, created and spent", "P2SH inputs and outputs, nested SegWit too", [["Inputs", plainP2sh], ["Inputs, nested SegWit too", daily(inputs.p2sh.count)], ["Outputs", daily(outputs.p2sh.count)]], 7),
      inAndOut("P2TR", "p2tr"),
      inAndOut("P2WPKH", "p2wpkh"),
      inAndOut("P2WSH", "p2wsh"),
      line("Count", "Number of inputs and outputs", [["Inputs", day.inputs], ["Outputs", day.outputs]], 7),
      line("Inputs per output", "Inputs per output", [["Inputs", `${day.inputs} / ${day.outputs}`]], 7),
    ]],
    ["UTXOs › By type › Compare", [line("Change per day", "Change in UTXO count per day by type", (/** @type {const} */ (["p2pk", "p2pkh", "p2wpkh", "p2ms", "p2sh", "p2wsh", "p2tr", "p2a"])).map((key) => [key.toUpperCase(), perDay(t.utxos.types[key].utxos.count.value)]), 7)]],
    ["Addresses › Reused", [percentLine("Direct reuse, share", "Share of outputs paying an address being spent", [["Share", percent(daily(t.addresses.reused.direct.outputCount), day.outputs)]], 7)]],
  ];
  const homes = new Map(placed.map(([home, charts]) => [home, charts.flat()]));
  return {
    /** The charts of the section at `home`, its place. @param {string} home */
    at: (home) => ((charts) => (homes.delete(home), charts ?? []))(homes.get(home)),
    left: () => [...homes.keys()],
  };
}
