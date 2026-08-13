const COVER_SCALE = 1.15;
const DURATION_MS = 700;
const EXPAND_OFFSET = 0.82;
const EXIT_SCALE = 1.08;

/** @param {HTMLElement} container */
export function createBlockOpenTransition(container) {
  const animations = /** @type {Animation[]} */ ([]);
  let backdrop = /** @type {HTMLDivElement | null} */ (null);
  let overlay = /** @type {HTMLButtonElement | null} */ (null);

  /** @param {HTMLButtonElement} cube */
  async function play(cube) {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const bounds = container.getBoundingClientRect();
    const rect = cube.getBoundingClientRect();
    const left = rect.left - bounds.left;
    const top = rect.top - bounds.top;
    const x = bounds.width / 2 - left - rect.width / 2;
    const y = bounds.height / 2 - top - rect.height / 2;
    const scale = COVER_SCALE * Math.max(
      bounds.width / rect.width,
      bounds.height / rect.height,
    );

    overlay = /** @type {HTMLButtonElement} */ (cube.cloneNode(true));
    overlay.querySelector("[data-block-interval]")?.remove();
    overlay.removeAttribute("data-enter");
    overlay.removeAttribute("data-press");
    overlay.removeAttribute("data-tip");
    overlay.removeAttribute("title");
    overlay.dataset.opening = "";
    overlay.tabIndex = -1;
    overlay.ariaHidden = "true";
    Object.assign(overlay.style, {
      bottom: "auto",
      left: `${left}px`,
      margin: "0",
      pointerEvents: "none",
      position: "fixed",
      right: "auto",
      top: `${top}px`,
      transformOrigin: "center",
      willChange: "transform",
      zIndex: "100",
    });

    backdrop = document.createElement("div");
    backdrop.dataset.blockOpeningBackdrop = "";
    Object.assign(backdrop.style, {
      background: "var(--black)",
      inset: "0",
      opacity: "0",
      pointerEvents: "none",
      position: "fixed",
      zIndex: "99",
    });

    container.style.zIndex = "calc(var(--layer-header) + 1)";
    container.append(backdrop, overlay);
    animations.push(
      overlay.animate(
        [
          {
            easing: "cubic-bezier(0.22, 1, 0.36, 1)",
            offset: 0,
            opacity: 1,
            transform: "translate3d(0, 0, 0) scale(1)",
          },
          {
            easing: "cubic-bezier(0.65, 0, 0.35, 1)",
            offset: 0.1,
            opacity: 1,
            transform: "translate3d(0, 0, 0) scale(1.04)",
          },
          {
            easing: "ease-in",
            offset: EXPAND_OFFSET,
            opacity: 1,
            transform: `translate3d(${x}px, ${y}px, 0) scale(${scale})`,
          },
          {
            offset: 1,
            opacity: 0,
            transform: `translate3d(${x}px, ${y}px, 0) scale(${scale * EXIT_SCALE})`,
          },
        ],
        {
          duration: DURATION_MS,
          fill: "forwards",
        },
      ),
      backdrop.animate(
        [
          { offset: 0, opacity: 0 },
          { offset: 0.1, opacity: 0.15 },
          { offset: EXPAND_OFFSET, opacity: 1 },
          { offset: 1, opacity: 1 },
        ],
        {
          duration: DURATION_MS,
          easing: "cubic-bezier(0.65, 0, 0.35, 1)",
          fill: "forwards",
        },
      ),
    );

    await Promise.allSettled(animations.map(({ finished }) => finished));
  }

  function clear() {
    for (const animation of animations) animation.cancel();
    animations.length = 0;
    backdrop?.remove();
    overlay?.remove();
    container.style.removeProperty("z-index");
    backdrop = null;
    overlay = null;
  }

  return /** @type {const} */ ({ clear, play });
}
