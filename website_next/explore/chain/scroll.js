/**
 * @param {HTMLElement} scrollElement
 * @param {Element} target
 * @param {"smooth" | "instant"} behavior
 */
export function scrollToElement(scrollElement, target, behavior) {
  const viewport = scrollElement.getBoundingClientRect();
  const rect = target.getBoundingClientRect();

  scrollElement.scrollTo({
    behavior,
    left:
      scrollElement.scrollLeft +
      rect.left -
      viewport.left -
      (viewport.width - rect.width) / 2,
    top: 0,
  });
}

/**
 * @param {HTMLElement} scrollElement
 * @param {Element | null | undefined} anchor
 * @param {DOMRect | undefined} anchorRect
 */
export function preserveScrollPosition(scrollElement, anchor, anchorRect) {
  if (!anchor || !anchorRect) return;

  const rect = anchor.getBoundingClientRect();

  scrollElement.scrollLeft += rect.left - anchorRect.left;
  scrollElement.scrollTop = 0;
}

/**
 * @param {HTMLElement} scrollElement
 * @param {HTMLElement} blocksElement
 */
export function syncDiagonalScroll(scrollElement, blocksElement) {
  scrollElement.scrollTop = 0;
  blocksElement.style.setProperty(
    "--chain-scroll-x",
    `${scrollElement.scrollLeft}px`,
  );
}

/** @param {HTMLElement} blocksElement */
export function isHorizontalLayout(blocksElement) {
  return getComputedStyle(blocksElement).flexDirection.startsWith("row");
}

/** @param {HTMLElement} scrollElement @param {boolean} horizontal */
export function olderRemaining(scrollElement, horizontal) {
  return horizontal
    ? scrollElement.scrollWidth -
        scrollElement.clientWidth -
        scrollElement.scrollLeft
    : scrollElement.scrollHeight -
        scrollElement.clientHeight -
        scrollElement.scrollTop;
}

/**
 * @param {HTMLElement} scrollElement
 * @param {boolean} horizontal
 * @param {number} viewports
 */
export function olderRunway(scrollElement, horizontal, viewports) {
  return (
    (horizontal ? scrollElement.clientWidth : scrollElement.clientHeight) *
    viewports
  );
}

/** @param {WheelEvent} event @param {boolean} horizontal */
export function olderWheelDelta(event, horizontal) {
  return Math.max(
    0,
    horizontal ? Math.max(event.deltaX, event.deltaY) : event.deltaY,
  );
}

/**
 * @param {HTMLElement} scrollElement
 * @param {Element} element
 * @param {boolean} horizontal
 */
export function distanceFromViewport(scrollElement, element, horizontal) {
  const viewport = scrollElement.getBoundingClientRect();
  const rect = element.getBoundingClientRect();

  if (horizontal) {
    if (rect.left > viewport.right) return rect.left - viewport.right;
    if (rect.right < viewport.left) return viewport.left - rect.right;
    return 0;
  }

  if (rect.top > viewport.bottom) return rect.top - viewport.bottom;
  if (rect.bottom < viewport.top) return viewport.top - rect.bottom;
  return 0;
}

/**
 * @param {HTMLElement} scrollElement
 * @param {HTMLElement} blocksElement
 */
export function findVisibleConfirmedHeight(scrollElement, blocksElement) {
  const viewport = scrollElement.getBoundingClientRect();
  const x = (viewport.left + viewport.right) / 2;
  const y = (viewport.top + viewport.bottom) / 2;

  for (const element of document.elementsFromPoint(x, y)) {
    const cube = element.closest("[data-cube][data-height]");

    if (
      cube instanceof HTMLElement &&
      blocksElement.contains(cube) &&
      !cube.hasAttribute("data-projected")
    ) {
      return Number(cube.dataset.height);
    }
  }

  return null;
}
