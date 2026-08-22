import { brk } from "../../../scripts/utils/client.js";
import { onPlainClick } from "../../../scripts/utils/dom.js";
import {
  createHeightElement,
  formatFeeRate,
} from "../../../scripts/explorer/render.js";
import { createCubeAnchor, createCubeDiv } from "./cube/index.js";

const LOOKAHEAD = 15;
const PROJECTED_LIMIT = 8;
const TARGET_BLOCK_SECONDS = 600;
const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

/** @type {HTMLElement} */ let explorerEl;
/** @type {HTMLDivElement} */ let chainEl;
/** @type {HTMLDivElement} */ let scrollEl;
/** @type {HTMLDivElement} */ let blocksEl;
/** @type {HTMLAnchorElement | null} */ let selectedCube = null;
/** @type {IntersectionObserver} */ let olderEdgeObserver;
/** @type {(block: BlockInfoV1) => void} */ let onSelect = () => {};
/** @type {(cube: HTMLAnchorElement) => void} */ let onCubeClick = () => {};
/** @type {() => void} */ let onTip = () => {};
/** @type {() => void} */ let onGenesis = () => {};

/** @type {Map<BlockHash, BlockInfoV1>} */
const blocksByHash = new Map();
/** @type {ReturnType<typeof createProjectedCube>[]} */
const projectedCubes = [];

let newestHeight = -1;
let oldestHeight = Infinity;
let newestTimestamp = 0;
let loadingOlder = false;
let loadingNewer = false;
let reachedTip = false;

/**
 * @param {HTMLElement} parent
 * @param {{
 *   onSelect: (block: BlockInfoV1) => void,
 *   onCubeClick: (cube: HTMLAnchorElement) => void,
 *   onTip: () => void,
 *   onGenesis: () => void,
 * }} callbacks
 */
export function initChain(parent, callbacks) {
  onSelect = callbacks.onSelect;
  onCubeClick = callbacks.onCubeClick;
  onTip = callbacks.onTip;
  onGenesis = callbacks.onGenesis;
  explorerEl = parent;

  chainEl = document.createElement("div");
  chainEl.id = "chain";
  parent.append(chainEl);

  chainEl.append(
    createEdgeLink("tip", "/block/tip", "Jump to chain tip", onTip),
    createEdgeLink("gen", "/block/0", "Jump to genesis block", onGenesis),
  );

  scrollEl = document.createElement("div");
  scrollEl.classList.add("scroll");
  chainEl.append(scrollEl);

  blocksEl = document.createElement("div");
  blocksEl.classList.add("blocks");
  scrollEl.append(blocksEl);

  olderEdgeObserver = new IntersectionObserver(
    (entries) => {
      if (entries[0].isIntersecting) loadOlder();
    },
    { root: scrollEl },
  );

  scrollEl.addEventListener(
    "scroll",
    () => {
      if (reachedTip || loadingNewer) return;
      if (scrollEl.scrollTop <= 50 && scrollEl.scrollLeft <= 50) loadNewer();
    },
    { passive: true },
  );
}

export function deselectCube() {
  if (selectedCube) selectedCube.classList.remove("selected");
  selectedCube = null;
}

/** @param {HTMLAnchorElement} cube @param {{ scroll?: "smooth" | "instant", silent?: boolean }} [opts] */
export function selectCube(cube, { scroll, silent } = {}) {
  if (cube !== selectedCube) {
    if (selectedCube) selectedCube.classList.remove("selected");
    selectedCube = cube;
    cube.classList.add("selected");
  }
  if (scroll) {
    cube.scrollIntoView({
      behavior: scroll,
      block: "center",
      inline: "center",
    });
  }
  if (!silent) {
    const hash = cube.dataset.hash;
    if (hash) {
      const block = blocksByHash.get(hash);
      if (block) onSelect(block);
    }
  }
}

