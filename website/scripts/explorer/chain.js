import { brk } from "../utils/client.js";
import { onPlainClick } from "../utils/dom.js";
import { createCube } from "./cube.js";
import { initMempool, renderMempool } from "./mempool.js";
import { createHeightElement, formatFeeRate } from "./render.js";

const LOOKAHEAD = 15;

/** @type {HTMLDivElement} */ let chainEl;
/** @type {HTMLDivElement} */ let scrollEl;
/** @type {HTMLDivElement} */ let blocksEl;
/** @type {HTMLAnchorElement | null} */ let selectedCube = null;
/** @type {IntersectionObserver} */ let olderObserver;
/** @type {(block: BlockInfoV1) => void} */ let onSelect = () => {};
/** @type {(cube: HTMLAnchorElement) => void} */ let onCubeClick = () => {};
/** @type {() => void} */ let onTip = () => {};
/** @type {() => void} */ let onGenesis = () => {};

/** @type {Map<BlockHash, BlockInfoV1>} */
const blocksByHash = new Map();

let newestHeight = -1;
let oldestHeight = Infinity;
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

  chainEl = document.createElement("div");
  chainEl.id = "chain";
  parent.append(chainEl);

  chainEl.append(
    createControlLink("tip", "/block/tip", "Jump to chain tip", onTip),
  );

  chainEl.append(
    createControlLink("gen", "/block/0", "Jump to genesis block", onGenesis),
  );

  scrollEl = document.createElement("div");
  scrollEl.classList.add("chain-scroll");
  chainEl.append(scrollEl);

  blocksEl = document.createElement("div");
  blocksEl.classList.add("blocks");
  scrollEl.append(blocksEl);

  initMempool(scrollEl);

  olderObserver = new IntersectionObserver(
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

/** @param {BlockHash | Height | null} [hashOrHeight] */
function findCube(hashOrHeight) {
  if (hashOrHeight == null) {
    return reachedTip && newestHeight >= 0
      ? /** @type {HTMLAnchorElement | null} */ (blocksEl.lastElementChild)
      : null;
  }
  const attr = typeof hashOrHeight === "number" ? "height" : "hash";
  return /** @type {HTMLAnchorElement | null} */ (
    blocksEl.querySelector(`[data-${attr}="${hashOrHeight}"]`)
  );
}

export function deselectCube() {
  if (selectedCube) selectedCube.classList.remove("selected");
  selectedCube = null;
}

/** @param {HTMLAnchorElement} cube @param {{ scroll?: "smooth" | "instant", silent?: boolean }} [opts] */
export function selectCube(cube, { scroll, silent } = {}) {
  const changed = cube !== selectedCube;
  if (changed) {
    if (selectedCube) selectedCube.classList.remove("selected");
    selectedCube = cube;
    cube.classList.add("selected");
  }
  if (scroll) {
    cube.scrollIntoView({ behavior: scroll, block: "center", inline: "center" });
  }
  if (!silent) {
    const hash = cube.dataset.hash;
    if (hash) {
      const block = blocksByHash.get(hash);
      if (block) onSelect(block);
    }
  }
}

function clear() {
  newestHeight = -1;
  oldestHeight = Infinity;
  loadingOlder = false;
  loadingNewer = false;
  reachedTip = false;
  selectedCube = null;
  blocksEl.innerHTML = "";
  olderObserver.disconnect();
}

function observeOldestEdge() {
  olderObserver.disconnect();
  const oldest = blocksEl.firstElementChild;
  if (oldest) olderObserver.observe(oldest);
}

/** @param {BlockInfoV1[]} blocks */
function appendNewerBlocks(blocks) {
  if (!blocks.length) return false;
  const anchor = blocksEl.lastElementChild;
  const anchorRect = anchor?.getBoundingClientRect();
  for (let i = blocks.length - 1; i >= 0; i--) {
    const b = blocks[i];
    if (b.height > newestHeight) {
      appendCube(createBlockCube(b));
    } else {
      blocksByHash.set(b.id, b);
    }
  }
  newestHeight = Math.max(newestHeight, blocks[0].height);

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
  for (const b of blocks) prependCube(createBlockCube(b));
  newestHeight = blocks[0].height;
  oldestHeight = blocks[blocks.length - 1].height;
  reachedTip = height == null;
  observeOldestEdge();

  if (!reachedTip) await loadNewer();
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
  for (const cube of blocksEl.children) cube.classList.add("skeleton");
  let startHash;
  try {
    const height = await resolveHeight(hashOrHeight);
    startHash = await loadInitial(height);
  } catch (e) {
    try { startHash = await loadInitial(null); } catch (_) { return; }
  }
  selectCube(/** @type {HTMLAnchorElement} */ (findCube(startHash)), { scroll: "instant", silent });
}

export async function poll() {
  if (!reachedTip) return;
  brk.getMempoolBlocks()
    .then(renderMempool)
    .catch((e) => console.error("mempool poll:", e));
  try {
    const blocks = await brk.getBlocksV1();
    appendNewerBlocks(blocks);
  } catch (e) {
    console.error("explorer poll:", e);
  }
}

async function loadOlder() {
  if (loadingOlder || oldestHeight <= 0) return;
  loadingOlder = true;
  try {
    const blocks = await brk.getBlocksV1FromHeight(oldestHeight - 1);
    for (const block of blocks) prependCube(createBlockCube(block));
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
    }
  } catch (e) {
    console.error("explorer loadNewer:", e);
  }
  loadingNewer = false;
}

