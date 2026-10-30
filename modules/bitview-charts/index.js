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
import { Crosshair } from "./crosshair.js";

export { Timeline, Pane, TimeAxis };
export { Line } from "./line.js";
export { Band } from "./band.js";
export { Bars } from "./bars.js";
export { Candles } from "./candles.js";
export { stack } from "./stack.js";
export { Custom } from "./custom.js";
export { picture } from "./picture.js";
export { Scale } from "./scale.js";
export { compact } from "./format.js";

/** @type {import("./pane.js").Theme} */
export const DARK = { background: "#0e0f11", text: "#8a8f98", grid: "rgba(255, 255, 255, 0.06)", font: "11px system-ui, sans-serif", crosshair: "rgba(247, 248, 248, 0.4)", badge: "#f7f8f8", badgeText: "#0e0f11", separator: "rgba(255, 255, 255, 0.14)" };
/** @type {import("./pane.js").Theme} */
export const LIGHT = { background: "#ffffff", text: "#6b6f76", grid: "rgba(0, 0, 0, 0.06)", font: "11px system-ui, sans-serif", crosshair: "rgba(14, 15, 17, 0.35)", badge: "#0e0f11", badgeText: "#ffffff", separator: "rgba(0, 0, 0, 0.12)" };

/** A touch held this long (ms) without moving shows the crosshair, the finger moving it then, not the chart. */
const HOLD = 350;
/** How near a separator between panes a pointer takes it (CSS pixels): a mouse's, a finger's; a pane's least height. */
const NEAR_MOUSE = 4, NEAR_TOUCH = 12, LEAST = 32;

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
    this.crosshair = new Crosshair();
    /** @type {((index: number, pane: number) => void)[]} */
    this.crosshairListeners = [];
    /** @type {((index: number, pane: number, series: number) => void)[]} */
    this.clickListeners = [];
    /** @type {Pane[]} */
    this.panes = [];
    this.dpr = 1;
    /** The left axis' width (0: none, no pane has a left scale): the plot after it. */
    this.left = 0;
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
    /** A touch's hold waiting to show the crosshair (a timer; 0: none). */
    this.holding = 0;
    // (Its canvases placed in it: a box for them, unless it's one already.)
    if (getComputedStyle(element).position === "static") element.style.position = "relative";
    Object.assign(element.style, { overflow: "hidden", touchAction: "none", userSelect: "none", webkitUserSelect: "none" });
    element.append(this.timeAxis.canvas, this.crosshair.canvas);
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
    // (The crosshair kept on a point there is, or gone.)
    if (this.crosshair.index === this.crosshair.index) this.setCrosshair(this.crosshair.index, this.crosshair.y);
    this.invalidate(true);
  }

  /** @param {ConstructorParameters<typeof Pane>[0]} [options] */
  addPane(options) {
    const pane = new Pane(options);
    this.panes.push(pane);
    this.element.insertBefore(pane.canvas, this.timeAxis.canvas);
    // (Its left scale come: room made for it.)
    pane.onLeft = () => this.layout();
    this.layout();
    return pane;
  }

  /** @param {Pane} pane */
  removePane(pane) {
    this.panes = this.panes.filter((each) => each !== pane);
    pane.canvas.remove();
    pane.leftCanvas?.remove();
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

  /**
   * The crosshair on a point (a logical index: the nearest whole one) at a height in the chart (CSS pixels from its top),
   * or gone (NaN). Told to the crosshair's listeners.
   * @param {number} index @param {number} [y]
   */
  setCrosshair(index, y = NaN) {
    const k = index === index && this.time.length ? Math.min(Math.max(Math.round(index), 0), this.time.length - 1) : NaN, crosshair = this.crosshair;
    let pane = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    const moved = k !== crosshair.index && !(k !== k && crosshair.index !== crosshair.index);
    (crosshair.index = k), (crosshair.y = y), (crosshair.pane = pane);
    if (moved) for (let l = 0; l < this.crosshairListeners.length; l++) this.crosshairListeners[l](k, pane);
    this.invalidate();
  }

  /**
   * Told of a click or a tap (a press let go where it began, the crosshair not shown by it): its point (a logical index,
   * the nearest whole one), its pane (-1: none), and the series hit there (its index in the pane, -1: none).
   * @param {(index: number, pane: number, series: number) => void} listener
   */
  onClick(listener) {
    this.clickListeners.push(listener);
    return () => void (this.clickListeners = this.clickListeners.filter((each) => each !== listener));
  }

  /** Told each time the crosshair moves to another point (NaN: gone), and the pane it's in. @param {(index: number, pane: number) => void} listener */
  onCrosshair(listener) {
    this.crosshairListeners.push(listener);
    return () => void (this.crosshairListeners = this.crosshairListeners.filter((each) => each !== listener));
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
    // (An axis at the left too, as wide as the right one, where a pane has a left scale: the plot and the time axis
    // after it.)
    const left = (this.left = this.panes.some((pane) => pane.left) ? this.axis : 0), width = Math.max(1, box.width - left);
    let top = 0;
    for (const [k, pane] of this.panes.entries()) {
      const bottom = k === this.panes.length - 1 ? room : Math.round(top + (room * pane.share) / shares);
      pane.place(top, width, bottom - top, this.axis, this.dpr);
      pane.canvas.style.left = `${left}px`;
      const had = pane.leftCanvas;
      pane.placeLeft(top, left, this.dpr);
      if (pane.leftCanvas && !had) this.element.insertBefore(pane.leftCanvas, this.timeAxis.canvas);
      if (pane.leftCanvas) pane.leftCanvas.style.display = left ? "block" : "none";
      pane.separated = k < this.panes.length - 1;
      top = bottom;
    }
    this.timeAxis.place(room, width, this.timeHeight, this.dpr);
    this.timeAxis.canvas.style.left = `${left}px`;
    this.crosshair.place(box.width, box.height, this.dpr);
    this.time.resize(Math.max(1, width - this.axis));
    // (The crosshair's pane found again: panes moved, came or went.)
    if (this.crosshair.index === this.crosshair.index) this.setCrosshair(this.crosshair.index, this.crosshair.y);
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
    this.crosshair.draw(time, this.panes, Math.max(0, this.height - this.timeHeight), this.left, this.theme, this.dpr);
    if (time.from !== this.drawnFrom || time.to !== this.drawnTo) {
      (this.drawnFrom = time.from), (this.drawnTo = time.to);
      for (let k = 0; k < this.rangeListeners.length; k++) this.rangeListeners[k](time.from, time.to);
    }
  }

  // Pointers: one drags (a pan, gliding on once let go), two pinch (a zoom about their middle, and a pan as it moves);
  // the wheel zooms about the pointer, a trackpad's sideways scroll pans. The crosshair: where a mouse is; a finger
  // held still (HOLD) shows it, moving it then instead of the chart, till it's let go. A finger pans once it's moved
  // past a slop of 6 pixels (a hold, else, would move the chart a little).
  #input() {
    const element = this.element, time = this.time, down = /** @type {Map<number, { x: number, y: number, startX: number, startY: number, panning: boolean }>} */ (new Map());
    let held = -1;
    // (Two fingers down in this gesture: its end no click.)
    let twoFingers = false;
    // (A separator dragged: the panes over and under it, the pointer dragging it (-1: none).)
    let sizing = /** @type {[Pane, Pane] | null} */ (null), sizer = -1;
    const stopSizing = () => void ((sizing = null), (sizer = -1), this.invalidate(true));
    /** The separator under a height in the chart (the pane over it), or -1. @param {number} y @param {boolean} touch */
    const separator = (y, touch) => {
      for (let k = 0; k < this.panes.length - 1; k++) if (Math.abs(this.panes[k].top + this.panes[k].height - y) <= (touch ? NEAR_TOUCH : NEAR_MOUSE)) return k;
      return -1;
    };
    const point = (/** @type {PointerEvent} */ event) => {
      const box = element.getBoundingClientRect();
      this.setCrosshair(time.index(event.clientX - box.left - this.left), event.clientY - box.top);
    };
    // (The last tenth of a second of a drag: its speed as it's let go.)
    const trail = new Float64Array(16), trailAt = new Float64Array(16);
    let trailEnd = 0, trailSize = 0;
    const plotX = (/** @type {number} */ clientX) => clientX - element.getBoundingClientRect().left - this.left;
    const signal = this.listening.signal;
    element.addEventListener("pointerdown", (event) => {
      // (A separator dragged: no other pointer does anything meanwhile.)
      if (event.button || sizer >= 0) return;
      element.setPointerCapture(event.pointerId);
      // (On a separator: it's dragged, nothing else.)
      const y = event.clientY - element.getBoundingClientRect().top, on = down.size ? -1 : separator(y, event.pointerType === "touch");
      if (on >= 0) return void ((sizing = [this.panes[on], this.panes[on + 1]]), (sizer = event.pointerId), (this.glide.speed = 0), this.setCrosshair(NaN));
      const touch = event.pointerType === "touch";
      down.set(event.pointerId, { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, panning: !touch });
      this.glide.speed = 0;
      trailSize = 0;
      clearTimeout(this.holding);
      this.holding = 0;
      // (A second finger: a pinch, the crosshair a held finger showed gone.)
      if (held >= 0) (held = -1), this.setCrosshair(NaN);
      if (down.size > 1) twoFingers = true;
      if (touch && down.size === 1) {
        const id = event.pointerId;
        this.holding = setTimeout(() => {
          const at = down.get(id);
          this.holding = 0;
          if (!at || at.panning || down.size !== 1) return;
          held = id;
          const box = element.getBoundingClientRect();
          this.setCrosshair(time.index(at.x - box.left - this.left), at.y - box.top);
        }, HOLD);
      }
    }, { signal });
    element.addEventListener("pointermove", (event) => {
      if (sizer === event.pointerId) return void (sizing && !this.#size(sizing[0], sizing[1], event.clientY - element.getBoundingClientRect().top) && stopSizing());
      const at = down.get(event.pointerId);
      if (!at && event.pointerType === "mouse") {
        // (A mouse over a separator: it shows it can be dragged, no crosshair there.)
        const over = separator(event.clientY - element.getBoundingClientRect().top, false) >= 0;
        element.style.cursor = over ? "row-resize" : "";
        return void (over ? this.setCrosshair(NaN) : point(event));
      }
      if (!at) return;
      if (held === event.pointerId) return void ((at.x = event.clientX), (at.y = event.clientY), point(event));
      if (!at.panning) {
        if (Math.hypot(event.clientX - at.startX, event.clientY - at.startY) <= 6) return;
        (at.panning = true), clearTimeout(this.holding), (this.holding = 0);
      }
      if (down.size === 1) {
        time.pan(event.clientX - at.x);
        trail[trailEnd] = event.clientX;
        trailAt[trailEnd] = event.timeStamp;
        trailEnd = (trailEnd + 1) % trail.length;
        trailSize = Math.min(trail.length, trailSize + 1);
      } else if (down.size === 2) {
        // (About the fingers' middle as it was, then moved with it: what's under each stays under it.)
        let other = at;
        for (const each of down.values()) if (each !== at) other = each;
        const was = Math.abs(at.x - other.x), now = Math.abs(event.clientX - other.x);
        if (was > 8 && now > 8) time.zoom(plotX((at.x + other.x) / 2), was / now);
        time.pan((event.clientX - at.x) / 2);
      }
      (at.x = event.clientX), (at.y = event.clientY);
      // (A mouse's crosshair where it is now, the chart moved under it.)
      if (event.pointerType === "mouse") point(event);
      this.invalidate();
    }, { signal });
    element.addEventListener("pointerleave", (event) => event.pointerType === "mouse" && !down.size && this.setCrosshair(NaN), { signal });
    const up = (/** @type {PointerEvent} */ event) => {
      if (sizer === event.pointerId) return void stopSizing();
      const at = down.get(event.pointerId);
      if (!at || !down.delete(event.pointerId)) return;
      const pinched = twoFingers;
      if (!down.size) twoFingers = false;
      clearTimeout(this.holding);
      this.holding = 0;
      // (The finger holding the crosshair let go: it goes; no glide.)
      if (held === event.pointerId) return void ((held = -1), (trailSize = 0), this.setCrosshair(NaN));
      // (Let go where it began: a click.)
      if (event.type === "pointerup" && !down.size && !pinched && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) < 6) {
        trailSize = 0;
        return void this.#click(event);
      }
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
    element.addEventListener("lostpointercapture", (event) => event.pointerId === sizer && stopSizing(), { signal });
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

  /** A click at a pointer's place: its point, pane and series told to the click's listeners. @param {PointerEvent} event */
  #click(event) {
    const box = this.element.getBoundingClientRect(), x = event.clientX - box.left - this.left, y = event.clientY - box.top, n = this.time.length;
    const index = n && x >= 0 && x <= this.time.width ? Math.min(Math.max(Math.round(this.time.index(x)), 0), n - 1) : NaN;
    let pane = -1, series = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    if (pane >= 0 && index === index) series = this.panes[pane].hit(this.time, index, y - this.panes[pane].top);
    for (let l = 0; l < this.clickListeners.length; l++) this.clickListeners[l](index, pane, series);
  }

  /**
   * The separator between `over` and `under` dragged to `y` (CSS pixels in the chart): their shares moved between them,
   * their total kept, neither under LEAST. Whether they're still next to each other (a pane removed: the drag's over).
   * @param {Pane} over @param {Pane} under @param {number} y
   */
  #size(over, under, y) {
    const k = this.panes.indexOf(over);
    if (k < 0 || this.panes[k + 1] !== under) return false;
    const both = over.height + under.height;
    if (both < 2 * LEAST) return true;
    const height = Math.min(Math.max(y - over.top, LEAST), both - LEAST), shares = over.share + under.share;
    over.share = (shares * height) / both;
    under.share = shares - over.share;
    this.layout();
    return true;
  }

  destroy() {
    cancelAnimationFrame(this.frame);
    clearTimeout(this.holding);
    this.frame = this.holding = 0;
    this.glide.speed = 0;
    this.crosshair.canvas.remove();
    this.listening.abort();
    this.resizer.disconnect();
    for (const pane of this.panes) pane.canvas.remove(), pane.leftCanvas?.remove();
    this.timeAxis.canvas.remove();
    this.panes = [];
  }
}
