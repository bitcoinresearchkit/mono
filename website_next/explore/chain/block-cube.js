import { createPoolLogo, getPoolDisplayName } from "../../pools/index.js";
import { formatFeeRate } from "../../utils/fee-rate.js";
import { createCubeButton, createCubeDiv } from "./cube/index.js";
import { onPlainClick } from "./events.js";
import {
  createHeightElement,
  dim,
  formatBtc,
  formatHeight,
  formatInterval,
  formatMegabytes,
  formatNumber,
  formatShortDate,
  formatTime,
  formatUtcOffset,
  formatYear,
} from "./format.js";

/** @typedef {import("../../modules/bitview-client/index.js").BlockInfoV1} Block */
/** @typedef {import("../../modules/bitview-client/index.js").MempoolBlock} MempoolBlock */

const CUBE_ENTER_STAGGER_MS = 60;

export function createPlaceholderCube() {
  const cube = document.createElement("div");

  cube.dataset.cube = "block";
  cube.dataset.placeholder = "";

  return cube;
}

/**
 * @param {Block} block
 * @param {(cube: HTMLButtonElement) => void} onOpen
 * @param {number} [enterIndex]
 */
export function createEnteringConfirmedCube(block, onOpen, enterIndex = 0) {
  const cube = createConfirmedCube(block, onOpen);

  markCubeEntering(cube, enterIndex);

  return cube;
}

/**
 * @param {Block} block
 * @param {(cube: HTMLButtonElement) => void} onOpen
 */
function createConfirmedCube(block, onOpen) {
  const { pool, totalFees, medianFee, feeRange, virtualSize } = block.extras;
  const cube = createCubeButton(Math.min(1, virtualSize / 1_000_000));

  cube.element.dataset.hash = block.id;
  cube.element.dataset.height = String(block.height);
  cube.element.dataset.timestamp = String(block.timestamp);
  cube.element.title = `Block ${formatNumber(block.height)}`;
  onPlainClick(cube.element, () => onOpen(cube.element));

  const date = document.createElement("p");
  const year = dim(formatYear(block.timestamp));
  const time = document.createElement("p");
  const utcOffset = dim(formatUtcOffset(block.timestamp));
  const [hh, mm, ss] = formatTime(block.timestamp);
  date.append(formatShortDate(block.timestamp));
  time.append(hh, dim(":"), mm, dim(":"), ss);
  cube.topFace.append(date, year, time, utcOffset);

  const height = document.createElement("p");
  height.dataset.cubeHeight = "";
  height.append(createHeightElement(block.height));

  const poolElement = document.createElement("div");
  const logo = createPoolLogo(pool);
  const name = document.createElement("span");
  const txs = document.createElement("p");
  const size = document.createElement("p");
  txs.append(
    formatNumber(block.txCount),
    " ",
    dim(block.txCount === 1 ? "tx" : "txs"),
  );
  size.append(formatMegabytes(block.size), " ", dim("MB"));
  poolElement.dataset.cubePool = "";
  name.textContent = getPoolDisplayName(pool.name);
  poolElement.append(logo, name);
  cube.rightFace.append(height, txs, size, poolElement);

  const total = document.createElement("p");
  const median = document.createElement("p");
  const range = document.createElement("p");
  const unit = document.createElement("p");
  total.append(dim("₿"), formatBtc(totalFees));
  median.append(dim("~"), formatFeeRate(medianFee));
  range.append(
    formatFeeRate(feeRange[0]),
    dim("-"),
    formatFeeRate(feeRange[6]),
  );
  unit.dataset.dim = "";
  unit.textContent = "sat/vB";
  cube.leftFace.append(total, median, range, unit);

  return cube.element;
}

/** @param {HTMLElement} cube @param {number} enterIndex */
function markCubeEntering(cube, enterIndex) {
  cube.dataset.enter = "";
  cube.style.setProperty(
    "--enter-delay",
    `${enterIndex * CUBE_ENTER_STAGGER_MS}ms`,
  );
  cube.addEventListener(
    "animationend",
    () => {
      cube.removeAttribute("data-enter");
    },
    { once: true },
  );
}