/** @param {BlockHash | Height | string | null} [hashOrHeight] @param {{ silent?: boolean }} [options] */
export async function goToCube(hashOrHeight, { silent } = {}) {
  if (hashOrHeight === "tip") hashOrHeight = null;
  if (typeof hashOrHeight === "string" && /^\d+$/.test(hashOrHeight)) {
    hashOrHeight = Number(hashOrHeight);
  }
  let cube = findCube(hashOrHeight);
  if (cube) {
    selectCube(cube, { scroll: "smooth", silent });
    return;
  }
  for (const cube of blocksEl.children) {
    if (!cube.classList.contains("projected")) cube.classList.add("skeleton");
  }
  explorerEl.classList.add("loading");
  let startHash;
  try {
    const height = await resolveHeight(hashOrHeight);
    startHash = await loadInitial(height);
  } catch (_) {
    try {
      startHash = await loadInitial(null);
    } catch (_) {
      explorerEl.classList.remove("loading");
      return;
    }
  }
  selectCube(/** @type {HTMLAnchorElement} */ (findCube(startHash)), {
    scroll: "instant",
    silent,
  });
  explorerEl.classList.remove("loading");
}

export async function poll() {
  if (!reachedTip) return;
  pollProjected();
  try {
    const blocks = await brk.getBlocksV1();
    appendNewerBlocks(blocks);
  } catch (e) {
    console.error("explorer poll:", e);
  }
}

function pollProjected() {
  return brk
    .getMempoolBlocks()
    .then(renderProjected)
    .catch((e) => console.error("mempool poll:", e));
}

/** @param {BlockHash | Height | null} [hashOrHeight] */
function findCube(hashOrHeight) {
  if (hashOrHeight == null) {
    return reachedTip && newestHeight >= 0 ? newestConfirmedCube() : null;
  }
  const attr = typeof hashOrHeight === "number" ? "height" : "hash";
  return /** @type {HTMLAnchorElement | null} */ (
    blocksEl.querySelector(`[data-${attr}="${hashOrHeight}"]`)
  );
}

function firstProjectedCube() {
  return projectedCubes[0]?.el ?? null;
}

function newestConfirmedCube() {
  const firstProj = firstProjectedCube();
  return /** @type {HTMLAnchorElement | null} */ (
    firstProj ? firstProj.previousElementSibling : blocksEl.lastElementChild
  );
}

function clear() {
  newestHeight = -1;
  oldestHeight = Infinity;
  newestTimestamp = 0;
  loadingOlder = false;
  loadingNewer = false;
  reachedTip = false;
  selectedCube = null;
  blocksEl.innerHTML = "";
  projectedCubes.length = 0;
  olderEdgeObserver.disconnect();
}

function observeOldestEdge() {
  olderEdgeObserver.disconnect();
  const oldest = blocksEl.firstElementChild;
  if (oldest) olderEdgeObserver.observe(oldest);
}

/** @param {BlockInfoV1[]} blocks */
function appendNewerBlocks(blocks) {
  if (!blocks.length) return false;
  const anchor = newestConfirmedCube();
  const anchorRect = anchor?.getBoundingClientRect();
  for (let i = blocks.length - 1; i >= 0; i--) {
    const b = blocks[i];
    if (b.height > newestHeight) appendConfirmed(createConfirmedCube(b));
    else blocksByHash.set(b.id, b);
  }
  newestHeight = Math.max(newestHeight, blocks[0].height);
  newestTimestamp = blocks[0].timestamp;
  refreshProjected();
  if (anchor && anchorRect) {
    const r = anchor.getBoundingClientRect();
    scrollEl.scrollTop += r.top - anchorRect.top;
    scrollEl.scrollLeft += r.left - anchorRect.left;
  }
  return true;
}

/** @param {number | null} [height] @returns {Promise<BlockHash>} */
async function loadInitial(height) {
  const blocks =
    height != null
      ? await brk.getBlocksV1FromHeight(height)
      : await brk.getBlocksV1();

  clear();
  for (const b of blocks) prependConfirmed(createConfirmedCube(b));
  newestHeight = blocks[0].height;
  oldestHeight = blocks[blocks.length - 1].height;
  newestTimestamp = blocks[0].timestamp;
  reachedTip = height == null;
  observeOldestEdge();

  if (!reachedTip) await loadNewer();
  // Await the projected cubes so the layout is complete before the caller
  // scrolls to the tip; otherwise they load late and push the tip out of view.
  else await pollProjected();
  return blocks[0].id;
}

/** @param {BlockHash | Height | null} [hashOrHeight] @returns {Promise<Height | null>} */
async function resolveHeight(hashOrHeight) {
  if (typeof hashOrHeight === "number") return hashOrHeight;
  if (typeof hashOrHeight === "string") {
    const cached = blocksByHash.get(hashOrHeight);
    if (cached) return cached.height;
    const block = await brk.getBlockV1(hashOrHeight);
    blocksByHash.set(hashOrHeight, block);
    return block.height;
  }
  return null;
}

