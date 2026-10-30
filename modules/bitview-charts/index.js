// @ts-check
// bitview-charts: time series charts drawn from columns. A chart is panes stacked over one time axis, every pane on the
// same timeline (one time a logical index); each pane's lines are columns on it (a Float64Array a line, NaN for none).
// Made for many series of thousands of points on a phone without its JIT: no object a point, nothing made a frame, a
// pan moving the picture already drawn where it can.
//
//   const chart = createChart(element);
//   chart.setTimes(times);                                   // Float64Array of seconds, rising
//   const pane = chart.addPane({ scale: { mode: "log" } });
//   pane.addLine({ values, color: "#f7931a" });              // Float64Array, an index a time
//   chart.show(365);                                         // the last year in view
//
// Dragged, a pane pans (a flick glides on, slowing as iOS's lists do); pinched or wheeled, it zooms about that point;
// a trackpad's sideways scroll pans.

import { Timeline } from "./timeline.js";
import { Pane } from "./pane.js";
import { TimeAxis } from "./axis.js";

export { Timeline, Pane, TimeAxis };
export { Line } from "./line.js";
export { Band } from "./band.js";
export { Bars } from "./bars.js";
export { Candles } from "./candles.js";
export { Scale } from "./scale.js";
export { compact } from "./format.js";

/** @type {import("./pane.js").Theme} */
export const DARK = { background: "#0e0f11", text: "#8a8f98", grid: "rgba(255, 255, 255, 0.06)", font: "11px system-ui, sans-serif" };
/** @type {import("./pane.js").Theme} */
export const LIGHT = { background: "#ffffff", text: "#6b6f76", grid: "rgba(0, 0, 0, 0.06)", font: "11px system-ui, sans-serif" };

/** A flick's speed kept each millisecond (iOS's lists' rate). */
const SLOWING = 0.998;

/**
 * @param {HTMLElement} element
 * @param {{ theme?: import("./pane.js").Theme, axis?: number, timeHeight?: number, rightOffset?: number, maxSpacing?: number }} [options]
 */
export function createChart(element, options) {
  return new Chart(element, options);
}

export class Chart {
  /**
   * @param {HTMLElement} element
   * @param {{ theme?: import("./pane.js").Theme, axis?: number, timeHeight?: number, rightOffset?: number, maxSpacing?: number }} [options]
   */
  constructor(element, { theme = DARK, axis = 64, timeHeight = 26, rightOffset, maxSpacing } = {}) {
    this.element = element;
    this.theme = theme;
    /** The axis' width at every pane's right (CSS pixels). */
    this.axis = axis;
    this.timeHeight = timeHeight;
    this.time = new Timeline({ rightOffset, maxSpacing });
    this.timeAxis = new TimeAxis();
    /** @type {Pane[]} */
    this.panes = [];
    this.dpr = 1;
    this.width = 0;
    this.height = 0;
    // (A frame asked for; drawn whole (sizes, panes or data changed); moved since the last frame; pictures moved, to
    // draw whole at the first frame nothing moves.)
    this.frame = 0;
    this.whole = true;
    this.moved = false;
    this.seams = false;
    /** A flick gliding on: its speed (CSS pixels a millisecond) and its last frame's time. */
    this.glide = { speed: 0, at: 0 };
    /** @type {((from: number, to: number) => void)[]} */
    this.rangeListeners = [];
    this.drawnFrom = NaN;
    this.drawnTo = NaN;
    // (Its frame's callback, made once: none made each frame.)
    this.tick = (/** @type {number} */ now) => this.#frame(now);
    /** Its listeners, gone with it (see destroy). */
    this.listening = new AbortController();
    // (Its canvases placed in it: a box for them, unless it's one already.)
    if (getComputedStyle(element).position === "static") element.style.position = "relative";
    Object.assign(element.style, { overflow: "hidden", touchAction: "none", userSelect: "none", webkitUserSelect: "none" });
    element.append(this.timeAxis.canvas);
    this.resizer = new ResizeObserver(() => this.layout());
    this.resizer.observe(element);
    this.#input();
    this.#watchDpr();
    this.layout();
  }