/** @param {string} name */
const poolSlug = (name) => name.toLowerCase().replace(/[^a-z0-9]/g, "");

const MONTHS = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

/** @param {number} unixSec */
function formatShortDate(unixSec) {
  const d = new Date(unixSec * 1000);
  return `${MONTHS[d.getMonth()]} ${d.getDate()}`;
}

/** @param {number} unixSec */
function formatHHMM(unixSec) {
  const d = new Date(unixSec * 1000);
  return [String(d.getHours()).padStart(2, "0"), String(d.getMinutes()).padStart(2, "0")];
}

/** @param {string} text @param {string} [cls] */
function span(text, cls) {
  const s = document.createElement("span");
  if (cls) s.classList.add(cls);
  s.textContent = text;
  return s;
}

/** @param {BlockInfoV1} block */
function createBlockCube(block) {
  const cubeElement = document.createElement("a");
  cubeElement.classList.add("cube");
  cubeElement.href = `/block/${block.id}`;
  cubeElement.dataset.hash = block.id;
  cubeElement.dataset.height = String(block.height);
  cubeElement.dataset.timestamp = String(block.timestamp);

  const { pool, medianFee, feeRange, virtualSize } = block.extras;
  const fill = Math.min(1, virtualSize / 1_000_000);
  const { topFace, rightFace, leftFace } = createCube(cubeElement, fill);
  blocksByHash.set(block.id, block);
  onPlainClick(cubeElement, () => onCubeClick(cubeElement));

  const minerName = pool.name;

  // Top: short date / HH:MM (colon dimmed).
  const dateP = document.createElement("p");
  dateP.textContent = formatShortDate(block.timestamp);
  const [hh, mm] = formatHHMM(block.timestamp);
  const timeP = document.createElement("p");
  timeP.append(hh, span(":", "dim"), mm);
  topFace.append(dateP, timeP);

  // Right: block height / raw pool-logo + miner name.
  const heightP = document.createElement("p");
  heightP.classList.add("height");
  heightP.append(createHeightElement(block.height));
  const poolDiv = document.createElement("div");
  poolDiv.classList.add("pool");
  const logo = document.createElement("img");
  logo.src = `/assets/pools/${poolSlug(minerName)}.svg`;
  logo.alt = "";
  logo.onerror = () => {
    logo.onerror = null;
    logo.src = "/assets/pools/default.svg";
  };
  const nameSpan = document.createElement("span");
  nameSpan.textContent = minerName.replace(/\s+(Pool|USA)$/i, "").trim();
  poolDiv.append(logo, nameSpan);
  rightFace.append(heightP, poolDiv);

  // Left: ~median / min-max / sat/vB fees stack.
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

  return cubeElement;
}

/** @param {HTMLElement} cube */
function setGap(cube) {
  const prev = /** @type {HTMLElement | null} */ (cube.previousElementSibling);
  if (!prev) return;
  const dt = Math.max(0, Number(cube.dataset.timestamp) - Number(prev.dataset.timestamp));
  cube.style.setProperty("--dt", String(dt));
}

/** @param {HTMLAnchorElement} cube */
function prependCube(cube) {
  const next = /** @type {HTMLElement | null} */ (blocksEl.firstElementChild);
  blocksEl.prepend(cube);
  if (next) setGap(next);
}

/** @param {HTMLAnchorElement} cube */
function appendCube(cube) {
  blocksEl.append(cube);
  setGap(cube);
}

/** @param {"tip" | "gen"} label @param {string} href @param {string} title @param {() => void} handler */
function createControlLink(label, href, title, handler) {
  const a = document.createElement("a");
  a.classList.add("chain-edge", label);
  a.href = href;
  a.title = title;
  a.textContent = label;
  onPlainClick(a, handler);
  return a;
}

