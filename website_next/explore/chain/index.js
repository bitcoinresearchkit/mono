import { createConfirmedBlocks } from "./confirmed.js";
import { chainClient } from "./client.js";
import { createEdgeButton } from "./edge.js";
import {
  distanceFromViewport,
  findVisibleConfirmedHeight,
  isHorizontalLayout,
  olderWheelDelta,
  preserveScrollPosition,
  syncDiagonalScroll,
} from "./scroll.js";
import { createJumpController } from "./jump.js";
import { createOlderBlocks } from "./older.js";
import { createNewBlockTransition } from "./new-block.js";
import { createProjectedBlocks } from "./projected.js";
import { hasReorganized } from "./reorg.js";
import { createTipVisibility } from "./tip.js";

const BLOCK_BATCH_SIZE = 15;
const CLOCK_INTERVAL = 1_000;
const EDGE_LOAD_DISTANCE = 50;
const POLL_INTERVAL = 5_000;

/** @typedef {import("../../modules/brk-client/index.js").BlockInfoV1} Block */
/** @typedef {import("../../modules/brk-client/index.js").MempoolBlock} MempoolBlock */

/** @param {string | number | null | undefined} hashOrHeight */
function normalizeTarget(hashOrHeight) {
  if (hashOrHeight === "tip") return null;
  if (typeof hashOrHeight === "string" && /^\d+$/.test(hashOrHeight)) {
    return Number(hashOrHeight);
  }

  return hashOrHeight;
}

/**
 * @param {{ onOpen?: (block: Block, cube: HTMLButtonElement) => void }} [options]
 */
