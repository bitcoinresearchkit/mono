// @ts-check
// The part of Lightweight Charts' API Studio uses, on bitview-charts: Studio's copy imports this in the chart library's
// place, its own code as it is, to try the library in the real app. Studio's glue, not the library's: what Studio does
// with the library (its options, its patches' calls: bv*), mapped onto bitview-charts' panes, scales and series. Phase
// one: the chart, its panes and scales, the view, the crosshair and clicks, pictures, and the standard series (lines,
// areas (stacked ones' fills to the line under them), histograms, baselines, candles), rows or columns onto one
// timeline. Phase two: custom series (Studio's clouds, heatmaps, bubbles: see Painted), overlay scales, its scales'
// and series' writers, primitives (its drawings, halving marks: their pane views, see Pane.paint) and markers (its
// signals' marks: see Marks). Not yet: primitives' axis tags, price lines of its own.

import { createChart as make, DARK, picture, compact, Line, Band, Bars, Candles, Custom, Scale } from "/modules/bitview-charts/index.js";

export const LineSeries = "Line", AreaSeries = "Area", HistogramSeries = "Histogram", BaselineSeries = "Baseline", CandlestickSeries = "Candlestick";
export const LineStyle = { Solid: 0, Dotted: 1, Dashed: 2, LargeDashed: 3, SparseDotted: 4 };
export const LineType = { Simple: 0, WithSteps: 1, Curved: 2 };
export const ColorType = { Solid: "solid", VerticalGradient: "gradient" };
export const CrosshairMode = { Normal: 0, Magnet: 1, Hidden: 2, MagnetOHLC: 3 };
export const TrackingModeExitMode = { OnTouchEnd: 0, OnNextTap: 1 };
export const MismatchDirection = { NearestLeft: -1, None: 0, NearestRight: 1 };
export const TickMarkType = { Year: 0, Month: 1, DayOfMonth: 2, Time: 3, TimeWithSeconds: 4 };
export const customSeriesDefaultOptions = { color: "#2196f3", visible: true, lastValueVisible: true, priceLineVisible: true, priceScaleId: "right" };

/** Markers (a plugin of the library's): a series' marks at dates, drawn over its pane's series (see Marks). @param {Series} series @param {any[]} [markers] */
export function createSeriesMarkers(series, markers = []) {
  return new Marks(series, markers);
}

/** @param {HTMLElement} element @param {any} [options] */
export function createChart(element, options = {}) {
  return new Chart(element, options);
}

const MODES = ["linear", "log", "percent", "indexed"];
// (What an autoscaleInfoProvider's `original` gives it here: a sign it kept the series' own range.)
const OWN = { priceRange: null }, original = () => OWN;

/** `b` merged into `a`, the objects in them too (as the chart library merged its options). @param {any} a @param {any} b @returns {any} */
function merged(a, b) {
  const out = { ...a };
  for (const [key, value] of Object.entries(b ?? {})) out[key] = value && typeof value === "object" && !Array.isArray(value) && a?.[key] && typeof a[key] === "object" ? merged(a[key], value) : value;
  return out;
}

/** Whether two arrays of times are the same, point for point. @param {Float64Array} a @param {Float64Array} b */
function sameTimes(a, b) {
  if (a === b) return true;
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}

/** The index of `time` in rising `times`, or -1 (exact), or the nearest one each way (`mismatch`). @param {Float64Array | number[]} times @param {number} time @param {number} [mismatch] */
function indexOf(times, time, mismatch = 0) {
  let lo = 0, hi = times.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    times[mid] < time ? (lo = mid + 1) : (hi = mid);
  }
  if (times[lo] === time) return lo;
  if (mismatch > 0) return lo < times.length ? lo : -1;
  if (mismatch < 0) return lo - 1;
  return -1;
}

class Chart {
  /** @param {HTMLElement} element @param {any} options */
  constructor(element, options) {
    // (Its own element in the page's box, as the chart library made one: Studio sizes and pins it while a motion stretches
    // its picture, its box laid out as it goes. Sized as asked, else as the box.)
    this.box = element;
    this.element = document.createElement("div");
    Object.assign(this.element.style, { width: options.width ? `${options.width}px` : "100%", height: options.height ? `${options.height}px` : "100%" });
    element.append(this.element);
    this.options = merged({}, options);
    this.engine = make(this.element, { theme: this.#theme(this.options), rightOffset: this.options.timeScale?.rightOffsetPixels ?? 48, dateFormat: this.options.localization?.timeFormatter });
    /** @type {Pane[]} */
    this.panesList = [];
    /** @type {Series[]} */
    this.all = [];
    /** The timeline: every series' times as one (Studio's are one already, mostly). @type {Float64Array} */
    this.times = new Float64Array(0);
    this.dirty = false;
    /** @type {((param: any) => void)[]} */
    this.crosshairListeners = [];
    /** @type {((param: any) => void)[]} */
    this.clickListeners = [];
    // (Each with the series under the point, as the chart library's: a pointer's moves (Studio lights a line's item) and
    // clicks (it selects its layer).)
    this.engine.onCrosshair((index, pane, y, input, x) => {
      const engine = this.engine.panes[pane], hit = input && engine && index === index ? engine.hit(this.engine.time, index, x, y) : -1;
      const param = { ...this.#param(index, pane, y, input), hoveredSeries: this.#owner(pane, hit) };
      for (const listener of this.crosshairListeners) listener(param);
    });
    this.engine.onClick((index, pane, hit, y) => {
      const param = { ...this.#param(index, pane, y, true), hoveredSeries: this.#owner(pane, hit) };
      for (const listener of this.clickListeners) listener(param);
    });
    this.timeScaleApi = new TimeScale(this);
    // (Its DOM as the chart library's was, where Studio looks for it: a table of rows, each pane's (see Pane) and, last,
    // the time axis' (its right corner's cell Studio's Full screen button's, the one cell taking the pointer), kept on
    // them as it's laid out; the library's canvases in their cells (Studio stretches the plots' and the time axis' as
    // its panel slides), none taking the pointer (the chart's element takes it).)
    this.table = document.createElement("table");
    this.table.style.cssText = "position: absolute; inset: 0; display: block; pointer-events: none";
    this.timeRow = row();
    /** @type {HTMLElement} */ (this.timeRow.children[2]).style.pointerEvents = "auto";
    this.timeFrame = frame();
    this.timeRow.children[1].append(this.timeFrame);
    this.table.append(this.timeRow);
    // (The crosshair's canvas after the table: Studio's magnifier draws the chart's canvases in their order, the
    // crosshair's last.)
    this.element.append(this.table, this.engine.crosshair.canvas);
    this.engine.onLayout(() => {
      const engine = this.engine;
      for (const pane of this.panesList) pane.place();
      placeRow(this.timeRow, engine.height - engine.timeHeight, engine.timeHeight, engine.left, engine.time.width, engine.axis);
      mount(engine.timeAxis.canvas, this.timeFrame);
    });
  }

  /** The library's theme from LWC's options: its colors, its font, its crosshair's and separators'. @param {any} options */
  #theme(options) {
    const layout = options.layout ?? {}, grid = options.grid?.horzLines ?? {}, cross = options.crosshair?.vertLine ?? {}, across = options.crosshair?.horzLine ?? {};
    return {
      ...DARK,
      // (A transparent one (Studio's) the page's ground under it: the library's canvases are opaque.)
      background: layout.background?.color && layout.background.color !== "transparent" ? layout.background.color : this.#ground(),
      ...(layout.textColor ? { text: layout.textColor } : {}),
      ...(grid.color ? { grid: grid.color } : {}),
      ...(cross.color ? { crosshair: cross.color } : {}),
      ...(across.color ? { across: across.color } : {}),
      ...(cross.labelBackgroundColor ? { badge: cross.labelBackgroundColor } : {}),
      ...(layout.panes?.separatorColor ? { separator: layout.panes.separatorColor } : {}),
      font: `${layout.fontSize ?? 11}px ${layout.fontFamily ?? "system-ui, sans-serif"}`,
    };
  }

  /**
   * The page's ground under the chart as it shows: its box's and its ancestors' backgrounds up to the first one that
   * isn't see-through, laid over each other (a translucent card over the page's ground: the color it makes).
   */
  #ground() {
    /** @type {string[]} */
    const layers = [];
    for (let element = /** @type {HTMLElement | null} */ (this.box); element; element = element.parentElement) {
      const color = getComputedStyle(element).backgroundColor, alpha = alphaOf(color);
      if (!color || !alpha) continue;
      layers.push(color);
      if (alpha >= 1) break;
    }
    if (!layers.length) return DARK.background;
    const ground = alphaOf(layers[layers.length - 1]) >= 1;
    if (layers.length === 1 && ground) return layers[0];
    // (Laid over each other on a pixel of a canvas, from the page's ground in (none opaque: the library's dark one);
    // that pixel read back.)
    const g = (ink ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d", { willReadFrequently: true })));
    g.globalCompositeOperation = "copy";
    g.fillStyle = ground ? layers[layers.length - 1] : DARK.background;
    g.fillRect(0, 0, 1, 1);
    g.globalCompositeOperation = "source-over";
    for (let k = layers.length - (ground ? 2 : 1); k >= 0; k--) (g.fillStyle = layers[k]), g.fillRect(0, 0, 1, 1);
    const [r, gr, b] = g.getImageData(0, 0, 1, 1).data;
    return `rgb(${r}, ${gr}, ${b})`;
  }

  /** A pane's percent scales written as the chart's localization writes percents (none: the library's). @param {Pane} pane */
  #percents(pane) {
    const local = this.options.localization ?? {};
    if (local.percentageFormatter) pane.engine.setPercentFormat(local.percentageFormatter, local.tickmarksPercentageFormatter ?? null);
  }

  /** The series (Studio's) that drew a pane's library series `hit` (-1: none): its own, or its edge. @param {number} pane @param {number} hit */
  #owner(pane, hit) {
    const drawn = hit >= 0 ? this.engine.panes[pane]?.series[hit] : null;
    return drawn ? this.all.find((each) => each.engine === drawn || each.edge === drawn) : undefined;
  }

  /** An LWC crosshair or click event. @param {number} index @param {number} pane @param {number} y @param {boolean} input */
  #param(index, pane, y, input) {
    if (index !== index) return { point: undefined, time: undefined, logical: undefined, paneIndex: undefined, seriesData: new Map(), sourceEvent: input ? {} : undefined };
    const x = this.engine.time.x(index);
    return { point: { x, y }, time: this.times[index], logical: index, paneIndex: pane >= 0 ? pane : undefined, seriesData: new Map(), sourceEvent: input ? {} : undefined };
  }

