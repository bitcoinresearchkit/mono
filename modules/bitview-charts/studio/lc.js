// @ts-check
// The part of Lightweight Charts' API Studio uses, on bitview-charts: Studio's copy imports this in the chart library's
// place, its own code as it is, to try the library in the real app. Studio's glue, not the library's: what Studio does
// with the library (its options, its patches' calls: bv*), mapped onto bitview-charts' panes, scales and series. Phase
// one: the chart, its panes and scales, the view, the crosshair and clicks, pictures, and the standard series (lines,
// areas (stacked ones' fills to the line under them), histograms, baselines, candles), rows or columns onto one
// timeline (the page's: see Chart.#sync). Phase two: custom series (Studio's clouds, heatmaps, bubbles: see Painted) and
// overlay scales. (Their order in their panes, and what Studio draws with them (its drawings, halving marks, lines
// across), Studio's own on the library.)

import { createChart as make, picture, Line, Band, Bars, Candles, Custom, Scale } from "/modules/bitview-charts/index.js";

export const LineSeries = "Line", AreaSeries = "Area", HistogramSeries = "Histogram", BaselineSeries = "Baseline", CandlestickSeries = "Candlestick";
export const LineStyle = { Solid: 0, Dotted: 1, Dashed: 2, LargeDashed: 3, SparseDotted: 4 };
export const LineType = { Simple: 0, WithSteps: 1, Curved: 2 };
export const ColorType = { Solid: "solid", VerticalGradient: "gradient" };
export const CrosshairMode = { Normal: 0, Magnet: 1, Hidden: 2, MagnetOHLC: 3 };
export const TrackingModeExitMode = { OnTouchEnd: 0, OnNextTap: 1 };
export const customSeriesDefaultOptions = { color: "#2196f3", visible: true, lastValueVisible: true, priceLineVisible: true, priceScaleId: "right" };

/** @param {HTMLElement} element @param {any} [options] */
export function createChart(element, options = {}) {
  return new Chart(element, options);
}

// (What an autoscaleInfoProvider's `original` gives it here: a sign it kept the series' own range.)
const OWN = { priceRange: null }, original = () => OWN;
/** A fit adding nothing (an edge: its series fitted by its band). */
const NOTHING = () => {};

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
    this.element = document.createElement("div");
    Object.assign(this.element.style, { width: options.width ? `${options.width}px` : "100%", height: options.height ? `${options.height}px` : "100%" });
    element.append(this.element);
    // (Its look, its time scale's room and spacing, its dates' words: the page's, set on the library.)
    this.engine = make(this.element);
    /** Its wrapper of each of the library's panes (made by the page). @type {WeakMap<any, Pane>} */
    this.panes = new WeakMap();
    /** @type {Series[]} */
    this.all = [];
    /** The timeline its series were last pushed onto (the library's, as the page set it: see #sync). @type {Float64Array} */
    this.times = new Float64Array(0);
    this.dirty = false;
  }

  // ── What Studio calls ──


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
    this.all = this.all.filter((each) => each !== series);
    this.changed();
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

  remove() {
    this.engine.destroy();
    this.element.remove();
  }

  // ── Its panes and timeline ──

  /** The library's pane at `index` (the page's: made before its series), wrapped. @param {number} index */
  #pane(index) {
    const engine = this.engine.panes[index];
    return this.panes.get(engine) ?? ((pane) => (this.panes.set(engine, pane), pane))(new Pane(this, engine));
  }

  /** A series' data or look changed: brought to the library before the next frame. */
  changed() {
    if (this.dirty) return;
    this.dirty = true;
    queueMicrotask(() => this.#sync());
  }

  /**
   * Every series' data onto the timeline the page set (the library's), each one's columns on it, stacks' fills to the
   * line under them. A new timeline pushes every series and draws whole; else each one changed alone (a live price's
   * point pushes the price alone), only the panes whose series changed (their versions) drawn anew.
   */
  #sync() {
    if (!this.dirty) return;
    this.dirty = false;
    const times = this.engine.time.times, fresh = times !== this.times;
    this.times = times;
    for (const series of this.all) if (fresh || series.dirty) (series.dirty = false), series.push(times);
    this.engine.invalidate(fresh);
  }
}

class Pane {
  /** @param {Chart} chart @param {import("/modules/bitview-charts/index.js").Pane} engine */
  constructor(chart, engine) {
    this.chart = chart;
    this.engine = engine;
    /** @type {Record<string, PriceScale>} */
    this.scales = {};
  }

  /** Its scale of that id: its right one, its left one, or an overlay (any other id: its series' own, unseen). @param {string} id */
  scaleApi(id) {
    return (this.scales[id] ??= new PriceScale(this, id));
  }
}

