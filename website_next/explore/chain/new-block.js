const DURATION_MS = 650;
const EASING = "cubic-bezier(0.22, 1, 0.36, 1)";

/**
 * @typedef {{ positions: Map<HTMLElement, DOMRect>, sourceRect: DOMRect | null }} BlockLayout
 */

/**
 * @param {Object} args
 * @param {HTMLElement} args.scrollElement
 * @param {HTMLElement} args.blocksElement
 * @param {() => HTMLElement | null} args.firstProjectedElement
 */
export function createNewBlockTransition({
  scrollElement,
  blocksElement,
  firstProjectedElement,
}) {
  /** @returns {BlockLayout} */
  function capture() {
    const viewport = scrollElement.getBoundingClientRect();
    const positions = new Map();

    for (const child of blocksElement.children) {
      if (!(child instanceof HTMLElement)) continue;

      const rect = child.getBoundingClientRect();
      const nearViewport =
        rect.right >= viewport.left - rect.width &&
        rect.left <= viewport.right + rect.width &&
        rect.bottom >= viewport.top - rect.height &&
        rect.top <= viewport.bottom + rect.height;

      if (nearViewport) positions.set(child, rect);
    }

    return {
      positions,
      sourceRect: firstProjectedElement()?.getBoundingClientRect() ?? null,
    };
  }

  /** @param {BlockLayout} before @param {HTMLElement[]} entering */
  async function play(before, entering) {
    for (const cube of entering) cube.removeAttribute("data-enter");
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    for (const cube of entering) cube.dataset.newBlockEnter = "";

    const animations = [];

    for (const [cube, oldRect] of before.positions) {
      if (!cube.isConnected) continue;

      const newRect = cube.getBoundingClientRect();
      const x = oldRect.left - newRect.left;
      const y = oldRect.top - newRect.top;

      if (Math.abs(x) < 0.5 && Math.abs(y) < 0.5) continue;

      animations.push(
        cube.animate(
          [
            { transform: `translate3d(${x}px, ${y}px, 0)` },
            { transform: "translate3d(0, 0, 0)" },
          ],
          { duration: DURATION_MS, easing: EASING },
        ),
      );
    }

    for (const cube of entering) {
      const visual = cube.querySelector("[data-cube-visual]");
      const newRect = cube.getBoundingClientRect();
      const x = (before.sourceRect?.left ?? newRect.left) - newRect.left;
      const y = (before.sourceRect?.top ?? newRect.top) - newRect.top;

      animations.push(
        cube.animate(
          [
            { transform: `translate3d(${x}px, ${y}px, 0)` },
            { transform: "translate3d(0, 0, 0)" },
          ],
          { duration: DURATION_MS, easing: EASING },
        ),
      );

      if (visual) {
        animations.push(
          visual.animate(
            [
              { opacity: 0, filter: "brightness(2.5)" },
              { offset: 0.35, opacity: 1, filter: "brightness(1.6)" },
              { opacity: 1, filter: "brightness(1)" },
            ],
            { duration: DURATION_MS, easing: EASING },
          ),
        );
      }
    }

    await Promise.allSettled(animations.map(({ finished }) => finished));
    for (const cube of entering) cube.removeAttribute("data-new-block-enter");
  }

  return /** @type {const} */ ({
    capture,
    play,
  });
}