  // ── What Studio calls ──

  /** @param {any} options */
  applyOptions(options) {
    this.options = merged(this.options, options);
    if (options.layout || options.grid || options.crosshair) this.engine.theme = this.#theme(this.options);
    // (Its halo come or changed: the outlined series edged in it.)
    if (options.layout && "bvHalo" in options.layout) for (const series of this.all) series.restyle();
    if (options.localization) for (const pane of this.panesList) this.#percents(pane);
    if (options.localization?.timeFormatter) this.engine.crosshair.setFormat(options.localization.timeFormatter);
    if (options.timeScale?.rightOffsetPixels !== undefined) this.engine.time.rightOffset = options.timeScale.rightOffsetPixels;
    this.engine.invalidate(true);
  }

  timeScale() {
    return this.timeScaleApi;
  }

  /** @param {string} id @param {number} [pane] */
  priceScale(id, pane = 0) {
    return this.#pane(pane).scaleApi(id);
  }

  panes() {
    return this.panesList;
  }

  chartElement() {
    return this.element;
  }

  /** @param {number} width @param {number} height */
  resize(width, height) {
    Object.assign(this.element.style, { width: `${width}px`, height: `${height}px` });
    this.engine.layout();
  }

  /** @param {any} type @param {any} options @param {number} [pane] */
  addSeries(type, options = {}, pane = 0) {
    const series = new Series(this, String(type), options, this.#pane(pane));
    this.all.push(series);
    return series;
  }

  /** @param {any} view @param {any} options @param {number} [pane] */
  addCustomSeries(view, options = {}, pane = 0) {
    const series = new Series(this, "Custom", { ...customSeriesDefaultOptions, ...view.defaultOptions?.(), ...options }, this.#pane(pane), view);
    this.all.push(series);
    return series;
  }

  /** @param {Series} series */
  removeSeries(series) {
    series.pane.engine.remove(series.engine);
    if (series.edge) series.pane.engine.remove(series.edge);
    if (series.beside) series.pane.engine.remove(series.beside);
    if (series.across) series.pane.engine.remove(series.across);
    for (const primitive of series.primitives) primitive.detached?.();
    for (const marks of series.pane.marks.filter((each) => each.series === series)) marks.detach();
    series.pane.list = series.pane.list.filter((each) => each !== series);
    series.pane.dirty = true;
    this.all = this.all.filter((each) => each !== series);
    this.#trimPanes();
    this.changed();
  }

  /** @param {(param: any) => void} listener */
  subscribeCrosshairMove(listener) {
    this.crosshairListeners.push(listener);
  }

  /** @param {(param: any) => void} listener */
  unsubscribeCrosshairMove(listener) {
    this.crosshairListeners = this.crosshairListeners.filter((each) => each !== listener);
  }

  /** @param {(param: any) => void} listener */
  subscribeClick(listener) {
    this.clickListeners.push(listener);
  }

  /** @param {number} price @param {number} time @param {Series} series */
  setCrosshairPosition(price, time, series) {
    const i = indexOf(this.times, time, 1), y = series.priceToCoordinate(price);
    this.engine.setCrosshair(i, y === null ? NaN : series.pane.engine.top + y);
  }

  clearCrosshairPosition() {
    this.engine.setCrosshair(NaN);
  }

  takeScreenshot() {
    this.#sync();
    return picture(this.engine);
  }

  /** The patches' flush: what's pending drawn now. */
  bvFlush() {
    this.#sync();
    this.engine.render(true);
  }

  /** @param {string} type @param {any} listener */
  addEventListener(type, listener) {
    this.element.addEventListener(type, listener);
  }

  remove() {
    this.engine.destroy();
    this.element.remove();
  }

  // ── Its panes and timeline ──

  /** The pane at `index`, made (and the ones before it) if it's not there. @param {number} index */
  #pane(index) {
    while (this.panesList.length <= index) {
      const pane = new Pane(this, this.engine.addPane({ scale: { top: 0.17, bottom: 0.08 }, left: { top: 0.17, bottom: 0.08 } }));
      this.#percents(pane);
      this.panesList.push(pane);
      pane.place();
    }
    return this.panesList[index];
  }

  /** Panes left with no series at the end go (as the library's did). */
  #trimPanes() {
    while (this.panesList.length > 1 && !this.panesList.at(-1)?.list.length) {
      const pane = /** @type {Pane} */ (this.panesList.pop());
      this.engine.removePane(pane.engine);
      pane.row.remove();
    }
  }

  /** A series' data or look changed: brought to the library before the next frame. */
  changed() {
    if (this.dirty) return;
    this.dirty = true;
    queueMicrotask(() => this.#sync());
  }

  /** Every series' data onto one timeline, each one's columns on it, stacks' fills to the line under them; drawn whole. */
  #sync() {
    if (!this.dirty) return;
    this.dirty = false;
    // (The timeline: Studio's series share one array of times, mostly; else, all their times as one.)
    const lists = this.all.map((series) => series.times).filter((times) => times.length);
    let times = lists[0] ?? new Float64Array(0);
    if (lists.some((list) => !sameTimes(list, times))) {
      const union = new Set();
      for (const list of lists) for (const t of list) union.add(t);
      times = Float64Array.from([...union].sort((a, b) => a - b));
    }
    // (Each series changed pushed (all of them on a new timeline: a live price's point pushes the price alone); a
    // pane's stack set again where one of its areas was. The same times, whatever array holds them, the same timeline.)
    const was = this.times, fresh = !sameTimes(times, was);
    if (fresh) this.times = times;
    else times = was;
    for (const series of this.all) if (fresh || series.dirty) (series.dirty = false), series.push(times), series.pane.dirty = true;
    if (fresh) for (const pane of this.panesList) for (const marks of pane.marks) marks.place();
    for (const pane of this.panesList) if (pane.dirty) (pane.dirty = false), pane.arrange(), pane.stack(), pane.formats();
    // (A new timeline draws everything whole; else only the panes whose series changed (their versions) draw anew.)
    if (fresh) this.engine.setTimes(times);
    else this.engine.invalidate();
  }
}

