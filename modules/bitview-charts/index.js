// @ts-check
// bitview-charts: time series charts drawn from columns. A chart is panes stacked over one time axis, every pane on the
// same timeline (one time a logical index); each pane's lines are columns on it (a Float64Array a line, NaN for none).
// Made for many series of thousands of points on a phone without its JIT: no object a point, next to nothing made a
// frame, a pan moving the picture already drawn where it can.
//
//   const chart = createChart(element);
//   chart.setTimes(times);                                   // Float64Array of seconds, rising
//   const pane = chart.addPane({ scale: { mode: "log" } });
//   pane.addLine({ values, color: "#f7931a" });              // Float64Array, an index a time
//   chart.show(365);                                         // the last year in view
//
// Dragged, a pane pans (a finger's flick glides on, slowing as a list does, a little sooner); pinched or wheeled, it zooms about that point;
// a trackpad's sideways scroll pans (a diagonal swipe both at once).

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

/** A touch held this long (ms) without moving shows the crosshair, the finger moving it then, not the chart (as Lightweight Charts' long tap). */
const HOLD = 240;
/** Mouse and pen input share precise pointing; touch uses slop, holding and momentum. @param {PointerEvent} event */
const fine = (event) => event.pointerType !== "touch";
/** How near a separator between panes a pointer takes it (CSS pixels): a mouse or pen's, a finger's; a pane's least height. */
const NEAR_FINE = 4, NEAR_TOUCH = 12, LEAST = 32;

// iOS WebKit can hand a touch to a native gesture and send it neither pointerup nor pointercancel (WebKit bug 239014).
// Only the browser's own touch list proves such a touch gone: a touch event with every finger on the screen new to it
// (at a touchend or touchcancel, none; at a touchstart, only those it starts), or a press taking the id of a touch still
// down. A press's pointerdown comes before its touchstart, a lift's pointerup before its touchend (Chromium and WebKit,
// iOS's too): those pressed since the last touch event are the new fingers. Anything short of that proof is left alone.
/** The touch pointers down on the page; of them, those pressed since the last touch event. */
const touchesDown = new Set(), touchesNew = new Set();
/** The presses held by touch, told of each touch proved gone, by its id (see press). @type {Set<(id: number) => void>} */
const touchOwners = new Set();
/** Charts needing to replay presses after all lost touches have been cancelled. @type {Set<() => void>} */
const touchRecoveries = new Set();
/** Touches proved gone, let go of by every owner before any press is replayed. @param {number[]} ids */
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
 * A press followed to its end, however it ends: every press a page holds goes through here, so none misses an end.
 * `move` hears each move of its pointer meanwhile. `end` is told once: `end(event, false)` at its pointerup (a
 * release), else `end(null, lost)` (let go of, no release) at its pointercancel, its capture lost by `captor`, the
 * window left, a mouse or pen moving with no button down (its release went elsewhere), or (`lost` true) the browser
 * forgetting its touch (see above: as it's proved gone, before the press that proves it). Its events are heard on the
 * window as they start out (the capture phase), before any element's listeners: only one before it on the window
 * itself could stop them.
 * Returns its stop, `end` untold; `signal`'s abort stops it too.
 * @param {PointerEvent} down the event it starts from (its pointerdown, or the move that made it one)
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
  // (The window's own blur: an element's doesn't bubble, nor reach a listener on the window but in the capture phase.)
  addEventListener("blur", (event) => event.target === window && ended(null), heard);
  if (touch) {
    const lost = (/** @type {number} */ id) => id === pointer && ended(null, true);
    touchOwners.add(lost);
    held.signal.addEventListener("abort", () => touchOwners.delete(lost));
  }
  return stop;
}

