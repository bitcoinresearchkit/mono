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

import { Timeline } from "./timeline.js";
import { Pane } from "./pane.js";
import { TimeAxis } from "./axis.js";
import { Crosshair } from "./crosshair.js";

export { Timeline, Pane, TimeAxis };
export { Line, FAINT } from "./line.js";
export { Band } from "./band.js";
export { Bars } from "./bars.js";
export { Candles, BODIES_FROM } from "./candles.js";
export { Bubbles } from "./bubbles.js";
export { Shade } from "./shade.js";
export { Grid } from "./grid.js";
export { Profile } from "./profile.js";
export { stack } from "./stack.js";
export { Custom } from "./custom.js";
export { Marks } from "./marks.js";
export { Levels } from "./levels.js";
export { Legend } from "./legend.js";
export { picture } from "./picture.js";
export { Scale } from "./scale.js";
export { compact, decimalsOf } from "./format.js";
export { channels } from "./color.js";

/** @type {import("./pane.js").Theme} */
export const DARK = { background: "#0e0f11", text: "#8a8f98", grid: "rgba(255, 255, 255, 0.06)", font: "11px system-ui, sans-serif", crosshair: "rgba(247, 248, 248, 0.4)", across: "rgba(247, 248, 248, 0.25)", badge: "#f7f8f8", separator: "rgba(255, 255, 255, 0.14)", active: "#f2660f" };
/** @type {import("./pane.js").Theme} */
export const LIGHT = { background: "#ffffff", text: "#6b6f76", grid: "rgba(0, 0, 0, 0.06)", font: "11px system-ui, sans-serif", crosshair: "rgba(14, 15, 17, 0.35)", across: "rgba(14, 15, 17, 0.2)", badge: "#0e0f11", separator: "rgba(0, 0, 0, 0.12)", active: "#f2660f" };

/** A touch held this long (ms) without moving shows the crosshair, the finger moving it then, not the chart (as Lightweight Charts' long tap). */
const HOLD = 240;
/** How near a separator between panes a pointer takes it (CSS pixels): a mouse's, a finger's; a pane's least height. */
const NEAR_MOUSE = 4, NEAR_TOUCH = 12, LEAST = 32;

