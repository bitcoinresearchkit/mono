import {
  createProjectedCube,
  setBlockInterval,
  updateProjectedCube,
  updateProjectedHeight,
  updateProjectedTime,
} from "./block-cube.js";
import { formatInterval } from "./format.js";

const PROJECTED_LIMIT = 8;
const TARGET_BLOCK_SECONDS = 600;

/** @typedef {import("../../modules/bitview-client/index.js").MempoolBlock} MempoolBlock */

/**
 * @param {Object} args
 * @param {HTMLElement} args.blocksElement
 * @param {() => boolean} args.isLayoutFrozen
 * @param {(element: Element) => boolean} args.isElementVisible
 */
export function createProjectedBlocks({
  blocksElement,
  isLayoutFrozen,
  isElementVisible,
}) {
  /** @type {ReturnType<typeof createProjectedCube>[]} */
  const cubes = [];
  let renderedHeight = -1;

  function firstElement() {
    return cubes[0]?.element ?? null;
  }

  function clear() {
    cubes.length = 0;
    renderedHeight = -1;
  }

  /** @param {MempoolBlock[]} blocks */
  function render(blocks) {
    const want = Math.min(blocks.length, PROJECTED_LIMIT);

    if (cubes.length !== want) renderedHeight = -1;

    while (cubes.length > want) {
      cubes.pop()?.element.remove();
    }

    while (cubes.length < want) {
      const cube = createProjectedCube(cubes.length + 1);

      if (cubes.length > 0) {
        setBlockInterval(cube.element, TARGET_BLOCK_SECONDS, "~10mn");
      }

      cubes.push(cube);
      blocksElement.append(cube.element);
    }

    for (let i = 0; i < want; i++) {
      updateProjectedCube(cubes[i], blocks[i]);
    }
  }

  /** @param {number} newestHeight @param {number} newestTimestamp */
  function refresh(newestHeight, newestTimestamp) {
    if (!cubes.length || newestHeight < 0 || !newestTimestamp) return;

    const now = Math.floor(Date.now() / 1_000);
    const elapsed = Math.max(0, now - newestTimestamp);
    const heightChanged = renderedHeight !== newestHeight;
    const updateLayout = !isLayoutFrozen();

    for (let i = 0; i < cubes.length; i++) {
      const cube = cubes[i];
      const timestamp = now + i * TARGET_BLOCK_SECONDS;

      if (updateLayout && i === 0) {
        setBlockInterval(cube.element, elapsed, formatInterval(elapsed));
      }

      if (heightChanged) updateProjectedHeight(cube, newestHeight + i + 1);
      updateProjectedTime(cube, timestamp);
    }

    renderedHeight = newestHeight;
  }

  function hasVisibleElement() {
    return cubes.some(({ element }) => isElementVisible(element));
  }

  return /** @type {const} */ ({
    clear,
    firstElement,
    hasVisibleElement,
    refresh,
    render,
  });
}
