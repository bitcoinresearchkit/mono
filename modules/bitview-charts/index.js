// @ts-check
// bitview-charts: no object per point and next to nothing allocated per frame, for phones without a JIT.

import { Timeline, lastIn, rightBlank } from "./timeline.js";
import { Controller } from "./controller.js";
import { Pane } from "./pane.js";
import { TimeAxis } from "./axis.js";
import { Crosshair } from "./crosshair.js";

export { Timeline, Pane, TimeAxis, lastIn, rightBlank };
export { Line, FAINT } from "./line.js";
export { Band } from "./band.js";
export { Bars, PARTIAL } from "./bars.js";
export { Candles, BODIES_FROM } from "./candles.js";
export { Bubbles } from "./bubbles.js";
export { Shade } from "./shade.js";
export { Grid } from "./grid.js";
export { Rows } from "./rows.js";
export { Profile, columnWidth } from "./profile.js";
export { stack } from "./stack.js";
export { Custom } from "./custom.js";
export { Marks } from "./marks.js";
export { Levels } from "./levels.js";
export { Legend } from "./legend.js";
export { picture } from "./picture.js";
export { Scale, indexed } from "./scale.js";
export { compact } from "./format.js";
export { channels, Gradient, gradientTable, inkOn, placeIndex } from "./color.js";

/** @type {import("./pane.js").Theme} */
export const DARK = { background: "#0e0f11", text: "#8a8f98", grid: "rgba(255, 255, 255, 0.06)", font: "12px system-ui, sans-serif", crosshair: "rgba(247, 248, 248, 0.4)", across: "rgba(247, 248, 248, 0.25)", badge: "#f7f8f8", separator: "rgba(255, 255, 255, 0.14)", active: "#f2660f" };
/** @type {import("./pane.js").Theme} */
export const LIGHT = { background: "#ffffff", text: "#6b6f76", grid: "rgba(0, 0, 0, 0.06)", font: "12px system-ui, sans-serif", crosshair: "rgba(14, 15, 17, 0.35)", across: "rgba(14, 15, 17, 0.2)", badge: "#0e0f11", separator: "rgba(0, 0, 0, 0.12)", active: "#f2660f" };

const HOLD = 240;
/** @param {PointerEvent} event */
const fine = (event) => event.pointerType !== "touch";
const NEAR_FINE = 4, NEAR_TOUCH = 12, LEAST = 32;

// iOS WebKit can hand a touch to a native gesture with no pointerup or pointercancel (WebKit bug 239014): only a
// touch event proves one gone (all its fingers new, or a press taking a down touch's id).
// A press's pointerdown comes before its touchstart, a lift's pointerup before its touchend (Chromium and WebKit, iOS too): touchesNew relies on it.
const touchesDown = new Set(), touchesNew = new Set();
/** @type {Set<(id: number) => void>} */
const touchOwners = new Set();
/** @type {Set<() => void>} */
const touchRecoveries = new Set();
/** Every owner lets go before any press is replayed. @param {number[]} ids */
const loseTouches = (ids) => {
  let error, failed = false;
  const attempt = (/** @type {() => void} */ action) => {
    try { action(); } catch (cause) { if (!failed) error = cause, failed = true; }
  };
  for (const id of ids) {
    touchesDown.delete(id);
    for (const owner of [...touchOwners]) attempt(() => owner(id));
  }
  for (const recover of touchRecoveries) attempt(recover);
  touchRecoveries.clear();
  if (failed) throw error;
};
const listen = { capture: true, passive: true };
addEventListener("pointerdown", (event) => {
  if (event.pointerType !== "touch") return;
  try { if (touchesDown.has(event.pointerId)) loseTouches([event.pointerId]); }
  finally { touchesDown.add(event.pointerId), touchesNew.add(event.pointerId); }
}, listen);
for (const type of ["pointerup", "pointercancel"]) addEventListener(type, (event) => touchesDown.delete(/** @type {PointerEvent} */ (event).pointerId), listen);
for (const type of ["touchstart", "touchend", "touchcancel"]) addEventListener(type, (event) => {
  const { touches, changedTouches } = /** @type {TouchEvent} */ (event);
  try {
    if (touches.length === (type === "touchstart" ? changedTouches.length : 0)) loseTouches([...touchesDown].filter((id) => !touchesNew.has(id)));
  } finally { touchesNew.clear(); }
}, listen);
/**
 * Every press goes through here, as iOS can drop a touch's end. `end` is told once: (event, false) or (null, lost).
 * @param {PointerEvent} down its pointerdown, or the move that made it one
 * @param {{ end: (release: PointerEvent | null, lost: boolean) => void, move?: (event: PointerEvent) => void, captor?: Element, signal?: AbortSignal }} how
 * @returns {() => void}
 */