/** An axis' least width (CSS pixels): a short label's, as "0". */
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
  /** Whether the crosshair was last set by pointer input. */
  #crossedInput = false;

  /**
   * @param {HTMLElement} element
   * @param {{ theme?: import("./pane.js").Theme, axis?: number, leftAxis?: number, timeHeight?: number, maxSpacing?: number, dateFormat?: (t: number) => string, embedded?: boolean }} [options]
   */
  constructor(element, { theme = DARK, axis = 64, leftAxis = axis, timeHeight = 26, maxSpacing, dateFormat, embedded = false } = {}) {
    this.element = element;
    /**
     * In a page that scrolls: a finger's mostly vertical swipe scrolls the page, the browser deciding (`touch-action:
     * pan-y`); the chart takes only what's sideways, a pinch and a held finger's crosshair (no separator or price axis by
     * finger: their drags are vertical). Else it takes every gesture (a chart filling its screen).
     */
    this.embedded = embedded;
    this.theme = theme;
    /**
     * Whether an axis is at every pane's right, and at their left where one has a left scale (0: none), and its width till
     * it's drawn; then as narrow as its labels and tags let it (see fitted). Then layout().
     */
    this.axis = axis;
    this.leftAxis = leftAxis;
    /** Each side's axis as shown, as its labels and tags need it (right, left; 0: not drawn yet, `axis`'s till then). */
    this.fitted = [0, 0];
    /** A render drawing again at the axes' new widths: in it they only widen (see render). */
    this.fitting = false;
    /** The axes fitted again at the next frame, narrower too (see fitAxes). */
    this.refit = false;
    this.timeHeight = timeHeight;
    this.time = new Timeline({ maxSpacing });
    /** Its listeners, gone with it (see destroy). */
    this.listening = new AbortController();
    /** How the view moves: the wheel, a flick gliding on, moves eased to a range (see Controller). */
    this.controller = new Controller(element, this.time, { move: () => this.invalidate(), left: () => this.left, signal: this.listening.signal, embedded });
    this.timeAxis = new TimeAxis();
    this.crosshair = new Crosshair();
    if (dateFormat) this.crosshair.setFormat(dateFormat);
    /** @type {((index: number, pane: number, y: number, input: boolean, x: number, series: number) => void)[]} */
    this.crosshairListeners = [];
    /** The pane the crosshair was last in (null: none). @type {Pane | null} */
    this.crossedPane = null;
    /** The supplied pointer x across the plot (NaN: the crosshair stays anchored to its index). */
    this.pointerX = NaN;
    /** A pointer's crosshair to be found again at the next frame (see #recross); its pane's changes as it was told. */
    this.recrossing = false;
    this.crossedChanges = -1;
    /** The view as the crosshair's series was last found under it (see render). */
    (this.crossedFrom = NaN), (this.crossedSpacing = NaN);
    /** The separator pointed at or dragged (the pane over it; none, null): drawn in the theme's active color. @type {Pane | null} */
    this.separatorLit = null;
    /** A separator being dragged: its panes laid out and drawn at each move, no crosshair or tag read meanwhile. */
    this.resizing = false;
    /** @type {((index: number, pane: number, series: number, y: number, x: number) => void)[]} */
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
    /** @type {((from: number, to: number) => void)[]} */
    this.rangeListeners = [];
    /** @type {(() => void)[]} */
    this.layoutListeners = [];
    /** @type {(() => void)[]} */
    this.timeResetListeners = [];
    this.drawnFrom = NaN;
    this.drawnTo = NaN;
    // (Its frame's callback, made once: none made each frame.)
    this.tick = (/** @type {number} */ now) => this.#frame(now);
    /** A touch's hold waiting to show the crosshair (a timer; 0: none). */
    this.holding = 0;
    /** A held finger's crosshair going at the next frame (a frame's request; 0: none): see the pointer's lifting. */
    this.letting = 0;
    // (Its canvases placed in it: a box for them, unless it's one already.)
    if (getComputedStyle(element).position === "static") element.style.position = "relative";
    Object.assign(element.style, { overflow: "hidden", touchAction: embedded ? "pan-y" : "none", userSelect: "none", webkitUserSelect: "none" });
    /**
     * A page's own element in the chart's corner beside the time axis, under its price axis (the right one, else the
     * left): its buttons (Full screen). Laid out with the chart, as wide as that axis; it takes the pointer.
     */
    this.corner = document.createElement("div");
    this.corner.style.cssText = "position: absolute; display: block";
    element.append(this.timeAxis.canvas, this.crosshair.canvas, this.corner);
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

  /**
   * The crosshair found again after the data or the layout changed: a pointer's at the next frame, on the point under it
   * once the panes are fitted (told once, see render); else kept on its point (one there is, or gone), its pane found
   * again.
   */
  #recross() {
    const crosshair = this.crosshair;
    if (crosshair.index !== crosshair.index) return;
    if (this.#crossedInput) this.recrossing = true;
    else this.setCrosshair(crosshair.index, crosshair.y);
  }

  /**
   * A mouse or pen dragging the view: its crosshair found again at the next frame where it is then (see render), its listeners
   * told once the panes are fitted to the view it moved to (not on the scale before).
   * @param {PointerEvent} event
   */
  #follow(event) {
    const box = this.element.getBoundingClientRect();
    (this.pointerX = event.clientX - box.left - this.left), (this.crosshair.y = event.clientY - box.top), (this.recrossing = true);
  }

  /** Its axes as narrow as their labels and tags let them, at the next frame (they only widen by themselves). */
  fitAxes() {
    this.refit = true;
    this.invalidate(true);
  }

  /**
   * The timeline's times (seconds, rising), every pane's lines' indices. An end in view follows added points
   * at its existing screen position; data updates preserve momentum.
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
    // (Its canvases before the time axis' and the crosshair's: the element's canvases in the order they're painted (a
    // page drawing them in turn, a magnifier, shows them as they are). Its frame over its plot.)
    this.timeAxis.canvas.before(pane.canvas, pane.axisCanvas);
    this.element.append(pane.frame);
    // (Its left scale come or gone, its layer over it come: laid out again.)
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

  /** The last `span` points in view; stop momentum if the view changes. @param {number} span */
  show(span) {
    const { from, to } = this.time;
    this.time.show(span);
    this.#halt(from, to);
    this.invalidate(true);
  }

  /** A flick ends if the view moved from `from`, `to`. @param {number} from @param {number} to */
  #halt(from, to) {
    if (this.time.from !== from || this.time.to !== to) this.controller.halt();
  }

  /**
   * The view: logical indices, fractions allowed, within the zoom's bounds (narrower, centred at the closest). As wide as
   * it is (a page's own pan: its glide, its navigator's drag), moved as a pan is, by whole pixels (the rest kept): the
   * panes' pictures moved, not drawn again; `exact`, at its endpoints as given (a view restored, dates gone to). An
   * effective change stops momentum.
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
   * Where a series draws `value`: CSS pixels from its pane's top, on its scale (a relative one read against its first
   * value in view: see Pane.y); NaN where it can't.
   * @param {import("./pane.js").Series} series @param {number} value
   */
  y(series, value) {
    const pane = this.panes.find((pane) => pane.series.includes(series));
    return pane ? pane.y(series, value, this.time) : NaN;
  }

  /** A series' value at a height (CSS pixels from its pane's top), in its own terms; NaN where it has none. @param {import("./pane.js").Series} series @param {number} y */
  value(series, y) {
    const pane = this.panes.find((pane) => pane.series.includes(series));
    return pane ? pane.value(series, y, this.time) : NaN;
  }

  /**
   * The crosshair on a point (a logical index: the nearest whole one) at a height in the chart (CSS pixels from its top),
   * or gone (NaN); past the last point, on the slots after it in view (as many as its spacing makes: the line follows
   * the pointer there, its date as the points' step goes on). Told to the crosshair's listeners (`input`: a pointer moved it, at `x` CSS pixels into the plot).
   * Pointer input marks placements and clears; supplied x anchors to the plot, otherwise to the index.
   * @param {number} index @param {number} [y] @param {boolean} [input] @param {number} [x]
   */
  setCrosshair(index, y = NaN, input = false, x = NaN) {
    this.#placeCrosshair(index, y, input, x);
    this.invalidate();
  }

  /** A pointer's crosshair gone, its origin kept for linked charts. */
  #clearPointer() {
    this.setCrosshair(NaN, NaN, true);
  }

  /** Placement and render refresh share one hit test and notification decision; a render needs no further frame.
   * @param {number} index @param {number} y @param {boolean} input @param {number} x */
  #placeCrosshair(index, y, input, x) {
    const time = this.time, crosshair = this.crosshair;
    const k = index === index && time.length ? Math.min(Math.max(Math.round(index), 0), time.lastSlot) : NaN;
    let pane = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    // (Its pane by itself, not its place: one removed, the next in its place is another.)
    const inPane = pane >= 0 ? this.panes[pane] : null;
    const moved = (k !== crosshair.index || y !== crosshair.y || inPane !== this.crossedPane || input !== this.#crossedInput || !Object.is(x, this.pointerX)) && !(k !== k && crosshair.index !== crosshair.index);
    const refresh = input && k === k && (this.recrossing || (inPane?.changes ?? -1) !== this.crossedChanges);
    const viewChanged = input && k === k && (time.from !== this.crossedFrom || time.spacing !== this.crossedSpacing);
    this.recrossing = false;
    (this.#crossedInput = input), (crosshair.index = k), (crosshair.y = y), (crosshair.pane = pane), (this.crossedPane = inPane), (this.pointerX = x);
    const within = inPane ? y - inPane.top : NaN, at = x === x ? x : k === k ? time.x(k) : NaN;
    // (A pointer's: the series it's on, found once (at the pointer, not the point), told and its dot marked by it.)
    if (!moved && !refresh && !viewChanged) return;
    const hit = input && k === k && inPane ? inPane.hit(time, k, at, within) : -1;
    const series = hit >= 0 && inPane ? inPane.series[hit] : null, hitChanged = crosshair.hit !== series;
    crosshair.hit = series;
    // (Recorded before listeners: their changes still need refreshing at the next draw.)
    (this.crossedChanges = inPane?.changes ?? -1), (this.crossedFrom = time.from), (this.crossedSpacing = time.spacing);
    if (!moved && !refresh && !hitChanged) return;
    // (Its listeners as they were: their list never changed in place (one added or leaving makes a new one), so one
    // leaving as it's told skips no other, one added isn't told this time.)
    const listeners = this.crosshairListeners;
    for (let l = 0; l < listeners.length; l++) listeners[l](k, pane, within, input, at, hit);
  }

  /**
   * Told of a click or a tap (a press let go where it began, the crosshair not shown by it): its point (a logical index,
   * the nearest whole one), its pane (-1: none), the series hit there (its index in the pane, -1: none), its height in
   * that pane and where across the plot (CSS pixels).
   * @param {(index: number, pane: number, series: number, y: number, x: number) => void} listener
   */
  onClick(listener) {
    this.clickListeners = [...this.clickListeners, listener];
    return () => void (this.clickListeners = this.clickListeners.filter((each) => each !== listener));
  }

  /**
   * Told each time the crosshair moves: its point (NaN: gone), the pane it's in (-1: none), its height in that pane,
   * whether pointer input set or cleared it, where across the plot (CSS pixels: the pointer's, else the point's), and the series a
   * pointer's on there (its index in the pane, as Pane's hit finds it; -1: none, or not a pointer's).
   * @param {(index: number, pane: number, y: number, input: boolean, x: number, series: number) => void} listener
   */
  onCrosshair(listener) {
    this.crosshairListeners = [...this.crosshairListeners, listener];
    return () => void (this.crosshairListeners = this.crosshairListeners.filter((each) => each !== listener));
  }

  /** Its look (DARK, LIGHT or a page's): its element's ground too (under the axis' corner). Then invalidate(true). */
  get theme() {
    return this.#theme;
  }

  set theme(theme) {
    this.#theme = theme;
    this.element.style.background = theme.background;
    // (Its legends' words in its font: every word on the chart in one face and size.)
    this.element.style.setProperty("--bc-font", theme.font);
  }

  /**
   * Told each time it's laid out (its size, its panes' places, its axes' widths: a page's own things kept on them; a
   * separator's drag, at each move).
   * @param {() => void} listener
   */
  onLayout(listener) {
    this.layoutListeners = [...this.layoutListeners, listener];
    return () => void (this.layoutListeners = this.layoutListeners.filter((each) => each !== listener));
  }

  /** Told of a double-click or a double tap on the time axis: the page's view to come back to (the library's own: none). @param {() => void} listener */
  onTimeReset(listener) {
    this.timeResetListeners = [...this.timeResetListeners, listener];
    return () => void (this.timeResetListeners = this.timeResetListeners.filter((each) => each !== listener));
  }

  /** Told each frame the view changed in. @param {(from: number, to: number) => void} listener */
  onRange(listener) {
    this.rangeListeners = [...this.rangeListeners, listener];
    return () => void (this.rangeListeners = this.rangeListeners.filter((each) => each !== listener));
  }

  /** The right axis' width as shown: as its labels and tags need (see fitted). */
  get axisShown() {
    return this.axis && (this.fitted[0] || this.axis);
  }

  /** The left axis' width as shown (where a pane has a left scale). */
  get leftShown() {
    return this.leftAxis && (this.fitted[1] || this.leftAxis);
  }

  /** Laid out again at the element's size (watched: needed only for a change it can't see), keeping the visible range. */
  layout() {
    const box = this.element.getBoundingClientRect(), wasLeft = this.left;
    this.dpr = devicePixelRatio || 1;
    this.width = box.width;
    this.height = box.height;
    const room = Math.max(0, box.height - this.timeHeight), shares = this.panes.reduce((sum, pane) => sum + pane.share, 0) || 1;
    // (An axis at the left too where a pane has a left scale: the plot and the time axis after it.)
    const left = (this.left = this.panes.some((pane) => pane.left) ? this.leftShown : 0), width = Math.max(1, box.width - left), axis = this.axisShown;
    // (A pointer's place kept where it is on the chart: the plot moved under it as a left axis comes or goes.)
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
      // (Its layer over everything: over the crosshair (see their z-index).)
      if (pane.overCanvas) {
        if (!pane.overCanvas.parentNode) this.element.append(pane.overCanvas);
        pane.overCanvas.style.left = `${left}px`;
      }
      pane.separated = true;
      top = bottom;
    }
    // (The time axis under the plot alone: a page stretching the plot stretches it alike. Under the axis, the chart's ground.)
    this.timeAxis.place(room, Math.max(1, width - axis), this.timeHeight, this.dpr);
    this.timeAxis.canvas.style.left = `${left}px`;
    const cornerWidth = axis || left;
    Object.assign(this.corner.style, { left: `${axis ? left + Math.max(1, width - axis) : 0}px`, top: `${room}px`, width: `${cornerWidth}px`, height: `${this.timeHeight}px`, display: cornerWidth ? "block" : "none" });
    this.crosshair.place(box.width, box.height, this.dpr);
    this.time.resize(Math.max(1, width - axis));
    // (The crosshair's pane found again: panes moved, came or went.)
    this.#recross();
    this.invalidate(true);
    const listeners = this.layoutListeners;
    for (let k = 0; k < listeners.length; k++) listeners[k]();
  }

  /**
   * A frame asked for: the view moved (a pan, a zoom), or `whole`, everything drawn anew (sizes, panes, a series'
   * values or look, the theme). After changing a series, ask for one.
   * @param {boolean} [whole]
   */
  invalidate(whole = false) {
    this.whole ||= whole;
    this.moved ||= !whole;
    this.#schedule();
  }

  /** Request a draw only while the chart is alive. */
  #schedule() {
    if (!this.listening.signal.aborted) this.frame ||= requestAnimationFrame(this.tick);
  }

  /** @param {number} now */
  #frame(now) {
    this.frame = 0;
    this.controller.step(now);
    // (Nothing moved since the last frame, its pictures moved or stretched before: drawn whole now, the seams gone; till
    // then they're kept, a move of less than a pixel moving nothing.)
    const whole = this.whole || (this.seams && !this.moved);
    this.moved = false;
    if (whole) this.seams = false;
    this.render(whole);
    if (this.seams) this.#schedule();
  }

  /** Drawn now (for a measurement: its frames draw it). @param {boolean} [whole] @returns {void} */
  render(whole = false) {
    this.whole = false;
    const time = this.time, crosshair = this.crosshair;
    for (let k = 0; k < this.panes.length; k++) {
      const pane = this.panes[k];
      pane.fit(time);
      if (pane.draw(time, this.theme, this.dpr, whole)) this.seams = true;
    }
    // (Each axis as wide as its labels and tags need (4 pixels a step, AXIS_LEAST at least), each side on its own,
    // from its first draw (`axis`, its width till then, no floor: a phone's plot not left short), the visible time
    // range kept as the plot's width changes: wider at once where they need it, never narrower by itself after (no
    // plot growing and shrinking as values come and go); as narrow as they let it only when fitted again (fitAxes: a
    // price axis' double-click, the page's own). Laid out and drawn whole again now (a picture
    // taken from this render has it all); in that draw, only wider (it ends).)
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
    // (The pointer refreshed on fitted geometry, through the same placement path as input. An unchanged pointer asks
    // for neither another hit test nor another frame.)
    if (this.recrossing || (this.#crossedInput && crosshair.index === crosshair.index)) {
      const x = this.pointerX;
      // (Over an axis now (one come under it): gone, as a pointer over an axis has none.)
      if (x === x && !(x >= 0 && x <= time.width)) this.#clearPointer();
      else this.#placeCrosshair(x === x ? time.index(x) : crosshair.index, crosshair.y, true, x);
    }
    this.timeAxis.draw(time, this.theme, this.dpr, time.width, this.panes);
    this.crosshair.draw(time, this.panes, Math.max(0, this.height - this.timeHeight), this.left, this.theme, this.dpr, this.timeAxis, this.separatorLit);
    // (Each legend pointed at the series read: the page's, crosshair or not (a tag, a row of its own, the pointer moved
    // onto the legend); else the one a pointer's on, none with the crosshair gone. Its items made here read at its
    // point, in this frame.)
    const read = crosshair.pointed !== undefined ? crosshair.pointed : crosshair.index === crosshair.index ? crosshair.hit : null;
    for (const pane of this.panes) if (pane.legend) pane.legend.point(pane.legend.itemOf(read)), pane.legend.read(crosshair.index, (series, v) => pane.words(series, v, time), (series) => pane.restOf(series, time));
    for (let k = 0; k < this.panes.length; k++) this.panes[k].drawOver(time, this.dpr);
    if (time.from !== this.drawnFrom || time.to !== this.drawnTo) {
      (this.drawnFrom = time.from), (this.drawnTo = time.to);
      const listeners = this.rangeListeners;
      for (let k = 0; k < listeners.length; k++) listeners[k](time.from, time.to);
    }
  }

  // Pointers: one drags (a pan, gliding on once let go), two pinch (a zoom about their middle, and a pan as it moves);
  // the wheel zooms about the pointer, a trackpad's sideways scroll pans (a diagonal swipe both). On an axis, a drag scales it: a price axis
  // about its middle (its range then set by hand, fitted again by a double-click, its width too: see fitAxes), the time axis about the plot's right
  // edge. The crosshair: where a mouse or pen is; a finger
  // held still (HOLD) shows it, moving it then instead of the chart, till it's let go. A finger pans once it's moved
  // past a slop of 6 pixels (a hold, else, would move the chart a little).
  #input() {
    /**
     * @typedef {{ pane: Pane | null, side: "right" | "left", a: number, b: number, at: number, held?: number }} Axis
     * An axis pressed, as it was then: a pane's price axis on `side` (its range, `a` to `b`; the press's height in the
     * pane, `at`), or the time axis (`pane` null: the spacing `a`, the point it holds `b` (its latest, else the view's
     * right end) at `held` of the plot's width, the press's x in the plot `at`). Scaled once the press has moved as far
     * as a click can't (a click on a tag scales nothing).
     */
    const element = this.element, time = this.time, down = /** @type {Map<number, { x: number, y: number, startX: number, startY: number, panning: boolean, moved: boolean, axis: Axis | null, pane: Pane | null, beside: PointerEvent | null, stop: () => void }>} */ (new Map());
    let held = -1;
    // (Two fingers down in this gesture: its end no click.)
    let twoFingers = false;
    // (A separator dragged: the panes over and under it, the pointer dragging it (-1: none).)
    let sizing = /** @type {[Pane, Pane] | null} */ (null), sizer = -1;
    /** Let go (`y`: a mouse or pen's height in the chart, its separator lit still where it's over one). @param {number} [y] */
    const stopSizing = (y = NaN) => {
      (sizing = null), (sizer = -1), (this.resizing = false), (this.separatorLit = this.panes[separator(y, false)] ?? null);
      this.invalidate(true);
    };
    /** The separator under a height in the chart (the pane over it), or -1. @param {number} y @param {boolean} touch */
    const separator = (y, touch) => {
      for (let k = 0; k < this.panes.length; k++) if (this.#next(k) >= 0 && Math.abs(this.panes[k].top + this.panes[k].height - y) <= (touch ? NEAR_TOUCH : NEAR_FINE)) return k;
      return -1;
    };
    // (Over the plot, the crosshair where it points; over an axis, none: its tags are read there.)
    const point = (/** @type {PointerEvent} */ event) => {
      const box = element.getBoundingClientRect(), x = event.clientX - box.left - this.left, over = x >= 0 && x <= time.width;
      // (A drag's crosshair still to be found (see #follow): found at the next frame where the pointer is then, on the
      // panes fitted to the view it moved to, told once.)
      if (this.recrossing) return void ((this.pointerX = x), (this.crosshair.y = event.clientY - box.top), this.invalidate());
      this.setCrosshair(over ? time.index(x) : NaN, event.clientY - box.top, true, over ? x : NaN);
    };
    // (The axis a drag from a point would scale, found without making anything (a mouse or pen's hover asks at each move): its
    // pane (none: the time axis) and side.)
    let foundPane = /** @type {Pane | null} */ (null), foundSide = /** @type {"right" | "left"} */ ("right");
    /** Whether a drag from a point in the chart (CSS pixels from its corner) would scale an axis (see foundPane). @param {number} x @param {number} y */
    const findAxis = (x, y) => {
      if (y >= this.height - this.timeHeight) return (foundPane = null), x >= this.left && x <= this.left + time.width;
      for (const pane of this.panes) {
        if (y < pane.top || y >= pane.top + pane.height) continue;
        // (A relative scale's range follows its bases as the view moves: never set by hand, as the chart library's.)
        const side = x >= this.left + pane.plot && pane.width > pane.plot ? "right" : x < this.left && pane.left ? "left" : null, scale = side === "left" ? pane.left : pane.scale;
        // (One fitted to nothing (an empty pane: its range a placeholder) none either.)
        return (foundPane = pane), (foundSide = side ?? "right"), Boolean(side && scale && !scale.relative && !scale.empty && scale.lo < scale.hi && (scale.mode !== "log" || scale.lo > 0));
      }
      return false;
    };
    /** The axis a press at a point would drag, as it is now, or null. @param {number} x @param {number} y @returns {Axis | null} */
    const axisAt = (x, y) => {
      if (!findAxis(x, y)) return null;
      const pane = foundPane, scale = foundSide === "left" ? pane?.left : pane?.scale;
      if (pane && scale) return { pane, side: foundSide, a: scale.lo, b: scale.hi, at: y - pane.top };
      // (The time axis': its latest point held where it is (its room after it as it is), where it's in view; else the plot's
      // right edge. As a share of the plot's width: an axis widening meanwhile keeps it in the plot.)
      const last = time.length - 1, held = last >= 0 && time.x(last) >= 0 && time.x(last) <= time.width ? time.x(last) / time.width : 1;
      return { pane: null, side: "right", a: time.spacing, b: held < 1 ? last : time.to, held, at: x - this.left };
    };
    /**
     * An axis dragged to a point (CSS pixels from the chart's corner): a price axis' range scaled about its middle (on a
     * log scale, its logarithms') by how far the press has come down its height, as the chart library's (a tenth of it
     * at least); the time axis' spacing by how far the press has come towards the plot's right edge, its latest point
     * held where it is (where it's in view: its room after it the same in pixels), else that edge.
     * @param {Axis} axis @param {number} x @param {number} y
     */
    const scaleAxis = (axis, x, y) => {
      const pane = axis.pane;
      if (!pane) {
        const width = time.width, start = width - axis.at, now = Math.min(Math.max(width - (x - this.left), 0), width);
        // (Its span within the zoom's bounds first: what it holds held at them too.)
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
     * A pane's scales set by hand (an axis dragged) slid with a drag `dy` CSS pixels down, as the time axis is with it
     * across (a fitted one fits the view as ever; a relative one reads its bases): its values follow the pointer.
     * @param {Pane} pane @param {number} dy
     */
    const slide = (pane, dy) => {
      if (!dy) return;
      for (const [scale, side, auto] of /** @type {const} */ ([[pane.scale, "right", pane.auto], [pane.left, "left", pane.leftAuto]]))
        if (scale && !auto && !scale.relative && !scale.empty) pane.setRange(scale.value(scale.y(scale.lo) - dy), scale.value(scale.y(scale.hi) - dy), side);
    };
    // (A price axis' last click or tap: when, where and which, for a second one on it (a double-click, a double tap: iOS
    // sends no dblclick) fitting it again; forgotten at any release that isn't a click.)
    let tapAt = -Infinity, tapX = 0, tapY = 0, tapPane = /** @type {Pane | null} */ (null), tapSide = "";
    const plotX = (/** @type {number} */ clientX) => clientX - element.getBoundingClientRect().left - this.left;
    const signal = this.listening.signal;
    /** A press taken from here on, its end told to ended. @param {PointerEvent} event */
    const hold = (event) => {
      const id = event.pointerId;
      element.setPointerCapture(id);
      return press(event, { captor: element, signal, end: (release, lost) => ended(id, release, lost) });
    };
    /** The separator over pane `on` dragged by a press. @param {PointerEvent} event @param {number} on */
    const size = (event, on) => {
      hold(event);
      // (A drag, not a click: the one a browser may send on its lift is taken (WebKit's, at the press's start, on what
      // slid under it there: a legend's item), any pointer; none past the next press, nor a key's (detail 0).)
      const swallow = (/** @type {MouseEvent} */ click) => click.detail && (click.stopPropagation(), click.preventDefault());
      addEventListener("click", swallow, { capture: true, once: true, signal });
      addEventListener("pointerdown", () => removeEventListener("click", swallow, true), { capture: true, once: true, signal });
      (sizing = [this.panes[on], this.panes[this.#next(on)]]), (sizer = event.pointerId), (this.resizing = true), (this.separatorLit = this.panes[on]), this.controller.stop(), this.#clearPointer();
    };
    /** The separator band's press that began on a page's own element (see take), till it goes up or down, or not. */
    let banded = /** @type {{ id: number, x: number, y: number, on: number } | null} */ (null);
    const take = (/** @type {PointerEvent} */ event) => {
      banded = null;
      if (event.button || sizer >= 0) return;
      const touch = !fine(event), page = touch && this.embedded, y = event.clientY - element.getBoundingClientRect().top, on = down.size || page ? -1 : separator(y, touch);
      // (A separator dragged: no other pointer does anything meanwhile. A press beginning on a page's own element in the
      // chart (a legend's button) is the page's: none of its canvases (wherever a page put them), not the chart's; one
      // more during the chart's own (a second finger: a pinch), the chart's wherever it lands. In a separator's band, the
      // separator's once it goes up or down (see banded): a tap, a sideways swipe, still the page's.)
      if (!down.size && event.target !== element && !this.#owns(event.target)) return void (on >= 0 && (banded = { id: event.pointerId, x: event.clientX, y: event.clientY, on }));
      // (On a separator: it's dragged, nothing else.)
      if (on >= 0) return size(event, on);
      const id = event.pointerId, stop = hold(event);
      const box = element.getBoundingClientRect(), found = down.size ? null : axisAt(event.clientX - box.left, y), axis = page && found?.pane ? null : found;
      down.set(event.pointerId, { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, panning: !touch, moved: false, axis, pane: this.panes.find((pane) => y >= pane.top && y < pane.top + pane.height) ?? null, beside: down.size ? event : null, stop });
      this.controller.grab();
      clearTimeout(this.holding);
      // (A held finger's crosshair going at the next frame: gone now.)
      if (this.letting) cancelAnimationFrame(this.letting), this.#clearPointer();
      this.holding = this.letting = 0;
      // (A second finger: a pinch, the crosshair a held finger showed gone.)
      if (held >= 0) (held = -1), this.#clearPointer();
      // (A second finger: a pinch, no axis dragged after it, the one finger left panning.)
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
      // (A press in a separator's band on a page's element: gone up or down as far as a click can't, more than sideways, the
      // separator's; sideways (a legend's row scrolled), the page's.)
      if (banded?.id === event.pointerId) {
        // (A mouse let go elsewhere: no press.)
        const dx = Math.abs(event.clientX - banded.x), dy = Math.abs(event.clientY - banded.y), pressed = !fine(event) || event.buttons;
        if (pressed && dy >= 6 && dy > dx) size(event, banded.on);
        if (!pressed || dx >= 6 || dy >= 6) banded = null;
        return;
      }
      if (sizer === event.pointerId) return void (sizing && !this.#size(sizing[0], sizing[1], event.clientY - element.getBoundingClientRect().top) && stopSizing());
      // (Another pointer meanwhile (a mouse beside a finger): nothing while a separator's dragged.)
      if (this.resizing) return;
      const at = down.get(event.pointerId);
      if (!at && fine(event)) {
        // (Over a page's own element over it (a legend): no crosshair, as off the chart.)
        if (event.target !== element && !this.#owns(event.target)) return void (this.separatorLit && ((this.separatorLit = null), this.invalidate()), this.#clearPointer());
        // (A mouse or pen over a separator, or an axis: it shows it can be dragged, no crosshair over a separator.)
        const box = element.getBoundingClientRect(), y = event.clientY - box.top, lit = separator(y, false), over = lit >= 0, axis = !over && findAxis(event.clientX - box.left, y);
        element.style.cursor = over ? "row-resize" : axis ? (foundPane ? "ns-resize" : "ew-resize") : "";
        // (The separator pointed at in the theme's active color: it can be dragged.)
        if (this.panes[lit] !== (this.separatorLit ?? undefined)) (this.separatorLit = this.panes[lit] ?? null), this.invalidate();
        return void (over ? this.#clearPointer() : point(event));
      }
      if (!at) return;
      // (Gone as far as a click can't, once: a drag back to where it began is no click.)
      if (!at.moved && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) >= 6) {
        at.moved = true;
        if (at.axis && this.#crossedInput) this.#clearPointer();
      }
      if (held === event.pointerId) return void ((at.x = event.clientX), (at.y = event.clientY), point(event));
      if (!at.panning) {
        if (Math.hypot(event.clientX - at.startX, event.clientY - at.startY) <= 6) return;
        (at.panning = true), clearTimeout(this.holding), (this.holding = 0);
      }
      // (An axis pressed: scaled once the press has moved as far as a click can't, no glide after.)
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
        // (About the fingers' middle as it was, then moved with it: what's under each stays under it.)
        let other = at;
        for (const each of down.values()) if (each !== at) other = each;
        this.controller.pinch(plotX((at.x + other.x) / 2), Math.abs(at.x - other.x), Math.abs(event.clientX - other.x), (event.clientX - at.x) / 2);
      }
      (at.x = event.clientX), (at.y = event.clientY);
      // (A mouse or pen's crosshair where it is now, the chart moved under it.)
      // (Its listeners told once the panes are fitted to the view it moved to: see render.)
      if (fine(event)) this.#follow(event);
      this.invalidate();
    }, { signal });
    for (const type of ["pointerup", "pointercancel"]) element.addEventListener(type, (event) => banded?.id === event.pointerId && (banded = null), { signal });
    element.addEventListener("pointerleave", (event) => fine(event) && !down.size && !this.resizing && ((this.separatorLit = null), this.#clearPointer()), { signal });
    /**
     * A press's end (see press): its release, or none (`lost`: its touch proved gone).
     * @param {number} id @param {PointerEvent | null} event @param {boolean} lost
     */
    const ended = (id, event, lost) => {
      if (sizer === id) return void ((tapAt = -Infinity), stopSizing(event && fine(event) ? event.clientY - element.getBoundingClientRect().top : NaN));
      const at = down.get(id);
      if (!at || !down.delete(id)) return;
      // (Its touch proved gone: those taken beside it as a pinch's second finger pressed again alone, once every lost one's let go.)
      if (lost) touchRecoveries.add(recover);
      const pinched = twoFingers;
      if (!down.size) twoFingers = false;
      clearTimeout(this.holding);
      this.holding = 0;
      // (The finger holding the crosshair let go: it goes at the next frame (a page's own lifting handled first, as a
      // touch's end came after it: Studio keeps it there); no glide.)
      if (held === id) return void ((held = -1), this.controller.forget(), (tapAt = -Infinity), (this.letting = requestAnimationFrame(() => ((this.letting = 0), this.#clearPointer()))));
      // (Let go where it began: a click.)
      if (event && !down.size && !pinched && !at.moved && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) < 6) {
        this.controller.forget();
        const axis = at.axis;
        const again = axis && axis.pane === tapPane && axis.side === tapSide && event.timeStamp - tapAt < 350 && Math.hypot(event.clientX - tapX, event.clientY - tapY) < 24;
        (tapAt = axis && !again ? event.timeStamp : -Infinity), (tapX = event.clientX), (tapY = event.clientY), (tapPane = axis?.pane ?? null), (tapSide = axis?.side ?? "");
        // (A price axis' second: fitted again; the time axis': its page's to say what the view comes back to.)
        if (again && axis.pane) axis.pane.setRange(undefined, undefined, axis.side), this.fitAxes();
        else if (again) for (const listener of this.timeResetListeners) listener();
        return void this.#click(event);
      }
      tapAt = -Infinity;
      // (A pinch's end, a finger at a time: no glide (a zoom has no momentum; the finger left lifts as it stops). A
      // mouse's or a pen's drag: none either, as a desktop's charts (a finger's flick alone glides on).)
      if (down.size || pinched || !event || fine(event)) return void this.controller.forget();
      this.controller.flick(event.timeStamp);
    };
    // (Embedded: the browser scrolls the page unless a touchmove's default is cancelled, which touch-action can't say
    // "vertical unless held" for: cancelled while a finger holds the crosshair or two of the chart's own fingers pinch
    // (`down` holds only pointers the chart took: a page's own control over it isn't counted). A pinch starts with
    // both fingers: one that joins a scroll already running is the browser's.)
    if (this.embedded) element.addEventListener("touchmove", (event) => (held >= 0 || down.size > 1) && event.cancelable && event.preventDefault(), { passive: false, signal });
    // (Cancel every forgotten touch first; only then replay presses still down that were taken as a second finger.)
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
   * The series whose tag is at a point (CSS pixels from its element's corner: on an axis, as last drawn; not one hidden
   * since), or null (none while a separator's dragged).
   * @param {number} x @param {number} y
   */
  tagAt(x, y) {
    if (this.resizing) return null;
    for (const pane of this.panes) {
      if (y < pane.top || y >= pane.top + pane.height) continue;
      // (As last drawn: one hidden since, none.)
      const line = x >= this.left + pane.plot && x < this.left + pane.width ? pane.lineAt(0, y - pane.top) : x >= 0 && x < this.left ? pane.lineAt(1, y - pane.top) : null;
      return line && !line.hidden ? line : null;
    }
    return null;
  }

  /** Whether `target` is one of its canvases (a pane's plot or axes, the time axis). @param {EventTarget | null} target */
  #owns(target) {
    if (target === this.timeAxis.canvas) return true;
    for (const pane of this.panes) if (target === pane.canvas || target === pane.axisCanvas || target === pane.leftCanvas) return true;
    return false;
  }

  /** A click at a pointer's place: its point, pane and series told to the click's listeners. @param {PointerEvent} event */
  #click(event) {
    // (None listening: nothing to find.)
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

  /** The pane under the one at `k` (past those with no height: a separator is between the panes shown), or -1 where it's the last or has none. @param {number} k */
  #next(k) {
    return this.panes[k].share > 0 ? this.panes.findIndex((pane, i) => i > k && pane.share > 0) : -1;
  }

  /**
   * The separator between `over` and `under` dragged to `y` (CSS pixels in the chart): their shares moved between them,
   * their total kept, neither under LEAST; laid out and drawn at their new heights at each move (what's dragged is what
   * shows: no picture stretched). Whether they're still next to each other (a pane removed: the drag's over).
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

/** A pane taken off its chart: its canvases and frame (its legend in it) gone, its chart no longer told of it (a page keeping it keeps no chart). @param {Pane} pane */
function detach(pane) {
  for (const element of [pane.canvas, pane.axisCanvas, pane.leftCanvas, pane.overCanvas, pane.frame]) element?.remove();
  pane.onLayout = () => {};
}
