import { getBtcParts, satsToUsd } from "../../../btc/index.js";
import { getUsdParts } from "../../../usd/index.js";
import { formatFeeRate } from "../../../utils/fee-rate.js";
import {
  DIFFICULTY_EPOCH_BLOCKS,
  HALVING_EPOCH_BLOCKS,
  formatBlockFill,
  formatBytes,
  formatDateTime,
  formatEpoch,
  formatNumber,
  formatPoolBlockNumber,
  formatWeight,
} from "../format.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */

/**
 * @typedef {Object} ReceiptRow
 * @property {string} label
 * @property {string} value
 * @property {string} [detail]
 * @property {"total" | "wide"} [variant]
 */

/**
 * @typedef {Object} ReceiptSection
 * @property {string} title
 * @property {ReceiptRow[]} rows
 */

/** @param {number} value */
function formatBtc(value) {
  return getBtcParts(value).map(({ text }) => text).join("");
}

/** @param {number} value */
function formatUsd(value) {
  return getUsdParts(value).map(({ text }) => text).join("");
}

/**
 * @param {number} sats
 * @param {number} price
 */
function formatSatsUsd(sats, price) {
  return formatUsd(satsToUsd(sats, price));
}

/** @param {Block} block */
export function createBlockUrl(block) {
  return new URL(`/block/${block.id}`, window.location.origin).href;
}

/** @param {string} url */
export function formatDisplayUrl(url) {
  return url.replace(/^https?:\/\//, "");
}

/**
 * @param {string} label
 * @param {string} value
 * @param {"total" | "wide"} [variant]
 * @returns {ReceiptRow}
 */
function createRow(label, value, variant) {
  return { label, value, ...(variant ? { variant } : {}) };
}

/**
 * @param {string} label
 * @param {string} value
 * @param {string} detail
 * @param {"total"} [variant]
 * @returns {ReceiptRow}
 */
function createAmountRow(label, value, detail, variant) {
  return { label, value, detail, ...(variant ? { variant } : {}) };
}

/**
 * @param {string} title
 * @param {ReceiptRow[]} rows
 * @returns {ReceiptSection}
 */
function createSection(title, rows) {
  return { title, rows };
}

/** @param {Block} block */
export function getReceiptSections(block) {
  const { extras } = block;
  const subsidy = extras.reward - extras.totalFees;

  return [
    createSection("Summary", [
      createRow("Time", formatDateTime(block.timestamp), "wide"),
      createRow("Price", formatUsd(extras.price)),
      createRow("Miner", extras.pool.name),
      createAmountRow(
        "Reward",
        formatBtc(extras.reward),
        formatSatsUsd(extras.reward, extras.price),
        "total",
      ),
      createAmountRow(
        "Fees",
        formatBtc(extras.totalFees),
        formatSatsUsd(extras.totalFees, extras.price),
      ),
      createRow("Tx", formatNumber(block.txCount)),
      createRow(
        "Size",
        `${formatBytes(block.size)} · ${formatBlockFill(block.weight)}`,
      ),
    ]),
    createSection("Reward split", [
      createAmountRow(
        "Subsidy",
        formatBtc(subsidy),
        formatSatsUsd(subsidy, extras.price),
      ),
      createAmountRow(
        "Fees",
        formatBtc(extras.totalFees),
        formatSatsUsd(extras.totalFees, extras.price),
      ),
    ]),
    createSection("Block", [
      createRow("Weight", formatWeight(block.weight)),
      createRow("Virtual", `${formatNumber(extras.virtualSize)} vB`),
      createRow("Avg tx", formatBytes(extras.avgTxSize)),
      createRow("Input", formatNumber(extras.totalInputs)),
      createRow("Output", formatNumber(extras.totalOutputs)),
    ]),
    createSection("Fees", [
      createRow("Median", `${formatFeeRate(extras.medianFee)} sat/vB`),
      createRow("Average", `${formatFeeRate(extras.avgFeeRate)} sat/vB`),
      createRow(
        "Range",
        `${formatFeeRate(extras.feeRange[0])} - ${formatFeeRate(extras.feeRange[6])} sat/vB`,
      ),
      createRow("Avg fee", `${formatNumber(extras.avgFee)} sat`),
      createRow("Median fee", `${formatNumber(extras.medianFeeAmt)} sat`),
    ]),
    createSection("Mining", [
      createRow("Pool slug", extras.pool.slug),
      createRow("Pool block", formatPoolBlockNumber(extras.pool.blockNumber)),
      createRow("Difficulty", formatNumber(block.difficulty), "wide"),
      createRow(
        "Diff epoch",
        formatEpoch(block.height, DIFFICULTY_EPOCH_BLOCKS),
      ),
      createRow("Halving", formatEpoch(block.height, HALVING_EPOCH_BLOCKS)),
    ]),
    createSection("Verify", [
      createRow("Height", `#${formatNumber(block.height)}`),
      createRow("Hash", block.id, "wide"),
      createRow("Previous", block.previousblockhash, "wide"),
    ]),
  ];
}