export function press(down, { end, move, captor, signal }) {
  const pointer = down.pointerId, held = new AbortController(), heard = { capture: true, passive: true, signal: held.signal };
  const stop = () => held.abort();
  const ended = (/** @type {PointerEvent | null} */ release, lost = false) => {
    if (held.signal.aborted) return;
    stop();
    end(release, lost);
  };
  /** @param {(event: PointerEvent) => void} handle */
  const own = (handle) => (/** @type {Event} */ event) => {
    if (/** @type {PointerEvent} */ (event).pointerId === pointer) handle(/** @type {PointerEvent} */ (event));
  };
  if (signal?.aborted) return stop;
  signal?.addEventListener("abort", stop, { signal: held.signal });
  const touch = down.pointerType === "touch";
  if (move || !touch) addEventListener("pointermove", own((event) => (!touch && !event.buttons ? ended(null) : move?.(event))), heard);
  addEventListener("pointerup", own(ended), heard);
  addEventListener("pointercancel", own(() => ended(null)), heard);
  captor?.addEventListener("lostpointercapture", own((event) => event.target === captor && ended(null)), heard);
  // (The window's own blur: an element's doesn't bubble, and reaches a window listener only in the capture phase.)
  addEventListener("blur", (event) => event.target === window && ended(null), heard);
  if (touch) {
    const lost = (/** @type {number} */ id) => id === pointer && ended(null, true);
    touchOwners.add(lost);
    held.signal.addEventListener("abort", () => touchOwners.delete(lost));
  }
  return stop;
}

const AXIS_LEAST = 28;

/**
 * @param {HTMLElement} element
 * @param {ConstructorParameters<typeof Chart>[1]} [options]
 */
export function createChart(element, options) {
  return new Chart(element, options);
}

export class Chart {
  /** @type {import("./pane.js").Theme} */
  #theme = DARK;
  #crossedInput = false;

  /**
   * @param {HTMLElement} element
   * @param {{ theme?: import("./pane.js").Theme, axis?: number, leftAxis?: number, timeHeight?: number, maxSpacing?: number, dateFormat?: (t: number) => string, embedded?: boolean }} [options]
   */
  constructor(element, { theme = DARK, axis = 64, leftAxis = axis, timeHeight = 26, maxSpacing, dateFormat, embedded = false } = {}) {
    this.element = element;
    this.embedded = embedded;
    this.theme = theme;
    this.axis = axis;
    this.leftAxis = leftAxis;
    this.fitted = [0, 0];
    this.fitting = false;
    this.refit = false;
    this.timeHeight = timeHeight;
    this.time = new Timeline({ maxSpacing });
    this.listening = new AbortController();
    this.controller = new Controller(element, this.time, { move: () => this.invalidate(), left: () => this.left, signal: this.listening.signal, embedded });
    this.timeAxis = new TimeAxis();
    this.crosshair = new Crosshair();
    if (dateFormat) this.crosshair.setFormat(dateFormat);
    /** @type {((index: number, pane: number, y: number, input: boolean, x: number, series: number) => void)[]} */
    this.crosshairListeners = [];
    /** @type {Pane | null} */
    this.crossedPane = null;
    /** NaN: the crosshair stays on its index. */
    this.pointerX = NaN;
    this.recrossing = false;
    this.crossedChanges = -1;
    (this.crossedFrom = NaN), (this.crossedSpacing = NaN);
    /** @type {Pane | null} */
    this.separatorLit = null;
    this.resizing = false;
    /** @type {((index: number, pane: number, series: number, y: number, x: number) => void)[]} */
    this.clickListeners = [];
    /** @type {Pane[]} */
    this.panes = [];
    this.dpr = 1;
    this.left = 0;
    this.width = 0;
    this.height = 0;
    this.frame = 0;
    this.whole = true;
    this.moved = false;
    this.seams = false;
    /** @type {((from: number, to: number) => void)[]} */
    this.rangeListeners = [];
    /** @type {(() => void)[]} */
    this.layoutListeners = [];
    /** @type {(() => void)[]} */
    this.timeResetListeners = [];
    this.drawnFrom = NaN;
    this.drawnTo = NaN;
    // (Made once: no closure per frame.)
    this.tick = (/** @type {number} */ now) => this.#frame(now);
    this.holding = 0;
    this.letting = 0;
    if (getComputedStyle(element).position === "static") element.style.position = "relative";
    Object.assign(element.style, { overflow: "hidden", touchAction: embedded ? "pan-y" : "none", userSelect: "none", webkitUserSelect: "none" });
    this.corner = document.createElement("div");
    this.corner.style.cssText = "position: absolute; display: block";
    element.append(this.timeAxis.canvas, this.crosshair.canvas, this.corner);
    this.resizer = new ResizeObserver(() => this.layout());
    this.resizer.observe(element);
    this.#input();
    this.#watchDpr();
    this.layout();
  }

  /** A dpr change (another display, a page zoom) resizes nothing the ResizeObserver sees. */
  #watchDpr() {
    const query = matchMedia(`(resolution: ${devicePixelRatio || 1}dppx)`);
    query.addEventListener("change", () => (this.layout(), this.#watchDpr()), { once: true, signal: this.listening.signal });
  }

  #recross() {
    const crosshair = this.crosshair;
    if (crosshair.index !== crosshair.index) return;
    if (this.#crossedInput) this.recrossing = true;
    else this.setCrosshair(crosshair.index, crosshair.y);
  }

  /**
     * Found again at the next frame, so listeners hear it on panes fitted to the new view.
     * @param {PointerEvent} event
     */
  #follow(event) {
    const box = this.element.getBoundingClientRect();
    (this.pointerX = event.clientX - box.left - this.left), (this.crosshair.y = event.clientY - box.top), (this.recrossing = true);
  }