class TimeScale {
  /** @param {Chart} chart */
  constructor(chart) {
    this.chart = chart;
  }

  get time() {
    return this.chart.engine.time;
  }

  width() {
    return this.time.width;
  }

  height() {
    return this.chart.engine.timeHeight;
  }

  options() {
    return { rightOffsetPixels: this.time.rightOffset, minBarSpacing: this.chart.options.timeScale?.minBarSpacing ?? 0.5, barSpacing: this.time.spacing, ...this.chart.options.timeScale };
  }

  /** @param {any} options */
  applyOptions(options) {
    this.chart.applyOptions({ timeScale: options });
  }

  getVisibleLogicalRange() {
    return this.time.length ? { from: this.time.from, to: this.time.to } : null;
  }

  /** @param {{ from: number, to: number }} range */
  setVisibleLogicalRange(range) {
    this.chart.engine.setRange(range.from, range.to);
  }

  getVisibleRange() {
    const times = this.chart.times, n = times.length;
    if (!n) return null;
    const at = (/** @type {number} */ i) => times[Math.min(Math.max(Math.round(i), 0), n - 1)];
    return { from: at(this.time.from), to: at(this.time.to) };
  }

  /** @param {{ from: number, to: number }} range */
  setVisibleRange(range) {
    const times = this.chart.times;
    this.chart.engine.setRange(indexOf(times, range.from, 1), indexOf(times, range.to, -1));
  }

  /** @param {number} time */
  timeToCoordinate(time) {
    const i = indexOf(this.chart.times, time);
    return i < 0 ? null : this.time.x(i);
  }

  /** @param {number} time @param {boolean} [nearest] */
  timeToIndex(time, nearest = false) {
    const i = indexOf(this.chart.times, time, nearest ? -1 : 0);
    return i < 0 ? null : i;
  }

  /** @param {number} logical */
  logicalToCoordinate(logical) {
    return this.time.x(logical);
  }

  /** @param {number} x */
  coordinateToLogical(x) {
    return this.time.index(x);
  }

  /** How far the view's right end is past the last point, in points. */
  scrollPosition() {
    return this.time.to - (this.time.length - 1);
  }

  fitContent() {
    this.chart.engine.show(this.time.length);
  }

  /** @param {(range: any) => void} listener */
  subscribeVisibleLogicalRangeChange(listener) {
    this.chart.engine.onRange((from, to) => listener({ from, to }));
  }

  /** @param {(width: number, height: number) => void} listener */
  subscribeSizeChange(listener) {
    new ResizeObserver(() => listener(this.width(), this.height())).observe(this.chart.element);
  }
}

class Pane {
  /** @param {Chart} chart @param {import("/modules/bitview-charts/index.js").Pane} engine */
  constructor(chart, engine) {
    this.chart = chart;
    this.engine = engine;
    /** @type {Series[]} */
    this.list = [];
    /** @type {Record<string, PriceScale>} */
    this.scales = {};
    /** A series of its pushed since its stack was set. */
    this.dirty = false;
    /** Each side's priceFormat as last given the library. @type {Record<string, any>} */
    this.written = {};
    /**
     * Its series' primitives' views under every series, and over everything (the crosshair too: the library's layer
     * over the pane, drawn whole each frame, as their labels and handles keep to the plot), once one's attached.
     * @type {Custom | null}
     */
    this.under = null;
    /** @type {Custom | null} */
    this.over = null;
    /** Its primitives told the chart moved this frame (see fresh). */
    this.updated = false;
    /** Its series' markers. @type {Marks[]} */
    this.marks = [];
    // (Its row in the chart's table (its box: getHTMLElement), over it: its plot's cell a frame over the plot, first in
    // it (its canvases in it, its legend put in there after them), its axes' cells theirs.)
    this.row = row();
    this.frame = frame();
    this.leftFrame = frame();
    this.row.children[0].append(this.leftFrame);
    this.row.children[1].append(this.frame);
    chart.table.insertBefore(this.row, chart.timeRow);
    this.target = new Target();
  }

  /** Its layers for its series' primitives' views: under every series, over everything (each series' own, just over it). */
  layers() {
    const pane = this.engine, first = () => 1;
    this.under ??= pane.add(new Custom({ draw: (g, at) => this.paint(g, at, "bottom", null), first }));
    this.over ??= pane.add(new Custom({ draw: (g, at) => this.paint(g, at, "top", null), first, axisTags: (side, add) => this.axisTags(side, add), timeTags: (add) => this.timeTags(add) }, { over: true }));
  }

  /**
   * Its series' primitives' views of one z-order drawn (Studio's drawings, its halving marks: the chart library's
   * primitives): `owner`'s alone ("normal": just over it), or all of the pane's ("bottom", under every series, drawn
   * first; "top", over everything, drawn last). Each primitive told the chart moved first (see fresh): by the layer
   * under the series wherever the pane's drawn (each strip), else by whatever reads them first.
   * @param {CanvasRenderingContext2D} g @param {import("/modules/bitview-charts/custom.js").View} at @param {string} z @param {Series | null} owner
   */
  paint(g, at, z, owner) {
    const target = this.target.set(g, at.time.width, at.scale.height, at.dpr);
    if (z === "bottom") (this.updated = false), this.fresh();
    else if (z === "top") this.fresh(), (this.updated = false);
    for (let k = 0; k < this.list.length; k++) {
      const series = this.list[k];
      // (A hidden series' primitives drawn all the same, as the chart library drew them: Studio's drawings on a price
      // layer ride its candles, hidden while they're too thin to read; Studio leaves out what it hides itself.)
      if (owner && series !== owner) continue;
      for (let p = 0; p < series.primitives.length; p++) {
        const primitive = series.primitives[p];
        const views = primitive.paneViews?.() ?? NONE;
        for (let v = 0; v < views.length; v++) if ((views[v].zOrder?.() ?? "normal") === z) rendererOf(views[v])?.draw(target);
      }
    }
  }

  /**
   * Its series' primitives told the chart moved (their updateAllViews), once a frame: by the first of its drawing to read
   * them (its layer under the series where the pane's drawn, else the time axis' tags or its layer over everything),
   * the layer over everything last (the next frame's anew).
   */
  fresh() {
    if (this.updated) return;
    this.updated = true;
    for (const series of this.list) for (const primitive of series.primitives) primitive.updateAllViews?.();
  }

  /**
   * Its series' primitives' tags on its axis on `side` (0, the right; 1, the left): their price axis views (a drawing's
   * value), each on its series' side (a hidden one's too, as its views are drawn).
   * @param {number} side @param {(y: number, text: string, color: string, ink?: string) => void} add
   */
  axisTags(side, add) {
    this.fresh();
    for (const series of this.list)
      if ((series.side === "left" ? 1 : 0) === side) for (const primitive of series.primitives) for (const view of primitive.priceAxisViews?.() ?? NONE) if (view.visible?.() !== false) add(view.coordinate(), view.text(), view.backColor(), view.textColor?.());
  }

  /** Its series' primitives' tags on the time axis: their time axis views (a drawing's date). @param {(x: number, text: string, color: string, ink?: string) => void} add */
  timeTags(add) {
    this.fresh();
    for (const series of this.list) for (const primitive of series.primitives) for (const view of primitive.timeAxisViews?.() ?? NONE) if (view.visible?.() !== false) add(view.coordinate(), view.text(), view.backColor(), view.textColor?.());
  }

  /**
   * Its series in the library in their order (Studio's), each with its edge and its own primitives' layer over it; its
   * markers over them all ("aboveSeries"); its primitives' layers under and over everything.
   */
  arrange() {
    /** @type {any[]} */
    const list = [];
    if (this.under) list.push(this.under);
    for (const series of this.list) {
      list.push(series.engine);
      if (series.edge) list.push(series.edge);
      if (series.across) list.push(series.across);
      if (series.beside) list.push(series.beside);
    }
    for (const marks of this.marks) list.push(marks.engine);
    if (this.over) list.push(this.over);
    this.engine.order(list);
  }

