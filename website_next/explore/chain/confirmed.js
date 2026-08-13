import { createEnteringConfirmedCube, setConfirmedInterval } from "./block-cube.js";
import { scrollToElement } from "./scroll.js";

/** @typedef {import("../../modules/brk-client/index.js").BlockInfoV1} Block */

/**
 * @param {Object} args
 * @param {HTMLElement} args.scrollElement
 * @param {HTMLElement} args.blocksElement
 * @param {() => Element | null} args.firstProjectedElement
 * @param {(block: Block, cube: HTMLButtonElement) => void} args.onOpen
 * @param {() => void} args.onScrollSelect
 */
export function createConfirmedBlocks({
  scrollElement,
  blocksElement,
  firstProjectedElement,
  onOpen,
  onScrollSelect,
}) {
  /** @type {HTMLButtonElement | null} */
  let tipCube = null;
  /** @type {Map<string, Block>} */
  const blocksByHash = new Map();

  function clear() {
    tipCube = null;
    blocksByHash.clear();
  }

  /** @param {Block} block */
  function cache(block) {
    blocksByHash.set(block.id, block);
  }

  /** @param {string} hash */
  function get(hash) {
    return blocksByHash.get(hash);
  }

  /** @param {string | number} hashOrHeight */
  function find(hashOrHeight) {
    const attribute = typeof hashOrHeight === "number" ? "height" : "hash";

    return /** @type {HTMLButtonElement | null} */ (
      blocksElement.querySelector(`[data-${attribute}="${hashOrHeight}"]`)
    );
  }

  function newest() {
    const firstProjected = firstProjectedElement();

    return /** @type {HTMLButtonElement | null} */ (
      firstProjected
        ? firstProjected.previousElementSibling
        : blocksElement.lastElementChild
    );
  }

  function markTip() {
    tipCube?.removeAttribute("data-tip");
    tipCube = newest();
    tipCube?.setAttribute("data-tip", "");
  }

  /** @param {HTMLButtonElement} cube */
  function open(cube) {
    const hash = cube.dataset.hash;
    const block = hash ? get(hash) : undefined;
    if (block) onOpen(block, cube);
  }

  /**
   * @param {HTMLButtonElement} cube
   * @param {"smooth" | "instant"} behavior
   */
  function scrollTo(cube, behavior) {
    scrollToElement(scrollElement, cube, behavior);
    onScrollSelect();
  }

  function markSkeletons() {
    for (const cube of blocksElement.children) {
      if (!cube.hasAttribute("data-projected")) {
        cube.setAttribute("data-skeleton", "");
      }
    }
  }

  /** @param {Block} block @param {number} [enterIndex] */
  function create(block, enterIndex = 0) {
    cache(block);

    return createEnteringConfirmedCube(block, open, enterIndex);
  }

  /** @param {Block} block @param {number} [enterIndex] */
  function prepend(block, enterIndex = 0) {
    const cube = create(block, enterIndex);
    const oldFirst = /** @type {HTMLElement | null} */ (
      blocksElement.firstElementChild
    );

    blocksElement.insertBefore(cube, oldFirst);
    if (oldFirst) setConfirmedInterval(oldFirst);

    return cube;
  }

  /** @param {Block} block */
  function append(block) {
    const cube = create(block);

    blocksElement.insertBefore(cube, firstProjectedElement());
    setConfirmedInterval(cube);

    return cube;
  }

  return /** @type {const} */ ({
    append,
    cache,
    clear,
    create,
    find,
    get,
    markSkeletons,
    markTip,
    newest,
    scrollTo,
    tipCube: () => tipCube,
    prepend,
  });
}