class PriceScale {
  /** @param {Pane} pane @param {string} id */
  constructor(pane, id) {
    this.pane = pane;
    /** Its side; an overlay's, none (its series drawn on the right one's side, its own scale theirs). @type {"right" | "left" | null} */
    this.side = id === "left" || id === "right" ? id : null;
    /** An overlay's own scale: its series' range (their autoscaleInfoProvider's), the pane's height. */
    this.overlay = this.side ? null : new Scale({ top: 0.2, bottom: 0.1 });
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
    /** Whether its rows are its own copy (Studio's arrays, given to setData, never changed). */
    this.ownRows = false;
    /** Its data or look changed since it was pushed. */
    this.dirty = true;
    /** A custom series' view, drawn as the chart library drew it. @type {Painted | null} */
    this.painted = view ? new Painted(this, view) : null;
    // (Its fitting by its autoscaleInfoProvider (see #fit), made once: nothing made a frame but what the provider makes.)
    this.fitter = (/** @type {number} */ a, /** @type {number} */ b, /** @type {boolean} */ positive, /** @type {Float64Array} */ out) => this.#fit(a, b, positive, out);
    this.original = () => this.#own();
    this.asked = new Float64Array(3);
    this.range = new Float64Array(2);
    this.info = { priceRange: { minValue: 0, maxValue: 0 } };
    this.engine = this.#make();
    pane.engine.add(this.engine, this.side);
    // (An area's or a baseline's line along its edge, over its fill: its tag and its price line its.)
    /** @type {Line | null} */
    this.edge = this.type === "Area" || this.type === "Baseline" ? pane.engine.add(new Line(), this.side) : null;
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

  /**
   * Its fitting by its autoscaleInfoProvider where it has one (a stack's: its edge then not fitted to, the series' range
   * its band's); a custom series' look from its options. (A standard series' look is the page's: the library's styles.)
   */
  #style() {
    const o = this.opts, engine = this.engine;
    if (engine instanceof Custom) return void ((engine.hidden = o.visible === false), /** @type {Painted} */ (this.painted).style());
    engine.fit = o.autoscaleInfoProvider ? this.fitter : null;
    if (this.edge) this.edge.fit = o.autoscaleInfoProvider ? NOTHING : null;
  }

  /**
   * Its autoscale info from point `a` to `b`, as the chart library's: its own range there (none: null), through its
   * autoscaleInfoProvider where it has one (Studio's: a stack's from zero or 0 to 100, a
   * carrier's none), `original` its own.
   * @param {number} a @param {number} b @param {boolean} positive
   */
  autoscale(a, b, positive) {
    (this.asked[0] = a), (this.asked[1] = b), (this.asked[2] = positive ? 1 : 0);
    const provider = this.opts.autoscaleInfoProvider;
    return provider ? provider(this.original) : this.#own();
  }

  /**
   * Its scale's range from it by its autoscaleInfoProvider, from point `a` to `b` widening `out`.
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  #fit(a, b, positive, out) {
    const range = this.autoscale(a, b, positive)?.priceRange;
    if (!range) return;
    if (range.minValue < out[0]) out[0] = range.minValue;
    if (range.maxValue > out[1]) out[1] = range.maxValue;
  }

  /** Its own autoscale info where it's asked (see autoscale): its range there; no range, null. */
  #own() {
    const range = this.range, asked = this.asked;
    (range[0] = Infinity), (range[1] = -Infinity);
    this.engine.extent(asked[0], asked[1], asked[2] === 1, range);
    if (!(range[0] <= range[1])) return null;
    (this.info.priceRange.minValue = range[0]), (this.info.priceRange.maxValue = range[1]);
    return this.info;
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
    // (Pushed again only where its push reads what changed (its place in a stack): a look alone is set (Studio's
    // bubbles' sizes, as the view moves, a spotlight's fading); a pane's writers or stack set again; shown or hidden,
    // drawn whole.)
    if ("visible" in options) this.chart.engine.invalidate(true);
    this.chart.changed();
  }

  /** @param {any[]} rows */
  setData(rows) {
    this.rows = rows;
    this.ownRows = false;
    // (In a loop: without a JIT, from() with a mapping costs several times as much.)
    (this.times = new Float64Array(rows.length)), (this.values = new Float64Array(rows.length));
    for (let i = 0; i < rows.length; i++) (this.times[i] = Number(rows[i].time)), (this.values[i] = rows[i].value ?? rows[i].close ?? NaN);
    this.dirty = true;
    this.chart.changed();
  }

  /** The patches' older rows put in front. @param {any[]} rows */
  bvPrepend(rows) {
    this.setData([...rows, ...(this.rows ?? Array.from(this.times, (time, i) => ({ time, value: this.values[i] })))]);
  }

  /** The last row set, or one added past it. @param {any} row */
  update(row) {
    // (A copy: Studio's own array, given to setData, stays as it gave it.)
    const rows = this.rows ? this.rows.slice() : Array.from(this.times, (time, i) => ({ time, value: this.values[i] }));
    if (rows.length && Number(rows[rows.length - 1].time) === Number(row.time)) rows[rows.length - 1] = row;
    else rows.push(row);
    this.setData(rows);
    this.ownRows = true;
  }

  /** A custom series' rows onto the timeline (its bars; its closes its values: see Painted); a standard one's data is the page's, set on the library. @param {Float64Array} times */
  push(times) {
    if (this.painted) this.columns = this.painted.push(times);
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
    // (On an overlay, its scale its own: the library's (fitted by its provider's range, as its extent gives none).)
    this.engine.overlay = overlay;
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
    const o = this.series.opts, bars = this.bars;
    this.scale = scale;
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