  /** Its scale of that id: its right one, its left one, or an overlay (any other id: its series' own, unseen). @param {string} id */
  scaleApi(id) {
    return (this.scales[id] ??= new PriceScale(this, id));
  }

  paneIndex() {
    return this.chart.panesList.indexOf(this);
  }

  getHTMLElement() {
    return this.row;
  }

  /** Its row over its pane, the library's canvases of it in its cells (its plot's first in its frame, its layer over it next). */
  place() {
    const engine = this.engine;
    placeRow(this.row, engine.top, engine.height, this.chart.engine.left, engine.plot, this.chart.engine.axis);
    mount(engine.canvas, this.frame, true);
    if (engine.overCanvas) mount(engine.overCanvas, this.frame, true), engine.canvas.after(engine.overCanvas);
    mount(engine.axisCanvas, /** @type {HTMLElement} */ (this.row.children[2]));
    if (engine.leftCanvas) mount(engine.leftCanvas, this.leftFrame);
  }

  getHeight() {
    return this.engine.height;
  }

  getStretchFactor() {
    return this.engine.share;
  }

  /** @param {number} share */
  setStretchFactor(share) {
    this.engine.share = share;
    this.chart.engine.layout();
  }

  getSeries() {
    return this.list;
  }

  /**
   * Its sides' writers, as the chart library's were: each scale's its first series' (its priceFormat's, a value's and its
   * ticks'); none, the library's own.
   */
  formats() {
    for (const side of /** @type {const} */ (["right", "left"])) {
      const format = this.list.find((series) => series.opts.priceScaleId === side)?.opts.priceFormat, was = this.written[side];
      if (was === format) continue;
      this.written[side] = format;
      this.engine.setFormat(format?.formatter ?? compact, format?.tickmarksFormatter ?? null, side);
    }
  }

  /**
   * Its stacked areas' fills each down to the stacked line under it on its side (the library's patch 41's bands), from
   * the stack's foot (Studio orders it last) up; the foot's an area to zero.
   */
  stack() {
    /** @type {Record<string, Float64Array | undefined>} */
    const under = {};
    for (let k = this.list.length - 1; k >= 0; k--) {
      const series = this.list[k], side = series.opts.bvBandFill;
      if (!side || !(series.engine instanceof Band)) continue;
      series.engine.set({ lower: under[side] ?? null, upper: series.columns, base: 0 });
      under[side] = series.columns;
    }
  }
}

class PriceScale {
  /** @param {Pane} pane @param {string} id */
  constructor(pane, id) {
    this.pane = pane;
    /** Its side; an overlay's, none (its series drawn on the right one's side, its own scale theirs). @type {"right" | "left" | null} */
    this.side = id === "left" || id === "right" ? id : null;
    this.opts = { mode: 0, autoScale: true, visible: true, scaleMargins: this.side ? { top: 0.17, bottom: 0.08 } : { top: 0.2, bottom: 0.1 } };
    /** An overlay's own scale: its series' range (their autoscaleInfoProvider's), the pane's height. */
    this.overlay = this.side ? null : new Scale({ top: 0.2, bottom: 0.1 });
  }

  /** The library's scale: the right one, the left one once a series is on it, or an overlay's own (its pane's height). */
  get scale() {
    if (this.overlay) return (this.overlay.height = this.pane.engine.height), this.overlay;
    return this.side === "left" ? this.pane.engine.left : this.pane.engine.scale;
  }

  options() {
    return this.opts;
  }

  /** @param {any} options */
  applyOptions(options) {
    this.opts = { ...this.opts, ...options };
    const scale = this.scale;
    if (scale) {
      if (options.mode !== undefined) scale.mode = /** @type {any} */ (MODES[options.mode] ?? "linear");
      if (options.scaleMargins) (scale.top = options.scaleMargins.top), (scale.bottom = options.scaleMargins.bottom);
    }
    // (Fitted again, or held where it is.)
    // (Its tags under Studio's legend: LWC patch 50.)
    if (this.side && options.bvTagsTop !== undefined) this.pane.engine.setTagsTop(options.bvTagsTop, this.side);
    if (this.side) {
      if (options.autoScale) this.pane.engine.setRange(undefined, undefined, this.side);
      else if (options.autoScale === false && scale) this.pane.engine.setRange(scale.lo, scale.hi, this.side);
    }
    this.pane.chart.engine.invalidate(true);
  }

  /** @param {boolean} on */
  setAutoScale(on) {
    this.applyOptions({ autoScale: on });
  }

  width() {
    return this.side === "left" ? this.pane.chart.engine.left : this.side ? this.pane.chart.engine.axis : 0;
  }

  /** The patches' range: what it shows, as values. */
  bvRange() {
    const scale = this.scale;
    return scale ? { from: scale.lo, to: scale.hi } : null;
  }

  /** @param {{ from: number, to: number }} range */
  bvSetRange(range) {
    if (this.side) this.pane.engine.setRange(range.from, range.to, this.side);
    else this.overlay?.fit(range.from, range.to);
    this.pane.chart.engine.invalidate(true);
  }

  /** What it'd show fitted to its series (an overlay, as it is: its series' own range). */
  bvAutoRange() {
    const engine = this.pane.engine, auto = this.side === "left" ? engine.leftAuto : engine.auto, scale = this.scale;
    if (!scale) return null;
    if (!this.side) return { from: scale.lo, to: scale.hi };
    const was = { lo: scale.lo, hi: scale.hi };
    this.side === "left" ? (engine.leftAuto = true) : (engine.auto = true);
    engine.fit(this.pane.chart.engine.time);
    const range = { from: scale.lo, to: scale.hi };
    this.side === "left" ? (engine.leftAuto = auto) : (engine.auto = auto);
    if (!auto) scale.fit(was.lo, was.hi);
    return range;
  }
}

class Series {
  /** @param {Chart} chart @param {string} type @param {any} options @param {Pane} pane @param {any} [view] */
  constructor(chart, type, options, pane, view) {
    this.chart = chart;
    this.type = type;
    this.view = view;
    this.pane = pane;
    this.opts = { visible: true, lastValueVisible: true, priceLineVisible: true, priceScaleId: "right", ...options };
    /** Its own times and values (Studio's rows or columns), and its values on the chart's timeline (pushed at sync). */
    /** @type {Float64Array} */
    this.times = new Float64Array(0);
    /** @type {Float64Array} */
    this.values = new Float64Array(0);
    /** @type {Float64Array} */
    this.columns = new Float64Array(0);
    /** @type {any[] | null} */
    this.rows = null;
    /** Whether its rows are its own copy, and its values (Studio's arrays, given to setData or bvSetColumns, never changed). */
    this.ownRows = false;
    this.ownValues = true;
    /** Its data or look changed since it was pushed. */
    this.dirty = true;
    /** @type {any[]} */
    this.primitives = [];
    /** Its primitives' views of the "normal" z-order, drawn just over it (made once one's attached). @type {Custom | null} */
    this.beside = null;
    /** Its lines across (price lines), and the layer they're drawn on, just over it (made with the first). @type {{ opts: any, dash: number[], options: () => any, applyOptions: (more: any) => void }[]} */
    this.levels = [];
    /** @type {Custom | null} */
    this.across = null;
    /** A custom series' view, drawn as the chart library drew it. @type {Painted | null} */
    this.painted = view ? new Painted(this, view) : null;
    this.engine = this.#make();
    pane.engine.add(this.engine, this.side);
    // (An area's or a baseline's line along its edge, over its fill: its tag and its price line its.)
    /** @type {Line | null} */
    this.edge = this.type === "Area" || this.type === "Baseline" ? pane.engine.add(new Line(), this.side) : null;
    pane.list.push(this);
    this.#style();
  }

  /** How it writes a value (its priceFormat's own writer; none, its scale's). */
  get format() {
    return this.opts.priceFormat?.formatter ?? null;
  }

  /** Its side: its scale's (an overlay's series on the right one's, their own scale theirs). */
  get side() {
    return this.opts.priceScaleId === "left" ? "left" : "right";
  }

  /** Its scale's: its side's, or an overlay's. */
  get scaleApi() {
    return this.pane.scaleApi(this.opts.priceScaleId || "right");
  }

