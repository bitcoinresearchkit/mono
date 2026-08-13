import { createPlaceholderCube, setConfirmedInterval } from "./block-cube.js";
import { olderRemaining, olderRunway } from "./scroll.js";

const OLDER_RESERVE_VIEWPORTS = 3;

/**
 * @typedef {import("../../modules/brk-client/index.js").BlockInfoV1} Block
 * @typedef {{ generation: number, startHeight: number, placeholders: HTMLElement[] }} OlderBatch
 */

/**
 * @param {Object} args
 * @param {HTMLElement} args.scrollElement
 * @param {HTMLElement} args.blocksElement
 * @param {number} args.batchSize
 * @param {() => boolean} args.isActive
 * @param {() => boolean} args.isHorizontal
 * @param {(startHeight: number) => Promise<Block[]>} args.fetchBlocks
 * @param {(block: Block, enterIndex?: number) => HTMLButtonElement} args.createCube
 * @param {() => boolean} args.isAborted
 * @param {(error: unknown) => void} args.onError
 */
export function createOlderBlocks({
  scrollElement,
  blocksElement,
  batchSize,
  isActive,
  isHorizontal,
  fetchBlocks,
  createCube,
  isAborted,
  onError,
}) {
  let oldestHeight = Infinity;
  let oldestReservedHeight = -1;
  let hydrating = false;
  let generation = 0;
  /** @type {OlderBatch[]} */
  const batches = [];

  function reset() {
    oldestHeight = Infinity;
    oldestReservedHeight = -1;
    hydrating = false;
    generation++;
    batches.length = 0;
  }

  /** @param {number} height */
  function setOldestHeight(height) {
    oldestHeight = height;
    oldestReservedHeight = height;
  }

  /**
   * @param {Element | null} anchor
   * @param {number} count
   */
  function prependPlaceholders(anchor, count) {
    const fragment = document.createDocumentFragment();
    const placeholders = /** @type {HTMLElement[]} */ ([]);

    for (let i = 0; i < count; i++) {
      const cube = createPlaceholderCube();

      placeholders.push(cube);
      fragment.append(cube);
    }

    blocksElement.insertBefore(fragment, anchor);

    return placeholders;
  }

  function reserveBatch() {
    if (!isActive() || oldestReservedHeight <= 0) return false;

    const anchor = blocksElement.firstElementChild;
    const count = Math.min(batchSize, oldestReservedHeight);
    const startHeight = oldestReservedHeight - 1;
    const placeholders = prependPlaceholders(anchor, count);

    if (!placeholders.length) return false;

    oldestReservedHeight -= placeholders.length;
    batches.push({ generation, startHeight, placeholders });
    void hydrateBatches();

    return true;
  }

  /** @param {number} [delta] */
  function reserve(delta = 0) {
    if (!isActive() || oldestReservedHeight <= 0) return;

    const horizontal = isHorizontal();
    const runway =
      olderRunway(scrollElement, horizontal, OLDER_RESERVE_VIEWPORTS) + delta;
    let remaining = olderRemaining(scrollElement, horizontal);

    while (remaining < runway) {
      if (!reserveBatch()) return;
      remaining = olderRemaining(scrollElement, horizontal);
    }
  }

  async function hydrateBatches() {
    if (hydrating) return;

    const currentGeneration = generation;

    hydrating = true;

    try {
      while (
        isActive() &&
        currentGeneration === generation &&
        batches[0]?.generation === currentGeneration
      ) {
        await hydrateBatch(batches[0]);
        if (batches[0]?.generation === currentGeneration) batches.shift();
      }
    } finally {
      if (currentGeneration === generation) hydrating = false;
    }
  }

  /** @param {OlderBatch} batch */
  async function hydrateBatch(batch) {
    try {
      const blocks = await fetchBlocks(batch.startHeight);

      if (
        batch.generation !== generation ||
        !batch.placeholders.some((placeholder) => placeholder.isConnected)
      ) {
        return;
      }

      const cubes = [...blocks].reverse().map((block, index, ordered) => {
        return createCube(block, ordered.length - index - 1);
      });

      for (let i = 0; i < batch.placeholders.length; i++) {
        const cube = cubes[i];

        if (cube) batch.placeholders[i].replaceWith(cube);
        else batch.placeholders[i].remove();
      }

      for (const cube of cubes) setConfirmedInterval(cube);

      const next = cubes.at(-1)?.nextElementSibling;
      if (next instanceof HTMLElement) setConfirmedInterval(next);

      if (blocks.length) {
        oldestHeight = blocks[blocks.length - 1].height;
      } else {
        oldestReservedHeight = oldestHeight;
      }

      reserve();
    } catch (error) {
      if (isAborted() || batch.generation !== generation) return;

      for (const placeholder of batch.placeholders) placeholder.remove();
      oldestReservedHeight = oldestHeight;
      onError(error);
    }
  }

  return /** @type {const} */ ({
    reserve,
    reset,
    setOldestHeight,
  });
}
