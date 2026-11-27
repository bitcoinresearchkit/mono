// @ts-check
import { press } from "./index.js";

const FEWEST = 10;
const JITTER = 3;

/**
 * @param {number} left @param {number} right @param {number} width @param {number} least @returns {[number, number]}
 */
export function windowOn(left, right, width, least) {
  const from = Math.min(left, width - least);
  return [from, from + Math.max(right - left, least)];
}

/** @typedef {{ from: number, to: number }} Range */
/**
 * @typedef {object} NavigatorHooks
 * @property {() => Range | null} overview null: nothing to navigate
 * @property {() => number} length
 * @property {() => unknown} data
 * @property {(range: Range, keep: "from" | "to" | "middle") => Range} within
 * @property {(range: Range) => void} glide
 * @property {HTMLElement} window
 * @property {number} least
 * @property {number} grip
 * @property {() => void} [pressed]
 * @property {() => void} [dropped]
 * @property {AbortSignal} [signal]
 */

export class Navigator {
  /** @type {{ mode: "pan" | "left" | "right", x: number, perPixel: number, range: Range, data: unknown, width: number } | null} */
  #drag = null;
  #element;
  #chart;
  #hooks;

  /** @param {HTMLElement} element @param {import("./index.js").Chart} chart @param {NavigatorHooks} hooks */
  constructor(element, chart, hooks) {
    this.#element = element;
    this.#chart = chart;
    this.#hooks = hooks;
    const listen = { signal: hooks.signal };
    element.style.touchAction = "pan-y";
    element.addEventListener("pointerdown", (event) => this.#down(event), listen);
    element.addEventListener("pointermove", (event) => {
      const range = chart.controller.range, span = range && hooks.overview();
      if (this.#drag || !range || !span) return;
      const rect = element.getBoundingClientRect(), x = event.clientX - rect.left, { left, right } = this.#place(rect.width, span, range);
      element.style.cursor = Math.abs(x - left) <= hooks.grip || Math.abs(x - right) <= hooks.grip ? "ew-resize" : "";
    }, listen);
  }

  get dragging() {
    return this.#drag !== null;
  }

  /** @param {number} width @param {Range} span @param {Range} range */
  #place(width, span, range) {
    const perPixel = (span.to - span.from) / width, px = (/** @type {number} */ at) => Math.max(0, Math.min(width, (at - span.from) / perPixel));
    const [left, right] = windowOn(px(range.from), px(range.to), width, this.#hooks.least);
    return { perPixel, left, right };
  }

  /**
   * A finger's press waits till it moves more sideways than up, or lifts: iOS cancels a vertical swipe's press only after a few moves.
   * @param {PointerEvent} event
   */
  #down(event) {
    if (event.button !== 0 || this.#drag) return;
    let taken = event.pointerType === "mouse" && this.#take(event);
    const sideways = (/** @type {PointerEvent} */ at) => Math.abs(at.clientX - event.clientX) > Math.max(JITTER, Math.abs(at.clientY - event.clientY));
    this.#element.setPointerCapture(event.pointerId);
    press(event, {
      captor: this.#element,
      move: (moved) => (taken ||= sideways(moved) && this.#take(event)) && this.#move(moved),
      end: (release) => (release && !taken && this.#take(event), this.#drop()),
    });
  }

  /** @param {PointerEvent} event */
  #take(event) {
    const element = this.#element, controller = this.#chart.controller, hooks = this.#hooks;
    const range = controller.range, span = range && hooks.overview();
    if (!range || !span) return controller.stop(), false;
    const rect = element.getBoundingClientRect(), x = event.clientX - rect.left;
    const { perPixel, left, right } = this.#place(rect.width, span, range), at = (/** @type {number} */ px) => span.from + px * perPixel;
    hooks.pressed?.();
    /** @type {"pan" | "left" | "right"} */
    let mode = "pan";
    if (Math.abs(x - left) <= hooks.grip) mode = "left";
    else if (Math.abs(x - right) <= hooks.grip) mode = "right";
    let start = range;
    if (mode === "pan" && (x < left || x > right)) {
      const half = (range.to - range.from) / 2, time = this.#chart.time;
      start = hooks.within({ from: at(x) - half, to: at(x) + half }, "middle");
      const by = time.length ? time.held(start.from, start.to) : 0;
      if (by) start = { from: start.from + by, to: start.to + by };
      hooks.glide(start);
    } else controller.stop();
    this.#drag = { mode, x, perPixel: hooks.length() / rect.width, range: start, data: hooks.data(), width: rect.width };
    hooks.window.classList.add("dragging");
    return true;
  }

  /** @param {PointerEvent} event */
  #move(event) {
    const drag = this.#drag, controller = this.#chart.controller, hooks = this.#hooks;
    if (!drag) return;
    const rect = this.#element.getBoundingClientRect(), x = event.clientX - rect.left;
    if (controller.target) {
      if (Math.abs(x - drag.x) < JITTER) return;
      const now = controller.range;
      controller.stop();
      if (now) Object.assign(drag, { x, range: now });
      return;
    }
    if (drag.data !== hooks.data() || drag.width !== rect.width) {
      const now = controller.range;
      if (!now || !hooks.overview()) return;
      Object.assign(drag, { x, perPixel: hooks.length() / rect.width, range: now, data: hooks.data(), width: rect.width });
    }
    const delta = (x - drag.x) * drag.perPixel;
    let { from, to } = drag.range;
    if (drag.mode === "pan") (from += delta), (to += delta);
    else if (drag.mode === "left") from = Math.min(from + delta, to - FEWEST);
    else to = Math.max(to + delta, from + FEWEST);
    const view = hooks.within({ from, to }, drag.mode === "left" ? "to" : drag.mode === "right" ? "from" : "middle");
    this.#chart.setRange(view.from, view.to);
  }

  #drop() {
    if (!this.#drag) return;
    this.#drag = null;
    this.#hooks.window.classList.remove("dragging");
    this.#hooks.dropped?.();
  }
}