async function loadOlder() {
  if (loadingOlder || oldestHeight <= 0) return;
  loadingOlder = true;
  try {
    const blocks = await brk.getBlocksV1FromHeight(oldestHeight - 1);
    for (const block of blocks) prependConfirmed(createConfirmedCube(block));
    if (blocks.length) {
      oldestHeight = blocks[blocks.length - 1].height;
      observeOldestEdge();
    }
  } catch (e) {
    console.error("explorer loadOlder:", e);
  }
  loadingOlder = false;
}

async function loadNewer() {
  if (loadingNewer || newestHeight === -1 || reachedTip) return;
  loadingNewer = true;
  try {
    const prevNewest = newestHeight;
    const blocks = await brk.getBlocksV1FromHeight(newestHeight + LOOKAHEAD);
    if (!appendNewerBlocks(blocks) || newestHeight === prevNewest) {
      reachedTip = true;
      await pollProjected();
    }
  } catch (e) {
    console.error("explorer loadNewer:", e);
  }
  loadingNewer = false;
}

/** @param {BlockInfoV1} block */
function createConfirmedCube(block) {
  const { pool, medianFee, feeRange, virtualSize } = block.extras;
  const fill = Math.min(1, virtualSize / 1_000_000);
  const { el, topFace, rightFace, leftFace } = createCubeAnchor(fill);
  el.href = `/block/${block.id}`;
  el.dataset.hash = block.id;
  el.dataset.height = String(block.height);
  el.dataset.timestamp = String(block.timestamp);
  blocksByHash.set(block.id, block);
  onPlainClick(el, () => onCubeClick(el));

  const dateP = document.createElement("p");
  dateP.textContent = formatShortDate(block.timestamp);
  const [hh, mm] = formatHHMM(block.timestamp);
  const timeP = document.createElement("p");
  timeP.append(hh, span(":", "dim"), mm);
  topFace.append(dateP, timeP);

  const heightP = document.createElement("p");
  heightP.classList.add("height");
  heightP.append(createHeightElement(block.height));
  const poolDiv = document.createElement("div");
  poolDiv.classList.add("pool");
  const logo = document.createElement("img");
  logo.src = `/assets/pools/${poolSlug(pool.name)}.svg`;
  logo.alt = "";
  logo.onerror = () => {
    logo.onerror = null;
    logo.src = "/assets/pools/default.svg";
  };
  const nameSpan = document.createElement("span");
  nameSpan.textContent = pool.name.replace(/\s+(Pool|USA)$/i, "").trim();
  poolDiv.append(logo, nameSpan);
  rightFace.append(heightP, poolDiv);

  const feesEl = document.createElement("div");
  feesEl.classList.add("fees");
  const avg = document.createElement("p");
  avg.append(span("~", "dim"), formatFeeRate(medianFee));
  const range = document.createElement("p");
  range.append(
    formatFeeRate(feeRange[0]),
    span("-", "dim"),
    formatFeeRate(feeRange[6]),
  );
  const unit = document.createElement("p");
  unit.classList.add("dim");
  unit.textContent = "sat/vB";
  feesEl.append(avg, range, unit);
  leftFace.append(feesEl);

  return el;
}

/** @param {HTMLElement} cube */
function setConfirmedInterval(cube) {
  const prev = /** @type {HTMLElement | null} */ (cube.previousElementSibling);
  if (!prev) return;
  const dt = Math.max(
    0,
    Number(cube.dataset.timestamp) - Number(prev.dataset.timestamp),
  );
  cube.style.setProperty("--block-interval", String(dt));
}

/** @param {HTMLAnchorElement} cube */
function prependConfirmed(cube) {
  const oldFirst = /** @type {HTMLElement | null} */ (blocksEl.firstElementChild);
  blocksEl.insertBefore(cube, oldFirst);
  if (oldFirst) setConfirmedInterval(oldFirst);
}

/** @param {HTMLAnchorElement} cube */
function appendConfirmed(cube) {
  blocksEl.insertBefore(cube, firstProjectedCube());
  setConfirmedInterval(cube);
}