  /** Narrowed to their labels at the next frame: by themselves axes only widen. */
  fitAxes() {
    this.refit = true;
    this.invalidate(true);
  }

  /**
     * Seconds, rising, one per index.
     * @param {Float64Array} times
     */
  setTimes(times) {
    this.time.setTimes(times);
    this.#recross();
    this.invalidate(true);
  }

  /** @param {ConstructorParameters<typeof Pane>[0]} [options] */
  addPane(options) {
    const pane = new Pane(options);
    this.panes.push(pane);
    // (In paint order, before the time axis' and the crosshair's: a page drawing them in turn sees them as shown.)
    this.timeAxis.canvas.before(pane.canvas, pane.axisCanvas);
    this.element.append(pane.frame);
    pane.onLayout = () => this.layout();
    this.layout();
    return pane;
  }

  /** @param {Pane} pane */
  removePane(pane) {
    this.panes = this.panes.filter((each) => each !== pane);
    detach(pane);
    this.layout();
  }

  /** @param {number} span */
  show(span) {
    const { from, to } = this.time;
    this.time.show(span);
    this.#halt(from, to);
    this.invalidate(true);
  }

  /** @param {number} from @param {number} to */
  #halt(from, to) {
    if (this.time.from !== from || this.time.to !== to) this.controller.halt();
  }

  /**
     * Whole pixels, so the panes' pictures move rather than redraw; `exact` keeps the endpoints as given.
     * @param {number} from @param {number} to @param {{ exact?: boolean }} [options]
     */
  setRange(from, to, options) {
    const was = this.time.from, wasTo = this.time.to;
    if (options?.exact === true) this.time.setRange(from, to);
    else this.time.slide(from, to);
    this.#halt(was, wasTo);
    this.invalidate();
  }

  /**
     * NaN where it can't.
     * @param {import("./pane.js").Series} series @param {number} value
     */
  y(series, value) {
    const pane = this.panes.find((pane) => pane.series.includes(series));
    return pane ? pane.y(series, value, this.time) : NaN;
  }

  /** NaN where it has none. @param {import("./pane.js").Series} series @param {number} y */
  value(series, y) {
    const pane = this.panes.find((pane) => pane.series.includes(series));
    return pane ? pane.value(series, y, this.time) : NaN;
  }

  /**
     * Index NaN: gone. `x` NaN: anchored to the index.
     * @param {number} index @param {number} [y] @param {boolean} [input] @param {number} [x]
     */
  setCrosshair(index, y = NaN, input = false, x = NaN) {
    this.#placeCrosshair(index, y, input, x);
    this.invalidate();
  }

  /** Told as input, so linked charts know a pointer cleared it. */
  #clearPointer() {
    this.setCrosshair(NaN, NaN, true);
  }

  /** @param {number} index @param {number} y @param {boolean} input @param {number} x */
  #placeCrosshair(index, y, input, x) {
    const time = this.time, crosshair = this.crosshair;
    const k = index === index && time.length ? Math.min(Math.max(Math.round(index), 0), time.lastSlot) : NaN;
    let pane = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    // (By its pane, not its place: one removed, the next takes its place.)
    const inPane = pane >= 0 ? this.panes[pane] : null;
    const moved = (k !== crosshair.index || y !== crosshair.y || inPane !== this.crossedPane || input !== this.#crossedInput || !Object.is(x, this.pointerX)) && !(k !== k && crosshair.index !== crosshair.index);
    const refresh = input && k === k && (this.recrossing || (inPane?.changes ?? -1) !== this.crossedChanges);
    const viewChanged = input && k === k && (time.from !== this.crossedFrom || time.spacing !== this.crossedSpacing);
    this.recrossing = false;
    (this.#crossedInput = input), (crosshair.index = k), (crosshair.y = y), (crosshair.pane = pane), (this.crossedPane = inPane), (this.pointerX = x);
    const within = inPane ? y - inPane.top : NaN, at = x === x ? x : k === k ? time.x(k) : NaN;
    if (!moved && !refresh && !viewChanged) return;
    const hit = input && k === k && inPane ? inPane.hit(time, k, at, within) : -1;
    const series = hit >= 0 && inPane ? inPane.series[hit] : null, hitChanged = crosshair.hit !== series;
    crosshair.hit = series;
    // (Recorded before listeners: their changes still need refreshing at the next draw.)
    (this.crossedChanges = inPane?.changes ?? -1), (this.crossedFrom = time.from), (this.crossedSpacing = time.spacing);
    if (!moved && !refresh && !hitChanged) return;
    // (Listener lists are replaced, never changed in place: one leaving as it's told skips no other.)
    const listeners = this.crosshairListeners;
    for (let l = 0; l < listeners.length; l++) listeners[l](k, pane, within, input, at, hit);
  }

  /**
   * @param {(index: number, pane: number, series: number, y: number, x: number) => void} listener
   */
  onClick(listener) {
    this.clickListeners = [...this.clickListeners, listener];
    return () => void (this.clickListeners = this.clickListeners.filter((each) => each !== listener));
  }

  /**
   * @param {(index: number, pane: number, y: number, input: boolean, x: number, series: number) => void} listener
   */
  onCrosshair(listener) {
    this.crosshairListeners = [...this.crosshairListeners, listener];
    return () => void (this.crosshairListeners = this.crosshairListeners.filter((each) => each !== listener));
  }

  /** Set it, then invalidate(true). */
  get theme() {
    return this.#theme;
  }

  set theme(theme) {
    this.#theme = theme;
    this.element.style.background = theme.background;
    this.element.style.setProperty("--bc-font", theme.font);
  }

  /**
   * @param {() => void} listener
   */
  onLayout(listener) {
    this.layoutListeners = [...this.layoutListeners, listener];
    return () => void (this.layoutListeners = this.layoutListeners.filter((each) => each !== listener));
  }

  /** @param {() => void} listener */
  onTimeReset(listener) {
    this.timeResetListeners = [...this.timeResetListeners, listener];
    return () => void (this.timeResetListeners = this.timeResetListeners.filter((each) => each !== listener));
  }

  /** @param {(from: number, to: number) => void} listener */
  onRange(listener) {
    this.rangeListeners = [...this.rangeListeners, listener];
    return () => void (this.rangeListeners = this.rangeListeners.filter((each) => each !== listener));
  }

  get axisShown() {
    return this.axis && (this.fitted[0] || this.axis);
  }

  get leftShown() {
    return this.leftAxis && (this.fitted[1] || this.leftAxis);
  }

  /** Watched; call it only for a change the ResizeObserver can't see. */
  layout() {
    const box = this.element.getBoundingClientRect(), wasLeft = this.left;
    this.dpr = devicePixelRatio || 1;
    this.width = box.width;
    this.height = box.height;
    const room = Math.max(0, box.height - this.timeHeight), shares = this.panes.reduce((sum, pane) => sum + pane.share, 0) || 1;
    const left = (this.left = this.panes.some((pane) => pane.left) ? this.leftShown : 0), width = Math.max(1, box.width - left), axis = this.axisShown;
    if (this.pointerX === this.pointerX) this.pointerX += wasLeft - left;
    let top = 0;
    const last = this.panes.findLastIndex((pane) => pane.share > 0);
    for (const [k, pane] of this.panes.entries()) {
      const bottom = !pane.share ? top : k === last ? room : Math.round(top + (room * pane.share) / shares);
      pane.place(top, width, bottom - top, axis, this.dpr);
      pane.canvas.style.left = pane.frame.style.left = `${left}px`;
      pane.axisCanvas.style.left = `${left + pane.plot}px`;
      const had = pane.leftCanvas;
      pane.placeLeft(top, left, this.dpr);
      if (pane.leftCanvas && !had) this.timeAxis.canvas.before(pane.leftCanvas);
      if (pane.leftCanvas) pane.leftCanvas.style.display = left ? "block" : "none";
      if (pane.overCanvas) {
        if (!pane.overCanvas.parentNode) this.element.append(pane.overCanvas);
        pane.overCanvas.style.left = `${left}px`;
      }
      pane.separated = true;
      top = bottom;
    }
    this.timeAxis.place(room, Math.max(1, width - axis), this.timeHeight, this.dpr);
    this.timeAxis.canvas.style.left = `${left}px`;
    const cornerWidth = axis || left;
    Object.assign(this.corner.style, { left: `${axis ? left + Math.max(1, width - axis) : 0}px`, top: `${room}px`, width: `${cornerWidth}px`, height: `${this.timeHeight}px`, display: cornerWidth ? "block" : "none" });
    this.crosshair.place(box.width, box.height, this.dpr);
    this.time.resize(Math.max(1, width - axis));
    this.#recross();
    this.invalidate(true);
    const listeners = this.layoutListeners;
    for (let k = 0; k < listeners.length; k++) listeners[k]();
  }

  /**
     * `whole` after a size, pane, series or theme change; else a view move.
     * @param {boolean} [whole]
     */
  invalidate(whole = false) {
    this.whole ||= whole;
    this.moved ||= !whole;
    this.#schedule();
  }

  #schedule() {
    if (!this.listening.signal.aborted) this.frame ||= requestAnimationFrame(this.tick);
  }

  /** @param {number} now */
  #frame(now) {
    this.frame = 0;
    this.controller.step(now);
    // (Once still, pictures moved or stretched before are drawn whole, the seams gone.)
    const whole = this.whole || (this.seams && !this.moved);
    this.moved = false;
    if (whole) this.seams = false;
    this.render(whole);
    if (this.seams) this.#schedule();
  }

  /** The series pointed at, the legend's lit one. */
  #focus() {
    const crosshair = this.crosshair;
    return crosshair.pointed !== undefined ? crosshair.pointed : crosshair.index === crosshair.index ? crosshair.hit : null;
  }

  /** @param {boolean} [whole] @returns {void} */
  render(whole = false) {
    this.whole = false;
    const time = this.time, crosshair = this.crosshair, focus = this.#focus();
    for (let k = 0; k < this.panes.length; k++) {
      const pane = this.panes[k];
      pane.focus = focus;
      pane.fit(time);
      if (pane.draw(time, this.theme, this.dpr, whole)) this.seams = true;
    }
    // (Axes widen at once and narrow only in fitAxes: no plot jitter. Drawn whole for a picture taken now.)
    {
      let right = 0, leftNeed = 0;
      for (let k = 0; k < this.panes.length; k++) (right = Math.max(right, this.panes[k].axisNeed[0])), (leftNeed = Math.max(leftNeed, this.panes[k].left ? this.panes[k].axisNeed[1] : 0));
      const fit = (/** @type {number} */ need) => Math.max(AXIS_LEAST, Math.ceil(need / 4) * 4), refit = this.refit;
      this.refit = false;
      const next = (/** @type {number} */ want, /** @type {number} */ fitted) => (want > fitted || refit || !fitted ? want : fitted);
      const wantRight = this.axis ? next(fit(right), this.fitted[0]) : 0, wantLeft = this.left ? next(fit(leftNeed), this.fitted[1]) : 0;
      const moved = (this.axis && wantRight !== this.axisShown) || (this.left && wantLeft !== this.leftShown);
      if (this.axis) this.fitted[0] = wantRight;
      if (this.left) this.fitted[1] = wantLeft;
      if (moved) {
        this.layout();
        const was = this.fitting;
        this.fitting = true;
        try {
          return this.render(true);
        } finally {
          this.fitting = was;
        }
      }
    }
    if (this.recrossing || (this.#crossedInput && crosshair.index === crosshair.index)) {
      const x = this.pointerX;
      if (x === x && !(x >= 0 && x <= time.width)) this.#clearPointer();
      else this.#placeCrosshair(x === x ? time.index(x) : crosshair.index, crosshair.y, true, x);
    }
    this.timeAxis.draw(time, this.theme, this.dpr, time.width, this.panes);
    this.crosshair.draw(time, this.panes, Math.max(0, this.height - this.timeHeight), this.left, this.theme, this.dpr, this.timeAxis, this.separatorLit);
    const read = this.#focus();
    // (Hit as placed above, after the panes drew: a yielded pane's tags follow it at the next frame.)
    if (read !== focus && this.panes.some((pane) => pane.yields[0] || pane.yields[1])) this.invalidate();
    for (const pane of this.panes) if (pane.legend) pane.legend.point(pane.legend.itemOf(read)), pane.legend.read(crosshair.index, (series, v) => pane.words(series, v, time), (series) => pane.restOf(series, time));
    for (let k = 0; k < this.panes.length; k++) this.panes[k].drawOver(time, this.dpr);
    if (time.from !== this.drawnFrom || time.to !== this.drawnTo) {
      (this.drawnFrom = time.from), (this.drawnTo = time.to);
      const listeners = this.rangeListeners;
      for (let k = 0; k < listeners.length; k++) listeners[k](time.from, time.to);
    }
  }

  #input() {
    /**
         * @typedef {{ pane: Pane | null, side: "right" | "left", a: number, b: number, at: number, held?: number }} Axis
         */
    const element = this.element, time = this.time, down = /** @type {Map<number, { x: number, y: number, startX: number, startY: number, panning: boolean, moved: boolean, axis: Axis | null, pane: Pane | null, beside: PointerEvent | null, stop: () => void, caught: boolean }>} */ (new Map());
    let held = -1;
    let twoFingers = false;
    let sizing = /** @type {[Pane, Pane] | null} */ (null), sizer = -1;
    /** @param {number} [y] */
    const stopSizing = (y = NaN) => {
      (sizing = null), (sizer = -1), (this.resizing = false), (this.separatorLit = this.panes[separator(y, false)] ?? null);
      this.invalidate(true);
    };
    /** @param {number} y @param {boolean} touch */
    const separator = (y, touch) => {
      for (let k = 0; k < this.panes.length; k++) if (this.#next(k) >= 0 && Math.abs(this.panes[k].top + this.panes[k].height - y) <= (touch ? NEAR_TOUCH : NEAR_FINE)) return k;
      return -1;
    };
    const point = (/** @type {PointerEvent} */ event) => {
      const box = element.getBoundingClientRect(), x = event.clientX - box.left - this.left, over = x >= 0 && x <= time.width;
      if (this.recrossing) return void ((this.pointerX = x), (this.crosshair.y = event.clientY - box.top), this.invalidate());
      this.setCrosshair(over ? time.index(x) : NaN, event.clientY - box.top, true, over ? x : NaN);
    };
    // (Found without allocating: a mouse's hover asks at each move.)
    let foundPane = /** @type {Pane | null} */ (null), foundSide = /** @type {"right" | "left"} */ ("right");
    /** @param {number} x @param {number} y */
    const findAxis = (x, y) => {
      if (y >= this.height - this.timeHeight) return (foundPane = null), x >= this.left && x <= this.left + time.width;
      for (const pane of this.panes) {
        if (y < pane.top || y >= pane.top + pane.height) continue;
        // (None on a relative scale (it follows its bases) or one fitted to nothing (its range a placeholder).)
        const side = x >= this.left + pane.plot && pane.width > pane.plot ? "right" : x < this.left && pane.left ? "left" : null, scale = side === "left" ? pane.left : pane.scale;
        return (foundPane = pane), (foundSide = side ?? "right"), Boolean(side && scale && !scale.relative && !scale.empty && scale.lo < scale.hi && (scale.mode !== "log" || scale.lo > 0));
      }
      return false;
    };
    /** @param {number} x @param {number} y @returns {Axis | null} */
    const axisAt = (x, y) => {
      if (!findAxis(x, y)) return null;
      const pane = foundPane, scale = foundSide === "left" ? pane?.left : pane?.scale;
      if (pane && scale) return { pane, side: foundSide, a: scale.lo, b: scale.hi, at: y - pane.top };
      // (A share of the plot's width, so an axis widening meanwhile keeps it in the plot.)
      const last = time.length - 1, held = last >= 0 && time.x(last) >= 0 && time.x(last) <= time.width ? time.x(last) / time.width : 1;
      return { pane: null, side: "right", a: time.spacing, b: held < 1 ? last : time.to, held, at: x - this.left };
    };
    /**
     * @param {Axis} axis @param {number} x @param {number} y
     */
    const scaleAxis = (axis, x, y) => {
      const pane = axis.pane;
      if (!pane) {
        const width = time.width, start = width - axis.at, now = Math.min(Math.max(width - (x - this.left), 0), width);
        if (start > 0 && now > 0) ((span, at) => time.setRange(axis.b - at * span, axis.b + (1 - at) * span))(time.within(width / ((axis.a * now) / start)), /** @type {number} */ (axis.held));
        return;
      }
      const scale = axis.side === "left" ? pane.left : pane.scale, h = pane.height;
      if (!scale) return;
      const k = Math.max(0.1, (h - axis.at + (h - 1) * 0.2) / (Math.max(0, h - (y - pane.top)) + (h - 1) * 0.2)), log = scale.mode === "log";
      const a = log ? Math.log10(axis.a) : axis.a, b = log ? Math.log10(axis.b) : axis.b, middle = (a + b) / 2, half = ((b - a) / 2) * k;
      pane.setRange(log ? 10 ** (middle - half) : middle - half, log ? 10 ** (middle + half) : middle + half, axis.side);
    };
    /**
         * @param {Pane} pane @param {number} dy
         */
    const slide = (pane, dy) => {
      if (!dy) return;
      for (const [scale, side, auto] of /** @type {const} */ ([[pane.scale, "right", pane.auto], [pane.left, "left", pane.leftAuto]]))
        if (scale && !auto && !scale.relative && !scale.empty) pane.setRange(scale.value(scale.y(scale.lo) - dy), scale.value(scale.y(scale.hi) - dy), side);
    };
    // (A price axis' last tap, for a double tap (iOS sends no dblclick).)
    let tapAt = -Infinity, tapX = 0, tapY = 0, tapPane = /** @type {Pane | null} */ (null), tapSide = "";
    const plotX = (/** @type {number} */ clientX) => clientX - element.getBoundingClientRect().left - this.left;
    const signal = this.listening.signal;
    /** @param {PointerEvent} event */
    const hold = (event) => {
      const id = event.pointerId;
      element.setPointerCapture(id);
      return press(event, { captor: element, signal, end: (release, lost) => ended(id, release, lost) });
    };
    /** @param {PointerEvent} event @param {number} on */
    const size = (event, on) => {
      hold(event);
      // (A drag's lift may still send a click (WebKit's, on what slid under the press): taken till the next press; a key's
      // (detail 0) never.)
      const swallow = (/** @type {MouseEvent} */ click) => click.detail && (click.stopPropagation(), click.preventDefault());
      addEventListener("click", swallow, { capture: true, once: true, signal });
      addEventListener("pointerdown", () => removeEventListener("click", swallow, true), { capture: true, once: true, signal });
      (sizing = [this.panes[on], this.panes[this.#next(on)]]), (sizer = event.pointerId), (this.resizing = true), (this.separatorLit = this.panes[on]), this.controller.stop(), this.#clearPointer();
    };
    let banded = /** @type {{ id: number, x: number, y: number, on: number } | null} */ (null);
    const take = (/** @type {PointerEvent} */ event) => {
      banded = null;
      if (event.button || sizer >= 0) return;
      const touch = !fine(event), page = touch && this.embedded, y = event.clientY - element.getBoundingClientRect().top, on = down.size || page ? -1 : separator(y, touch);
      // (A press on a page's own element is the page's, unless it joins the chart's own.)
      if (!down.size && event.target !== element && !this.#owns(event.target)) return void (on >= 0 && (banded = { id: event.pointerId, x: event.clientX, y: event.clientY, on }));
      if (on >= 0) return size(event, on);
      const id = event.pointerId, stop = hold(event);
      const box = element.getBoundingClientRect(), found = down.size ? null : axisAt(event.clientX - box.left, y), axis = page && found?.pane ? null : found;
      const caught = this.controller.grab();
      down.set(event.pointerId, { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, panning: !touch, moved: false, axis, pane: this.panes.find((pane) => y >= pane.top && y < pane.top + pane.height) ?? null, beside: down.size ? event : null, stop, caught });
      clearTimeout(this.holding);
      if (this.letting) cancelAnimationFrame(this.letting), this.#clearPointer();
      this.holding = this.letting = 0;
      if (held >= 0) (held = -1), this.#clearPointer();
      if (down.size > 1) {
        twoFingers = true;
        for (const each of down.values()) each.axis = null;
      }
      if (!touch && down.size === 1 && !axis) point(event);
      if (touch && down.size === 1 && !found) {
        this.holding = setTimeout(() => {
          const at = down.get(id);
          this.holding = 0;
          if (!at || at.panning || down.size !== 1) return;
          held = id;
          const box = element.getBoundingClientRect();
          this.setCrosshair(time.index(at.x - box.left - this.left), at.y - box.top, true, at.x - box.left - this.left);
        }, HOLD);
      }
    };
    element.addEventListener("pointerdown", take, { signal });
    element.addEventListener("pointermove", (event) => {
      if (banded?.id === event.pointerId) {
        // (A mouse let go elsewhere: no press.)
        const dx = Math.abs(event.clientX - banded.x), dy = Math.abs(event.clientY - banded.y), pressed = !fine(event) || event.buttons;
        if (pressed && dy >= 6 && dy > dx) size(event, banded.on);
        if (!pressed || dx >= 6 || dy >= 6) banded = null;
        return;
      }
      if (sizer === event.pointerId) return void (sizing && !this.#size(sizing[0], sizing[1], event.clientY - element.getBoundingClientRect().top) && stopSizing());
      if (this.resizing) return;
      const at = down.get(event.pointerId);
      if (!at && fine(event)) {
        if (event.target !== element && !this.#owns(event.target)) return void (this.separatorLit && ((this.separatorLit = null), this.invalidate()), this.#clearPointer());
        const box = element.getBoundingClientRect(), y = event.clientY - box.top, lit = separator(y, false), over = lit >= 0, axis = !over && findAxis(event.clientX - box.left, y);
        element.style.cursor = over ? "row-resize" : axis ? (foundPane ? "ns-resize" : "ew-resize") : "";
        if (this.panes[lit] !== (this.separatorLit ?? undefined)) (this.separatorLit = this.panes[lit] ?? null), this.invalidate();
        return void (over ? this.#clearPointer() : point(event));
      }
      if (!at) return;
      if (!at.moved && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) >= 6) {
        at.moved = true;
        if (at.axis && this.#crossedInput) this.#clearPointer();
      }
      if (held === event.pointerId) return void ((at.x = event.clientX), (at.y = event.clientY), point(event));
      if (!at.panning) {
        // A finger pans only past a 6 px slop, else a hold would move the chart a little.
        if (Math.hypot(event.clientX - at.startX, event.clientY - at.startY) <= 6) return;
        (at.panning = true), clearTimeout(this.holding), (this.holding = 0);
      }
      if (down.size === 1 && at.axis) {
        if (!at.moved) return;
        const box = element.getBoundingClientRect();
        scaleAxis(at.axis, event.clientX - box.left, event.clientY - box.top);
        (at.x = event.clientX), (at.y = event.clientY);
        this.controller.forget();
        return void this.invalidate();
      }
      if (down.size === 1) {
        this.controller.drag(event.clientX - at.x, event.clientX, event.timeStamp);
        if (at.pane) slide(at.pane, event.clientY - at.y);
      } else if (down.size === 2) {
        let other = at;
        for (const each of down.values()) if (each !== at) other = each;
        this.controller.pinch(plotX((at.x + other.x) / 2), Math.abs(at.x - other.x), Math.abs(event.clientX - other.x), (event.clientX - at.x) / 2, event.timeStamp);
      }
      (at.x = event.clientX), (at.y = event.clientY);
      if (fine(event)) this.#follow(event);
      this.invalidate();
    }, { signal });
    for (const type of ["pointerup", "pointercancel"]) element.addEventListener(type, (event) => banded?.id === event.pointerId && (banded = null), { signal });
    element.addEventListener("pointerleave", (event) => fine(event) && !down.size && !this.resizing && ((this.separatorLit = null), this.#clearPointer()), { signal });
    /**
     * @param {number} id @param {PointerEvent | null} event @param {boolean} lost
     */
    const ended = (id, event, lost) => {
      if (sizer === id) return void ((tapAt = -Infinity), stopSizing(event && fine(event) ? event.clientY - element.getBoundingClientRect().top : NaN));
      const at = down.get(id);
      if (!at || !down.delete(id)) return;
      // (Second fingers taken beside a lost touch are replayed alone once every lost one's let go.)
      if (lost) touchRecoveries.add(recover);
      const pinched = twoFingers;
      if (!down.size) twoFingers = false;
      clearTimeout(this.holding);
      this.holding = 0;
      // (The held crosshair goes at the next frame, after the page's own lift handling.)
      if (held === id) return void ((held = -1), this.controller.forget(), (tapAt = -Infinity), (this.letting = requestAnimationFrame(() => ((this.letting = 0), this.#clearPointer()))));
      if (event && !down.size && !pinched && !at.moved && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) < 6) {
        this.controller.forget();
        // A tap that caught a glide only stops it (nor is it the first of a double tap).
        if (at.caught) return void (tapAt = -Infinity);
        const axis = at.axis;
        const again = axis && axis.pane === tapPane && axis.side === tapSide && event.timeStamp - tapAt < 350 && Math.hypot(event.clientX - tapX, event.clientY - tapY) < 24;
        (tapAt = axis && !again ? event.timeStamp : -Infinity), (tapX = event.clientX), (tapY = event.clientY), (tapPane = axis?.pane ?? null), (tapSide = axis?.side ?? "");
        if (again && axis.pane) axis.pane.setRange(undefined, undefined, axis.side), this.fitAxes();
        else if (again) for (const listener of this.timeResetListeners) listener();
        return void this.#click(event);
      }
      tapAt = -Infinity;
      if (down.size || !event || fine(event)) return void this.controller.forget();
      this.controller.flick(event.timeStamp, pinched);
    };
    // (Embedded, touch-action can't say "vertical unless held": touchmove is cancelled while a finger holds the crosshair
    // or two of the chart's fingers pinch; one joining a running scroll is the browser's.)
    if (this.embedded) element.addEventListener("touchmove", (event) => (held >= 0 || down.size > 1) && event.cancelable && event.preventDefault(), { passive: false, signal });
    const recover = () => {
      let error, failed = false;
      for (const [id, at] of [...down]) if (at.beside && touchesDown.has(id) && touchesNew.has(id)) {
        down.delete(id), at.stop(), down.size || (twoFingers = false);
        try { take(at.beside); } catch (cause) { if (!failed) error = cause, failed = true; }
      }
      if (failed) throw error;
    };
  }

  /**
     * Null while a separator's dragged.
     * @param {number} x @param {number} y
     */
  tagAt(x, y) {
    if (this.resizing) return null;
    for (const pane of this.panes) {
      if (y < pane.top || y >= pane.top + pane.height) continue;
      const line = x >= this.left + pane.plot && x < this.left + pane.width ? pane.lineAt(0, y - pane.top) : x >= 0 && x < this.left ? pane.lineAt(1, y - pane.top) : null;
      return line && !line.hidden ? line : null;
    }
    return null;
  }

  /** @param {EventTarget | null} target */
  #owns(target) {
    if (target === this.timeAxis.canvas) return true;
    for (const pane of this.panes) if (target === pane.canvas || target === pane.axisCanvas || target === pane.leftCanvas) return true;
    return false;
  }

  /** @param {PointerEvent} event */
  #click(event) {
    if (!this.clickListeners.length) return;
    const box = this.element.getBoundingClientRect(), x = event.clientX - box.left - this.left, y = event.clientY - box.top, n = this.time.length;
    const index = n && x >= 0 && x <= this.time.width ? Math.min(Math.max(Math.round(this.time.index(x)), 0), n - 1) : NaN;
    let pane = -1, series = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    if (pane >= 0 && index === index) series = this.panes[pane].hit(this.time, index, x, y - this.panes[pane].top);
    const within = pane >= 0 ? y - this.panes[pane].top : NaN;
    const listeners = this.clickListeners;
    for (let l = 0; l < listeners.length; l++) listeners[l](index, pane, series, within, x);
  }

  /** @param {number} k */
  #next(k) {
    return this.panes[k].share > 0 ? this.panes.findIndex((pane, i) => i > k && pane.share > 0) : -1;
  }

  /**
     * False once they're no longer next to each other.
     * @param {Pane} over @param {Pane} under @param {number} y
     */
  #size(over, under, y) {
    const k = this.panes.indexOf(over);
    if (k < 0 || this.panes[this.#next(k)] !== under) return false;
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
    cancelAnimationFrame(this.letting);
    clearTimeout(this.holding);
    this.frame = this.holding = this.letting = 0;
    this.crosshair.canvas.remove();
    this.listening.abort();
    this.resizer.disconnect();
    for (const pane of this.panes) detach(pane);
    this.timeAxis.canvas.remove();
    this.corner.remove();
    this.panes = [];
  }
}

/** Its chart no longer told of it, so a page keeping it keeps no chart. @param {Pane} pane */
function detach(pane) {
  for (const element of [pane.canvas, pane.axisCanvas, pane.leftCanvas, pane.overCanvas, pane.frame]) element?.remove();
  pane.onLayout = () => {};
}
