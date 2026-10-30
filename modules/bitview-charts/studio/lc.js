// @ts-check
// The part of Lightweight Charts' API Studio uses, on bitview-charts: Studio's copy imports this in the chart library's
// place, its own code as it is, to try the library in the real app. Studio's glue, not the library's: what Studio does
// with the library (its options, its patches' calls: bv*), mapped onto bitview-charts' panes, scales and series. Phase
// one: the chart, its panes and scales, the view, the crosshair and clicks, pictures, and the standard series (lines,
// areas (stacked ones' fills to the line under them), histograms, baselines, candles), rows or columns onto one
// timeline. Not yet: custom series (Studio's clouds, heatmaps, bubbles) and primitives (its drawings, marks), kept but
// not drawn; markers and price lines of its own, none.

import { createChart as make, DARK, picture, Line, Band, Bars, Candles } from "/modules/bitview-charts/index.js";

export const LineSeries = "Line", AreaSeries = "Area", HistogramSeries = "Histogram", BaselineSeries = "Baseline", CandlestickSeries = "Candlestick";
export const LineStyle = { Solid: 0, Dotted: 1, Dashed: 2, LargeDashed: 3, SparseDotted: 4 };
export const LineType = { Simple: 0, WithSteps: 1, Curved: 2 };
export const ColorType = { Solid: "solid", VerticalGradient: "gradient" };
export const CrosshairMode = { Normal: 0, Magnet: 1, Hidden: 2, MagnetOHLC: 3 };
export const TrackingModeExitMode = { OnTouchEnd: 0, OnNextTap: 1 };
export const MismatchDirection = { NearestLeft: -1, None: 0, NearestRight: 1 };
export const TickMarkType = { Year: 0, Month: 1, DayOfMonth: 2, Time: 3, TimeWithSeconds: 4 };
export const customSeriesDefaultOptions = { color: "#2196f3", visible: true, lastValueVisible: true, priceLineVisible: true, priceScaleId: "right" };

/** Markers (a plugin of the library's): none drawn yet. @param {any} _series @param {any[]} [_markers] */
export function createSeriesMarkers(_series, _markers) {
  return { setMarkers() {}, markers: () => [], detach() {} };
}

/** @param {HTMLElement} element @param {any} [options] */
export function createChart(element, options = {}) {
  return new Chart(element, options);
}