  /** Its series in the library, by its type. */
  #make() {
    switch (this.type) {
      case "Area":
      case "Baseline":
        return new Band({ upper: new Float64Array(0) });
      case "Histogram":
        return new Bars({});
      case "Candlestick":
        return new Candles({ open: new Float64Array(0), high: new Float64Array(0), low: new Float64Array(0), close: new Float64Array(0) });
      case "Custom":
        return /** @type {Painted} */ (this.painted).engine;
      default:
        return new Line();
    }
  }

  /** Its look in the library again (the chart's halo changed). */
  restyle() {
    this.#style();
  }

  /** Its look in the library, from its options; outlined (`bvOutline`, LWC patch 53), in the chart's halo. */
  #style() {
    const o = this.opts, engine = this.engine, shown = o.visible !== false, outline = (o.bvOutline && this.chart.options.layout?.bvHalo) || null;
    // (The crosshair's dot on its line, and its dots at each point, as the chart library's options give them.)
    const marker = o.crosshairMarkerVisible === false ? 0 : (o.crosshairMarkerRadius ?? 4), dots = o.pointMarkersVisible ? (o.pointMarkersRadius ?? 3) : 0;
    engine.hidden = !shown;
    if (engine instanceof Custom) return /** @type {Painted} */ (this.painted).style();
    const tag = o.lastValueVisible !== false && shown, priceLine = o.priceLineVisible !== false && shown;
    if (this.edge) {
      this.edge.hidden = !shown;
      this.edge.setStyle({ color: o.lineColor ?? o.topLineColor ?? o.color ?? this.edge.color, width: o.lineWidth ?? 1.5, tag, priceLine, stroke: o.lineVisible !== false, format: this.format, outline, marker, dots });
    }
    // (Bars' and candles' corners as Studio rounds them (cornerRadius, by the spacing: LWC patch 7), else square.)
    if (engine instanceof Bars) return void ((engine.color = o.color ?? engine.color), (engine.base = o.base ?? 0), (engine.outline = outline), (engine.radius = o.cornerRadius ?? 0), engine.version++);
    if (engine instanceof Line) return void engine.setStyle({ color: o.color ?? o.lineColor ?? engine.color, width: o.lineWidth ?? 1.5, tag, priceLine, stroke: o.lineVisible !== false, format: this.format, outline, marker, dots });
    if (engine instanceof Band) return void engine.set({ color: o.topColor ?? o.topFillColor1 ?? o.color ?? engine.color, below: this.type === "Baseline" ? o.bottomFillColor1 ?? null : null, base: this.type === "Baseline" ? o.baseValue?.price ?? 0 : 0 });
    if (engine instanceof Candles) return void engine.setStyle({ up: o.upColor, down: o.downColor, outline, radius: o.cornerRadius ?? 0 });
  }

  // ── What Studio calls ──

  seriesType() {
    return this.type;
  }

  options() {
    return this.opts;
  }

  /** @param {any} options */
  applyOptions(options) {
    this.opts = { ...this.opts, ...options };
    this.#style();
    this.dirty = true;
    this.chart.changed();
  }

  /** @param {any[]} rows */
  setData(rows) {
    this.rows = rows;
    (this.ownRows = false), (this.ownValues = true);
    this.times = Float64Array.from(rows, (row) => Number(row.time));
    this.values = Float64Array.from(rows, (row) => (row.value ?? row.close ?? NaN));
    this.dirty = true;
    this.chart.changed();
  }

  /** The patches' columns: times and values. @param {number[] | Float64Array} times @param {Float64Array | number[]} values */
  bvSetColumns(times, values) {
    this.rows = null;
    this.ownValues = false;
    this.times = times instanceof Float64Array ? times : Float64Array.from(times);
    this.values = values instanceof Float64Array ? values : Float64Array.from(values);
    this.dirty = true;
    this.chart.changed();
  }

  /** The patches' older rows put in front. @param {any[]} rows */
  bvPrepend(rows) {
    this.setData([...rows, ...(this.rows ?? Array.from(this.times, (time, i) => ({ time, value: this.values[i] })))]);
  }

  /** The last row set, or one added past it. @param {any} row */
  update(row) {
    if (this.#updateLast(row)) return;
    // (A copy: Studio's own array, given to setData, stays as it gave it.)
    const rows = this.rows ? this.rows.slice() : Array.from(this.times, (time, i) => ({ time, value: this.values[i] }));
    if (rows.length && Number(rows[rows.length - 1].time) === Number(row.time)) rows[rows.length - 1] = row;
    else rows.push(row);
    this.setData(rows);
    this.ownRows = true;
  }

  /**
   * Its last row, at the same time, set in place (a live price within its interval, each second): a line's or candles'
   * point (no colors of its own), its rows and values its own copies (made once: Studio's arrays never written), nothing
   * made again but that point; its pane alone drawn anew. Whether it was.
   * @param {any} row
   */
  #updateLast(row) {
    const n = this.times.length, engine = this.engine, last = n - 1;
    if (!n || this.dirty || Number(row.time) !== this.times[last] || row.color || this.rows?.[last]?.color || !(engine instanceof Candles || engine.constructor === Line)) return false;
    const i = indexOf(this.chart.times, this.times[last]);
    if (i < 0) return false;
    if (this.rows) {
      if (!this.ownRows) (this.rows = this.rows.slice()), (this.ownRows = true);
      this.rows[last] = row;
    } else if (!this.ownValues) (this.values = this.values.slice()), (this.ownValues = true), this.push(this.chart.times);
    const value = row.value ?? row.close ?? NaN;
    (this.values[last] = value), (this.columns[i] = value);
    if (engine instanceof Candles) engine.set(i, row.open ?? NaN, row.high ?? NaN, row.low ?? NaN, row.close ?? NaN);
    else engine.set(i, value);
    this.chart.engine.invalidate();
    return true;
  }

  /** Its values on the chart's timeline, its rows' colors and breaks with them. @param {Float64Array} times */
  push(times) {
    // (A custom series' rows its bars; its "closes" its values, its rows read by them.)
    if (this.painted) return void (this.columns = this.painted.push(times));
    const own = this.times, rows = this.rows, engine = this.engine;
    const same = sameTimes(own, times);
    const on = (/** @type {Float64Array} */ column) => (same ? column : ((out) => (own.forEach((t, k) => ((i) => i >= 0 && (out[i] = column[k]))(indexOf(times, t))), out))(new Float64Array(times.length).fill(NaN)));
    this.columns = on(this.values);
    if (engine instanceof Candles && rows) {
      const column = (/** @type {string} */ key) => on(Float64Array.from(rows, (row) => row[key] ?? NaN));
      engine.setValues({ open: column("open"), high: column("high"), low: column("low"), close: column("close") });
      return;
    }
    if (engine instanceof Band) {
      // (Stacked: its lower edge its stack's (see Pane.stack); else an area, to its base.)
      engine.set({ upper: this.columns, ...(this.opts.bvBandFill ? {} : { lower: null }) });
      const edge = /** @type {Line} */ (this.edge);
      edge.setValues(this.columns);
      // (A baseline's line in its top color over its base, its bottom one under it.)
      if (this.type === "Baseline") {
        const base = this.opts.baseValue?.price ?? 0, colors = Uint8Array.from(this.columns, (v) => (v >= base ? 0 : 1));
        edge.setStyle({ colors, palette: [this.opts.topLineColor ?? edge.color, this.opts.bottomLineColor ?? edge.color] });
      }
      return;
    }
    // (Rows with colors of their own: a palette, the series' own color first (a row without one), each point an index
    // into it; a transparent one, a break. None: no palette, no breaks, as before.)
    if (engine instanceof Line) {
      if (rows && rows.some((row) => row.color)) {
        const palette = [this.opts.color ?? this.opts.lineColor ?? engine.color], colors = new Uint8Array(times.length), breaks = new Uint8Array(times.length);
        rows.forEach((row) => {
          const i = indexOf(times, Number(row.time));
          if (i < 0) return;
          if (row.color === "transparent") breaks[i] = 1;
          else if (row.color) {
            let k = palette.indexOf(row.color);
            if (k < 0) (k = palette.length), palette.push(row.color);
            colors[i] = Math.min(k, 255);
          }
        });
        engine.setStyle({ colors, palette, breaks });
      } else if (engine.colors || engine.breaks) engine.setStyle({ colors: null, palette: [], breaks: null });
      engine.setValues(this.columns);
    }
  }

  /** Its scale (an overlay's own), read against its own first value in view where it's relative. */
  #scale() {
    const overlay = this.scaleApi.overlay;
    if (overlay) return (overlay.height = this.pane.engine.height), overlay;
    const pane = this.pane.engine, scale = pane.scaleOf(pane.series.indexOf(this.engine));
    if (scale.relative) scale.setBase(this.engine.first(Math.max(0, Math.floor(this.chart.engine.time.from))));
    return scale;
  }

  /** @param {number} price */
  priceToCoordinate(price) {
    const y = this.#scale().y(price);
    return Number.isFinite(y) ? y : null;
  }

  /** A height's price (a relative scale's own terms turned back into its price). @param {number} y */
  coordinateToPrice(y) {
    const scale = this.#scale(), v = scale.value(y);
    return scale.relative ? (v - scale.c) / scale.m : v;
  }

  priceScale() {
    return this.scaleApi;
  }

  priceFormatter() {
    const format = this.opts.priceFormat?.formatter;
    return { format: (/** @type {number} */ v) => (format ? format(v) : String(v)) };
  }

  /** The row at a point (or the nearest one each way). @param {number} index @param {number} [mismatch] */
  dataByIndex(index, mismatch = 0) {
    const times = this.chart.times, columns = this.columns;
    const has = (/** @type {number} */ k) => k >= 0 && k < columns.length && columns[k] === columns[k];
    // (From outside the timeline, a way toward it: from its nearer end.)
    let i = mismatch > 0 ? Math.max(0, Math.round(index)) : mismatch < 0 ? Math.min(columns.length - 1, Math.round(index)) : Math.round(index);
    if (!has(i)) {
      if (!mismatch) return null;
      while (i >= 0 && i < columns.length && !has(i)) i += mismatch > 0 ? 1 : -1;
      if (!has(i)) return null;
    }
    if (this.rows) {
      const k = indexOf(this.times, times[i]);
      if (k >= 0) return this.rows[k];
    }
    return { time: times[i], value: columns[i] };
  }

  getPane() {
    return this.pane;
  }

  /** Its place among its pane's series (theirs in the library arranged so at the next sync). @param {number} order */
  setSeriesOrder(order) {
    this.pane.list = this.pane.list.filter((each) => each !== this);
    this.pane.list.splice(order, 0, this);
    this.pane.dirty = true;
    this.chart.changed();
  }

  /** A primitive of Studio's on it (its drawings, halving marks): drawn by its pane views (see Pane.paint). @param {any} primitive */
  attachPrimitive(primitive) {
    this.primitives.push(primitive);
    this.pane.layers();
    this.beside ??= this.pane.engine.add(new Custom({ draw: (g, at) => this.pane.paint(g, at, "normal", this), first: () => 1 }));
    primitive.attached?.({ chart: this.chart, series: this, requestUpdate: () => this.chart.engine.invalidate(true) });
    this.pane.dirty = true;
    this.chart.changed();
  }

  /** @param {any} primitive */
  detachPrimitive(primitive) {
    this.primitives = this.primitives.filter((each) => each !== primitive);
    primitive.detached?.();
    this.chart.engine.invalidate(true);
  }

  /**
   * A line across at a price (the chart library's price line: Studio's levels, hung on a carrier series): drawn just
   * over its series, across its pane, in its color, width and style; its value on the axis (`axisLabelVisible`).
   * @param {any} options
   */
  createPriceLine(options) {
    const line = { opts: { ...options }, dash: dashOf(options), options: () => line.opts, applyOptions: (/** @type {any} */ more) => ((line.opts = { ...line.opts, ...more }), (line.dash = dashOf(line.opts)), this.#leveled()) };
    this.levels.push(line);
    this.across ??= this.pane.engine.add(new Custom({ draw: (g, at) => this.#drawLevels(g, at), first: (i) => this.engine.first(i), axisTags: (side, add) => this.#levelTags(side, add) }), this.side);
    this.pane.dirty = true;
    this.#leveled();
    return line;
  }

  /** @param {any} line */
  removePriceLine(line) {
    this.levels = this.levels.filter((each) => each !== line);
    this.#leveled();
  }

  /** Its lines across changed: drawn whole. */
  #leveled() {
    this.across?.changed();
    this.chart.changed();
  }

  /**
   * Its lines across drawn (from x `x0` to `x1` of the plot), their dashes where they'd be on the timeline (a moved
   * picture has them where a whole draw does).
   * @param {CanvasRenderingContext2D} g @param {import("/modules/bitview-charts/custom.js").View} at
   */
  #drawLevels(g, at) {
    const { time, scale, x0, x1, dpr } = at;
    for (const line of this.levels) {
      const o = line.opts, y = Math.round(scale.y(o.price) * dpr) / dpr + 0.5 / dpr;
      if (o.lineVisible === false || y !== y) continue;
      g.strokeStyle = o.color;
      g.lineWidth = o.lineWidth ?? 1;
      g.setLineDash(line.dash);
      g.lineDashOffset = line.dash.length ? (x0 + time.from * time.spacing) % (line.dash[0] + line.dash[1]) : 0;
      g.beginPath();
      g.moveTo(x0, y);
      g.lineTo(x1, y);
      g.stroke();
    }
  }

  /** Its lines across' values on its axis on `side`, each in its color. @param {number} side @param {(y: number, text: string, color: string) => void} add */
  #levelTags(side, add) {
    if ((this.side === "left" ? 1 : 0) !== side) return;
    const scale = this.#scale(), format = this.priceFormatter().format;
    for (const line of this.levels) if (line.opts.axisLabelVisible !== false) add(scale.y(line.opts.price), format(line.opts.price), line.opts.color);
  }

  /** Whether a point (in the client's coordinates) is on its tag, as last drawn (LWC patch 51). @param {number} x @param {number} y */
  bvTagHit(x, y) {
    if (this.opts.visible === false) return false;
    const box = this.chart.element.getBoundingClientRect(), line = this.chart.engine.tagAt(x - box.left, y - box.top);
    return line !== null && (line === this.engine || line === this.edge);
  }
}

/** A line style's dashes at a width (the chart library's: dotted, dashed, large dashed, sparse dotted; solid, none). @param {any} options */
function dashOf(options) {
  const w = options.lineWidth ?? 1;
  return [[], [w, w], [2 * w, 2 * w], [6 * w, 6 * w], [w, 4 * w]][options.lineStyle ?? 0] ?? [];
}

/** A number rounded up, then down to an odd one, or an even one (as the chart library sized its marks). @param {number} x */
const odd = (x) => ((n) => (n % 2 ? n : n - 1))(Math.ceil(x)), even = (/** @type {number} */ x) => ((n) => (n % 2 ? n - 1 : n))(Math.ceil(x));

/**
 * A series' marks (the chart library's markers plugin: Studio's signals' marks), drawn over its pane's series as the
 * plugin drew them, to its sizes: each at its date, above its point (its candle's high), below it (its low), or on it;
 * an arrow, a circle or a square, as tall as a point is wide (12 to 30 pixels; times its `size`), several at one point
 * stacked away from it; its words (`text`) past it, in its color, edged in the chart's halo (`layout.bvHalo`, its
 * patch 47). In the strips a pan draws, those whose shape or words reach in.
 */
class Marks {
  /** @param {Series} series @param {any[]} list */
  constructor(series, list) {
    this.series = series;
    /** @type {any[]} */
    this.list = [];
    /** Each mark's point on the timeline (-1: none), and its words' width (CSS pixels). */
    this.at = new Int32Array(0);
    this.widths = new Float64Array(0);
    /** How far past its point a mark reaches (its words' half, a shape's), for a strip's. */
    this.reach = 15;
    this.engine = series.pane.engine.add(new Custom({ draw: (g, at) => this.#draw(g, at.time, at.scale, at.x0, at.x1), first: (i) => series.engine.first(i) }), series.side);
    series.pane.marks.push(this);
    series.pane.dirty = true;
    series.chart.changed();
    this.setMarkers(list);
  }

  /** @param {any[]} list */
  setMarkers(list) {
    this.list = list;
    this.place();
  }

  markers() {
    return this.list;
  }

  detach() {
    const pane = this.series.pane;
    pane.engine.remove(this.engine);
    pane.marks = pane.marks.filter((each) => each !== this);
    this.series.chart.engine.invalidate(true);
  }

  /** Each mark's point on the chart's timeline, its words measured; drawn anew. */
  place() {
    const times = this.series.chart.times, list = this.list, g = (unseen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d")));
    g.font = this.series.chart.engine.theme.font;
    this.at = Int32Array.from(list, (mark) => indexOf(times, Number(mark.time)));
    this.widths = Float64Array.from(list, (mark) => (mark.text ? g.measureText(mark.text).width : 0));
    this.reach = 18 + Math.max(0, ...this.widths) / 2;
    this.engine.changed();
    this.series.chart.engine.invalidate(true);
  }

  /**
   * Its marks from x `x0` to `x1` (and those reaching in) drawn on `g`.
   * @param {CanvasRenderingContext2D} g @param {import("/modules/bitview-charts/index.js").Timeline} time @param {Scale} scale
   * @param {number} x0 @param {number} x1
   */
  #draw(g, time, scale, x0, x1) {
    const engine = this.series.engine, list = this.list, at = this.at, n = list.length;
    // (Its series hidden: none shown, as the plugin drew none.)
    if (engine.hidden) return;
    // (Its points' heights: a candle's high and low, else its value; an area's, its top.)
    const values = engine instanceof Candles ? engine.close : engine instanceof Band ? engine.upper : /** @type {Line} */ (engine).values;
    const highs = engine instanceof Candles ? engine.high : values, lows = engine instanceof Candles ? engine.low : values;
    const unit = Math.min(Math.max(time.spacing, 12), 30), height = even(odd(unit)), margin = Math.max(odd(unit * 0.1), 3), chart = this.series.chart;
    const font = chart.engine.theme.font, words = parseFloat(font) || 11, halo = chart.options.layout?.bvHalo;
    g.font = font;
    g.textAlign = "center";
    g.textBaseline = "middle";
    g.lineWidth = 3;
    g.lineJoin = "round";
    // (Several at one point: each one further from it, above and below apart.)
    let point = -1, above = 0, below = 0;
    for (let k = 0; k < n; k++) {
      const i = at[k], mark = list[k];
      if (i < 0) continue;
      if (i !== point) (point = i), (above = margin), (below = margin);
      const size = height * (mark.size ?? 1), half = size / 2, position = mark.position;
      let y, textY;
      if (position === "aboveBar") {
        y = scale.y(highs[i]) - half - above;
        textY = y - half - words * 0.6;
        above += size + margin + (mark.text ? words * 1.2 : 0);
      } else if (position === "belowBar") {
        y = scale.y(lows[i]) + half + below;
        textY = y + half + margin + words * 0.6;
        below += size + margin + (mark.text ? words * 1.2 : 0);
      } else (y = scale.y(values[i])), (textY = y + half + margin + words * 0.6);
      const x = time.x(i);
      if (y !== y || x < x0 - this.reach || x > x1 + this.reach) continue;
      if (mark.text) {
        g.fillStyle = mark.color;
        if (halo) (g.strokeStyle = halo), g.strokeText(mark.text, x, textY);
        g.fillText(mark.text, x, textY);
      }
      if (!size) continue;
      g.fillStyle = mark.color;
      g.beginPath();
      if (mark.shape === "circle") g.arc(x, y, (odd(Math.max(size, 12) * 0.8) - 1) / 2, 0, Math.PI * 2);
      else if (mark.shape === "square") ((side) => g.rect(x - side / 2, y - side / 2, side, side))(odd(Math.max(size, 12) * 0.7));
      else {
        // (An arrow: its head toward its point, its stem half as wide behind it.)
        const up = mark.shape === "arrowUp" ? 1 : -1, head = (odd(Math.max(size, 12)) - 1) / 2, stem = (odd(size / 2) - 1) / 2;
        g.moveTo(x - head, y);
        g.lineTo(x, y - up * head);
        g.lineTo(x + head, y);
        g.lineTo(x + stem, y);
        g.lineTo(x + stem, y + up * head);
        g.lineTo(x - stem, y + up * head);
        g.lineTo(x - stem, y);
        g.closePath();
      }
      g.fill();
    }
  }
}

/** A row of the chart's table (a pane's, the time axis'): its left axis', plot's and right axis' cells. */
function row() {
  const tr = document.createElement("tr");
  tr.style.cssText = "position: absolute; left: 0; right: 0; display: block";
  for (let k = 0; k < 3; k++) {
    const td = document.createElement("td");
    td.style.cssText = "position: absolute; top: 0; bottom: 0; display: block; padding: 0";
    tr.append(td);
  }
  return tr;
}

/** A cell's frame (the chart library's: the canvases and what Studio puts over them, in it). */
function frame() {
  const div = document.createElement("div");
  div.style.cssText = "position: relative; width: 100%; height: 100%";
  return div;
}

/** One of the library's canvases in a cell of the table (`first`: before what's there), at its corner. @param {HTMLCanvasElement} canvas @param {HTMLElement} into @param {boolean} [first] */
function mount(canvas, into, first = false) {
  if (canvas.parentElement !== into) first ? into.prepend(canvas) : into.append(canvas);
  Object.assign(canvas.style, { left: "0px", top: "0px" });
}

/** A row laid over the chart from `top`, `height` tall: its cells the left axis' `left`, the plot's `plot`, the right axis' `axis` wide. @param {HTMLElement} tr @param {number} top @param {number} height @param {number} left @param {number} plot @param {number} axis */
function placeRow(tr, top, height, left, plot, axis) {
  Object.assign(tr.style, { top: `${top}px`, height: `${height}px` });
  const [a, b, c] = /** @type {HTMLElement[]} */ ([...tr.children]);
  Object.assign(a.style, { left: "0px", width: `${left}px` });
  Object.assign(b.style, { left: `${left}px`, width: `${plot}px` });
  Object.assign(c.style, { left: `${left + plot}px`, width: `${axis}px` });
}

/** No views. */
const NONE = /** @type {any[]} */ ([]);

/** Each view's renderer, asked once (Studio's draw what their views hold now, whichever is asked: none made a frame). @type {WeakMap<any, any>} */
const renderers = new WeakMap();
/** @param {any} view */
function rendererOf(view) {
  let renderer = renderers.get(view);
  if (renderer === undefined) renderers.set(view, (renderer = view.renderer?.() ?? null));
  return renderer;
}

/**
 * What the chart library gave a renderer to draw on: the canvas, in device pixels (`useBitmapCoordinateSpace`) or CSS
 * ones (`useMediaCoordinateSpace`), and its sizes; set at each draw (one each drawer: nothing made a frame).
 */
class Target {
  constructor() {
    this.scope = { context: /** @type {CanvasRenderingContext2D} */ (/** @type {unknown} */ (null)), mediaSize: { width: 0, height: 0 }, bitmapSize: { width: 0, height: 0 }, horizontalPixelRatio: 1, verticalPixelRatio: 1 };
    this.useBitmapCoordinateSpace = (/** @type {(scope: any) => void} */ draw) => (this.scope.context.setTransform(1, 0, 0, 1, 0, 0), draw(this.scope));
    this.useMediaCoordinateSpace = (/** @type {(scope: any) => void} */ draw) => {
      const dpr = this.scope.horizontalPixelRatio;
      this.scope.context.setTransform(dpr, 0, 0, dpr, 0, 0);
      draw(this.scope);
    };
  }

  /** On `g`, `width` by `height` CSS pixels, `dpr` device pixels each. @param {CanvasRenderingContext2D} g @param {number} width @param {number} height @param {number} dpr */
  set(g, width, height, dpr) {
    const scope = this.scope;
    (scope.context = g), (scope.horizontalPixelRatio = scope.verticalPixelRatio = dpr);
    (scope.mediaSize.width = width), (scope.mediaSize.height = height);
    (scope.bitmapSize.width = Math.round(width * dpr)), (scope.bitmapSize.height = Math.round(height * dpr));
    return this;
  }
}

/** The first index in `places[0, n)` (rising) at or past `at`. @param {Int32Array} places @param {number} n @param {number} at */
function lowerBound(places, n, at) {
  let lo = 0, hi = n;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    places[mid] < at ? (lo = mid + 1) : (hi = mid);
  }
  return lo;
}

/** A canvas a hit's drawing goes to (its marks found, not seen). @type {CanvasRenderingContext2D | null} */
let unseen = null;

/** A pixel's canvas the page's ground is found on (see Chart's ground). @type {CanvasRenderingContext2D | null} */
let ink = null;

/** A computed color's alpha, 0 to 1 (rgba's last, a color function's after "/"; none, 1; "transparent", 0). @param {string} color */
function alphaOf(color) {
  if (color === "transparent") return 0;
  const alpha = /(?:rgba\(.*,|\/)\s*([\d.]+%?)\s*\)$/.exec(color)?.[1];
  return alpha === undefined ? 1 : alpha.endsWith("%") ? parseFloat(alpha) / 100 : parseFloat(alpha);
}

/**
 * A custom series' view (Studio's clouds, heatmaps, bubbles: the chart library's custom series), drawn by a Custom of
 * bitview-charts as the chart library drew it: its rows its bars (each its place on the timeline, `time`, and as drawn
 * its x; whitespace, by its view, none), the strip being drawn its range (and the bars whose marks reach into it: a
 * bubble's radius; a heatmap's, all in view: what it draws, it keeps), its renderer given the canvas in device pixels
 * (or CSS ones) and its scale's y. Fitted to its rows' prices (its priceValueBuilder's) or by its
 * autoscaleInfoProvider; on an overlay, its scale its own (fixed by it). Hit where its renderer's hitTest says, its
 * marks around the point drawn unseen first (only the strips' are kept as drawn). Nothing made a frame.
 */
class Painted {
  /** @param {Series} series @param {any} view */
  constructor(series, view) {
    this.series = series;
    this.view = view;
    // (Its renderer once: Studio's draws read its view's data and options, whichever renderer's asked.)
    this.renderer = view.renderer();
    /** @type {{ x: number, time: number, originalData: any, barColor: string | undefined }[]} */
    this.bars = [];
    /** Its bars' places on the timeline, rising (`count` of them). */
    this.places = new Int32Array(0);
    this.count = 0;
    /** On the timeline: each row's lowest and highest price, and its last (its close: a relative scale's base, its value). */
    this.lows = new Float64Array(0);
    this.highs = this.lows;
    this.closes = this.lows;
    /** Its lows and highs as lines (in no pane): their extents by blocks, not point by point, each frame. */
    this.lowLine = new Line();
    this.highLine = new Line();
    this.data = { bars: this.bars, barSpacing: 1, visibleRange: { from: 0, to: 0 } };
    /** The scale it's drawn on now. @type {Scale} */
    this.scale = new Scale();
    this.toY = (/** @type {number} */ price) => this.scale.y(price);
    this.target = new Target();
    this.engine = new Custom({
      draw: (g, at) => this.#draw(g, at.time, at.scale, at.x0, at.x1, at.dpr),
      extent: (a, b, positive, out) => this.#extent(a, b, positive, out),
      first: (i) => this.#first(i),
      distance: (_i, y, scale, x) => this.#distance(scale, x, y),
    });
  }

  /**
   * Its look from its options: hidden, an overlay's range; drawn whole each frame where what it draws hangs on the whole
   * view (Studio's profile at the plot's edge; a heatmap's days, grouped by how many are in view).
   */
  style() {
    const o = this.series.opts, overlay = this.series.scaleApi.overlay;
    this.engine.fixed = Boolean(o.bvFixed) || "heatId" in o;
    // (On an overlay, not read on its side's scale: its pane's grid not that side's for it.)
    this.engine.scaled = !overlay;
    if (overlay) {
      const info = o.autoscaleInfoProvider?.(original);
      if (info?.priceRange) overlay.fit(info.priceRange.minValue, info.priceRange.maxValue);
    }
    this.engine.changed();
  }

  /** Its rows onto the timeline: its bars, their prices' columns; its closes returned (its values). @param {Float64Array} times */
  push(times) {
    const rows = this.series.rows ?? [], view = this.view, n = times.length;
    this.lows = new Float64Array(n).fill(NaN);
    this.highs = new Float64Array(n).fill(NaN);
    this.closes = new Float64Array(n).fill(NaN);
    this.bars = this.data.bars = [];
    this.places = new Int32Array(rows.length);
    let m = 0;
    for (const row of rows) {
      if (view.isWhitespace?.(row)) continue;
      const i = indexOf(times, Number(row.time));
      if (i < 0) continue;
      this.bars.push({ x: 0, time: i, originalData: row, barColor: row.color });
      this.places[m++] = i;
      const prices = view.priceValueBuilder(row);
      let lo = Infinity, hi = -Infinity;
      for (const price of prices) if (price === price) (lo = Math.min(lo, price)), (hi = Math.max(hi, price));
      if (lo <= hi) (this.lows[i] = lo), (this.highs[i] = hi), (this.closes[i] = prices[prices.length - 1]);
    }
    this.count = m;
    this.lowLine.setValues(this.lows);
    this.highLine.setValues(this.highs);
    this.engine.changed();
    return this.closes;
  }

  /**
   * Drawn on `g` from x `x0` to `x1` of the plot: its bars there (and those reaching in) placed, its view updated, its
   * renderer's draw.
   * @param {CanvasRenderingContext2D} g @param {import("/modules/bitview-charts/index.js").Timeline} time @param {Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  #draw(g, time, scale, x0, x1, dpr) {
    const o = this.series.opts, overlay = this.series.scaleApi.overlay, bars = this.bars;
    this.scale = overlay ? ((overlay.height = scale.height), overlay) : scale;
    // (Its marks' reach past their point: a bubble's radius, its ring. One drawn whole each frame (a heatmap: it keeps
    // the days it draws, asks for those missing), all in view.)
    const all = this.engine.fixed, reach = (o.size ?? 0) / 2 + 4;
    const a = all ? time.from - 1 : time.index(x0 - reach) - 1, b = all ? time.to + 1 : time.index(x1 + reach) + 1;
    const from = lowerBound(this.places, this.count, Math.floor(a)), to = lowerBound(this.places, this.count, Math.ceil(b) + 1);
    for (let k = from; k < to; k++) bars[k].x = time.x(bars[k].time);
    const data = this.data;
    (data.barSpacing = time.spacing), (data.visibleRange.from = from), (data.visibleRange.to = to);
    this.view.update(data, o);
    this.renderer.draw(this.target.set(g, time.width, scale.height, dpr), this.toY);
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  #extent(a, b, positive, out) {
    // (An overlay's its own; a provider's range, none, or its own.)
    if (this.series.scaleApi.overlay) return;
    const info = this.series.opts.autoscaleInfoProvider?.(original);
    if (info !== undefined && info !== OWN) {
      const range = info?.priceRange;
      if (range && range.minValue < out[0]) out[0] = range.minValue;
      if (range && range.maxValue > out[1]) out[1] = range.maxValue;
      return;
    }
    // (Its lows' and highs' extremes read block by block, as a line's are: on a log scale, those above 0.)
    this.lowLine.extent(a, b, positive, out);
    this.highLine.extent(a, b, positive, out);
  }

  /** Its first close at or past `i` (on an overlay, its scale its own: any, 1, so a relative pane draws it). @param {number} i */
  #first(i) {
    if (this.series.scaleApi.overlay) return 1;
    const closes = this.closes;
    for (let k = Math.max(0, i); k < closes.length; k++) if (closes[k] === closes[k]) return closes[k];
    return NaN;
  }

  /**
   * How far a point is from it: none (0) where its renderer's hitTest finds one of its marks, its marks around the point
   * drawn unseen first; else never hit.
   * @param {Scale} scale @param {number} x @param {number} y
   */
  #distance(scale, x, y) {
    if (!this.renderer.hitTest) return Infinity;
    unseen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
    const chart = this.series.chart.engine;
    this.#draw(unseen, chart.time, scale, x, x, chart.dpr);
    return this.renderer.hitTest(x, y) ? 0 : Infinity;
  }
}