/** @param {MempoolBlock[]} blocks */
function renderProjected(blocks) {
  const want = Math.min(blocks.length, PROJECTED_LIMIT);

  while (projectedCubes.length > want) {
    const last = projectedCubes.pop();
    if (last) last.el.remove();
  }
  while (projectedCubes.length < want) {
    const cube = createProjectedCube();
    projectedCubes.push(cube);
    blocksEl.append(cube.el);
  }
  for (let i = 0; i < want; i++) updateProjectedCube(projectedCubes[i], blocks[i]);
  refreshProjected();
}

function createProjectedCube() {
  const cube = createCubeDiv();
  cube.el.classList.add("projected");

  const date = document.createTextNode("");
  const hh = document.createTextNode("");
  const mm = document.createTextNode("");
  const dateP = document.createElement("p");
  dateP.append(date);
  const timeP = document.createElement("p");
  timeP.append(hh, span(":", "dim"), mm);
  cube.topFace.append(dateP, timeP);

  const txs = document.createTextNode("");
  const txsUnit = document.createTextNode("");
  const txsP = document.createElement("p");
  txsP.append(txs);
  const txsUnitP = document.createElement("p");
  txsUnitP.classList.add("dim");
  txsUnitP.append(txsUnit);
  cube.rightFace.append(txsP, txsUnitP);

  const median = document.createTextNode("");
  const rangeLo = document.createTextNode("");
  const rangeHi = document.createTextNode("");
  const medianP = document.createElement("p");
  medianP.append(span("~", "dim"), median);
  const rangeP = document.createElement("p");
  rangeP.append(rangeLo, span("-", "dim"), rangeHi);
  const unitP = document.createElement("p");
  unitP.classList.add("dim");
  unitP.textContent = "sat/vB";
  cube.leftFace.append(medianP, rangeP, unitP);

  return {
    ...cube,
    parts: { date, hh, mm, txs, txsUnit, median, rangeLo, rangeHi },
  };
}

/**
 * @param {ReturnType<typeof createProjectedCube>} cube
 * @param {MempoolBlock} block
 */
function updateProjectedCube(cube, block) {
  cube.el.style.setProperty(
    "--fill",
    String(Math.min(1, block.blockVSize / 1_000_000)),
  );
  const p = cube.parts;
  p.txs.nodeValue = block.nTx.toLocaleString();
  p.txsUnit.nodeValue = block.nTx === 1 ? "tx" : "txs";
  p.median.nodeValue = formatFeeRate(block.medianFee);
  p.rangeLo.nodeValue = formatFeeRate(block.feeRange[0]);
  p.rangeHi.nodeValue = formatFeeRate(block.feeRange[6]);
}

function refreshProjected() {
  if (!projectedCubes.length || !newestTimestamp) return;
  const now = Math.floor(Date.now() / 1000);
  const elapsed = Math.max(0, now - newestTimestamp);
  for (let i = 0; i < projectedCubes.length; i++) {
    const cube = projectedCubes[i];
    const interval = i === 0 ? elapsed : TARGET_BLOCK_SECONDS;
    cube.el.style.setProperty("--block-interval", String(interval));
    const ts = now + i * TARGET_BLOCK_SECONDS;
    const [hh, mm] = formatHHMM(ts);
    cube.parts.date.nodeValue = formatShortDate(ts);
    cube.parts.hh.nodeValue = hh;
    cube.parts.mm.nodeValue = mm;
  }
}

/** @param {"tip" | "gen"} label @param {string} href @param {string} title @param {() => void} handler */
function createEdgeLink(label, href, title, handler) {
  const a = document.createElement("a");
  a.classList.add("edge", label);
  a.href = href;
  a.title = title;
  a.textContent = label;
  onPlainClick(a, handler);
  return a;
}

/** @param {string} text @param {string} [cls] */
function span(text, cls) {
  const s = document.createElement("span");
  if (cls) s.classList.add(cls);
  s.textContent = text;
  return s;
}

/** @param {string} name */
const poolSlug = (name) => name.toLowerCase().replace(/[^a-z0-9]/g, "");

/** @param {number} unixSec */
function formatShortDate(unixSec) {
  const d = new Date(unixSec * 1000);
  return `${MONTHS[d.getMonth()]} ${d.getDate()}`;
}

/** @param {number} unixSec */
function formatHHMM(unixSec) {
  const d = new Date(unixSec * 1000);
  return [
    String(d.getHours()).padStart(2, "0"),
    String(d.getMinutes()).padStart(2, "0"),
  ];
}