  /** Laid out again when the device's pixels a CSS pixel change (another display, a zoom of the page): its size alone wouldn't say. */
  #watchDpr() {
    const query = matchMedia(`(resolution: ${devicePixelRatio || 1}dppx)`);
    query.addEventListener("change", () => (this.layout(), this.#watchDpr()), { once: true, signal: this.listening.signal });
  }

  /** The timeline's times (seconds, rising), every pane's lines' indices. @param {Float64Array} times */
  setTimes(times) {
    this.time.setTimes(times);
    this.invalidate(true);
  }

  /** @param {ConstructorParameters<typeof Pane>[0]} [options] */
  addPane(options) {
    const pane = new Pane(options);
    this.panes.push(pane);
    this.element.insertBefore(pane.canvas, this.timeAxis.canvas);
    this.layout();
    return pane;
  }

  /** @param {Pane} pane */
  removePane(pane) {
    this.panes = this.panes.filter((each) => each !== pane);
    pane.canvas.remove();
    this.layout();
  }

  /** The last `span` points in view. @param {number} span */
  show(span) {
    this.time.show(span);
    this.invalidate(true);
  }

  /** The view: logical indices, fractions allowed. @param {number} from @param {number} to */
  setRange(from, to) {
    this.time.setRange(from, to);
    this.invalidate(true);
  }

  get range() {
    return { from: this.time.from, to: this.time.to };
  }

  /** Told each frame the view changed in. @param {(from: number, to: number) => void} listener */
  onRange(listener) {
    this.rangeListeners.push(listener);
    return () => void (this.rangeListeners = this.rangeListeners.filter((each) => each !== listener));
  }

  /** Laid out again at the element's size (watched: needed only for a change it can't see). */
  layout() {
    const box = this.element.getBoundingClientRect();
    this.dpr = devicePixelRatio || 1;
    this.width = box.width;
    this.height = box.height;
    const room = Math.max(0, box.height - this.timeHeight), shares = this.panes.reduce((sum, pane) => sum + pane.share, 0) || 1;
    let top = 0;
    for (const [k, pane] of this.panes.entries()) {
      const bottom = k === this.panes.length - 1 ? room : Math.round(top + (room * pane.share) / shares);
      pane.place(top, box.width, bottom - top, this.axis, this.dpr);
      top = bottom;
    }
    this.timeAxis.place(room, box.width, this.timeHeight, this.dpr);
    this.time.resize(Math.max(1, box.width - this.axis));
    this.invalidate(true);
  }

  /**
   * A frame asked for: the view moved (a pan, a zoom), or `whole`, everything drawn anew (sizes, panes, a series'
   * values or look, the theme). After changing a series, ask for one.
   * @param {boolean} [whole]
   */
  invalidate(whole = false) {
    this.whole ||= whole;
    this.moved ||= !whole;
    this.frame ||= requestAnimationFrame(this.tick);
  }

  /** @param {number} now */
  #frame(now) {
    this.frame = 0;
    const glide = this.glide;
    if (glide.speed) {
      const dt = Math.min(64, now - glide.at);
      glide.at = now;
      glide.speed *= SLOWING ** dt;
      if (Math.abs(glide.speed) < 0.02 || !this.time.pan(glide.speed * dt)) glide.speed = 0;
      else (this.moved = true), (this.frame = requestAnimationFrame(this.tick));
    }
    // (Nothing moved since the last frame, its pictures moved or stretched before: drawn whole now, the seams gone; till
    // then they're kept, a move of less than a pixel moving nothing.)
    const whole = this.whole || (this.seams && !this.moved);
    this.moved = false;
    if (whole) this.seams = false;
    this.render(whole);
    if (this.seams) this.frame ||= requestAnimationFrame(this.tick);
  }

