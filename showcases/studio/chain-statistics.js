// @ts-check
/**
 * Chain statistics: every chart of mainnet.observer's kept under its own name, computed from Bitview's series, a day's
 * value the change of a running total and a share the ratio of two. Every count behind them is a series of its own,
 * defined once on the server (the AntPool group, proof of work, signature bytes); these presets only combine them.
 */

/** @import { SeriesTree, SeriesPattern, Pool, Concentration } from "../../modules/bitview-client/index.js" */
/** @import { Facts, ChartBuilder, SectionBuilder, SectionNode } from "./presets.js" */
/**
 * @typedef {Pick<SeriesPattern<unknown>, "name">} NamedSeries
 * @typedef {{ cumulative: NamedSeries, sum?: Record<string, NamedSeries> }} Cumulative
 * @typedef {{ [K in keyof SeriesTree["pools"]]: SeriesTree["pools"][K] extends Pool ? K : never }[keyof SeriesTree["pools"]]} PoolKey
 */

/**
 * @param {{ t: SeriesTree, facts: Facts | undefined, chart: ChartBuilder, section: SectionBuilder, ramp: (count: number) => readonly string[], flowWindow: (sums: Cumulative["sum"], days: number) => NamedSeries | undefined }} parts
 * @returns {SectionNode}
 */
