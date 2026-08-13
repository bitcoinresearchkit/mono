import { loadBlockPreview } from "../../block/preview/data.js";
import { createXrayRenderer } from "./renderer.js";

const CACHE_LIMIT = 8;
const DETACH_DELAY_MS = 220;
const INTENT_DELAY_MS = 150;

/** @typedef {import("../../../modules/brk-client/index.js").BlockInfoV1} Block */
/** @typedef {import("../../block/preview/data.js").BlockPreviewData} BlockPreviewData */

/**
 * @param {Object} args
 * @param {HTMLElement} args.blocksElement
 * @param {(hash: string) => Block | undefined} args.getBlock
 */
export function createBlockXray({ blocksElement, getBlock }) {
  const renderer = createXrayRenderer();
  const cache = /** @type {Map<string, BlockPreviewData>} */ (new Map());
  let activeCube = /** @type {HTMLButtonElement | null} */ (null);
  let controller = /** @type {AbortController | null} */ (null);
  let detachId = 0;
  let intentId = 0;
  let pendingCube = /** @type {HTMLButtonElement | null} */ (null);

  /** @param {PointerEvent} event */
  function enter(event) {
    const cube = getEventCube(event);

    if (!cube || cube.contains(/** @type {Node | null} */ (event.relatedTarget))) {
      return;
    }

    cancelIntent();
    pendingCube = cube;
    intentId = window.setTimeout(() => void activate(cube), INTENT_DELAY_MS);
  }

  /** @param {PointerEvent} event */
  function leave(event) {
    const cube = getEventCube(event);

    if (!cube || cube.contains(/** @type {Node | null} */ (event.relatedTarget))) {
      return;
    }

    if (pendingCube === cube) cancelIntent();
    if (activeCube === cube) clearActive();
  }

  /** @param {HTMLButtonElement} cube */
  async function activate(cube) {
    const hash = cube.dataset.hash;
    const block = hash ? getBlock(hash) : undefined;

    pendingCube = null;
    intentId = 0;
    if (!block || !cube.matches(":hover")) return;

    clearActive();
    window.clearTimeout(detachId);
    activeCube = cube;
    if (renderer.attach(cube, hash)) return;

    controller = new AbortController();

    try {
      const data = await load(block, controller.signal);

      if (activeCube === cube && !controller.signal.aborted) {
        renderer.render(hash, data);
      }
    } catch (error) {
      if (!controller.signal.aborted) console.error("explore block x-ray:", error);
    }
  }

  /** @param {Block} block @param {AbortSignal} signal */
  async function load(block, signal) {
    const cached = cache.get(block.id);

    if (cached) {
      cache.delete(block.id);
      cache.set(block.id, cached);
      return cached;
    }

    const data = await loadBlockPreview(block, signal);

    cache.set(block.id, data);
    if (cache.size > CACHE_LIMIT) cache.delete(cache.keys().next().value);

    return data;
  }

  function cancelIntent() {
    window.clearTimeout(intentId);
    intentId = 0;
    pendingCube = null;
  }

  function clearActive() {
    controller?.abort();
    controller = null;
    if (!activeCube) return;

    const cube = activeCube;

    activeCube = null;
    renderer.hide(cube);
    detachId = window.setTimeout(() => renderer.detach(cube), DETACH_DELAY_MS);
  }

  function clear() {
    cancelIntent();
    clearActive();
  }

  /** @param {PointerEvent} event */
  function getEventCube(event) {
    if (event.pointerType !== "mouse" || !(event.target instanceof Element)) {
      return null;
    }

    return /** @type {HTMLButtonElement | null} */ (
      event.target.closest("[data-cube][data-hash]")
    );
  }

  blocksElement.addEventListener("pointerover", enter);
  blocksElement.addEventListener("pointerout", leave);

  return /** @type {const} */ ({ clear });
}
