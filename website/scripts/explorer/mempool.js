import { createCube } from "./cube.js";
import { formatFeeRate } from "./render.js";
import { createSpan } from "../utils/dom.js";

const NUM_BLOCKS = 8;

/**
 * @typedef {{
 *   el: HTMLElement,
 *   topFace: HTMLDivElement,
 *   rightFace: HTMLDivElement,
 *   leftFace: HTMLDivElement,
 * }} Cube
 */

/** @type {HTMLDivElement | null} */ let mempoolBlocksEl = null;
/** @type {Cube[]} */ const cubes = [];

/** @param {HTMLElement} parent  the `.chain-scroll` element */
export function initMempool(parent) {
  mempoolBlocksEl = document.createElement("div");
  mempoolBlocksEl.classList.add("mempool-blocks");
  mempoolBlocksEl.hidden = true;
  parent.prepend(mempoolBlocksEl);
}

/** @param {MempoolBlock[]} blocks */
export function renderMempool(blocks) {
  if (!mempoolBlocksEl) return;
  mempoolBlocksEl.hidden = blocks.length === 0;
  const want = Math.min(blocks.length, NUM_BLOCKS);
  while (cubes.length > want) {
    const last = cubes.pop();
    if (last) last.el.remove();
  }
  while (cubes.length < want) {
    const cube = createMempoolCube(cubes.length);
    cubes.push(cube);
    mempoolBlocksEl.append(cube.el);
  }
  for (let i = 0; i < want; i++) updateMempoolCube(cubes[i], blocks[i], i);
}

/** @param {number} position @returns {Cube} */
function createMempoolCube(position) {
  const el = document.createElement("div");
  el.classList.add("cube", "projected");
  if (position === 0) el.classList.add("next");
  const { topFace, rightFace, leftFace } = createCube(el, 0);
  return { el, topFace, rightFace, leftFace };
}

/**
 * @param {Cube} cube
 * @param {MempoolBlock} block
 * @param {number} position
 */
function updateMempoolCube(cube, block, position) {
  const fill = Math.min(1, block.blockVSize / 1_000_000);
  cube.el.style.setProperty("--fill", String(fill));

  cube.topFace.textContent = "";
  const label = document.createElement("p");
  label.textContent = position === 0 ? "next" : `+${position}`;
  cube.topFace.append(label);

  cube.rightFace.textContent = "";
  const txs = document.createElement("p");
  txs.textContent = block.nTx.toLocaleString();
  const txsUnit = document.createElement("p");
  txsUnit.classList.add("dim");
  txsUnit.textContent = block.nTx === 1 ? "tx" : "txs";
  cube.rightFace.append(txs, txsUnit);

  cube.leftFace.textContent = "";
  const median = document.createElement("p");
  const tilde = createSpan("~");
  tilde.classList.add("dim");
  median.append(tilde, formatFeeRate(block.medianFee));
  const range = document.createElement("p");
  const dash = createSpan("-");
  dash.classList.add("dim");
  range.append(formatFeeRate(block.feeRange[0]), dash, formatFeeRate(block.feeRange[6]));
  const unit = document.createElement("p");
  unit.classList.add("dim");
  unit.textContent = "sat/vB";
  cube.leftFace.append(median, range, unit);
}