export function chainStatistics({ t, facts, chart, section, ramp, flowWindow }) {
  /** A series' change over a span. @param {NamedSeries} series @param {string} span */
  const yearChange = (series, span) => `flow(${series.name}, ${span})`;
  /** A series' change over a day. @param {NamedSeries} series */
  const perDay = (series) => `diff(${series.name}, 1d)`;
  /** A running total's change over a day. @param {Cumulative} total */
  const daily = (total) => perDay(total.cumulative);
  /** `part` as a percentage of `whole`. @param {string} part @param {string} whole */
  const percent = (part, whole) => `100 * (${part}) / (${whole})`;
  /** What `all` holds beyond `parts`. @param {string} all @param {string[]} parts */
  const rest = (all, parts) => `${all} - ${parts.map((part) => `(${part})`).join(" - ")}`;
  /** Lines, each a label and a formula. @param {string} title @param {string[][]} lines @param {number} days @param {{ format?: string, log?: number[], maxInterval?: string }} [options] */
  const line = (title, lines, days, { format, log, maxInterval } = {}) => chart(title, title, lines.map(([label, formula], i) => ({ formula, label, color: ramp(lines.length)[i], ...(format ? { format } : {}), ...(maxInterval ? { maxInterval } : {}) })), { ...(log ? { log } : {}), ...(days > 1 ? { average: { 1: `${days}d` } } : {}) });
  /** Fixed native event windows: weekly and monthly charts, each sampled no coarser than its window. */
  const events = (/** @type {string} */ title, /** @type {[string, Cumulative][]} */ totals, days = 7, options = {}) => (days === 7 ? [7, 30] : [days]).map((span) => {
    const name = span === days ? title : `${title}, per ${span} days`;
    const lines = totals.map(([label, total]) => {
      const node = flowWindow(total.sum, span);
      if (node) return [`${label} per ${span} days`, node.name];
      console.warn(`Missing published ${span}-day sum for ${name} (${total.cumulative.name})`);
      return null;
    });
    return lines.every((line) => line !== null) ? line(name, /** @type {string[][]} */ (lines), 1, { ...options, maxInterval: span === 7 ? "week1" : "month1" }) : null;
  });
  /** A calendar year's flow, from the preceding year's end. */
  const yearly = (/** @type {string} */ title, /** @type {NamedSeries} */ total, format = "number") => chart(title, title, [{ formula: yearChange(total, "1y"), label: title, type: "histogram", colors: 1, color: "orange", format }], { index: "year1" });
  /** Lines of percentages. @param {string} title @param {string[][]} lines @param {number} days */
  const percentLine = (title, lines, days) => line(title, lines, days, { format: "percent" });
  /** Areas stacked into shares of their sum. @param {string} title @param {string[][]} parts @param {number} days */
  const shareStack = (title, parts, days) => chart(title, title, parts.map(([label, formula], i) => ({ formula, label, type: "area", color: ramp(parts.length)[i] })), { stack: { 1: "percent" }, ...(days > 1 ? { average: { 1: `${days}d` } } : {}) });

  const day = { blocks: daily(t.blocks.count), txs: daily(t.transactions.count), inputs: daily(t.inputs.count), outputs: daily(t.outputs.count) };
  /** Per block, over the day. @param {string} formula */
  const perBlock = (formula) => `(${formula}) / ${day.blocks}`;

  const types = /** @type {const} */ (["p2pk", "p2pkh", "p2wpkh", "p2ms", "p2sh", "p2wsh", "p2tr", "p2a"]);
  const label = (/** @type {string} */ key) => ({ opReturn: "OP_RETURN", emptyScript: "empty script", unknownScript: "unknown script" })[key] ?? key.toUpperCase();
  const inputs = t.inputs.types, outputs = t.outputs.types, nested = t.inputs.nestedSegwit, multisig = t.inputs.multisig;
  const scriptPath = daily(t.inputs.p2trScriptPath);
  const keyPath = `${daily(inputs.p2tr.count)} - ${scriptPath}`;
  const plainP2sh = `${daily(inputs.p2sh.count)} - ${daily(nested.p2wpkh)} - ${daily(nested.p2wsh)}`;

  const { ecdsa, schnorr } = t.inputs.signatures;
  const lengths = (/** @type {const} */ ([70, 71, 72, 73, 74])).map((n) => [`${n} byte`, daily(ecdsa.length[`_${n}Bytes`])]).concat([["75 byte or more", daily(ecdsa.length.over74Bytes)]]);
  /** ECDSA signatures with and without a property. @param {string} title @param {Cumulative} total @param {string} yes @param {string} no */
  const ecdsaSplit = (title, total, yes, no) => shareStack(title, [[yes, daily(total)], [no, `${daily(ecdsa.count)} - ${daily(total)}`]], 30);
  const sighash = (/** @type {string} */ title, /** @type {keyof typeof ecdsa.sighash} */ key) => events(title, [["SigHashes", ecdsa.sighash[key]]]);
  const opcodes = (/** @type {string} */ title, /** @type {(keyof typeof t.inputs.opcodes)[]} */ keys) => events(title, keys.map((key) => [key.toUpperCase(), t.inputs.opcodes[key]]));

  const fees = t.transactions.fees;
  /** @type {[keyof typeof fees.txCountByFeeRate, string][]} */
  const feeRanges = [["under0p1SatVb", "under 0.1"], ["_0p1To0p25SatVb", "0.1 to 0.25"], ["_0p25To0p5SatVb", "0.25 to 0.5"], ["_0p5To1SatVb", "0.5 to 1"], ["_1To2SatVb", "1 to 2"], ["_2To5SatVb", "2 to 5"], ["_5To10SatVb", "5 to 10"], ["_10To25SatVb", "10 to 25"], ["_25To50SatVb", "25 to 50"], ["_50To100SatVb", "50 to 100"], ["_100To250SatVb", "100 to 250"], ["_250To500SatVb", "250 to 500"], ["_500To1kSatVb", "500 to 1,000"], ["over1kSatVb", "over 1,000"]];
  const underOneSatVb = [daily(fees.zeroFee.txCount), ...feeRanges.slice(0, 4).map(([key]) => daily(fees.txCountByFeeRate[key]))].join(" + ");

  const work = t.blocks.work.cumulative, hashrate = t.mining.hashrate.value, difficulty = t.blocks.difficulty.value;
  const workPerBlock = perBlock(perDay(work));

  const pools = t.pools;
  /** Some of a day's blocks, as a share of all. @param {string} blocks */
  const ofBlocks = (blocks) => percent(blocks, day.blocks);
  const poolLine = (/** @type {string} */ name, /** @type {Pool} */ pool) => percentLine(`${name} hashrate`, [[`${name} share`, ofBlocks(daily(pool.blockCount))]], 7);
  const group = daily(pools.groups.antpool.blockCount);
  const large = (/** @type {[string, Pool][]} */ ([["Foundry USA", pools.foundryusa], ["F2Pool", pools.f2pool], ["ViaBTC", pools.viabtc], ["SpiderPool", pools.spiderpool]])).map(([name, pool]) => [name, daily(pool.blockCount)]);
  const poolOf = (/** @type {string} */ slug) => pools[/** @type {PoolKey} */ (slug in pools ? slug : `_${slug}`)];
  /** The largest pools of each day, as a share of its blocks. @param {string} title @param {Concentration} tops */
  const centralization = (title, tops) => percentLine(title, (/** @type {const} */ ([2, 3, 4, 5, 6])).map((n) => [`top ${n} pools`, ofBlocks(tops[`top${n}`].name)]), 30);

  const txs = t.transactions, features = txs.features, timelocks = txs.timelocks, spending = txs.spending;
  const ofTxs = (/** @type {string} */ title, /** @type {string} */ name, /** @type {Cumulative} */ total, /** @type {number} */ days) => percentLine(title, [[name, percent(daily(total), day.txs)]], days);
  const within = t.inputs.spentWithin;
  /** @type {[string, Cumulative][]} */
  const spendAge = [["same block", t.inputs.sameBlock], ["≤ 1 block", within._1Block], ["≤ 6 blocks", within._6Blocks], ["≤ 144 blocks", within._144Blocks], ["≤ 2016 blocks", within._2016Blocks]];
  const utxos = t.utxos.types;
  const inAndOut = (/** @type {string} */ title, /** @type {keyof typeof inputs} */ key) => events(title, [["inputs", inputs[key].count], ["outputs", outputs[key].count]]);

  return section("Chain statistics", [
    section("Yearly", [
      yearly("Transaction fees per year", t.mining.rewards.fees.cumulative.btc, "btc"),
      yearly("Empty blocks per year", t.blocks.emptyCount.cumulative),
      yearly("Blocks per year", t.blocks.count.cumulative),
      yearly("Transactions per year", t.transactions.count.cumulative),
    ]),
    section("Bitcoin script", [
      ecdsaSplit("S-values in ECDSA signatures", ecdsa.lowS, "low S", "high S"),
      shareStack("Length of ECDSA signatures", [["<70 byte", rest(daily(ecdsa.count), lengths.map(([, formula]) => formula))], ...lengths], 30),
      shareStack("r and s-values in Bitcoin ECDSA signatures", [["low r & low S", daily(ecdsa.lowRLowS)], ["high r & high S", `${daily(ecdsa.count)} - ${daily(ecdsa.lowR)} - ${daily(ecdsa.lowS)} + ${daily(ecdsa.lowRLowS)}`], ["low r & high S", `${daily(ecdsa.lowR)} - ${daily(ecdsa.lowRLowS)}`], ["high r & low S", `${daily(ecdsa.lowS)} - ${daily(ecdsa.lowRLowS)}`]], 30),
      ecdsaSplit("r-values in ECDSA signatures", ecdsa.lowR, "low r", "high r"),
      ecdsaSplit("DER encoded ECDSA signatures", ecdsa.strictDer, "DER encoded", "not DER encoded"),
      shareStack("Public key compression (ECDSA)", [["Uncompressed (input)", daily(t.inputs.pubkeys.uncompressed)], ["Compressed (input)", daily(t.inputs.pubkeys.compressed)], ["Compressed (output)", daily(t.outputs.pubkeys.compressed)], ["Uncompressed (output)", daily(t.outputs.pubkeys.uncompressed)]], 30),
      ...sighash("SigHash ALL", "all"),
      ...sighash("SigHash ALL | ACP", "allAnyoneCanPay"),
      ...sighash("SigHash NONE | ACP", "noneAnyoneCanPay"),
      ...sighash("SigHash SINGLE | ACP", "singleAnyoneCanPay"),
      ...sighash("SigHash NONE", "none"),
      ...sighash("SigHash SINGLE", "single"),
      line("Total signature count", [["ECDSA", ecdsa.count.cumulative.name], ["Schnorr", schnorr.count.cumulative.name]], 1),
      line("Signature count per day", [["ECDSA", daily(ecdsa.count)], ["Schnorr", daily(schnorr.count)]], 7),
      line("Total ECDSA and Schnorr signature size", [["GB of ECDSA signatures", `${ecdsa.bytes.cumulative.name} / 1000000000`], ["GB of Schnorr signatures", `${schnorr.bytes.cumulative.name} / 1000000000`]], 1),
      ...opcodes("Signature checks by opcode", ["checksig", "checksigverify", "checkmultisig", "checksigadd"]),
      ...opcodes("Timelocks by opcode", ["checklocktimeverify", "checksequenceverify"]),
      ...opcodes("Hashes by opcode", ["hash160", "sha256"]),
      ...opcodes("Branches by opcode", ["if", "notif"]),
      line("Signature verifications per block", [["ECDSA and Schnorr", perBlock(`${daily(ecdsa.count)} + ${daily(schnorr.count)}`)]], 7),
    ]),
    section("Blocks", [
      line("Blockchain growth per day", [["chain growth rate", daily(t.blocks.size)]], 30),
      line("Blockchain size", [["chain size", t.blocks.size.cumulative.name]], 1),
      line("Sigops per block", [["block sigops", perBlock(daily(t.blocks.sigopCost))]], 30),
      line("Block size", [["block size in bytes", perBlock(daily(t.blocks.size))]], 7),
      line("Block weight", [["block weight", perBlock(daily(t.blocks.weight))]], 7),
      ...events("Empty blocks", [["empty blocks", t.blocks.emptyCount]]),
      line("Blocks per day", [["blocks per day", day.blocks]], 7),
      ...events("BIP-110 signaling blocks", [["blocks signaling on bit 4 (BIP-91 in 2017)", t.blocks.versionBits.bit4]]),
      percentLine("Coinbases with a witness", [["coinbases with a witness", ofBlocks(daily(t.inputs.coinbaseWitness))]], 7),
      line("Cumulative unclaimed coins", [["unclaimed coins", t.mining.rewards.unclaimed.cumulative.btc.name]], 1, { format: "btc" }),
      chart("Coinbase output value", "Coinbase output value", (/** @type {const} */ ([["block subsidy", t.mining.rewards.subsidy], ["transaction fees", t.mining.rewards.fees]])).map(([name, node], i) => ({ formula: perBlock(perDay(node.cumulative.btc)), label: name, format: "btc", type: "area", color: ramp(2)[i] })), { stack: { 1: "stack" }, average: { 1: "7d" } }),
    ]),
    section("Fees", [
      percentLine("Sub 1 sat/vByte transactions", [["sub 1 sat/vbyte", percent(underOneSatVb, day.txs)]], 7),
      ...events("Zero-Fee transactions", [["zero-fee transactions", fees.zeroFee.txCount]]),
      shareStack("Transaction fee rate distribution", [["zero fee", daily(fees.zeroFee.txCount)], ...feeRanges.map(([key, name]) => [name, daily(fees.txCountByFeeRate[key])])], 90),
      shareStack("Fee rates by block space", feeRanges.map(([key, name]) => [name, daily(fees.vsizeByEffectiveFeeRate[key])]), 30),
      line("Fee rate in dollars", [["median fee rate", `${fees.effectiveFeeRate.median.name} * ${t.price.spot.name} / 100000000`]], 7, { format: "usd" }),
    ]),
    section("Inputs", [
      line("Cumulative inputs", [["Inputs (cumulative)", t.inputs.count.cumulative.name]], 1),
      shareStack("Multisig input types", [["P2MS", daily(inputs.p2ms.count)], ["P2SH", daily(multisig.p2sh)], ["nested P2WSH", daily(multisig.p2shP2wsh)], ["P2WSH", daily(multisig.p2wsh)]], 30),
      ...events("Nested P2WPKH inputs", [["nested P2WPKH inputs", nested.p2wpkh]]),
      ...events("Nested P2WSH inputs", [["nested P2WSH inputs", nested.p2wsh]]),
      shareStack("P2SH input type distribution", [["P2SH", plainP2sh], ["nested P2WPKH", daily(nested.p2wpkh)], ["nested P2WSH", daily(nested.p2wsh)]], 30),
      line("P2TR key-path spends", [["P2TR key-path inputs", keyPath]], 7),
      shareStack("P2TR key- and script-path spends", [["key-path", keyPath], ["script-path", scriptPath]], 30),
      line("P2TR script-path spends", [["P2TR script-path inputs", scriptPath]], 7),
      line("Inputs per transaction", [["inputs per transaction", `${day.inputs} / ${day.txs}`]], 7),
      shareStack("SegWit input types", [["nested P2WPKH", daily(nested.p2wpkh)], ["nested P2WSH", daily(nested.p2wsh)], ["P2WPKH", daily(inputs.p2wpkh.count)], ["P2WSH", daily(inputs.p2wsh.count)], ["P2TR key-path", keyPath], ["P2TR script-path", scriptPath]], 30),
      percentLine("Inputs spending multisig (excl. Taproot)", [["inputs spending multisig", percent([daily(inputs.p2ms.count), daily(multisig.p2sh), daily(multisig.p2shP2wsh), daily(multisig.p2wsh)].join(" + "), day.inputs)]], 7),
      line("Cumulative inputs by script type", types.map((key) => [label(key), inputs[key].count.cumulative.name]), 1),
      shareStack("Input types by count", [...types.filter((key) => key !== "p2sh" && key !== "p2tr").map((key) => [label(key), daily(inputs[key].count)]), ["P2SH", plainP2sh], ["nested P2WPKH", daily(nested.p2wpkh)], ["nested P2WSH", daily(nested.p2wsh)], ["P2TR key-path", keyPath], ["P2TR script-path", scriptPath]], 30),
    ]),
    section("Outputs", (() => {
      const coinbase = t.outputs.coinbase;
      const coinbaseTypes = /** @type {const} */ (["p2pk", "p2pkh", "p2wpkh", "p2ms", "p2sh", "p2wsh", "p2tr", "p2a", "emptyScript", "unknownScript", "opReturn"]);
      const counted = (/** @type {keyof typeof outputs} */ key) => (key === "opReturn" ? t.opReturn.outputCount : outputs[key].count);
      /** @type {(typeof types[number] | "opReturn" | "emptyScript" | "unknownScript")[]} */
      const outputTypes = [...types.slice(0, 7), "opReturn", "p2a", "emptyScript", "unknownScript"];
      return [
        shareStack("Coinbase output types by count", coinbaseTypes.map((key) => [label(key), daily(coinbase.types[key])]), 30),
        ...events("OP_RETURN outputs", [["OP_RETURN outputs", t.opReturn.outputCount]]),
        shareStack("Output types by value", outputTypes.map((key) => [label(key), perDay((key === "opReturn" ? t.outputs.opReturnValue : outputs[key].value).cumulative.btc)]), 30),
        shareStack("Output types by count", outputTypes.map((key) => [label(key), daily(counted(key))]), 30),
        line("Cumulative BIP47 notifications", [["cumulative BIP47 notifications", t.opReturn.protocols.bip47.outputCount.cumulative.name]], 1),
        line("Cumulative OP_RETURN data bytes", [["OP_RETURN data bytes", t.opReturn.dataBytes.cumulative.name]], 1),
        line("Cumulative outputs", [["Outputs (cumulative)", t.outputs.count.cumulative.name]], 1),
        line("OP_RETURN data bytes", [["OP_RETURN data bytes", daily(t.opReturn.dataBytes)]], 1),
        ...events("OmniLayer (OP_RETURN)", [["OmniLayer", t.opReturn.protocols.omni.outputCount]]),
        ...events("Stacks block commit (OP_RETURN)", [["Stacks block commit", t.opReturn.protocols.stacks.outputCount]]),
        ...events("P2TR outputs", [["P2TR outputs", outputs.p2tr.count]]),
        line("Outputs per Coinbase transaction", [["Outputs", perBlock(daily(coinbase.count))]], 7),
        line("Outputs per transaction", [["Outputs", `${day.outputs} / ${day.txs}`]], 7),
        line("Cumulative Runestones (OP_RETURN)", [["cumulative runestones", t.opReturn.protocols.runes.outputCount.cumulative.name]], 1),
        ...events("Runestone (OP_RETURN)", [["runestones", t.opReturn.protocols.runes.outputCount]]),
        line("Cumulative outputs by script type", types.map((key) => [label(key), outputs[key].count.cumulative.name]), 7),
        line("Cumulative OP_RETURN output values", [["BTC in OP_RETURN", t.outputs.opReturnValue.cumulative.btc.name]], 7, { format: "btc" }),
        ...events("OP_RETURN outputs by size", (/** @type {const} */ ([["over 42 bytes", "over42Bytes"], ["over 520 bytes", "over520Bytes"], ["over 10,000 bytes", "over10kBytes"]])).map(([name, key]) => [name, t.opReturn.sizes[key].outputCount]), 30, { log: [1] }),
        line("Bytes saved by x-only keys", [["at least", outputs.p2tr.count.cumulative.name]], 1),
      ];
    })()),
    section("Inputs and outputs", [
      line("Native SegWit v0 inputs and outputs", [["inputs", `${daily(inputs.p2wpkh.count)} + ${daily(inputs.p2wsh.count)}`], ["outputs", `${daily(outputs.p2wpkh.count)} + ${daily(outputs.p2wsh.count)}`]], 7),
      ...inAndOut("P2A inputs and outputs", "p2a"),
      ...inAndOut("P2MS inputs and outputs", "p2ms"),
      ...inAndOut("P2PK inputs and outputs", "p2pk"),
      ...inAndOut("P2PKH inputs and outputs", "p2pkh"),
      line("P2SH inputs and outputs (including nested SegWit)", [["P2SH inputs", plainP2sh], ["P2SH, nested P2WPKH, nested P2WSH inputs", daily(inputs.p2sh.count)], ["P2SH outputs", daily(outputs.p2sh.count)]], 7),
      ...inAndOut("P2TR inputs and outputs", "p2tr"),
      ...inAndOut("P2WPKHv0 inputs and outputs", "p2wpkh"),
      ...inAndOut("P2WSHv0 inputs and outputs", "p2wsh"),
      percentLine("Inputs spending UTXOs created in the same block", [["inputs spending newly created UTXOs", percent(daily(t.inputs.sameBlock), day.inputs)]], 7),
      line("Inputs and outputs", [["inputs", day.inputs], ["outputs", day.outputs]], 7),
      ...events("Inputs revealing an Inscription", [["inscription inputs", t.inputs.inscriptions]]),
      line("Inputs per outputs", [["Inputs per Outputs", `${day.inputs} / ${day.outputs}`]], 7),
    ]),
    section("Mining", [
      line("Block time", [["block time", `1440 / ${day.blocks}`]], 7),
      line("Cumulative log2(work)", [["cumulative log2(work)", `ln(${work.name}) / ln(2)`]], 1),
      line("Cumulative work", [["cumulative work", work.name]], 1, { log: [1] }),
      line("Difficulty", [["difficulty", difficulty.name]], 7),
      chart("Hashrate and difficulty", "Hashrate and difficulty", [{ formula: hashrate.name, label: "Hashrate", color: "orange" }, { formula: difficulty.name, label: "Difficulty", color: "sky", pane: 2 }], { log: [1, 2], average: { 1: "7d" } }),
      line("Hashrate", [["hashrate", hashrate.name]], 7),
      line("log2(work) per block", [["log2(work)", `ln(${workPerBlock}) / ln(2)`]], 1),
      line("Proof-of-Work equivalent days", [["PoW-equivalent days", `${work.name} / ${hashrate.name} / 86400`]], 7),
      line("Work per block", [["work", workPerBlock]], 1, { log: [1] }),
    ]),
    section("Mining pools", [
      poolLine("AntPool", pools.antpool),
      poolLine("F2Pool", pools.f2pool),
      poolLine("Foundry", pools.foundryusa),
      poolLine("MARAPool", pools.marapool),
      poolLine("Ocean", pools.ocean),
      poolLine("Unknown", pools.unknown),
      poolLine("ViaBTC", pools.viabtc),
      percentLine("AntPool & friends hashrate", [["AntPool & friends share", ofBlocks(group)]], 7),
      percentLine("AntPool & friends", [["AntPool & friends", group], ...large].map(([name, blocks]) => [name, ofBlocks(blocks)]), 30),
      shareStack("Hashrate distribution", [["AntPool & friends", group], ...large, ["others", rest(day.blocks, [group, ...large.map(([, blocks]) => blocks)])]], 30),
      percentLine("Current top 5 mining pools", (facts?.topPools ?? []).map((/** @type {string} */ slug) => [/** @type {Facts} */ (facts).names.get(slug) ?? slug, ofBlocks(daily(poolOf(slug).blockCount))]), 30),
      centralization("Mining centralization index", pools.concentration),
      centralization("Mining centralization index (with proxy pools)", pools.concentrationGrouped),
    ]),
    section("Payments", (() => {
      const payments = txs.payments;
      return [
        line("Payments per day", [["Payments", daily(payments.count)]], 7),
        line("Payments per transaction", [["Payments per Transaction", `${daily(payments.count)} / ${day.txs}`]], 7),
        percentLine("Payments signaling RBF", [["RBF signaling", percent(daily(payments.rbfSignaling), daily(payments.count))]], 7),
        percentLine("SegWit spending payments", [["Payments spending SegWit", percent(daily(payments.segwitSpending), daily(payments.count))]], 7),
        chart("Payments and block size", "Payments and block size", [{ formula: daily(payments.count), label: "Payments", color: "orange" }, { formula: perBlock(daily(t.blocks.size)), label: "Block size", format: "bytes", color: "sky", pane: 2 }], { average: { 1: "7d", 2: "7d" } }),
      ];
    })()),
    section("Transactions", [
      shareStack("Transaction versions", [["v1", daily(txs.versions.v1)], ["v2", daily(txs.versions.v2)], ["v3", daily(txs.versions.v3)], ["unknown version", daily(txs.versions.other)]], 30),
      ofTxs("BIP-69 compliant transactions", "BIP-69 compliant", features.bip69, 7),
      line("Cumulative transactions", [["Transactions (cumulative)", txs.count.cumulative.name]], 1),
      ofTxs("Height-based time locks", "lock-by-block-height", timelocks.height, 7),
      ofTxs("Share of transactions revealing an Inscription", "inscription transactions", txs.inscription, 7),
      ...events("Transactions revealing inscriptions", [["inscription transactions", txs.inscription]]),
      ofTxs("Transactions with unenforced locktimes", "unenforced locktime", timelocks.unenforced, 7),
      line("Transactions per block", [["Transactions per block", perBlock(day.txs)]], 30),
      line("Transactions per day", [["Transactions per day", day.txs]], 7),
      shareStack("Legacy to SegWit inputs transition", [["legacy only", daily(spending.onlyLegacy)], ["mixed", daily(spending.segwitAndLegacy)], ["SegWit only", daily(spending.onlySegwit)]], 30),
      ofTxs("Explicit RBF signaling transactions", "signaling explicit RBF", features.rbfSignaling, 7),
      line("Transaction size", [["vsize", `${daily(txs.vsize)} / ${day.txs}`], ["size", `${daily(t.blocks.size)} / ${day.txs}`]], 7),
      ofTxs("Transactions spending UTXOs created in the same block", "spending newly created UTXOs", txs.sameBlockSpending, 7),
      percentLine("SegWit spending transactions", [["Transactions spending SegWit", percent(`${daily(spending.onlySegwit)} + ${daily(spending.segwitAndLegacy)}`, day.txs)]], 7),
      ofTxs("Taproot spending transactions", "Transactions spending Taproot", inputs.p2tr.txCount, 7),
      ofTxs("Time-based time locks", "lock-by-block-time", timelocks.time, 1),
      shareStack("Transaction input/output distribution", [["1 in, 1 out", daily(features.oneInputOneOutput)], ["1 in, 2 out", daily(features.oneInputTwoOutput)], ["1 in, N out", `${daily(features.oneInput)} - ${daily(features.oneInputOneOutput)} - ${daily(features.oneInputTwoOutput)}`], ["N in, 1 out", `${daily(features.oneOutput)} - ${daily(features.oneInputOneOutput)}`], ["others", `${day.txs} - ${daily(features.oneInput)} - ${daily(features.oneOutput)} + ${daily(features.oneInputOneOutput)}`]], 7),
      ...events("Ephemeral dust", [["transactions", txs.ephemeralDust]]),
      percentLine("Coinbase locktime set", [["coinbase locktime set", ofBlocks(daily(t.blocks.coinbaseLocktime.nonzero))]], 30),
      percentLine("BIP-54 coinbase locktime set", [["BIP-54 locktime set", ofBlocks(daily(t.blocks.coinbaseLocktime.bip54))]], 7),
      percentLine("BIP-69 compliance and chance", [["BIP-69 compliant", percent(daily(features.bip69), day.txs)], ["expected by chance", percent(daily(features.bip69ExpectedByChance), day.txs)]], 7),
      percentLine("Relative locks", [["block-count locks", percent(daily(timelocks.relativeHeight), day.txs)], ["time locks", percent(daily(timelocks.relativeTime), day.txs)]], 7),
      percentLine("Direct address reuse", [["outputs paying an address being spent", percent(daily(t.addresses.reused.direct.outputCount), day.outputs)]], 7),
    ]),
    section("UTXO set", [
      line("UTXO count", [["UTXO count", t.holders.all.utxos.count.value.name]], 1),
      chart("UTXO set composition", "UTXO set composition", types.map((key, i) => ({ series: utxos[key].utxos.count.value.name, label: label(key), type: "area", color: ramp(types.length)[i] })), { stack: { 1: "stack" } }),
      line("UTXO set delta by script type", types.map((key) => [label(key), perDay(utxos[key].utxos.count.value)]), 7),
      percentLine("UTXO spend age", spendAge.map(([name, total]) => [name, percent(daily(total), day.inputs)]), 30),
    ]),
  ]);
}