const MODES = ["linear", "log", "percent", "indexed"];

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
    this.element = element;
    this.options = { ...options };
    this.engine = make(element, { theme: this.#theme(options), rightOffset: options.timeScale?.rightOffsetPixels ?? 48 });
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
    this.engine.onCrosshair((index, pane, y, input) => {
      const param = this.#param(index, pane, y, input);
      for (const listener of this.crosshairListeners) listener(param);
    });
    this.engine.onClick((index, pane, hit, y) => {
      const engine = this.engine.panes[pane], drawn = engine && hit >= 0 ? engine.series[hit] : null, series = drawn ? this.all.find((each) => each.engine === drawn || each.edge === drawn) : undefined;
      const param = { ...this.#param(index, pane, y, true), hoveredSeries: series };
      for (const listener of this.clickListeners) listener(param);
    });
    this.timeScaleApi = new TimeScale(this);
  }

  /** The library's theme from LWC's options. @param {any} options */
  #theme(options) {
    const layout = options.layout ?? {}, grid = options.grid?.horzLines ?? {};
    return {
      ...DARK,
      ...(layout.background?.color && layout.background.color !== "transparent" ? { background: layout.background.color } : {}),
      ...(layout.textColor ? { text: layout.textColor } : {}),
      ...(grid.color ? { grid: grid.color } : {}),
      font: `${layout.fontSize ?? 11}px ${layout.fontFamily ?? "system-ui, sans-serif"}`,
    };
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
    this.options = { ...this.options, ...options, timeScale: { ...this.options.timeScale, ...options.timeScale } };
    if (options.layout || options.grid) this.engine.theme = this.#theme(this.options);
    if (options.timeScale?.rightOffsetPixels !== undefined) this.engine.time.rightOffset = options.timeScale.rightOffsetPixels;
    this.engine.invalidate(true);
  }

  timeScale() {
    return this.timeScaleApi;
  }

  /** @param {string} id @param {number} [pane] */
  priceScale(id, pane = 0) {
    return this.#pane(pane).scaleApi(id === "left" ? "left" : "right");
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
    const series = new Series(this, "Custom", { ...customSeriesDefaultOptions, ...options }, this.#pane(pane), view);
    this.all.push(series);
    return series;
  }

  /** @param {Series} series */
  removeSeries(series) {
    series.pane.engine.remove(series.engine);
    if (series.edge) series.pane.engine.remove(series.edge);
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
  }

  // ── Its panes and timeline ──

  /** The pane at `index`, made (and the ones before it) if it's not there. @param {number} index */
  #pane(index) {
    while (this.panesList.length <= index) this.panesList.push(new Pane(this, this.engine.addPane({ scale: { top: 0.17, bottom: 0.08 }, left: { top: 0.17, bottom: 0.08 } })));
    return this.panesList[index];
  }

  /** Panes left with no series at the end go (as the library's did). */
  #trimPanes() {
    while (this.panesList.length > 1 && !this.panesList.at(-1)?.list.length) {
      const pane = /** @type {Pane} */ (this.panesList.pop());
      this.engine.removePane(pane.engine);
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
    for (const pane of this.panesList) if (pane.dirty) (pane.dirty = false), pane.stack();
    if (fresh) this.engine.setTimes(times);
    this.engine.invalidate(true);
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
  }

  /** @param {"right" | "left"} side */
  scaleApi(side) {
    return (this.scales[side] ??= new PriceScale(this, side));
  }

  paneIndex() {
    return this.chart.panesList.indexOf(this);
  }

  getHTMLElement() {
    return this.engine.canvas;
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
  /** @param {Pane} pane @param {"right" | "left"} side */
  constructor(pane, side) {
    this.pane = pane;
    this.side = side;
    this.opts = { mode: 0, autoScale: true, visible: true, scaleMargins: { top: 0.17, bottom: 0.08 } };
  }

  /** The library's scale: the right one, or the left one once a series is on it. */
  get scale() {
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
    if (options.autoScale) this.pane.engine.setRange(undefined, undefined, this.side);
    else if (options.autoScale === false && scale) this.pane.engine.setRange(scale.lo, scale.hi, this.side);
    this.pane.chart.engine.invalidate(true);
  }

  /** @param {boolean} on */
  setAutoScale(on) {
    this.applyOptions({ autoScale: on });
  }

  width() {
    return this.side === "left" ? this.pane.chart.engine.left : this.pane.chart.engine.axis;
  }

  /** The patches' range: what it shows, as values. */
  bvRange() {
    const scale = this.scale;
    return scale ? { from: scale.lo, to: scale.hi } : null;
  }

  /** @param {{ from: number, to: number }} range */
  bvSetRange(range) {
    this.pane.engine.setRange(range.from, range.to, this.side);
    this.pane.chart.engine.invalidate(true);
  }

  /** What it'd show fitted to its series. */
  bvAutoRange() {
    const engine = this.pane.engine, auto = this.side === "left" ? engine.leftAuto : engine.auto, scale = this.scale;
    if (!scale) return null;
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
    /** Its data or look changed since it was pushed. */
    this.dirty = true;
    /** @type {any[]} */
    this.primitives = [];
    this.engine = this.#make();
    pane.engine.add(this.engine, this.side);
    // (An area's or a baseline's line along its edge, over its fill: its tag and its price line its.)
    /** @type {Line | null} */
    this.edge = this.type === "Area" || this.type === "Baseline" ? pane.engine.add(new Line(), this.side) : null;
    pane.list.push(this);
    this.#style();
  }

  get side() {
    return this.opts.priceScaleId === "left" ? "left" : "right";
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
        // (Phase two: Studio's custom views drawn. Meanwhile, kept and not drawn.)
        return Object.assign(new Line(), { hidden: true });
      default:
        return new Line();
    }
  }

  /** Its look in the library, from its options. */
  #style() {
    const o = this.opts, engine = this.engine, shown = o.visible !== false;
    engine.hidden = !shown || this.type === "Custom";
    const tag = o.lastValueVisible !== false && shown, priceLine = o.priceLineVisible !== false && shown;
    if (this.edge) {
      this.edge.hidden = !shown;
      this.edge.setStyle({ color: o.lineColor ?? o.topLineColor ?? o.color ?? this.edge.color, width: o.lineWidth ?? 1.5, tag, priceLine, stroke: o.lineVisible !== false });
    }
    if (engine instanceof Bars) return void ((engine.color = o.color ?? engine.color), (engine.base = o.base ?? 0), engine.version++);
    if (engine instanceof Line) return void engine.setStyle({ color: o.color ?? o.lineColor ?? engine.color, width: o.lineWidth ?? 1.5, tag, priceLine, stroke: o.lineVisible !== false });
    if (engine instanceof Band) return void engine.set({ color: o.topColor ?? o.topFillColor1 ?? o.color ?? engine.color, below: this.type === "Baseline" ? o.bottomFillColor1 ?? null : null, base: this.type === "Baseline" ? o.baseValue?.price ?? 0 : 0 });
    if (engine instanceof Candles) return void engine.setStyle({ up: o.upColor, down: o.downColor });
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
    this.times = Float64Array.from(rows, (row) => Number(row.time));
    this.values = Float64Array.from(rows, (row) => (row.value ?? row.close ?? NaN));
    this.dirty = true;
    this.chart.changed();
  }

  /** The patches' columns: times and values. @param {number[] | Float64Array} times @param {Float64Array | number[]} values */
  bvSetColumns(times, values) {
    this.rows = null;
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
    // (A copy: Studio's own array, given to setData, stays as it gave it.)
    const rows = this.rows ? this.rows.slice() : Array.from(this.times, (time, i) => ({ time, value: this.values[i] }));
    if (rows.length && Number(rows[rows.length - 1].time) === Number(row.time)) rows[rows.length - 1] = row;
    else rows.push(row);
    this.setData(rows);
  }

  /** Its values on the chart's timeline, its rows' colors and breaks with them. @param {Float64Array} times */
  push(times) {
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

  /** Its scale, read against its own first value in view where it's relative. */
  #scale() {
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
    return this.pane.scaleApi(this.side);
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

  /** @param {number} order */
  setSeriesOrder(order) {
    // (Its series in the library (and its edge's, right over it) at its place among the pane's, as Studio orders them.)
    const engine = this.pane.engine, own = this.edge ? [this.engine, this.edge] : [this.engine], side = engine.sides[engine.series.indexOf(this.engine)];
    for (const each of own) {
      const k = engine.series.indexOf(each);
      engine.series.splice(k, 1), engine.sides.splice(k, 1);
    }
    const before = this.pane.list.filter((each) => each !== this)[order], at = before ? engine.series.indexOf(before.engine) : engine.series.length;
    engine.series.splice(at, 0, ...own), engine.sides.splice(at, 0, ...own.map(() => side));
    this.pane.list = this.pane.list.filter((each) => each !== this);
    this.pane.list.splice(order, 0, this);
    this.pane.dirty = true;
    this.chart.changed();
  }

  /** @param {any} primitive */
  attachPrimitive(primitive) {
    this.primitives.push(primitive);
  }

  /** @param {any} primitive */
  detachPrimitive(primitive) {
    this.primitives = this.primitives.filter((each) => each !== primitive);
  }

  /** @param {any} options */
  createPriceLine(options) {
    return { options: () => options, applyOptions() {} };
  }

  removePriceLine() {}

  /** The patches' tag hit (none yet). @param {number} _x @param {number} _y */
  bvTagHit(_x, _y) {
    return false;
  }
}
