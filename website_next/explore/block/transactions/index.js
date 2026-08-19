import { formatBlockFill, formatBytes, formatNumber } from "../format.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */

/**
 * @param {string} label
 * @param {string | Node} value
 */
function createBlockMetaItem(label, value) {
  const item = document.createElement("div");
  const name = document.createElement("span");
  const data = document.createElement("strong");

  name.textContent = label;
  data.append(value);
  item.append(name, data);

  return item;
}

/** @param {Block} block */
export function createTransactionPane(block) {
  const { extras } = block;
  const meta = document.createElement("div");

  meta.dataset.blockMeta = "";
  meta.append(
    createBlockMetaItem(
      "Block",
      `${formatBytes(block.size)} · ${formatBlockFill(block.weight)}`,
    ),
    createBlockMetaItem("Tx", formatNumber(block.txCount)),
    createBlockMetaItem("Input", formatNumber(extras.totalInputs)),
    createBlockMetaItem("Output", formatNumber(extras.totalOutputs)),
  );

  return meta;
}
