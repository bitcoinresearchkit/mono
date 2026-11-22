// @ts-check
// A chart's navigator: a strip of its whole history (the page paints it), the view a window on it. The window dragged
// pans the view, an edge of it moves that end (a zoom), a press off it glides the view there (centred, as long as it
// was) and a drag goes on from where the glide has got to. A finger's mostly vertical swipe on it scrolls the page
// (`touch-action: pan-y`), so a finger's press is taken only once it moves more sideways than up or down, or lifts: a
// scroll that starts on it neither jumps nor stops the view. One behaviour for every page that has one.
import { press } from "./index.js";

/** The fewest points an edge's drag leaves in view. */
const FEWEST = 10;
/** CSS pixels a press moves before it counts as moved (a drag while its jump glides, a finger's sideways move): less is jitter. */
const JITTER = 3;

/**
 * The window's edges as shown, in pixels of a strip `width` wide: no narrower than `least`, within the strip.
 * @param {number} left @param {number} right @param {number} width @param {number} least @returns {[number, number]}
 */
export function windowOn(left, right, width, least) {
  const from = Math.min(left, width - least);
  return [from, from + Math.max(right - left, least)];
}

/** @typedef {{ from: number, to: number }} Range */
/**
 * What a page tells its navigator.
 * @typedef {object} NavigatorHooks
 * @property {() => Range | null} overview the points across the strip as it's painted now; null, nothing to navigate
 * @property {() => number} length the points across it at rest: a drag's scale
 * @property {() => unknown} data what the chart shows: changed under a drag, the drag's taken up again from the view
 * @property {(range: Range, keep: "from" | "to" | "middle") => Range} within a view no longer than the page shows, keeping the end named
 * @property {(range: Range) => void} glide the page's eased move of the view
 * @property {HTMLElement} window the element that's `dragging` while the window's held
 * @property {number} least the window's least width shown, CSS pixels (see windowOn)
 * @property {number} grip how near an edge, CSS pixels, a press takes it
 * @property {() => void} [pressed] told as a press takes the window, its hit test done
 * @property {() => void} [dropped] told as the press lets go, `dragging` gone
 * @property {AbortSignal} [signal]
 */

export class Navigator {
  /** The window held: by its body or an edge, where from, the view then, and the scale. @type {{ mode: "pan" | "left" | "right", x: number, perPixel: number, range: Range, data: unknown, width: number } | null} */
  #drag = null;
  #element;
  #chart;
  #hooks;

  /** @param {HTMLElement} element the strip @param {import("./index.js").Chart} chart @param {NavigatorHooks} hooks */
  constructor(element, chart, hooks) {
    this.#element = element;
    this.#chart = chart;
    this.#hooks = hooks;
    const listen = { signal: hooks.signal };
    element.style.touchAction = "pan-y";
    element.addEventListener("pointerdown", (event) => this.#down(event), listen);
    // (Pointed at, not held: the cursor says an edge can be dragged.)
    element.addEventListener("pointermove", (event) => {
      const range = chart.controller.range, span = range && hooks.overview();
      if (this.#drag || !range || !span) return;
      const rect = element.getBoundingClientRect(), x = event.clientX - rect.left, { left, right } = this.#place(rect.width, span, range);
      element.style.cursor = Math.abs(x - left) <= hooks.grip || Math.abs(x - right) <= hooks.grip ? "ew-resize" : "";
    }, listen);
  }

  /** Whether the window's held. */
  get dragging() {
    return this.#drag !== null;
  }

  /** The window on a strip `width` pixels wide over `span`: its edges (see windowOn), and the points a pixel. @param {number} width @param {Range} span @param {Range} range */
  #place(width, span, range) {
    const perPixel = (span.to - span.from) / width, px = (/** @type {number} */ at) => Math.max(0, Math.min(width, (at - span.from) / perPixel));
    const [left, right] = windowOn(px(range.from), px(range.to), width, this.#hooks.least);
    return { perPixel, left, right };
  }

  /** A press: a mouse's taken at once; a finger's once it has moved more sideways than up or down, or as it lifts (a
   * tap: its jump). A vertical swipe is the browser's to scroll, and it cancels the press only after sending a few
   * moves (iOS). @param {PointerEvent} event */
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

  /** The window taken by a press: its hit test, a jump's glide or the view stopped, and held. Whether there's a view to
   * take. @param {PointerEvent} event */
  #take(event) {
    const element = this.#element, controller = this.#chart.controller, hooks = this.#hooks;
    const range = controller.range, span = range && hooks.overview();
    if (!range || !span) return controller.stop(), false;
    const rect = element.getBoundingClientRect(), x = event.clientX - rect.left;
    // (Hit-tested on the strip as it's painted (mid-move, too), which then lands where it was going.)
    const { perPixel, left, right } = this.#place(rect.width, span, range), at = (/** @type {number} */ px) => span.from + px * perPixel;
    hooks.pressed?.();
    /** @type {"pan" | "left" | "right"} */
    let mode = "pan";
    if (Math.abs(x - left) <= hooks.grip) mode = "left";
    else if (Math.abs(x - right) <= hooks.grip) mode = "right";
    let start = range;
    if (mode === "pan" && (x < left || x > right)) {
      // (A jump: the same window centred where it's pressed, glided to as every move of the view's goes, the drag
      // going on from there; aimed where the chart holds it on its data.)
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

  /** The window dragged by its press's pointer (see press). @param {PointerEvent} event */
  #move(event) {
    const drag = this.#drag, controller = this.#chart.controller, hooks = this.#hooks;
    if (!drag) return;
    const rect = this.#element.getBoundingClientRect(), x = event.clientX - rect.left;
    // (Dragged on as its jump glides: from where the view is now, the glide given up; a click's jitter, not a drag.)
    if (controller.target) {
      if (Math.abs(x - drag.x) < JITTER) return;
      const now = controller.range;
      controller.stop();
      if (now) Object.assign(drag, { x, range: now });
      return;
    }
    // (The chart's data or the strip changed under it (another interval, a block, a resize): taken up again from
    // here, not moved by the old one's points a pixel.)
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
    // (An edge dragged out past the most the view shows: the other edge gives way.)
    const view = hooks.within({ from, to }, drag.mode === "left" ? "to" : drag.mode === "right" ? "from" : "middle");
    this.#chart.setRange(view.from, view.to);
  }

  /** The press let go (see press). */
  #drop() {
    if (!this.#drag) return;
    this.#drag = null;
    this.#hooks.window.classList.remove("dragging");
    this.#hooks.dropped?.();
  }
}