export function createChain({ onOpen = () => {} } = {}) {
  const element = document.createElement("div");
  const scrollElement = document.createElement("div");
  const blocksElement = document.createElement("div");
  const tipButton = createEdgeButton("tip", "↑", "←", "Jump to chain tip", () => {
    jumpToTip();
  });
  const jump = createJumpController(element, () => {
    const tipCube = confirmed.tipCube();
    if (tipCube) confirmed.scrollTo(tipCube, "instant");
  });

  element.id = "chain";
  scrollElement.dataset.chainScroll = "";
  blocksElement.dataset.chainBlocks = "";
  scrollElement.append(blocksElement);
  element.append(tipButton, scrollElement);

  const projected = createProjectedBlocks({
    blocksElement,
    isLayoutFrozen: () => tipButton.hasAttribute("data-visible"),
    isElementVisible,
  });
  const confirmed = createConfirmedBlocks({
    scrollElement,
    blocksElement,
    firstProjectedElement: projected.firstElement,
    onOpen,
    onScrollSelect: () => tip.schedule(),
  });
  const tip = createTipVisibility({
    button: tipButton,
    reachedTip: () => reachedTip,
    newestHeight: () => newestHeight,
    tipCube: confirmed.tipCube,
    visibleConfirmedHeight: () =>
      findVisibleConfirmedHeight(scrollElement, blocksElement),
    hasVisibleProjected: () => projected.hasVisibleElement(),
    isElementVisible,
  });
  const newBlock = createNewBlockTransition({
    scrollElement,
    blocksElement,
    firstProjectedElement: projected.firstElement,
  });
  let active = false;
  let newestHeight = -1;
  let newestTimestamp = 0;
  let loadingNewer = false;
  let polling = false;
  let reachedTip = false;

  /** @type {number | undefined} */
  let pollId;

  /** @type {number | undefined} */
  let clockId;

  /** @type {AbortController} */
  let controller = new AbortController();

  const older = createOlderBlocks({
    scrollElement,
    blocksElement,
    batchSize: BLOCK_BATCH_SIZE,
    isActive: () => active,
    isHorizontal,
    fetchBlocks: (startHeight) =>
      chainClient.getBlocksFromHeight(startHeight, controller.signal),
    createCube: confirmed.create,
    isAborted: () => controller.signal.aborted,
    onError: (error) => logChainError("explore older:", error),
  });

  /**
   * @param {string} label
   * @param {unknown} error
   */
  function logChainError(label, error) {
    if (!controller.signal.aborted) console.error(label, error);
  }

  /** @param {string | number | null | undefined} hashOrHeight */
  function findCube(hashOrHeight) {
    if (hashOrHeight == null) {
      return reachedTip && newestHeight >= 0 ? confirmed.newest() : null;
    }

    return confirmed.find(hashOrHeight);
  }

  function jumpToTip() {
    if (confirmed.tipCube()) jump.jump();
  }

  function isHorizontal() {
    return isHorizontalLayout(blocksElement);
  }

  function clear() {
    newestHeight = -1;
    newestTimestamp = 0;
    loadingNewer = false;
    reachedTip = false;
    confirmed.clear();
    blocksElement.textContent = "";
    projected.clear();
    older.reset();
    tip.setVisible(false);
  }

  /** @param {Block[]} blocks */
  function appendNewerBlocks(blocks) {
    if (!blocks.length || blocks[0].height <= newestHeight) return false;

    const anchor = confirmed.newest();
    const anchorRect = anchor?.getBoundingClientRect();
    const transition = reachedTip ? newBlock.capture() : null;
    const entering = [];

    for (let i = blocks.length - 1; i >= 0; i--) {
      const block = blocks[i];

      if (block.height > newestHeight) {
        entering.push(confirmed.append(block));
      }
    }

    newestHeight = Math.max(newestHeight, blocks[0].height);
    newestTimestamp = blocks[0].timestamp;
    confirmed.markTip();
    refreshProjected();

    preserveScrollPosition(scrollElement, anchor, anchorRect);
    syncDiagonalScroll(scrollElement, blocksElement);
    if (transition && entering.length) void newBlock.play(transition, entering);

    tip.sync();

    return true;
  }

  /** @param {number | null} [height] */
  async function loadInitial(height) {
    const blocks =
      height != null
        ? await chainClient.getBlocksFromHeight(height, controller.signal)
        : await chainClient.getBlocks(controller.signal);

    clear();

    for (const [index, block] of blocks.entries()) {
      confirmed.prepend(block, index);
    }

    newestHeight = blocks[0].height;
    older.setOldestHeight(blocks[blocks.length - 1].height);
    newestTimestamp = blocks[0].timestamp;
    reachedTip = height == null;
    confirmed.markTip();
    older.reserve();

    if (reachedTip) await pollProjected();
    else await loadNewer();

    return blocks[0].id;
  }

  /** @param {string | number | null | undefined} hashOrHeight */
  async function resolveHeight(hashOrHeight) {
    if (typeof hashOrHeight === "number") return hashOrHeight;

    if (typeof hashOrHeight === "string") {
      const cached = confirmed.get(hashOrHeight);
      if (cached) return cached.height;

      const block = await chainClient.getBlock(hashOrHeight, controller.signal);
      confirmed.cache(block);

      return block.height;
    }

    return null;
  }

  /** @param {string | number | null | undefined} [hashOrHeight] */
  async function goToCube(hashOrHeight) {
    if (!active) return;

    hashOrHeight = normalizeTarget(hashOrHeight);

    const existing = findCube(hashOrHeight);
    if (existing) {
      confirmed.scrollTo(existing, "smooth");
      return;
    }

    confirmed.markSkeletons();
    element.dataset.loading = "";

    try {
      const height = await resolveHeight(hashOrHeight);
      const startHash = await loadInitial(height);
      const cube = findCube(startHash);
      if (cube) confirmed.scrollTo(cube, "instant");
    } catch (error) {
      logChainError("explore chain load:", error);
    } finally {
      delete element.dataset.loading;
    }
  }

  async function pollProjected() {
    try {
      renderProjected(
        await chainClient.getMempoolBlocks(controller.signal),
      );
    } catch (error) {
      logChainError("explore mempool:", error);
    }
  }

  async function poll() {
    if (!active || !reachedTip || polling) return;

    polling = true;

    try {
      const [blocks] = await Promise.all([
        chainClient.getBlocks(controller.signal),
        pollProjected(),
      ]);

      if (hasReorganized(blocks, confirmed.find)) {
        await recoverReorganization();
      } else {
        appendNewerBlocks(blocks);
      }
    } catch (error) {
      logChainError("explore chain poll:", error);
    } finally {
      polling = false;
    }
  }

  async function recoverReorganization() {
    element.dataset.reorganizing = "";

    try {
      const tipHash = await loadInitial(null);
      const tipCube = findCube(tipHash);

      if (tipCube) confirmed.scrollTo(tipCube, "instant");
    } finally {
      delete element.dataset.reorganizing;
    }
  }

  async function loadNewer() {
    if (!active || loadingNewer || newestHeight === -1 || reachedTip) return;

    loadingNewer = true;

    try {
      const prevNewest = newestHeight;
      const blocks = await chainClient.getBlocksFromHeight(
        newestHeight + BLOCK_BATCH_SIZE,
        controller.signal,
      );

      if (!appendNewerBlocks(blocks) || newestHeight === prevNewest) {
        reachedTip = true;
        await pollProjected();
      }
    } catch (error) {
      logChainError("explore newer:", error);
    } finally {
      loadingNewer = false;
    }
  }

  /** @param {MempoolBlock[]} blocks */
  function renderProjected(blocks) {
    projected.render(blocks);
    confirmed.markTip();
    refreshProjected();
  }

  function refreshProjected() {
    projected.refresh(newestHeight, newestTimestamp);
  }

  /** @param {Element} element */
  function cubeDistanceFromViewport(element) {
    return distanceFromViewport(scrollElement, element, isHorizontal());
  }

  /** @param {Element} element */
  function isElementVisible(element) {
    return cubeDistanceFromViewport(element) === 0;
  }

  function shouldLoadNewer() {
    const cube = confirmed.newest();

    return cube != null && cubeDistanceFromViewport(cube) <= EDGE_LOAD_DISTANCE;
  }

  scrollElement.addEventListener(
    "wheel",
    (event) => {
      const horizontal = isHorizontal();

      if (horizontal && Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
        scrollElement.scrollLeft += event.deltaY;
      }

      older.reserve(olderWheelDelta(event, horizontal));
    },
    { passive: true },
  );

  scrollElement.addEventListener(
    "scroll",
    () => {
      syncDiagonalScroll(scrollElement, blocksElement);
      tip.schedule();
      older.reserve();

      if (reachedTip || loadingNewer) return;
      if (shouldLoadNewer()) void loadNewer();
    },
    { passive: true },
  );

  syncDiagonalScroll(scrollElement, blocksElement);

  /** @param {boolean} nextActive */
  function setActive(nextActive) {
    if (active === nextActive) return;

    active = nextActive;

    if (active) {
      controller = new AbortController();

      if (newestHeight === -1) void goToCube(null);
      else void poll();

      clockId = window.setInterval(refreshProjected, CLOCK_INTERVAL);
      pollId = window.setInterval(() => void poll(), POLL_INTERVAL);
      return;
    }

    if (pollId !== undefined) {
      window.clearInterval(pollId);
      pollId = undefined;
    }

    if (clockId !== undefined) {
      window.clearInterval(clockId);
      clockId = undefined;
    }

    tip.cancel();
    jump.cancel();
    controller.abort();
  }

  return /** @type {const} */ ({
    element,
    setActive,
  });
}