/** @param {number} [enterIndex] */
export function createProjectedCube(enterIndex = 0) {
  const cube = createCubeDiv();
  const boundary = document.createElement("div");
  const date = document.createTextNode("");
  const year = document.createTextNode("");
  const hh = document.createTextNode("");
  const mm = document.createTextNode("");
  const ss = document.createTextNode("");
  const utcOffset = document.createTextNode("");
  const heightPrefix = document.createTextNode("");
  const height = document.createTextNode("");
  const txs = document.createTextNode("");
  const txsUnit = document.createTextNode("");
  const size = document.createTextNode("");
  const totalFees = document.createTextNode("");
  const median = document.createTextNode("");
  const rangeLo = document.createTextNode("");
  const rangeHi = document.createTextNode("");

  const dateElement = document.createElement("p");
  const yearElement = dim("");
  const timeElement = document.createElement("p");
  const utcOffsetElement = dim("");
  const heightElement = document.createElement("p");
  const heightPrefixElement = dim("");
  const txsElement = document.createElement("p");
  const txsUnitElement = dim("");
  const sizeElement = document.createElement("p");
  const minerElement = dim("UNMINED");
  const totalFeesElement = document.createElement("p");
  const medianElement = document.createElement("p");
  const rangeElement = document.createElement("p");
  const unitElement = document.createElement("p");

  cube.element.dataset.projected = "";
  boundary.dataset.chainBoundary = "";
  boundary.append(createBoundaryLabels(), createBoundaryLabels());
  cube.element.append(boundary);
  markCubeEntering(cube.element, enterIndex);
  yearElement.append(year);
  utcOffsetElement.append(utcOffset);
  dateElement.append(date);
  timeElement.append(hh, dim(":"), mm, dim(":"), ss);
  cube.topFace.append(dateElement, yearElement, timeElement, utcOffsetElement);

  heightElement.dataset.cubeHeight = "";
  heightPrefixElement.append(heightPrefix);
  heightElement.append(heightPrefixElement, height);
  txsUnitElement.append(txsUnit);
  txsElement.append(txs, " ", txsUnitElement);
  sizeElement.append(size, " ", dim("MB"));
  cube.rightFace.append(heightElement, txsElement, sizeElement, minerElement);

  totalFeesElement.append(dim("₿"), totalFees);
  medianElement.append(dim("~"), median);
  rangeElement.append(rangeLo, dim("-"), rangeHi);
  unitElement.dataset.dim = "";
  unitElement.textContent = "sat/vB";
  cube.leftFace.append(
    totalFeesElement,
    medianElement,
    rangeElement,
    unitElement,
  );

  return {
    ...cube,
    parts: {
      date,
      year,
      hh,
      mm,
      ss,
      utcOffset,
      heightPrefix,
      height,
      txs,
      txsUnit,
      size,
      totalFees,
      median,
      rangeLo,
      rangeHi,
    },
  };
}

function createBoundaryLabels() {
  const labels = document.createElement("div");
  const projected = document.createElement("span");
  const confirmed = document.createElement("span");

  labels.dataset.boundaryLabels = "";
  projected.textContent = "PROJECTED";
  confirmed.textContent = "CONFIRMED";
  labels.append(projected, confirmed);

  return labels;
}

/** @param {ReturnType<typeof createProjectedCube>} cube @param {MempoolBlock} block */
export function updateProjectedCube(cube, block) {
  cube.element.style.setProperty(
    "--fill",
    String(Math.min(1, block.blockVSize / 1_000_000)),
  );

  cube.parts.txs.nodeValue = formatNumber(block.nTx);
  cube.parts.txsUnit.nodeValue = block.nTx === 1 ? "tx" : "txs";
  cube.parts.size.nodeValue = formatMegabytes(block.blockSize);
  cube.parts.totalFees.nodeValue = formatBtc(block.totalFees);
  cube.parts.median.nodeValue = formatFeeRate(block.medianFee);
  cube.parts.rangeLo.nodeValue = formatFeeRate(block.feeRange[0]);
  cube.parts.rangeHi.nodeValue = formatFeeRate(block.feeRange[6]);
}

/** @param {ReturnType<typeof createProjectedCube>} cube @param {number} height */
export function updateProjectedHeight(cube, height) {
  const [prefix, value] = formatHeight(height);

  cube.parts.heightPrefix.nodeValue = prefix;
  cube.parts.height.nodeValue = value;
}

/** @param {ReturnType<typeof createProjectedCube>} cube @param {number} timestamp */
export function updateProjectedTime(cube, timestamp) {
  const [hh, mm, ss] = formatTime(timestamp);

  cube.parts.date.nodeValue = formatShortDate(timestamp);
  cube.parts.year.nodeValue = formatYear(timestamp);
  cube.parts.hh.nodeValue = hh;
  cube.parts.mm.nodeValue = mm;
  cube.parts.ss.nodeValue = ss;
  cube.parts.utcOffset.nodeValue = formatUtcOffset(timestamp);
}

/** @param {HTMLElement} cube */
export function setConfirmedInterval(cube) {
  const prev = /** @type {HTMLElement | null} */ (cube.previousElementSibling);
  if (!prev?.dataset.timestamp) return;

  const seconds = Math.max(
    0,
    Number(cube.dataset.timestamp) - Number(prev.dataset.timestamp),
  );

  setBlockInterval(cube, seconds, formatInterval(seconds));
}

/** @param {HTMLElement} cube @param {number} seconds @param {string} label */
export function setBlockInterval(cube, seconds, label) {
  cube.style.setProperty("--block-interval", String(seconds));

  let interval = /** @type {HTMLElement | null} */ (
    cube.querySelector(":scope > [data-block-interval]")
  );

  if (!interval) {
    const full = document.createElement("span");
    const compact = document.createElement("span");

    interval = document.createElement("div");
    interval.dataset.blockInterval = "";
    interval.ariaHidden = "true";
    full.dataset.intervalFull = "";
    compact.dataset.intervalCompact = "";
    interval.append(full, compact);
    cube.append(interval);
  }

  const full = interval.querySelector("[data-interval-full]");
  const compact = interval.querySelector("[data-interval-compact]");

  if (full) full.textContent = label;
  if (compact) {
    compact.textContent = `~${Math.max(1, Math.round(seconds / 60))}mn`;
  }
}