  /** Drawn now (for a measurement: its frames draw it). @param {boolean} [whole] */
  render(whole = false) {
    this.whole = false;
    const time = this.time;
    for (let k = 0; k < this.panes.length; k++) {
      const pane = this.panes[k];
      pane.fit(time);
      if (pane.draw(time, this.theme, this.dpr, whole)) this.seams = true;
    }
    this.timeAxis.draw(time, this.theme, this.dpr, time.width);
    if (time.from !== this.drawnFrom || time.to !== this.drawnTo) {
      (this.drawnFrom = time.from), (this.drawnTo = time.to);
      for (let k = 0; k < this.rangeListeners.length; k++) this.rangeListeners[k](time.from, time.to);
    }
  }

  // Pointers: one drags (a pan, gliding on once let go), two pinch (a zoom about their middle, and a pan as it moves);
  // the wheel zooms about the pointer, a trackpad's sideways scroll pans.
  #input() {
    const element = this.element, time = this.time, down = /** @type {Map<number, { x: number }>} */ (new Map());
    // (The last tenth of a second of a drag: its speed as it's let go.)
    const trail = new Float64Array(16), trailAt = new Float64Array(16);
    let trailEnd = 0, trailSize = 0;
    const plotX = (/** @type {number} */ clientX) => clientX - element.getBoundingClientRect().left;
    const signal = this.listening.signal;
    element.addEventListener("pointerdown", (event) => {
      if (event.button) return;
      element.setPointerCapture(event.pointerId);
      down.set(event.pointerId, { x: event.clientX });
      this.glide.speed = 0;
      trailSize = 0;
    }, { signal });
    element.addEventListener("pointermove", (event) => {
      const point = down.get(event.pointerId);
      if (!point) return;
      if (down.size === 1) {
        time.pan(event.clientX - point.x);
        trail[trailEnd] = event.clientX;
        trailAt[trailEnd] = event.timeStamp;
        trailEnd = (trailEnd + 1) % trail.length;
        trailSize = Math.min(trail.length, trailSize + 1);
      } else if (down.size === 2) {
        // (About the fingers' middle as it was, then moved with it: what's under each stays under it.)
        let other = point;
        for (const each of down.values()) if (each !== point) other = each;
        const was = Math.abs(point.x - other.x), now = Math.abs(event.clientX - other.x);
        if (was > 8 && now > 8) time.zoom(plotX((point.x + other.x) / 2), was / now);
        time.pan((event.clientX - point.x) / 2);
      }
      point.x = event.clientX;
      this.invalidate();
    }, { signal });
    const up = (/** @type {PointerEvent} */ event) => {
      if (!down.delete(event.pointerId)) return;
      if (down.size || event.type === "pointercancel" || trailSize < 2) return void (trailSize = 0);
      // (Its speed over its last tenth of a second; held still that long, none.)
      const last = (trailEnd + trail.length - 1) % trail.length;
      let first = last;
      for (let k = 1; k < trailSize; k++) {
        const i = (last - k + trail.length) % trail.length;
        if (trailAt[last] - trailAt[i] > 100) break;
        first = i;
      }
      const ms = trailAt[last] - trailAt[first], stale = event.timeStamp - trailAt[last] > 60;
      trailSize = 0;
      if (ms > 0 && !stale) (this.glide.speed = (trail[last] - trail[first]) / ms), (this.glide.at = performance.now()), this.invalidate();
    };
    element.addEventListener("pointerup", up, { signal });
    element.addEventListener("pointercancel", up, { signal });
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? this.height : 1;
      if (event.ctrlKey || Math.abs(event.deltaY) >= Math.abs(event.deltaX)) time.zoom(plotX(event.clientX), Math.exp(event.deltaY * scale * (event.ctrlKey ? 0.01 : 0.002)));
      else time.pan(-event.deltaX * scale);
      this.glide.speed = 0;
      this.invalidate();
    }, { passive: false, signal });
  }

  destroy() {
    cancelAnimationFrame(this.frame);
    this.frame = 0;
    this.glide.speed = 0;
    this.listening.abort();
    this.resizer.disconnect();
    for (const pane of this.panes) pane.canvas.remove();
    this.timeAxis.canvas.remove();
    this.panes = [];
  }
}
