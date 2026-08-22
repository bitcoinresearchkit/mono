const OFFSET = 12;
const EDGE_PADDING = 8;

/** @typedef {"auto" | "above"} TooltipPlacement */

/**
 * @param {HTMLElement} parent
 */
export function createTooltipView(parent) {
  const element = document.createElement("div");
  element.hidden = true;
  element.setAttribute("role", "tooltip");
  parent.append(element);

  return {
    /**
     * @param {PointerEvent} event
     * @param {string} text
     * @param {{ placement?: TooltipPlacement }} [options]
     */
    show(event, text, { placement = "auto" } = {}) {
      if (element.textContent !== text) element.textContent = text;
      element.hidden = false;
      place(event, parent, element, placement);
    },
    hide() {
      element.hidden = true;
    },
  };
}

/**
 * @param {PointerEvent} event
 * @param {HTMLElement} parent
 * @param {HTMLElement} element
 * @param {TooltipPlacement} placement
 */
function place(event, parent, element, placement) {
  const parentRect = parent.getBoundingClientRect();
  const x = event.clientX - parentRect.left;
  const y = event.clientY - parentRect.top;
  const width = element.offsetWidth;
  const height = element.offsetHeight;

  let left = placement === "above" ? x - width / 2 : x + OFFSET;
  let top = placement === "above" ? y - height - OFFSET : y + OFFSET;

  if (left + width + EDGE_PADDING > parentRect.width) {
    left = x - width - OFFSET;
  }
  if (placement === "above" && top < EDGE_PADDING) {
    top = y + OFFSET;
  } else if (top + height + EDGE_PADDING > parentRect.height) {
    top = y - height - OFFSET;
  }

  element.style.left = `${clamp(left, EDGE_PADDING, parentRect.width - width - EDGE_PADDING)}px`;
  element.style.top = `${clamp(top, EDGE_PADDING, parentRect.height - height - EDGE_PADDING)}px`;
}

/**
 * @param {number} value
 * @param {number} min
 * @param {number} max
 */
function clamp(value, min, max) {
  return Math.min(Math.max(value, min), Math.max(min, max));
}
