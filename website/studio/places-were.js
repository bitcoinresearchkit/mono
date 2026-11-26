// @ts-check
// Where presets were before they moved: each row a place then and the place it is now, a chart's or a section's (and
// so of everything under it). A "*" in a step stands for any words there, the same on both sides, in order. A chart
// folded into another opens that one at a window: "#window=" and its short name after the place now. Old addresses
// keep opening them (moved.txt, made from these by the site build), and charts saved from them find them
// (adoptPresets); a row is kept for good.
// Rows are only ever added, at the end: an older place is followed through the rows before its own.

const CS = "Network › Chain statistics";
/** @type {[string, string][]} */
const PLACES_WERE = [
  ["Price › All-time high › Drawdown", "Price › All-time high, drawdown"],
  ["Price › All-time high › Days since", "Price › All-time high, days since"],
  ["Price › Moving averages › SMA", "Price › Moving averages, SMA"],
  ["Price › Moving averages › EMA", "Price › Moving averages, EMA"],
  ["Price › Range › High and low", "Price › Range"],
  ["Valuation › On-chain prices", "Valuation › On-chain prices › Compare"],
  ["Valuation › Cost basis clouds", "Valuation › On-chain prices › Cost basis clouds"],
  ["Valuation › Compass", "Valuation › On-chain prices › Compass"],
  ["Valuation › Cointime caps", "Valuation › On-chain prices › Cointime caps"],
  ["Valuation › Cointime prices", "Valuation › On-chain prices › Cointime prices"],
  ["Valuation › Rarity meter › * › Bands", "Valuation › Rarity meter › *, bands"],
  ["Valuation › Rarity meter › * › Level and score", "Valuation › Rarity meter › *, level"],
  ["Valuation › Rarity meter › Components › * › * › Bands", "Valuation › Rarity meter › Components › * › *, bands"],
  ["Valuation › Rarity meter › Components › * › * › Ratios", "Valuation › Rarity meter › Components › * › *, ratios"],
  ["Valuation › Rarity meter › Components › * › * › * › Bands", "Valuation › Rarity meter › Components › * › * › *, bands"],
  ["Valuation › Rarity meter › Components › * › * › * › Ratios", "Valuation › Rarity meter › Components › * › * › *, ratios"],
  ["Bedrock", "Valuation › Bedrock"],
  ["Valuation › Cumulative destruction", "Valuation › On-chain prices › Cumulative destruction"],
  ["Valuation › Delta price", "Valuation › On-chain prices › Delta price"],
  // The whole market at the top of its subjects; its totals and NUPL merged into the charts that were already there.
  ["Holders › All coins › Supply › Total", "Supply › Circulating"],
  ["Holders › All coins › Supply › In profit and loss", "Supply › In profit and loss"],
  ["Holders › All coins › Supply › Share in profit", "Supply › Share in profit"],
  ["Holders › All coins › Supply › Change", "Supply › Change"],
  ["Holders › All coins › Capital › Total", "Valuation › Market and realized cap"],
  ["Holders › All coins › Capital", "Valuation › Realized cap"],
  ["Holders › All coins › Cost basis", "Valuation › Cost basis"],
  ["Holders › All coins › Unrealized › NUPL", "Valuation › NUPL"],
  ["Holders › All coins › Realized › Profit and loss", "Profit and loss › Realized profit and loss"],
  ["Holders › All coins › Realized › *", "Profit and loss › *"],
  ["Holders › All coins › Unrealized › Profit and loss", "Profit and loss › Unrealized profit and loss"],
  ["Holders › All coins › Unrealized › Relative to market cap", "Profit and loss › Relative to market cap"],
  ["Holders › All coins › UTXOs › Change", "Activity › UTXO change"],
  ["Holders › All coins › UTXOs › Spent", "Activity › UTXOs spent"],
  ["Holders › All coins › Activity › *", "Activity › *"],
  ["Supply › Velocity", "Activity › Velocity"],
  ["Coin age › Last moved", "Supply › Last moved"],
  ["Cointime › Supply", "Supply › Vaulted and active"],
  ["Cointime › Activity", "Activity › Cointime"],
  ["Cointime › Value › Value *", "Activity › Cointime › Cointime value *"],
  // Fees, a subject of their own, from the miners' rewards and the transactions'.
  ["Mining › Rewards › Fees", "Fees › Fees paid"],
  ["Mining › Rewards › Fees per block", "Fees › Fees per block"],
  ["Mining › Rewards › Fees per block in dollars", "Fees › Fees per block, in dollars"],
  ["Network › Transactions › Fee rate", "Fees › Fee rate"],
  ["Network › Transactions › Fee rate and fullness", "Fees › Fee rate and fullness"],
  ["Network › Transactions › By fee rate", "Fees › By fee rate"],
  ["Network › Transactions › Block space by effective fee rate", "Fees › Block space by effective fee rate"],
  ["Network › Transactions › Fee", "Fees › Fee per transaction"],
  ["Network › Transactions › Volume", "Activity › Transfer volume"],
  ["Network › Transactions › CPFP › Parents", "Network › Transactions › CPFP, parents"],
  ["Network › Transactions › CPFP › Children", "Network › Transactions › CPFP, children"],
  ["Network › Outputs › Coinbase › Count", "Network › Outputs › Coinbase, count"],
  ["Network › Outputs › Coinbase › By type", "Network › Outputs › Coinbase, by type"],
  // Chain statistics, once a section of their own: each now beside the native charts of its subject, or one of them
  // at a window.
  [`${CS} › Yearly › Transaction fees per year`, "Fees › Fees paid#window=1y"],
  [`${CS} › Yearly › Empty blocks per year`, "Network › Blocks › Empty#window=1y"],
  [`${CS} › Yearly › Blocks per year`, "Network › Blocks › Count#window=1y"],
  [`${CS} › Yearly › Transactions per year`, "Network › Transactions › Count#window=1y"],
  [`${CS} › Bitcoin script › S-values in ECDSA signatures`, "Network › Inputs › Signatures › ECDSA › Low S#window=24h"],
  [`${CS} › Bitcoin script › Length of ECDSA signatures`, "Network › Inputs › Signatures › ECDSA › By length › Compare#window=24h"],
  [`${CS} › Bitcoin script › r and s-values in Bitcoin ECDSA signatures`, "Network › Inputs › Signatures › ECDSA › Low R and low S#window=24h"],
  [`${CS} › Bitcoin script › r-values in ECDSA signatures`, "Network › Inputs › Signatures › ECDSA › Low R#window=24h"],
  [`${CS} › Bitcoin script › DER encoded ECDSA signatures`, "Network › Inputs › Signatures › ECDSA › Strict DER#window=24h"],
  [`${CS} › Bitcoin script › SigHash ALL, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_ALL#window=30d"],
  [`${CS} › Bitcoin script › SigHash ALL | ACP, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_ALL|SIGHASH_ANYONECANPAY#window=30d"],
  [`${CS} › Bitcoin script › SigHash NONE | ACP, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_NONE|SIGHASH_ANYONECANPAY#window=30d"],
  [`${CS} › Bitcoin script › SigHash SINGLE | ACP, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_SINGLE|SIGHASH_ANYONECANPAY#window=30d"],
  [`${CS} › Bitcoin script › SigHash NONE, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_NONE#window=30d"],
  [`${CS} › Bitcoin script › SigHash SINGLE, by window`, "Network › Inputs › Signatures › ECDSA › By sighash type › SIGHASH_SINGLE#window=30d"],
  [`${CS} › Bitcoin script › Total signature count`, "Network › Inputs › Signatures › Compare › Signatures#window=all"],
  [`${CS} › Bitcoin script › Signature count per day`, "Network › Inputs › Signatures › Compare › Signatures#window=24h"],
  [`${CS} › Bitcoin script › Total ECDSA and Schnorr signature size`, "Network › Inputs › Signatures › Compare › Bytes#window=all"],
  [`${CS} › Bitcoin script › Signature checks by opcode, by window`, "Network › Inputs › Opcodes › Compare#window=30d"],
  [`${CS} › Bitcoin script › Timelocks by opcode, by window`, "Network › Inputs › Opcodes › Compare#window=30d"],
  [`${CS} › Bitcoin script › Hashes by opcode, by window`, "Network › Inputs › Opcodes › Compare#window=30d"],
  [`${CS} › Bitcoin script › Branches by opcode, by window`, "Network › Inputs › Opcodes › Compare#window=30d"],
  [`${CS} › Blocks › Blockchain growth per day`, "Network › Blocks › Size#window=24h"],
  [`${CS} › Blocks › Blockchain size`, "Network › Blocks › Size#window=all"],
  [`${CS} › Blocks › Empty blocks, by window`, "Network › Blocks › Empty#window=30d"],
  [`${CS} › Blocks › Blocks per day`, "Network › Blocks › Count#window=24h"],
  [`${CS} › Blocks › BIP-110 signaling blocks, by window`, "Network › Blocks › Version bit 4#window=30d"],
  [`${CS} › Blocks › Cumulative unclaimed coins`, "Mining › Rewards › Unclaimed#window=all"],
  [`${CS} › Fees › Zero-Fee transactions, by window`, "Fees › By fee rate › Zero fee#window=30d"],
  [`${CS} › Fees › Transaction fee rate distribution`, "Fees › By fee rate › Compare › Transactions#window=24h"],
  [`${CS} › Fees › Fee rates by block space`, "Fees › Block space by effective fee rate › Compare › Block space (vsize)#window=24h"],
  [`${CS} › Inputs › Cumulative inputs`, "Network › Inputs › Count#window=all"],
  [`${CS} › Inputs › Nested P2WPKH inputs, by window`, "Network › Inputs › Nested SegWit › Compare › Nested SegWit inputs#window=30d"],
  [`${CS} › Inputs › Nested P2WSH inputs, by window`, "Network › Inputs › Nested SegWit › Compare › Nested SegWit inputs#window=30d"],
  [`${CS} › Inputs › P2TR script-path spends`, "Network › Inputs › Taproot script path#window=24h"],
  [`${CS} › Inputs › Cumulative inputs by script type`, "Network › Inputs › By type › Compare › Inputs#window=all"],
  [`${CS} › Inputs › Input types by count`, "Network › Inputs › By type › Compare › Inputs#window=24h"],
  [`${CS} › Outputs › Coinbase output types by count`, "Network › Outputs › Coinbase, by type › Compare › Coinbase outputs#window=24h"],
  [`${CS} › Outputs › OP_RETURN outputs, by window`, "Network › OP_RETURN › Outputs#window=30d"],
  [`${CS} › Outputs › Output types by value`, "Network › Outputs › By type › Compare › Value#window=24h"],
  [`${CS} › Outputs › Output types by count`, "Network › Outputs › By type › Compare › Outputs#window=24h"],
  [`${CS} › Outputs › Cumulative BIP47 notifications`, "Network › OP_RETURN › Protocols › BIP-47 › Outputs#window=all"],
  [`${CS} › Outputs › Cumulative OP_RETURN data bytes`, "Network › OP_RETURN › Data#window=all"],
  [`${CS} › Outputs › Cumulative outputs`, "Network › Outputs › Count#window=all"],
  [`${CS} › Outputs › OP_RETURN data bytes`, "Network › OP_RETURN › Data#window=24h"],
  [`${CS} › Outputs › OmniLayer (OP_RETURN), by window`, "Network › OP_RETURN › Protocols › Omni › Outputs#window=30d"],
  [`${CS} › Outputs › Stacks block commit (OP_RETURN), by window`, "Network › OP_RETURN › Protocols › Stacks › Outputs#window=30d"],
  [`${CS} › Outputs › P2TR outputs, by window`, "Network › Outputs › By type › P2TR › Count#window=30d"],
  [`${CS} › Outputs › Cumulative Runestones (OP_RETURN)`, "Network › OP_RETURN › Protocols › Runes › Outputs#window=all"],
  [`${CS} › Outputs › Runestone (OP_RETURN), by window`, "Network › OP_RETURN › Protocols › Runes › Outputs#window=30d"],
  [`${CS} › Outputs › Cumulative outputs by script type`, "Network › Outputs › By type › Compare › Outputs#window=all"],
  [`${CS} › Outputs › Cumulative OP_RETURN output values`, "Network › OP_RETURN › Value#window=all"],
  [`${CS} › Outputs › OP_RETURN outputs by size, by window`, "Network › OP_RETURN › Sizes › Compare"],
  [`${CS} › Outputs › Bytes saved by x-only keys`, "Network › Outputs › By type › P2TR › Count#window=all"],
  [`${CS} › Inputs and outputs › Inputs revealing an Inscription, by window`, "Network › Inputs › Inscriptions#window=30d"],
  [`${CS} › Mining › Block time`, "Network › Blocks › Interval"],
  [`${CS} › Mining › Cumulative work`, "Mining › Hashing › Block work#window=all"],
  [`${CS} › Mining › Difficulty`, "Mining › Difficulty › History"],
  [`${CS} › Mining › Hashrate`, "Mining › Hashing › Hash rate"],
  [`${CS} › Mining pools › AntPool hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › F2Pool hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › Foundry hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › MARAPool hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › Ocean hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › Unknown hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › ViaBTC hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › AntPool & friends hashrate`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › AntPool & friends`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › Hashrate distribution`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Mining pools › Current top 5 mining pools`, "Mining › Pools › Compare › Share#window=24h"],
  [`${CS} › Payments › Payments per day`, "Network › Transactions › Payments › Count#window=24h"],
  [`${CS} › Transactions › Transaction versions`, "Network › Transactions › Versions › Compare › Transactions#window=24h"],
  [`${CS} › Transactions › Cumulative transactions`, "Network › Transactions › Count#window=all"],
  [`${CS} › Transactions › Transactions revealing inscriptions, by window`, "Network › Transactions › Inscriptions › Count#window=30d"],
  [`${CS} › Transactions › Transactions per block`, "Network › Transactions › Count per block"],
  [`${CS} › Transactions › Transactions per day`, "Network › Transactions › Count#window=24h"],
  [`${CS} › Transactions › Legacy to SegWit inputs transition`, "Network › Transactions › Spending › Compare › Transactions#window=24h"],
  [`${CS} › Transactions › SegWit spending transactions`, "Network › Transactions › Spending › Compare › Share#window=24h"],
  [`${CS} › Transactions › Taproot spending transactions`, "Network › Inputs › By type › Compare › Transaction share#window=24h"],
  [`${CS} › Transactions › Ephemeral dust, by window`, "Network › Transactions › Features › Spending ephemeral dust#window=30d"],
  [`${CS} › UTXO set › UTXO count`, "UTXOs › Count"],
  [`${CS} › UTXO set › UTXO set composition`, "UTXOs › By type › Compare › UTXO count"],
  [`${CS} › Mining › Cumulative log2(work)`, "Mining › Hashing › Chainwork, log2"],
  [`${CS} › Mining › log2(work) per block`, "Mining › Hashing › Work per block, log2"],
  [`${CS} › Mining › Proof-of-Work equivalent days`, "Mining › Hashing › Chainwork, in days at each day's hash rate"],
  [`${CS} › Mining › Work per block`, "Mining › Hashing › Work per block"],
  [`${CS} › Mining › Hashrate and difficulty`, "Mining › Difficulty › Hash rate and difficulty"],
  [`${CS} › Blocks › Coinbase output value`, "Mining › Rewards › Reward per block, subsidy and fees"],
  [`${CS} › Mining pools › Mining centralization index`, "Mining › Pools › Centralization, top pools' share"],
  [`${CS} › Mining pools › Mining centralization index (with proxy pools)`, "Mining › Pools › Centralization, AntPool group as one"],
  [`${CS} › Fees › Sub 1 sat/vByte transactions`, "Fees › By fee rate › Under 1 sat/vB, share of transactions"],
  [`${CS} › Fees › Fee rate in dollars`, "Fees › Median fee rate, in dollars"],
  [`${CS} › Transactions › Coinbase locktime set`, "Network › Blocks › Coinbase locktime › Nonzero, share"],
  [`${CS} › Transactions › BIP-54 coinbase locktime set`, "Network › Blocks › Coinbase locktime › BIP-54, share"],
  [`${CS} › Blocks › Sigops per block`, "Network › Blocks › Signature operations per block"],
  [`${CS} › Blocks › Block size`, "Network › Blocks › Size per block, average"],
  [`${CS} › Blocks › Block weight`, "Network › Blocks › Weight per block, average"],
  [`${CS} › Payments › Payments per transaction`, "Network › Transactions › Payments › Per transaction"],
  [`${CS} › Payments › Payments signaling RBF`, "Network › Transactions › Payments › RBF signaling, share"],
  [`${CS} › Payments › SegWit spending payments`, "Network › Transactions › Payments › SegWit inputs, share"],
  [`${CS} › Payments › Payments and block size`, "Network › Transactions › Payments › Against block size"],
  [`${CS} › Transactions › Explicit RBF signaling transactions`, "Network › Transactions › Features › RBF signaling › Share"],
  [`${CS} › Transactions › BIP-69 compliant transactions`, "Network › Transactions › Features › BIP-69, share"],
  [`${CS} › Transactions › Transactions spending UTXOs created in the same block`, "Network › Transactions › Features › Spending same-block outputs, share"],
  [`${CS} › Transactions › BIP-69 compliance and chance`, "Network › Transactions › Features › BIP-69, share against chance"],
  [`${CS} › Transactions › Height-based time locks`, "Network › Transactions › Timelocks › Height-based absolute, share"],
  [`${CS} › Transactions › Transactions with unenforced locktimes`, "Network › Transactions › Timelocks › Unenforced, share"],
  [`${CS} › Transactions › Time-based time locks`, "Network › Transactions › Timelocks › Time-based absolute, share"],
  [`${CS} › Transactions › Relative locks`, "Network › Transactions › Timelocks › Relative, share"],
  [`${CS} › Transactions › Share of transactions revealing an Inscription`, "Network › Transactions › Inscriptions › Share of transactions"],
  [`${CS} › Transactions › Transaction size`, "Network › Transactions › Size, average"],
  [`${CS} › Transactions › Transaction input/output distribution`, "Network › Transactions › By inputs and outputs"],
  [`${CS} › Bitcoin script › Public key compression (ECDSA)`, "Network › Inputs › Signatures › Public keys, compressed and uncompressed"],
  [`${CS} › Bitcoin script › Signature verifications per block`, "Network › Inputs › Signatures › Verifications per block"],
  [`${CS} › Inputs and outputs › Inputs spending UTXOs created in the same block`, "Network › Inputs › Spent within › Same block, share of inputs"],
  [`${CS} › UTXO set › UTXO spend age`, "Network › Inputs › Spent within › Share of inputs"],
  [`${CS} › Blocks › Coinbases with a witness`, "Network › Inputs › Coinbase witness, share of blocks"],
  [`${CS} › Inputs › Multisig input types`, "Network › Inputs › Multisig, by type"],
  [`${CS} › Inputs › P2SH input type distribution`, "Network › Inputs › P2SH, plain and nested SegWit"],
  [`${CS} › Inputs › P2TR key-path spends`, "Network › Inputs › Taproot key path"],
  [`${CS} › Inputs › P2TR key- and script-path spends`, "Network › Inputs › Taproot key and script path"],
  [`${CS} › Inputs › Inputs per transaction`, "Network › Inputs › Per transaction"],
  [`${CS} › Inputs › SegWit input types`, "Network › Inputs › SegWit, by type"],
  [`${CS} › Inputs › Inputs spending multisig (excl. Taproot)`, "Network › Inputs › Multisig, share of inputs"],
  [`${CS} › Outputs › Outputs per Coinbase transaction`, "Network › Outputs › Coinbase outputs per block"],
  [`${CS} › Outputs › Outputs per transaction`, "Network › Outputs › Per transaction"],
  [`${CS} › Inputs and outputs › Native SegWit v0 inputs and outputs`, "Network › Inputs and outputs › Native SegWit v0, created and spent"],
  [`${CS} › Inputs and outputs › P2A inputs and outputs, by window`, "Network › Inputs and outputs › P2A, created and spent"],
  [`${CS} › Inputs and outputs › P2MS inputs and outputs, by window`, "Network › Inputs and outputs › P2MS, created and spent"],
  [`${CS} › Inputs and outputs › P2PK inputs and outputs, by window`, "Network › Inputs and outputs › P2PK, created and spent"],
  [`${CS} › Inputs and outputs › P2PKH inputs and outputs, by window`, "Network › Inputs and outputs › P2PKH, created and spent"],
  [`${CS} › Inputs and outputs › P2SH inputs and outputs (including nested SegWit)`, "Network › Inputs and outputs › P2SH, created and spent"],
  [`${CS} › Inputs and outputs › P2TR inputs and outputs, by window`, "Network › Inputs and outputs › P2TR, created and spent"],
  [`${CS} › Inputs and outputs › P2WPKHv0 inputs and outputs, by window`, "Network › Inputs and outputs › P2WPKH, created and spent"],
  [`${CS} › Inputs and outputs › P2WSHv0 inputs and outputs, by window`, "Network › Inputs and outputs › P2WSH, created and spent"],
  [`${CS} › Inputs and outputs › Inputs and outputs`, "Network › Inputs and outputs › Count"],
  [`${CS} › Inputs and outputs › Inputs per outputs`, "Network › Inputs and outputs › Inputs per output"],
  [`${CS} › UTXO set › UTXO set delta by script type`, "UTXOs › By type › Compare › Change per day"],
  [`${CS} › Transactions › Direct address reuse`, "Addresses › Reused › Direct reuse, share"],
  // A cohort's Activity of transfer volume alone, lifted.
  ["UTXOs › * › * › Activity › Transfer volume", "UTXOs › * › * › Activity, transfer volume"],
  ["UTXOs › * › * › Activity › Transfer volume share", "UTXOs › * › * › Activity, transfer volume share"],
  ["Addresses › * › * › Activity › Transfer volume", "Addresses › * › * › Activity, transfer volume"],
  ["Addresses › * › * › Activity › Transfer volume share", "Addresses › * › * › Activity, transfer volume share"],
  // Holders hold every grouping of coins: by age, profit, wakefulness and mobility too.
  ["Coin age › Age bands", "Holders › Age bands"],
  ["Coin age › Under an age", "Holders › Under an age"],
  ["Coin age › Over an age", "Holders › Over an age"],
  ["Coin age › Halving epochs", "Holders › Halving epochs"],
  ["Coin age › Vintages", "Holders › Vintages"],
  ["Holders › Age cutoffs › Under *", "Holders › Under an age › Under *"],
  ["Holders › Age cutoffs › Over *", "Holders › Over an age › Over *"],
  ["Holders › Age cutoffs › Compare", "Holders › Over an age › Compare"],
  ["Profitability", "Holders › Profit bands"],
  ["Cointime › Awake coins", "Holders › Awake coins"],
  ["Coinflow › Mobile coins", "Holders › Mobile coins"],
  ["Cointime › Age bands › Compare", "Holders › Age bands › Compare"],
  ["Coinflow › Age bands › Compare", "Holders › Age bands › Compare"],
  // (Before the bands: Compare isn't one of them.)
  ["Cointime › Age bands › Compare › *", "Holders › Age bands › Compare › *"],
  ["Coinflow › Age bands › Compare › *", "Holders › Age bands › Compare › *"],
  ["Cointime › Age bands › * › Coin days consumed", "Holders › Age bands › * › Activity › Coin days consumed"],
  ["Cointime › Age bands › * › Wakefulness", "Holders › Age bands › * › Activity › Wakefulness"],
  ["Cointime › Age bands › * › Awake and dormant", "Holders › Age bands › * › Activity › Awake and dormant"],
  ["Coinflow › Age bands › * › Spending rate", "Holders › Age bands › * › Activity › Spending rate"],
  ["Coinflow › Age bands › * › Spending exposure", "Holders › Age bands › * › Activity › Spending exposure"],
  ["Coinflow › Age bands › * › Mobility", "Holders › Age bands › * › Activity › Mobility"],
  ["Coinflow › Age bands › * › Mobile and immobile", "Holders › Age bands › * › Activity › Mobile and immobile"],
  // A folder of two lifted (presets.js lift).
  ["Holders › Profit bands › * › * › Supply › Total", "Holders › Profit bands › * › * › Supply, total"],
  ["Holders › Profit bands › * › * › Supply › Share of *", "Holders › Profit bands › * › * › Supply, share of *"],
  ["Holders › Profit bands › * › * › Capital › Total", "Holders › Profit bands › * › * › Capital, total"],
  ["Holders › Profit bands › * › * › Capital › Share of *", "Holders › Profit bands › * › * › Capital, share of *"],
  ["Holders › Profit bands › * › * › * › Supply › Total", "Holders › Profit bands › * › * › * › Supply, total"],
  ["Holders › Profit bands › * › * › * › Supply › Share of *", "Holders › Profit bands › * › * › * › Supply, share of *"],
  ["Holders › Profit bands › * › * › * › Capital › Total", "Holders › Profit bands › * › * › * › Capital, total"],
  ["Holders › Profit bands › * › * › * › Capital › Share of *", "Holders › Profit bands › * › * › * › Capital, share of *"],
  // A member's realized price first, at its root.
  ["Holders › Short-term holders › Cost basis › Realized price", "Holders › Short-term holders › Realized price"],
  ["Holders › Long-term holders › Cost basis › Realized price", "Holders › Long-term holders › Realized price"],
  ["Holders › Age bands › * › Cost basis › Realized price", "Holders › Age bands › * › Realized price"],
  ["Holders › Under an age › * › Cost basis › Realized price", "Holders › Under an age › * › Realized price"],
  ["Holders › Over an age › * › Cost basis › Realized price", "Holders › Over an age › * › Realized price"],
  ["Holders › Awake coins › * › Cost basis › Realized price", "Holders › Awake coins › * › Realized price"],
  ["Holders › Awake coins › Age cutoffs › * › Cost basis › Realized price", "Holders › Awake coins › Age cutoffs › * › Realized price"],
  ["Holders › Mobile coins › * › Cost basis › Realized price", "Holders › Mobile coins › * › Realized price"],
  ["Holders › Mobile coins › Age cutoffs › * › Cost basis › Realized price", "Holders › Mobile coins › Age cutoffs › * › Realized price"],
  // Each subject its own preset: a pair split in two.
  ["Mining › Hashing › Hash price and value", "Mining › Hashing › Hash price"],
  // Entry price, said plainly.
  ["Holders › Entry price › Discount coins", "Holders › Bought at a discount or premium › Bought at a discount"],
  ["Holders › Entry price › Premium coins", "Holders › Bought at a discount or premium › Bought at a premium"],
  ["Holders › Entry price", "Holders › Bought at a discount or premium"]
];

/** @typedef {{ steps: string[], wild: number, row: number, window?: string }} Side */
/** @param {string} written @param {number} row @returns {Side} */
const side = (written, row) => {
  const [place, state] = written.split("#"), steps = place.split(" › "), window = state?.match(/^window=(\w+)$/)?.[1];
  if (state !== undefined && !window) throw new Error(`places-were.js: a state other than a window, ${written}`);
  return { steps, wild: steps.filter((step) => step.includes("*")).length, row, ...(window ? { window } : {}) };
};
/** @typedef {{ named: Map<string, Step>, wild: [string, Step][], rows: [Side, Side][] }} Step */
/** The rows as a tree of their first side's steps: a place is looked up in as many steps as it has, not row by row. @param {[Side, Side][]} pairs */
const tree = (pairs) => {
  const root = /** @type {Step} */ ({ named: new Map(), wild: [], rows: [] });
  for (const pair of pairs) {
    let at = root;
    for (const step of pair[0].steps) {
      const known = step.includes("*") ? at.wild.find(([wild]) => wild === step)?.[1] : at.named.get(step), next = known ?? { named: new Map(), wild: [], rows: [] };
      if (!known) step.includes("*") ? at.wild.push([step, next]) : at.named.set(step, next);
      at = next;
    }
    at.rows.push(pair);
  }
  return root;
};
const sides = PLACES_WERE.map(([was, now], row) => [side(was, row), side(now, row)]);
const toWas = tree(sides.map(([was, now]) => [now, was])), toNow = tree(sides.map(([was, now]) => [was, now]));
/** @typedef {{ path: string[], row: number, window?: string }} Place */
/**
 * Each row's other side for `path`, of the rows `taken` (all of them by default), the most specific first (the longest,
 * then the fewest wildcards, then the first written), with the window a folded chart opens at.
 * @param {Step} root @param {string[]} path @param {(row: number) => boolean} [taken] @returns {Place[]}
 */
const across = (root, path, taken = () => true) => {
  const found = /** @type {{ from: Side, to: Side, words: string[] }[]} */ ([]);
  /** @param {Step} at @param {number} depth @param {string[]} words */
  const walk = (at, depth, words) => {
    for (const [from, to] of at.rows) if (taken(from.row)) found.push({ from, to, words });
    if (depth === path.length) return;
    const named = at.named.get(path[depth]), step = path[depth];
    if (named) walk(named, depth + 1, words);
    for (const [wild, next] of at.wild) {
      const star = wild.indexOf("*"), tail = wild.length - star - 1;
      if (step.length >= wild.length - 1 && step.startsWith(wild.slice(0, star)) && step.endsWith(wild.slice(star + 1))) walk(next, depth + 1, [...words, step.slice(star, step.length - tail)]);
    }
  };
  walk(root, 0, []);
  return found
    .sort((a, b) => b.from.steps.length - a.from.steps.length || a.from.wild - b.from.wild || a.from.row - b.from.row)
    .map(({ from, to, words }) => ((left) => ({ path: [...to.steps.map((step) => (step.includes("*") ? step.replace("*", /** @type {string} */ (left.shift())) : step)), ...path.slice(from.steps.length)], row: from.row, ...(from.window ? { window: from.window } : {}) }))([...words]));
};
/**
 * The places a place was, a merged or folded one's too, and those before them (a row's old place moved by an older
 * row: rows are kept in the order they were written), the nearest first; none if it never moved.
 * @param {string[]} path
 */
export const placeWas = (path) => {
  const found = across(toWas, path);
  for (let i = 0; i < found.length; i++) for (const before of across(toWas, found[i].path, (row) => row < found[i].row)) found.push(found[i].window ? { ...before, window: found[i].window } : before);
  return found;
};
/** A place moved since, where it is now, through each row written after the one that moved it; null if it never moved. @param {string[]} path */
export const placeNow = (path) => {
  let now = across(toNow, path)[0];
  for (let next = now; next; next = across(toNow, next.path, (row) => row > next.row)[0]) now = next;
  return now?.path ?? null;
};