/** An axis' least width (CSS pixels): a short label's, as "0". */
const AXIS_LEAST = 28;
/** A flick's speed kept each millisecond: a chart's glide, two thirds of iOS's lists' (0.998), not years past what it was flicked for. */
const SLOWING = 0.997;
/** A flick's speed (CSS pixels a millisecond) as its finger's over its last moments (this many milliseconds); slower, none (a drag let go where it stopped). */
const FLICK_SPAN = 50, FLICK_LEAST = 0.25;

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

  /**
   * @param {HTMLElement} element
   * @param {{ theme?: import("./pane.js").Theme, axis?: number, leftAxis?: number, timeHeight?: number, rightOffset?: number, maxSpacing?: number, dateFormat?: (t: number) => string }} [options]
   */
  constructor(element, { theme = DARK, axis = 64, leftAxis = axis, timeHeight = 26, rightOffset, maxSpacing, dateFormat } = {}) {
    this.element = element;
    this.theme = theme;
    /**
     * Whether an axis is at every pane's right, and at their left where one has a left scale (0: none), and its width till
     * it's drawn; then as narrow as its labels and tags let it (see fitted). Then layout().
     */
    this.axis = axis;
    this.leftAxis = leftAxis;
    /** Each side's axis as shown, as its labels and tags need it (right, left; 0: not drawn yet, `axis`'s). */
    this.fitted = [0, 0];
    /** A render drawing again at the axes' new widths: in it they only widen (see render). */
    this.fitting = false;
    /** The axes fitted again at the next frame, narrower too (see fitAxes). */
    this.refit = false;
    /** The view set anew since the last frame (a timeline, the latest shown): unseen, its axes fitted keep its right end. */
    this.placed = true;
    this.timeHeight = timeHeight;
    this.time = new Timeline({ rightOffset, maxSpacing });
    this.timeAxis = new TimeAxis();
    this.crosshair = new Crosshair();
    if (dateFormat) this.crosshair.setFormat(dateFormat);
    /** @type {((index: number, pane: number, y: number, input: boolean, x: number, series: number) => void)[]} */
    this.crosshairListeners = [];
    /** The pane the crosshair was last in (null: none). @type {Pane | null} */
    this.crossedPane = null;
    /** The pointer's x across the plot as the crosshair was last set by it (NaN: none). */
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
    /**
     * A flick gliding on: its speed (CSS pixels a millisecond), its last frame's time, its speed kept each millisecond
     * (SLOWING; more where it'd pass an end: it eases into it, see #fling), and its way left to that end.
     */
    this.glide = { speed: 0, at: 0, slowing: SLOWING, left: Infinity };
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
    /** Its listeners, gone with it (see destroy). */
    this.listening = new AbortController();
    /** A touch's hold waiting to show the crosshair (a timer; 0: none). */
    this.holding = 0;
    /** A held finger's crosshair going at the next frame (a frame's request; 0: none): see the pointer's lifting. */
    this.letting = 0;
    // (Its canvases placed in it: a box for them, unless it's one already.)
    if (getComputedStyle(element).position === "static") element.style.position = "relative";
    Object.assign(element.style, { overflow: "hidden", touchAction: "none", userSelect: "none", webkitUserSelect: "none" });
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
    if (this.pointerX === this.pointerX) this.recrossing = true;
    else this.setCrosshair(crosshair.index, crosshair.y);
  }

  /**
   * A mouse dragging the view: its crosshair found again at the next frame where it is then (see render), its listeners
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

  /** The timeline's times (seconds, rising), every pane's lines' indices. @param {Float64Array} times */
  setTimes(times) {
    this.time.setTimes(times);
    this.placed = true;
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

  /** The last `span` points in view. @param {number} span */
  show(span) {
    this.time.show(span);
    this.placed = true;
    this.invalidate(true);
  }

  /**
   * The view: logical indices, fractions allowed. As wide as it is (a page's own pan: its glide, its navigator's drag),
   * moved as a pan is, by whole pixels (the rest kept): the panes' pictures moved, not drawn again.
   * @param {number} from @param {number} to
   */
  setRange(from, to) {
    const time = this.time, span = to - from;
    // (From where it truly is: its kept part of a pixel counted once.)
    if (time.length && Math.abs(span - (time.to - time.from)) <= span * 1e-9) time.pan((time.from - from) * time.spacing - time.rest);
    else time.setRange(from, to);
    this.invalidate();
  }

  get range() {
    return { from: this.time.from, to: this.time.to };
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
   * or gone (NaN). Told to the crosshair's listeners (`input`: a pointer moved it, at `x` CSS pixels into the plot).
   * @param {number} index @param {number} [y] @param {boolean} [input] @param {number} [x]
   */
  setCrosshair(index, y = NaN, input = false, x = NaN) {
    const k = index === index && this.time.length ? Math.min(Math.max(Math.round(index), 0), this.time.length - 1) : NaN, crosshair = this.crosshair;
    let pane = -1;
    for (let p = 0; p < this.panes.length; p++) if (y >= this.panes[p].top && y < this.panes[p].top + this.panes[p].height) pane = p;
    // (Its pane by itself, not its place: one removed, the next in its place is another.)
    const inPane = pane >= 0 ? this.panes[pane] : null;
    const moved = (k !== crosshair.index || y !== crosshair.y || inPane !== this.crossedPane || (x === x && x !== this.pointerX)) && !(k !== k && crosshair.index !== crosshair.index);
    (crosshair.index = k), (crosshair.y = y), (crosshair.pane = pane), (this.crossedPane = inPane), (this.pointerX = x);
    const within = pane >= 0 ? y - this.panes[pane].top : NaN, at = x === x ? x : k === k ? this.time.x(k) : NaN;
    // (A pointer's: the series it's on, found once (at the pointer, not the point), told and its dot marked by it.)
    if (!moved) return void this.invalidate();
    const hit = input && k === k && pane >= 0 ? this.panes[pane].hit(this.time, k, at, within) : -1;
    crosshair.hit = hit >= 0 ? this.panes[pane].series[hit] : null;
    // (Its listeners as they were: their list never changed in place (one added or leaving makes a new one), so one
    // leaving as it's told skips no other, one added isn't told this time.)
    const listeners = this.crosshairListeners;
    for (let l = 0; l < listeners.length; l++) listeners[l](k, pane, within, input, at, hit);
    this.invalidate();
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
   * whether a pointer moved it, where across the plot (CSS pixels: the pointer's, else the point's), and the series a
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

  /**
   * Laid out again at the element's size (watched: needed only for a change it can't see); `kept`, every point where it
   * is on the screen (an axis fitted to its labels: it grows over the plot's edge or gives it back, nothing under it
   * moving), else the view's right end (a box resized).
   * @param {boolean} [kept]
   */
  layout(kept = false) {
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
    for (const [k, pane] of this.panes.entries()) {
      const bottom = k === this.panes.length - 1 ? room : Math.round(top + (room * pane.share) / shares);
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
    this.time.resize(Math.max(1, width - axis), kept ? left - wasLeft : NaN);
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
    this.frame ||= requestAnimationFrame(this.tick);
  }

  /** @param {number} now */
  #frame(now) {
    this.frame = 0;
    const glide = this.glide;
    if (glide.speed) {
      // (Its move over the frame as its speed falls through it (not at its end's speed); come to its least speed, a
      // flick easing into an end moves the rest of its way there.)
      const dt = Math.min(64, now - glide.at), decay = glide.slowing ** dt, ln = -Math.log(glide.slowing);
      glide.at = now;
      let by = ln < Infinity ? (glide.speed * (1 - decay)) / ln : 0;
      glide.speed *= decay;
      const ends = Math.abs(glide.speed) < 0.02;
      if (ends && glide.left < Infinity) by = Math.sign(by || glide.speed) * glide.left;
      glide.left -= Math.abs(by);
      if (!this.time.pan(by) || ends) glide.speed = 0;
      else (this.moved = true), (this.frame = requestAnimationFrame(this.tick));
      if (ends && by) (this.moved = true), (this.frame ||= requestAnimationFrame(this.tick));
    }
    // (Nothing moved since the last frame, its pictures moved or stretched before: drawn whole now, the seams gone; till
    // then they're kept, a move of less than a pixel moving nothing.)
    const whole = this.whole || (this.seams && !this.moved);
    this.moved = false;
    if (whole) this.seams = false;
    this.render(whole);
    if (this.seams) this.frame ||= requestAnimationFrame(this.tick);
  }

  /** Drawn now (for a measurement: its frames draw it). @param {boolean} [whole] @returns {void} */
  render(whole = false) {
    this.whole = false;
    const time = this.time, crosshair = this.crosshair, x = this.pointerX;
    for (let k = 0; k < this.panes.length; k++) {
      const pane = this.panes[k];
      // (Its tags read the crosshair's point while it's on: each line's value there, by its line.)
      pane.tagsAt = crosshair.index;
      pane.fit(time);
      if (pane.draw(time, this.theme, this.dpr, whole)) this.seams = true;
    }
    // (Each axis as wide as its labels and tags need (4 pixels a step, AXIS_LEAST at least), each side on its own,
    // every point kept where it is on the screen (the axis over the plot's edge, not the plot shifted; a view set anew,
    // not seen yet, its right end and the room after its latest point as set): wider at once where they need it, never
    // narrower by itself (no plot growing and shrinking as values come and go); as narrow as they let it only when
    // fitted again (fitAxes: a price axis' double-click, the page's own). Laid out and drawn whole again now (a picture
    // taken from this render has it all); in that draw, only wider (it ends).)
    {
      let right = 0, leftNeed = 0;
      for (let k = 0; k < this.panes.length; k++) (right = Math.max(right, this.panes[k].axisNeed[0])), (leftNeed = Math.max(leftNeed, this.panes[k].left ? this.panes[k].axisNeed[1] : 0));
      const fit = (/** @type {number} */ need) => Math.max(AXIS_LEAST, Math.ceil(need / 4) * 4), refit = this.refit;
      this.refit = false;
      const next = (/** @type {number} */ want, /** @type {number} */ shown) => (want > shown || refit ? want : shown);
      const wantRight = this.axis ? next(fit(right), this.axisShown) : 0, wantLeft = this.left ? next(fit(leftNeed), this.leftShown) : 0;
      if ((this.axis && wantRight !== this.axisShown) || (this.left && wantLeft !== this.leftShown)) {
        if (this.axis) this.fitted[0] = wantRight;
        if (this.left) this.fitted[1] = wantLeft;
        this.layout(!this.placed);
        const was = this.fitting;
        this.fitting = true;
        try {
          return this.render(true);
        } finally {
          this.fitting = was;
        }
      }
    }
    // (A pointer's crosshair on the point under it as the view moves beneath it (a wheel's scroll, a glide) or its data or
    // layout change (see #recross), its listeners told with the panes fitted to the view: that point within the data, as
    // setCrosshair has it (past its end, its last), so a still pointer there asks for no frame.)
    // (Its pane's scale or series changed under it since it was told (a zoom, a live price):
    // what it hits read again.)
    const under = this.panes[crosshair.pane];
    // (The view moved under a still pointer (a zoom, a pan within a point): the series it's on found again, told where
    // that's another.)
    let changed = Boolean(under) && under.changes !== this.crossedChanges;
    if (under && !changed && x === x && crosshair.index === crosshair.index && (time.from !== this.crossedFrom || time.spacing !== this.crossedSpacing) && crosshair.hit !== (under.series[under.hit(time, crosshair.index, x, crosshair.y - under.top)] ?? null)) changed = true;
    (this.crossedFrom = time.from), (this.crossedSpacing = time.spacing);
    if (x === x && (this.recrossing || crosshair.index === crosshair.index)) {
      // (Over an axis now (one come under it): gone, as a pointer over an axis has none.)
      if (!(x >= 0 && x <= time.width)) (this.recrossing = false), this.setCrosshair(NaN);
      else if (this.recrossing || changed || Math.min(Math.max(Math.round(time.index(x)), 0), time.length - 1) !== crosshair.index) {
        // (Found again, told even where its point and height stay: what's under it may not.)
        if (this.recrossing || changed) (this.recrossing = false), (crosshair.index = NaN);
        this.setCrosshair(time.index(x), crosshair.y, true, x);
      }
    }
    this.crossedChanges = this.panes[crosshair.pane]?.changes ?? -1;
    // (Found again under a still pointer (the view moved beneath it): its panes' tags at its point now, in this render.)
    for (let k = 0; k < this.panes.length; k++) this.panes[k].retag(crosshair.index, time, this.theme, this.dpr);
    this.timeAxis.draw(time, this.theme, this.dpr, time.width, this.panes);
    this.crosshair.draw(time, this.panes, Math.max(0, this.height - this.timeHeight), this.left, this.theme, this.dpr, this.timeAxis, this.separatorLit);
    // (Each legend pointed at the series read: the page's, crosshair or not (a tag, a row of its own, the pointer moved
    // onto the legend); else the one a pointer's on, none with the crosshair gone. Its items made here read at its
    // point, in this frame.)
    const read = crosshair.pointed !== undefined ? crosshair.pointed : crosshair.index === crosshair.index ? crosshair.hit : null;
    for (const pane of this.panes) if (pane.legend) pane.legend.point(pane.legend.itemOf(read)), pane.legend.read(crosshair.index, time, (series, v) => pane.words(series, v, time));
    for (let k = 0; k < this.panes.length; k++) this.panes[k].drawOver(time, this.dpr);
    this.placed = false;
    if (time.from !== this.drawnFrom || time.to !== this.drawnTo) {
      (this.drawnFrom = time.from), (this.drawnTo = time.to);
      const listeners = this.rangeListeners;
      for (let k = 0; k < listeners.length; k++) listeners[k](time.from, time.to);
    }
  }

  // Pointers: one drags (a pan, gliding on once let go), two pinch (a zoom about their middle, and a pan as it moves);
  // the wheel zooms about the pointer, a trackpad's sideways scroll pans (a diagonal swipe both). On an axis, a drag scales it: a price axis
  // about its middle (its range then set by hand, fitted again by a double-click, its width too: see fitAxes), the time axis about the plot's right
  // edge. The crosshair: where a mouse is; a finger
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
    const element = this.element, time = this.time, down = /** @type {Map<number, { x: number, y: number, startX: number, startY: number, panning: boolean, moved: boolean, axis: Axis | null, pane: Pane | null }>} */ (new Map());
    let held = -1;
    // (Two fingers down in this gesture: its end no click.)
    let twoFingers = false;
    // (A separator dragged: the panes over and under it, the pointer dragging it (-1: none).)
    let sizing = /** @type {[Pane, Pane] | null} */ (null), sizer = -1;
    /** Let go (`y`: a mouse's height in the chart, its separator lit still where it's over one). @param {number} [y] */
    const stopSizing = (y = NaN) => {
      (sizing = null), (sizer = -1), (this.resizing = false), (this.separatorLit = this.panes[separator(y, false)] ?? null);
      this.invalidate(true);
    };
    /** The separator under a height in the chart (the pane over it), or -1. @param {number} y @param {boolean} touch */
    const separator = (y, touch) => {
      for (let k = 0; k < this.panes.length - 1; k++) if (Math.abs(this.panes[k].top + this.panes[k].height - y) <= (touch ? NEAR_TOUCH : NEAR_MOUSE)) return k;
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
    // (The axis a drag from a point would scale, found without making anything (a mouse's hover asks at each move): its
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
    // (A drag's last moves: its speed as it's let go (see FLICK_SPAN).)
    const trail = new Float64Array(16), trailAt = new Float64Array(16);
    let trailEnd = 0, trailSize = 0;
    const plotX = (/** @type {number} */ clientX) => clientX - element.getBoundingClientRect().left - this.left;
    const signal = this.listening.signal;
    element.addEventListener("pointerdown", (event) => {
      // (A separator dragged: no other pointer does anything meanwhile. A press beginning on a page's own element in the
      // chart (a legend's button) is the page's: none of its canvases (wherever a page put them), not the chart's; one
      // more during the chart's own (a second finger: a pinch), the chart's wherever it lands.)
      if (event.button || sizer >= 0 || (!down.size && event.target !== element && !this.#owns(event.target))) return;
      element.setPointerCapture(event.pointerId);
      // (On a separator: it's dragged, nothing else.)
      const y = event.clientY - element.getBoundingClientRect().top, on = down.size ? -1 : separator(y, event.pointerType === "touch");
      if (on >= 0) return void ((sizing = [this.panes[on], this.panes[on + 1]]), (sizer = event.pointerId), (this.resizing = true), (this.separatorLit = this.panes[on]), (this.glide.speed = 0), this.setCrosshair(NaN));
      const touch = event.pointerType === "touch", box = element.getBoundingClientRect();
      down.set(event.pointerId, { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, panning: !touch, moved: false, axis: down.size ? null : axisAt(event.clientX - box.left, y), pane: this.panes.find((pane) => y >= pane.top && y < pane.top + pane.height) ?? null });
      this.glide.speed = 0;
      trailSize = 0;
      clearTimeout(this.holding);
      // (A held finger's crosshair going at the next frame: gone now.)
      if (this.letting) cancelAnimationFrame(this.letting), this.setCrosshair(NaN);
      this.holding = this.letting = 0;
      // (A second finger: a pinch, the crosshair a held finger showed gone.)
      if (held >= 0) (held = -1), this.setCrosshair(NaN);
      // (A second finger: a pinch, no axis dragged after it, the one finger left panning.)
      if (down.size > 1) {
        twoFingers = true;
        for (const each of down.values()) each.axis = null;
      }
      if (touch && down.size === 1 && !down.get(event.pointerId)?.axis) {
        const id = event.pointerId;
        this.holding = setTimeout(() => {
          const at = down.get(id);
          this.holding = 0;
          if (!at || at.panning || down.size !== 1) return;
          held = id;
          const box = element.getBoundingClientRect();
          this.setCrosshair(time.index(at.x - box.left - this.left), at.y - box.top, true, at.x - box.left - this.left);
        }, HOLD);
      }
    }, { signal });
    element.addEventListener("pointermove", (event) => {
      if (sizer === event.pointerId) return void (sizing && !this.#size(sizing[0], sizing[1], event.clientY - element.getBoundingClientRect().top) && stopSizing());
      // (Another pointer meanwhile (a mouse beside a finger): nothing while a separator's dragged.)
      if (this.resizing) return;
      const at = down.get(event.pointerId);
      if (!at && event.pointerType === "mouse") {
        // (Over a page's own element over it (a legend): no crosshair, as off the chart.)
        if (event.target !== element && !this.#owns(event.target)) return void (this.separatorLit && ((this.separatorLit = null), this.invalidate()), this.setCrosshair(NaN));
        // (A mouse over a separator, or an axis: it shows it can be dragged, no crosshair over a separator.)
        const box = element.getBoundingClientRect(), y = event.clientY - box.top, lit = separator(y, false), over = lit >= 0, axis = !over && findAxis(event.clientX - box.left, y);
        element.style.cursor = over ? "row-resize" : axis ? (foundPane ? "ns-resize" : "ew-resize") : "";
        // (The separator pointed at in the theme's active color: it can be dragged.)
        if (this.panes[lit] !== (this.separatorLit ?? undefined)) (this.separatorLit = this.panes[lit] ?? null), this.invalidate();
        return void (over ? this.setCrosshair(NaN) : point(event));
      }
      if (!at) return;
      // (Gone as far as a click can't, once: a drag back to where it began is no click.)
      at.moved ||= Math.hypot(event.clientX - at.startX, event.clientY - at.startY) >= 6;
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
        (at.x = event.clientX), (at.y = event.clientY), (trailSize = 0);
        return void this.invalidate();
      }
      if (down.size === 1) {
        time.pan(event.clientX - at.x);
        if (at.pane) slide(at.pane, event.clientY - at.y);
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
      // (Its listeners told once the panes are fitted to the view it moved to: see render.)
      if (event.pointerType === "mouse") this.#follow(event);
      this.invalidate();
    }, { signal });
    element.addEventListener("pointerleave", (event) => event.pointerType === "mouse" && !down.size && !this.resizing && ((this.separatorLit = null), this.setCrosshair(NaN)), { signal });
    const up = (/** @type {PointerEvent} */ event) => {
      if (sizer === event.pointerId) return void ((tapAt = -Infinity), stopSizing(event.pointerType === "mouse" ? event.clientY - element.getBoundingClientRect().top : NaN));
      const at = down.get(event.pointerId);
      if (!at || !down.delete(event.pointerId)) return;
      const pinched = twoFingers;
      if (!down.size) twoFingers = false;
      clearTimeout(this.holding);
      this.holding = 0;
      // (The finger holding the crosshair let go: it goes at the next frame (a page's own lifting handled first, as a
      // touch's end came after it: Studio keeps it there); no glide.)
      if (held === event.pointerId) return void ((held = -1), (trailSize = 0), (this.letting = requestAnimationFrame(() => ((this.letting = 0), this.setCrosshair(NaN)))));
      // (Let go where it began: a click.)
      if (event.type === "pointerup" && !down.size && !pinched && !at.moved && Math.hypot(event.clientX - at.startX, event.clientY - at.startY) < 6) {
        trailSize = 0;
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
      if (down.size || pinched || event.type === "pointercancel" || event.pointerType !== "touch" || trailSize < 2) return void (trailSize = 0);
      // (Its speed as it was let go: over its last moves alone (a finger slowing to a stop lets go slow), and only if it
      // was still moving then (held still a moment, none); too slow, none.)
      const last = (trailEnd + trail.length - 1) % trail.length;
      let first = last;
      for (let k = 1; k < trailSize; k++) {
        const i = (last - k + trail.length) % trail.length;
        if (trailAt[last] - trailAt[i] > FLICK_SPAN) break;
        first = i;
      }
      const ms = trailAt[last] - trailAt[first], speed = ms > 0 ? (trail[last] - trail[first]) / ms : 0, stale = event.timeStamp - trailAt[last] >= FLICK_SPAN;
      trailSize = 0;
      if (!stale && Math.abs(speed) >= FLICK_LEAST) this.#fling(speed);
    };
    element.addEventListener("pointerup", up, { signal });
    element.addEventListener("lostpointercapture", (event) => event.pointerId === sizer && stopSizing(), { signal });
    element.addEventListener("pointercancel", up, { signal });
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? this.height : 1;
      // (A pinch zooms alone; a scroll both at once, a trackpad's diagonal swipe panning and zooming together.)
      if (event.deltaY) time.zoom(plotX(event.clientX), Math.exp(event.deltaY * scale * (event.ctrlKey ? 0.01 : 0.001)));
      if (event.deltaX && !event.ctrlKey) time.pan(-event.deltaX * scale);
      this.glide.speed = 0;
      this.invalidate();
    }, { passive: false, signal });
  }

  /**
   * The line whose tag is at a point (CSS pixels from its element's corner: on an axis, as last drawn; not one hidden
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

  /**
   * A flick let go at `speed` (CSS pixels a millisecond): it glides on, slowing by SLOWING a millisecond (its travel `speed /
   * -ln(SLOWING)`); where that would pass an end, slowing faster to come to rest there (held at it, it stopped dead).
   * @param {number} speed
   */
  #fling(speed) {
    const glide = this.glide, room = this.time.room(speed), passes = Math.abs(speed) / -Math.log(SLOWING) > room;
    glide.slowing = passes ? Math.exp(-Math.abs(speed) / Math.max(room, 1e-9)) : SLOWING;
    glide.left = passes ? room : Infinity;
    glide.speed = speed;
    glide.at = performance.now();
    this.invalidate();
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

  /**
   * The separator between `over` and `under` dragged to `y` (CSS pixels in the chart): their shares moved between them,
   * their total kept, neither under LEAST; laid out and drawn at their new heights at each move (what's dragged is what
   * shows: no picture stretched). Whether they're still next to each other (a pane removed: the drag's over).
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
    cancelAnimationFrame(this.letting);
    clearTimeout(this.holding);
    this.frame = this.holding = this.letting = 0;
    this.glide.speed = 0;
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
