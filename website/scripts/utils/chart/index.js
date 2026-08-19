import {
  createChart as untypedLcCreateChart,
  CandlestickSeries,
  HistogramSeries,
  LineSeries,
  BaselineSeries,
} from "../../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs";
import { createLegend, createSeriesLegend } from "./legend.js";
import { capture } from "./capture.js";
import { colors } from "../colors.js";
import { createRadios, createSelect, getElementById } from "../dom.js";
import { createPersistedValue } from "../persisted.js";
import { onChange as onThemeChange } from "../theme.js";
import { throttle, debounce } from "../timing.js";
import { serdeBool, INDEX_FROM_LABEL } from "../serde.js";
import { stringToId, numberToShortUSFormat } from "../format.js";
import { style } from "../elements.js";
import { Unit } from "../units.js";

/**
 * @typedef {_ISeriesApi<LCSeriesType>} ISeries
 * @typedef {_ISeriesApi<'Candlestick'>} CandlestickISeries
 * @typedef {_ISeriesApi<'Histogram'>} HistogramISeries
 * @typedef {_ISeriesApi<'Line'>} LineISeries
 * @typedef {_ISeriesApi<'Baseline'>} BaselineISeries
 *
 * @typedef {_LineSeriesPartialOptions} LineSeriesPartialOptions
 * @typedef {_HistogramSeriesPartialOptions} HistogramSeriesPartialOptions
 * @typedef {_BaselineSeriesPartialOptions} BaselineSeriesPartialOptions
 * @typedef {_CandlestickSeriesPartialOptions} CandlestickSeriesPartialOptions
 */

/**
 * @template T
 * @typedef {Object} Series
 * @property {string} id
 * @property {number} paneIndex
 * @property {PersistedValue<boolean>} active
 * @property {(value: boolean) => void} setActive
 * @property {(order: number) => void} setOrder
 * @property {() => void} highlight
 * @property {() => void} tame
 * @property {() => void} refresh
 * @property {number} generation
 * @property {() => boolean} hasData
 * @property {() => void} [fetch]
 * @property {string | null} url
 * @property {() => readonly T[]} getData
 * @property {(data: T) => void} update
 * @property {VoidFunction} remove
 */

/**
 * @typedef {SingleValueData | CandlestickData | LineData | BaselineData | HistogramData | WhitespaceData} AnyChartData
 * @typedef {Series<AnyChartData>} AnySeries
 */

/**
 * @typedef {_SingleValueData} SingleValueData
 * @typedef {_CandlestickData} CandlestickData
 * @typedef {_LineData} LineData
 * @typedef {_BaselineData} BaselineData
 * @typedef {_HistogramData} HistogramData
 *
 * @typedef {Object} Legend
 * @property {HTMLLegendElement} element
 * @property {(element: HTMLElement) => void} setPrefix
 * @property {() => void} clearPrefix
 * @property {(args: { series: AnySeries, name: string, order: number, colors: Color[] }) => void} addOrReplace
 * @property {(index: number) => void} removeFrom
 */

const lineWidth = /** @type {1} */ (/** @type {unknown} */ (1.5));

const MAX_SIZE = 10_000;

let hintShown = false;

/** @typedef {{ label: string, index: IndexLabel, from: number }} RangePreset */

/** @returns {RangePreset[]} */
function getRangePresets() {
  const now = new Date();
  const y = now.getUTCFullYear();
  const m = now.getUTCMonth();
  const d = now.getUTCDate();
  /** @param {number} months @param {number} [days] */
  const ago = (months, days = 0) =>
    Math.floor(Date.UTC(y, m - months, d - days) / 1000);

  /** @type {RangePreset[]} */
  const presets = [
    { label: "1w", index: /** @type {IndexLabel} */ ("30mn"), from: ago(0, 7) },
    { label: "1m", index: /** @type {IndexLabel} */ ("1h"), from: ago(1) },
    { label: "3m", index: /** @type {IndexLabel} */ ("4h"), from: ago(3) },
    { label: "6m", index: /** @type {IndexLabel} */ ("12h"), from: ago(6) },
    { label: "1y", index: /** @type {IndexLabel} */ ("1d"), from: ago(12) },
    { label: "4y", index: /** @type {IndexLabel} */ ("3d"), from: ago(48) },
    { label: "8y", index: /** @type {IndexLabel} */ ("1w"), from: ago(96) },
  ];

  const ytdFrom = Math.floor(Date.UTC(y, 0, 1) / 1000);
  const ri = presets.findIndex((e) => e.from <= ytdFrom);
  const insertAt = ri === -1 ? presets.length : ri;
  presets.splice(insertAt, 0, {
    label: "ytd",
    index: presets[Math.min(insertAt, presets.length - 1)].index,
    from: ytdFrom,
  });

  presets.push({
    label: "all",
    index: /** @type {IndexLabel} */ ("1w"),
    from: -Infinity,
  });

  return presets;
}

/**
 * @param {Object} args
 * @param {HTMLElement} args.parent
 * @param {BitviewClient} args.brk
 * @param {true} [args.fitContent]
 */
export function createChart({ parent, brk, fitContent }) {
  const baseUrl = brk.baseUrl.replace(/\/$/, "");

  /** @type {string} */
  let storageId = "";

  /** @param {ChartableIndex} idx */
  const getTimeEndpoint = (idx) =>
    idx === "height"
      ? brk.series.indexes.timestamp.monotonic.by[idx]
      : brk.series.indexes.timestamp.resolutions.by[idx];

  const index = {
    /** @type {Set<(index: ChartableIndex) => void>} */
    onChange: new Set(),

    get() {
      return INDEX_FROM_LABEL[index.name.value];
    },

    name: createPersistedValue({
      defaultValue: /** @type {IndexLabel} */ ("1d"),
      storageKey: "chart-index",
      urlKey: "i",
      serialize: (v) => v,
      deserialize: (s) =>
        /** @type {IndexLabel} */ (s in INDEX_FROM_LABEL ? s : "1d"),
      onChange: () => {
        range.set(null);
        index.onChange.forEach((cb) => cb(index.get()));
      },
    }),
  };

  // Generation counter - incremented on any context change (index, blueprints, unit)
  // Used to detect and ignore stale operations (in-flight fetches, etc.)
  let generation = 0;

  const time = {
    /** @type {SeriesData<number> | null} */
    data: null,
    /** @type {Set<(data: SeriesData<number>) => void>} */
    callbacks: new Set(),
    /** @type {ReturnType<typeof getTimeEndpoint> | null} */
    endpoint: null,

    /** @param {ChartableIndex} idx */
    setIndex(idx) {
      this.data = null;
      this.callbacks = new Set();
      this.endpoint = getTimeEndpoint(idx);
    },

    fetch() {
      const endpoint = this.endpoint;
      if (!endpoint) return;
      const currentGen = generation;
      const cached = cache.get(endpoint.path);
      if (cached) {
        this.data = cached;
      }
      endpoint.slice(-MAX_SIZE).fetch((/** @type {AnySeriesData} */ result) => {
        if (currentGen !== generation) return;
        cache.set(endpoint.path, result);
        this.data = result;
        this.callbacks.forEach((cb) => cb(result));
      });
    },
  };

  /** @type {Map<string, AnySeriesData>} */
  const cache = new Map();

  // Range state: localStorage stores all ranges per-index, URL stores current range only
  /** @typedef {{ from: number, to: number }} Range */
  const ranges = createPersistedValue({
    defaultValue: /** @type {Record<string, Range>} */ ({}),
    storageKey: "chart-ranges",
    serialize: JSON.stringify,
    deserialize: JSON.parse,
  });

  const range = createPersistedValue({
    defaultValue: /** @type {Range | null} */ (null),
    urlKey: "r",
    serialize: (v) => (v ? `${v.from.toFixed(2)}_${v.to.toFixed(2)}` : ""),
    deserialize: (s) => {
      if (!s) return null;
      const [from, to] = s.split("_").map(Number);
      return !isNaN(from) && !isNaN(to) ? { from, to } : null;
    },
  });

  /** @returns {Range | null} */
  const getRange = () => range.value ?? ranges.value[index.name.value] ?? null;

  /** @param {Range} value */
  const setRange = (value) => {
    ranges.set({ ...ranges.value, [index.name.value]: value });
    range.set(value);
  };

  const legends = [createSeriesLegend(), createSeriesLegend()];

  const root = document.createElement("div");
  root.classList.add("chart");
  parent.append(root);

  const chartEl = document.createElement("div");
  root.append(chartEl);

  const ichart = /** @type {typeof CreateLCChart} */ (untypedLcCreateChart)(
    chartEl,
    /** @satisfies {DeepPartial<ChartOptions>} */ ({
      autoSize: true,
      layout: {
        fontFamily: style.fontFamily,
        background: { color: "transparent" },
        attributionLogo: false,
        panes: {
          enableResize: false,
        },
      },
      rightPriceScale: {
        borderVisible: false,
      },
      timeScale: {
        borderVisible: false,
        enableConflation: true,
        ...(fitContent
          ? {
              minBarSpacing: 0.001,
            }
          : {}),
      },
      localization: {
        priceFormatter: numberToShortUSFormat,
        locale: "en-us",
      },
      crosshair: {
        mode: 3,
      },
      ...(fitContent
        ? {
            handleScale: false,
            handleScroll: false,
          }
        : {}),
    }),
  );
  // Takes a bit more space sometimes but it's better UX than having the scale being resized on option change
  ichart.priceScale("right").applyOptions({
    minimumWidth: 80,
  });

  ichart.panes().at(0)?.setStretchFactor(1);

  /** @typedef {(visibleBarsCount: number) => void} ZoomChangeCallback */

  const initialRange = getRange();
  if (initialRange) {
    ichart.timeScale().setVisibleLogicalRange(initialRange);
  }

  // Flag to prevent range persistence until first data load completes
  // This prevents the URL range from being overwritten during chart initialization
  let initialLoadComplete = false;

  let visibleBarsCount = initialRange
    ? initialRange.to - initialRange.from
    : Infinity;

  /** @param {number} count */
  const getDotsRadius = (count) =>
    count > 1000 ? 1 : count > 200 ? 1.5 : count > 100 ? 2 : 3;

  /** @type {Set<ZoomChangeCallback>} */
  const onZoomChange = new Set();

  const debouncedSetRange = debounce((/** @type {Range | null} */ range) => {
    if (!initialLoadComplete) return;
    if (range && range.from < range.to) {
      setRange({ from: range.from, to: range.to });
    }
  }, 100);
  index.onChange.add(() => debouncedSetRange.cancel());

  const throttledZoom = throttle((/** @type {Range} */ range) => {
    if (!initialLoadComplete) return;
    const count = range.to - range.from;
    if (count === visibleBarsCount) return;
    visibleBarsCount = count;
    onZoomChange.forEach((cb) => cb(count));
  }, 100);

  ichart.timeScale().subscribeVisibleLogicalRangeChange((range) => {
    if (!range) return;
    throttledZoom(range);
    debouncedSetRange(range);
  });

  function applyColors() {
    const defaultColor = colors.default();
    const offColor = colors.gray();
    const borderColor = colors.border();
    const offBorderColor = colors.offBorder();
    ichart.applyOptions({
      layout: {
        textColor: offColor,
        panes: {
          separatorColor: borderColor,
        },
      },
      crosshair: {
        horzLine: {
          color: offColor,
          labelBackgroundColor: defaultColor,
        },
        vertLine: {
          color: offColor,
          labelBackgroundColor: defaultColor,
        },
      },
      grid: {
        horzLines: {
          color: offBorderColor,
        },
        vertLines: {
          color: offBorderColor,
        },
      },
    });
  }
  applyColors();
  const removeThemeListener = onThemeChange(applyColors);

  /** @type {Partial<Record<ChartableIndex, number>>} */
  const minBarSpacingByIndex = {
    month1: 1,
    month3: 2,
    month6: 3,
    year1: 6,
    year10: 60,
  };

  /** @param {ChartableIndex} index */
  function applyIndexSettings(index) {
    const minBarSpacing = minBarSpacingByIndex[index] ?? 0.5;

    ichart.applyOptions({
      timeScale: {
        timeVisible:
          index === "height" ||
          index === "epoch" ||
          index === "halving" ||
          index.startsWith("minute") ||
          index.startsWith("hour"),
        ...(!fitContent
          ? {
              minBarSpacing,
            }
          : {}),
      },
    });
  }
  applyIndexSettings(index.get());
  index.onChange.add(applyIndexSettings);

  // Periodic refresh of active series data
  const refreshInterval = setInterval(() => serieses.refreshAll(), 30_000);
  const onVisibilityChange = () => {
    if (!document.hidden) serieses.refreshAll();
  };
  document.addEventListener("visibilitychange", onVisibilityChange);

  if (fitContent) {
    new ResizeObserver(() => ichart.timeScale().fitContent()).observe(chartEl);
  }

  const panes = {
    initialized: false,

    /**
     * @param {number} paneIndex
     * @param {(pane: IPaneApi<Time>, parent: ChildNode) => void} callback
     * @param {number} [retries]
     */
    whenReady(paneIndex, callback, retries = 10) {
      const pane = ichart.panes().at(paneIndex);
      const parent = pane?.getHTMLElement()?.children?.item(1)?.firstChild;
      if (pane && parent) {
        callback(pane, parent);
      } else if (retries > 0) {
        requestAnimationFrame(() =>
          this.whenReady(paneIndex, callback, retries - 1),
        );
      }
    },

    setup() {
      if (this.initialized) return;
      this.initialized = true;

      for (let i = 0; i < ichart.panes().length; i++) {
        this.whenReady(i, (pane, parent) => {
          parent.appendChild(legends[i].element);
          injectScaleSelector(i, pane);
          applyScaleForUnit(i);
          this.updateSize(i);
        });
      }
    },

    /** @param {number} paneIndex */
    isAllHidden(paneIndex) {
      for (const s of serieses.all) {
        if (s.paneIndex === paneIndex && s.active.value) return false;
      }
      return true;
    },

    /** @param {number} paneIndex */
    updateSize(paneIndex) {
      const pane = ichart.panes().at(paneIndex);
      if (!pane) return;
      if (this.isAllHidden(paneIndex)) {
        const collapsedHeight = paneIndex === 0 ? 32 : 64;
        const chartHeight = ichart.chartElement().clientHeight;
        pane.setStretchFactor(
          chartHeight > 0
            ? collapsedHeight / (chartHeight - collapsedHeight)
            : 0,
        );
      } else {
        pane.setStretchFactor(1);
      }
    },
  };

  const serieses = {
    /** @type {Set<AnySeries>} */
    all: new Set(),

    refreshAll() {
      time.fetch();
      serieses.all.forEach((s) => {
        if (s.active.value) s.fetch?.();
      });
    },

    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {Color[]} args.colors
     * @param {AnySeriesPattern} args.source
     * @param {number} args.paneIndex
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {boolean} [args.defaultActive]
     * @param {(order: number) => void} args.setOrder
     * @param {(active: boolean, highlighted: boolean) => void} args.applyOptions
     * @param {() => readonly AnyChartData[]} args.getData
     * @param {(data: AnyChartData[]) => void} args.setData
     * @param {(data: AnyChartData) => void} args.update
     * @param {() => void} args.onRemove
     */
    create({
      source,
      name,
      order,
      paneIndex,
      unit,
      key: customKey,
      defaultActive,
      colors,
      setOrder,
      applyOptions,
      getData,
      setData,
      update,
      onRemove,
    }) {
      const key = customKey ?? stringToId(name);
      const id = `${unit.id}-${key}`;

      const active = createPersistedValue({
        defaultValue: defaultActive ?? true,
        storageKey: `${storageId}-p${paneIndex}-${key}`,
        urlKey: `${paneIndex === 0 ? "t" : "b"}-${key}`,
        ...serdeBool,
      });

      setOrder(-order);

      let highlighted = true;
      function refresh() {
        applyOptions(active.value, highlighted);
      }
      refresh();
      const removeThemeListener = onThemeChange(refresh);

      const seriesGeneration = generation;
      const state = {
        hasData: false,
        lastTime: -Infinity,
        /** @type {string | null} */
        lastStamp: null,
        /** @type {string | null} */
        lastTimeStamp: null,
        /** @type {number | null} */
        lastVersion: null,
        /** @type {number | null} */
        lastTimeVersion: null,
        /** @type {VoidFunction | null} */
        fetch: null,
        /** @type {((data: SeriesData<number>) => void) | null} */
        onTime: null,
        reset() {
          this.hasData = false;
          this.lastTime = -Infinity;
          this.lastStamp = null;
          this.lastTimeStamp = null;
          this.lastVersion = null;
          this.lastTimeVersion = null;
        },
        /**
         * @param {string | null} valuesStamp
         * @param {string} timeStamp
         * @param {number | null} valuesVersion
         * @param {number} timeVersion
         */
        shouldProcess(valuesStamp, timeStamp, valuesVersion, timeVersion) {
          if (
            valuesStamp === this.lastStamp &&
            timeStamp === this.lastTimeStamp
          )
            return false;
          // Version change means data was recomputed, needs full reload
          if (
            valuesVersion !== this.lastVersion ||
            timeVersion !== this.lastTimeVersion
          ) {
            this.hasData = false;
            this.lastTime = -Infinity;
          }
          this.lastStamp = valuesStamp;
          this.lastTimeStamp = timeStamp;
          this.lastVersion = valuesVersion;
          this.lastTimeVersion = timeVersion;
          return true;
        },
      };

      /** @type {AnySeries} */
      const series = {
        active,
        setActive(value) {
          const wasActive = active.value;
          active.set(value);
          refresh();
          if (value && !wasActive) {
            state.fetch?.();
          }
          panes.updateSize(paneIndex);
        },
        setOrder,
        highlight() {
          if (highlighted) return;
          highlighted = true;
          refresh();
        },
        tame() {
          if (!highlighted) return;
          highlighted = false;
          refresh();
        },
        refresh,
        generation: seriesGeneration,
        hasData: () => state.hasData,
        fetch: () => state.fetch?.(),
        id,
        paneIndex,
        url: null,
        getData,
        update,
        remove() {
          if (state.onTime) time.callbacks.delete(state.onTime);
          removeThemeListener();
          onRemove();
          serieses.all.delete(series);
        },
      };

      serieses.all.add(series);

      /** @param {ChartableIndex} idx */
      function setupIndexEffect(idx) {
        // Reset data state for new index
        state.reset();
        state.fetch = null;

        const _valuesEndpoint = source.by[idx];
        // Gracefully skip - source may be about to be removed by option change
        if (!_valuesEndpoint) return;
        const valuesEndpoint = _valuesEndpoint;

        series.url = `${baseUrl}${valuesEndpoint.path}`;

        legends[paneIndex].addOrReplace({
          series,
          name,
          colors,
          order,
        });

        /**
         * @param {number[]} indexes
         * @param {(number | boolean | null | [number, number, number, number])[]} values
         */
        function processData(indexes, values) {
          const length = Math.min(indexes.length, values.length);

          // Find start index for processing
          let startIdx = 0;
          if (state.hasData) {
            // Binary search to find first index where time >= state.lastTime
            let lo = 0;
            let hi = length;
            while (lo < hi) {
              const mid = (lo + hi) >>> 1;
              if (indexes[mid] < state.lastTime) {
                lo = mid + 1;
              } else {
                hi = mid;
              }
            }
            startIdx = lo;
            if (startIdx >= length) return; // No new data
          }

          /**
           * @param {number} i
           * @returns {LineData | CandlestickData}
           */
          function buildDataPoint(i) {
            const time = /** @type {Time} */ (indexes[i]);
            const v = values[i];
            if (v === null) {
              return { time, value: NaN };
            } else if (typeof v === "number") {
              return { time, value: v };
            } else if (typeof v === "boolean") {
              return { time, value: Number(v) };
            } else {
              if (!Array.isArray(v) || v.length !== 4)
                throw new Error(`Expected OHLC tuple, got: ${v}`);
              const [open, high, low, close] = v;
              return { time, open, high, low, close };
            }
          }

          if (!state.hasData) {
            // Initial load: build full array
            const data = /** @type {LineData[] | CandlestickData[]} */ (
              Array.from({ length })
            );

            let prevTime = null;
            let timeOffset = 0;

            for (let i = 0; i < length; i++) {
              const time = indexes[i];
              const sameTime = prevTime === time;
              if (sameTime) {
                timeOffset += 1;
              }
              const offsetedI = i - timeOffset;
              const point = buildDataPoint(i);
              if (sameTime && "open" in point) {
                const prev = /** @type {CandlestickData} */ (data[offsetedI]);
                point.open = prev.open;
                point.high = Math.max(prev.high, point.high);
                point.low = Math.min(prev.low, point.low);
              }
              data[offsetedI] = point;
              prevTime = time;
            }

            data.length -= timeOffset;

            setData(data);
            state.hasData = true;
            state.lastTime =
              /** @type {number} */ (data.at(-1)?.time) ?? -Infinity;

            // Restore saved range or use defaults
            // RAF for Safari compatibility
            requestAnimationFrame(() => {
              if (seriesGeneration !== generation) return;
              const savedRange = getRange();
              if (savedRange) {
                ichart.timeScale().setVisibleLogicalRange({
                  from: savedRange.from,
                  to: savedRange.to,
                });
              } else if (fitContent) {
                ichart.timeScale().fitContent();
              } else if (
                (minBarSpacingByIndex[idx] ?? 0) >=
                /** @type {number} */ (minBarSpacingByIndex.month3)
              ) {
                ichart
                  .timeScale()
                  .setVisibleLogicalRange({ from: -1, to: data.length });
              }
              // Delay until chart has applied the range
              requestAnimationFrame(() => {
                if (seriesGeneration !== generation) return;
                initialLoadComplete = true;
                blueprints.onDataLoaded?.();
              });
            });
          } else {
            // Incremental update: only process new data points
            for (let i = startIdx; i < length; i++) {
              const point = buildDataPoint(i);
              update(point);
              state.lastTime = /** @type {number} */ (point.time);
            }
          }
        }

        async function fetchAndProcess() {
          /** @type {SeriesData<number> | null} */
          let timeData = null;
          /** @type {(number | boolean | null | [number, number, number, number])[] | null} */
          let valuesData = null;
          /** @type {string | null} */
          let valuesStamp = null;
          /** @type {number | null} */
          let valuesVersion = null;

          function tryProcess() {
            if (seriesGeneration !== generation) return;
            if (!timeData || !valuesData) return;
            if (
              !state.shouldProcess(
                valuesStamp,
                timeData.stamp,
                valuesVersion,
                timeData.version,
              )
            )
              return;
            if (timeData.data.length && valuesData.length) {
              processData(timeData.data, valuesData);
            }
          }

          // Register for shared time data
          state.onTime = (result) => {
            timeData = result;
            tryProcess();
          };
          time.callbacks.add(state.onTime);
          if (time.data) state.onTime(time.data);

          const cachedValues = cache.get(valuesEndpoint.path);
          if (cachedValues) {
            valuesData = cachedValues.data;
            valuesStamp = cachedValues.stamp;
            valuesVersion = cachedValues.version;
            tryProcess();
          }
          await valuesEndpoint.slice(-MAX_SIZE).fetch((result) => {
            cache.set(valuesEndpoint.path, result);
            valuesData = result.data;
            valuesStamp = result.stamp;
            valuesVersion = result.version;
            tryProcess();
          });
        }

        state.fetch = fetchAndProcess;

        // Initial fetch if active
        if (active.value) {
          fetchAndProcess();
        }
      }

      setupIndexEffect(index.get());

      return series;
    },

    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {AnySeriesPattern} args.source
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {number} [args.paneIndex]
     * @param {[Color, Color]} [args.colors] - [upColor, downColor] for legend
     * @param {boolean} [args.defaultActive]
     * @param {CandlestickSeriesPartialOptions} [args.options]
     */
    addCandlestick({
      source,
      name,
      key,
      order,
      unit,
      paneIndex = 0,
      colors: customColors,
      defaultActive,
      options,
    }) {
      const upColor = customColors?.[0] ?? colors.background;
      const downColor = customColors?.[1] ?? colors.default;

      const candlestickISeries = /** @type {CandlestickISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Candlestick'>} */ (CandlestickSeries),
          { visible: false, borderVisible: true, ...options },
          paneIndex,
        )
      );

      const lineISeries = /** @type {LineISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Line'>} */ (LineSeries),
          { visible: false, lineWidth, priceLineVisible: true },
          paneIndex,
        )
      );

      /** @param {number} barCount */
      const shouldShowLine = (barCount) => barCount > 500;
      let showLine = shouldShowLine(visibleBarsCount);

      /** @type {ZoomChangeCallback} */
      function handleZoom(count) {
        if (!series.hasData()) return;
        const newShowLine = shouldShowLine(count);
        if (newShowLine === showLine) return;
        showLine = newShowLine;
        series.refresh();
      }

      const series = serieses.create({
        colors: [upColor, downColor],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder(order) {
          candlestickISeries.setSeriesOrder(order);
          lineISeries.setSeriesOrder(order);
        },
        applyOptions(active, highlighted) {
          candlestickISeries.applyOptions({
            visible: active && !showLine,
            lastValueVisible: highlighted,
            priceLineColor: colors.default.highlight(highlighted),
            upColor: upColor.highlight(highlighted),
            downColor: downColor.highlight(highlighted),
            wickUpColor: colors.default.highlight(highlighted),
            wickDownColor: colors.default.highlight(highlighted),
            borderUpColor: colors.default.highlight(highlighted),
            borderDownColor: colors.default.highlight(highlighted),
          });
          lineISeries.applyOptions({
            visible: active && showLine,
            lastValueVisible: highlighted,
            color: colors.default.highlight(highlighted),
          });
        },
        setData: (data) => {
          const cdata = /** @type {CandlestickData[]} */ (data);
          candlestickISeries.setData(cdata);
          lineISeries.setData(
            cdata.map((d) => ({ time: d.time, value: d.close })),
          );
          requestAnimationFrame(() => {
            if (generation !== series.generation) return;
            showLine = shouldShowLine(visibleBarsCount);
            series.refresh();
          });
        },
        update: (data) => {
          const cd = /** @type {CandlestickData} */ (data);
          candlestickISeries.update(cd);
          lineISeries.update({ time: cd.time, value: cd.close });
        },
        getData: () => candlestickISeries.data(),
        onRemove: () => {
          onZoomChange.delete(handleZoom);
          ichart.removeSeries(candlestickISeries);
          ichart.removeSeries(lineISeries);
        },
      });

      onZoomChange.add(handleZoom);
      return series;
    },
    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {AnySeriesPattern} args.source
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {Color | [Color, Color]} [args.color] - Single color or [positive, negative] colors
     * @param {(value: number) => Color} [args.colorFn]
     * @param {number} [args.paneIndex]
     * @param {boolean} [args.defaultActive]
     * @param {HistogramSeriesPartialOptions} [args.options]
     */
    addHistogram({
      source,
      name,
      key,
      color = colors.bi.p1,
      colorFn,
      order,
      unit,
      paneIndex = 0,
      defaultActive,
      options,
    }) {
      const isDualColor = Array.isArray(color);
      const positiveColor = isDualColor ? color[0] : color;
      const negativeColor = isDualColor ? color[1] : color;

      const iseries = /** @type {HistogramISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Histogram'>} */ (HistogramSeries),
          { priceLineVisible: false, ...options },
          paneIndex,
        )
      );

      return serieses.create({
        colors: isDualColor ? [positiveColor, negativeColor] : [positiveColor],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder: (order) => iseries.setSeriesOrder(order),
        applyOptions(active, highlighted) {
          iseries.applyOptions({
            visible: active,
            lastValueVisible: highlighted,
            color: positiveColor.highlight(highlighted),
          });
        },
        setData: (data) => {
          if (colorFn) {
            iseries.setData(
              data.map((d) => ({
                ...d,
                color:
                  "value" in d
                    ? (colorFn(d.value) ?? (() => "transparent"))()
                    : "transparent",
              })),
            );
          } else if (isDualColor) {
            iseries.setData(
              data.map((d) => ({
                ...d,
                color:
                  "value" in d && d.value >= 0
                    ? positiveColor()
                    : negativeColor(),
              })),
            );
          } else {
            iseries.setData(data);
          }
        },
        update: (data) => iseries.update(data),
        getData: () => iseries.data(),
        onRemove: () => ichart.removeSeries(iseries),
      });
    },
    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {AnySeriesPattern} args.source
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {Color} args.color
     * @param {(value: number) => Color} [args.colorFn]
     * @param {number} [args.paneIndex]
     * @param {boolean} [args.defaultActive]
     * @param {LineSeriesPartialOptions} [args.options]
     */
    addLine({
      source,
      name,
      key,
      order,
      color,
      colorFn,
      unit,
      paneIndex = 0,
      defaultActive,
      options,
    }) {
      const iseries = /** @type {LineISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Line'>} */ (LineSeries),
          { lineWidth, priceLineVisible: false, ...options },
          paneIndex,
        )
      );

      const showLastValue = options?.lastValueVisible !== false;

      return serieses.create({
        colors: [color],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder: (order) => iseries.setSeriesOrder(order),
        applyOptions(active, highlighted) {
          iseries.applyOptions({
            visible: active,
            lastValueVisible: showLastValue && highlighted,
            color: color.highlight(highlighted),
          });
        },
        setData: (data) => {
          if (colorFn) {
            iseries.setData(
              data.map((d) => ({
                ...d,
                color: "value" in d ? (colorFn(d.value) ?? color)() : color(),
              })),
            );
          } else {
            iseries.setData(data);
          }
        },
        update: (data) => iseries.update(data),
        getData: () => iseries.data(),
        onRemove: () => ichart.removeSeries(iseries),
      });
    },
    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {AnySeriesPattern} args.source
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {Color} args.color
     * @param {number} [args.paneIndex]
     * @param {boolean} [args.defaultActive]
     * @param {LineSeriesPartialOptions} [args.options]
     */
    addDots({
      source,
      name,
      key,
      order,
      color,
      unit,
      paneIndex = 0,
      defaultActive,
      options,
    }) {
      const iseries = /** @type {LineISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Line'>} */ (LineSeries),
          {
            priceLineVisible: false,
            lineVisible: false,
            pointMarkersVisible: true,
            pointMarkersRadius: 1,
            ...options,
          },
          paneIndex,
        )
      );

      let radius = getDotsRadius(visibleBarsCount);

      /** @type {ZoomChangeCallback} */
      function handleZoom(count) {
        if (!series.hasData()) return;
        const newRadius = getDotsRadius(count);
        if (newRadius === radius) return;
        radius = newRadius;
        series.refresh();
      }

      const series = serieses.create({
        colors: [color],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder: (order) => iseries.setSeriesOrder(order),
        applyOptions(active, highlighted) {
          iseries.applyOptions({
            visible: active,
            lastValueVisible: highlighted,
            color: color.highlight(highlighted),
            pointMarkersRadius: radius,
          });
        },
        setData: (data) => iseries.setData(data),
        update: (data) => iseries.update(data),
        getData: () => iseries.data(),
        onRemove: () => {
          onZoomChange.delete(handleZoom);
          ichart.removeSeries(iseries);
        },
      });

      onZoomChange.add(handleZoom);
      return series;
    },
    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {number} args.order
     * @param {AnySeriesPattern} args.source
     * @param {Unit} args.unit
     * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
     * @param {number} [args.paneIndex]
     * @param {boolean} [args.defaultActive]
     * @param {Color} [args.topColor]
     * @param {Color} [args.bottomColor]
     * @param {BaselineSeriesPartialOptions} [args.options]
     */
    addBaseline({
      source,
      name,
      key,
      order,
      unit,
      paneIndex = 0,
      defaultActive,
      topColor = colors.bi.p1[0],
      bottomColor = colors.bi.p1[1],
      options,
    }) {
      const iseries = /** @type {BaselineISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Baseline'>} */ (BaselineSeries),
          {
            lineWidth,
            baseValue: {
              price: options?.baseValue?.price ?? 0,
            },
            ...options,
            priceLineVisible: false,
            bottomFillColor1: "transparent",
            bottomFillColor2: "transparent",
            topFillColor1: "transparent",
            topFillColor2: "transparent",
            lineVisible: true,
          },
          paneIndex,
        )
      );

      return serieses.create({
        colors: [topColor, bottomColor],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder: (order) => iseries.setSeriesOrder(order),
        applyOptions(active, highlighted) {
          iseries.applyOptions({
            visible: active,
            lastValueVisible: highlighted,
            topLineColor: topColor.highlight(highlighted),
            bottomLineColor: bottomColor.highlight(highlighted),
          });
        },
        setData: (data) => iseries.setData(data),
        update: (data) => iseries.update(data),
        getData: () => iseries.data(),
        onRemove: () => ichart.removeSeries(iseries),
      });
    },

    /**
     * Add a DotsBaseline series (baseline with point markers instead of line)
     * @param {Object} args
     * @param {AnySeriesPattern} args.source
     * @param {string} args.name
     * @param {string} [args.key]
     * @param {number} args.order
     * @param {Unit} args.unit
     * @param {number} [args.paneIndex]
     * @param {boolean} [args.defaultActive]
     * @param {Color} [args.topColor]
     * @param {Color} [args.bottomColor]
     * @param {BaselineSeriesPartialOptions} [args.options]
     */
    addDotsBaseline({
      source,
      name,
      key,
      order,
      unit,
      paneIndex = 0,
      defaultActive,
      topColor = colors.bi.p1[0],
      bottomColor = colors.bi.p1[1],
      options,
    }) {
      const iseries = /** @type {BaselineISeries} */ (
        ichart.addSeries(
          /** @type {SeriesDefinition<'Baseline'>} */ (BaselineSeries),
          {
            lineWidth,
            baseValue: {
              price: options?.baseValue?.price ?? 0,
            },
            ...options,
            priceLineVisible: false,
            bottomFillColor1: "transparent",
            bottomFillColor2: "transparent",
            topFillColor1: "transparent",
            topFillColor2: "transparent",
            lineVisible: false,
            pointMarkersVisible: true,
            pointMarkersRadius: 1,
          },
          paneIndex,
        )
      );

      let radius = getDotsRadius(visibleBarsCount);

      /** @type {ZoomChangeCallback} */
      function handleZoom(count) {
        if (!series.hasData()) return;
        const newRadius = getDotsRadius(count);
        if (newRadius === radius) return;
        radius = newRadius;
        series.refresh();
      }

      const series = serieses.create({
        colors: [topColor, bottomColor],
        name,
        key,
        order,
        paneIndex,
        unit,
        defaultActive,
        source,
        setOrder: (order) => iseries.setSeriesOrder(order),
        applyOptions(active, highlighted) {
          iseries.applyOptions({
            visible: active,
            lastValueVisible: highlighted,
            topLineColor: topColor.highlight(highlighted),
            bottomLineColor: bottomColor.highlight(highlighted),
            pointMarkersRadius: radius,
          });
        },
        setData: (data) => iseries.setData(data),
        update: (data) => iseries.update(data),
        getData: () => iseries.data(),
        onRemove: () => {
          onZoomChange.delete(handleZoom);
          ichart.removeSeries(iseries);
        },
      });

      onZoomChange.add(handleZoom);
      return series;
    },
  };

  /** @param {number} paneIndex */
  function applyScaleForUnit(paneIndex) {
    const pane = ichart.panes().at(paneIndex);
    if (!pane) return;
    const persisted = scalePersistedValues[paneIndex];
    if (!persisted) return;
    try {
      pane.priceScale("right").applyOptions({
        mode: persisted.value === "lin" ? 0 : 1,
      });
    } catch {}
  }

  /** @type {Record<number, ReturnType<typeof createPersistedValue<"lin" | "log">>>} */
  const scalePersistedValues = {};
  /** @type {Record<number, HTMLElement | undefined>} */
  const scaleSelectorElements = {};

  /**
   * @param {number} paneIndex
   * @param {IPaneApi<Time>} pane
   */
  function injectScaleSelector(paneIndex, pane) {
    const id = `${storageId}-scale`;
    const defaultValue = paneIndex === 0 ? "log" : "lin";

    let persisted = scalePersistedValues[paneIndex];
    if (!persisted) {
      persisted = createPersistedValue({
        defaultValue: /** @type {"lin" | "log"} */ (defaultValue),
        storageKey: `${storageId}-p${paneIndex}-scale`,
        urlKey: paneIndex === 0 ? "price_scale" : "unit_scale",
        serialize: (v) => v,
        deserialize: (s) => /** @type {"lin" | "log"} */ (s),
      });
      scalePersistedValues[paneIndex] = persisted;
    }

    // Inject into the price scale td (last td in the pane's tr)
    const paneEl = pane.getHTMLElement();
    const tr = paneEl?.closest("tr");
    const td = tr?.querySelector("td:last-child");
    if (!td) return;

    scaleSelectorElements[paneIndex]?.remove();

    /** @type {HTMLTableCellElement} */ (td).style.position = "relative";

    const radios = createRadios({
      choices: /** @type {const} */ (["lin", "log"]),
      id: stringToId(`${id} ${paneIndex}`),
      initialValue: persisted.value,
      onChange(value) {
        persisted.set(value);
        applyScaleForUnit(paneIndex);
      },
      toTitle: (c) => (c === "lin" ? "Linear scale" : "Logarithmic scale"),
    });
    scaleSelectorElements[paneIndex] = radios;
    td.append(radios);
  }

  const blueprints = {
    /** @type {{ map: Map<Unit, AnyFetchedSeriesBlueprint[]>, series: AnySeries[], unit: Unit | null }[]} */
    panes: [
      { map: new Map(), series: [], unit: null },
      { map: new Map(), series: [], unit: null },
    ],

    /** @type {VoidFunction | undefined} */
    onDataLoaded: undefined,

    /** @param {number} paneIndex */
    rebuildPane(paneIndex) {
      const pane = this.panes[paneIndex];
      const { map, series, unit } = pane;
      const legend = legends[paneIndex];

      if (!unit) {
        series.forEach((s) => s.remove());
        pane.series = [];
        legend.removeFrom(0);
        return;
      }

      const idx = index.get();
      legend.removeFrom(0);

      // Store old series to remove AFTER adding new ones
      // This prevents pane collapse which loses scale settings
      const oldSeries = [...series];
      pane.series = [];

      const defaultColor = unit === Unit.usd ? colors.usd : colors.bitcoin;

      map.get(unit)?.forEach((blueprint, order) => {
        if (!Object.keys(blueprint.series.by).includes(idx)) return;

        const common = {
          source: blueprint.series,
          name: blueprint.title,
          key: blueprint.key,
          defaultActive: blueprint.defaultActive,
          options: blueprint.options,
          paneIndex,
          unit,
          order,
        };

        switch (blueprint.type) {
          case "Baseline":
            pane.series.push(
              serieses.addBaseline({
                ...common,
                topColor: blueprint.colors?.[0] ?? blueprint.color,
                bottomColor: blueprint.colors?.[1] ?? blueprint.color,
              }),
            );
            break;
          case "DotsBaseline":
            pane.series.push(
              serieses.addDotsBaseline({
                ...common,
                topColor: blueprint.colors?.[0] ?? blueprint.color,
                bottomColor: blueprint.colors?.[1] ?? blueprint.color,
              }),
            );
            break;
          case "Histogram":
            pane.series.push(
              serieses.addHistogram({
                ...common,
                color: blueprint.color,
                colorFn: blueprint.colorFn,
              }),
            );
            break;
          case "Candlestick":
            pane.series.push(
              serieses.addCandlestick({ ...common, colors: blueprint.colors }),
            );
            break;
          case "Price":
            if (idx === "height" || idx.startsWith("minute")) {
              pane.series.push(
                serieses.addLine({
                  ...common,
                  color: colors.default,
                  options: { ...common.options, priceLineVisible: true },
                }),
              );
            } else {
              pane.series.push(
                serieses.addCandlestick({
                  ...common,
                  source: blueprint.ohlcSeries,
                  colors: blueprint.colors,
                }),
              );
            }
            break;
          case "Dots":
            pane.series.push(
              serieses.addDots({
                ...common,
                color: blueprint.color ?? defaultColor,
              }),
            );
            break;
          case "Line":
          case undefined:
            pane.series.push(
              serieses.addLine({
                ...common,
                color: blueprint.color ?? defaultColor,
                colorFn: blueprint.colorFn,
              }),
            );
        }
      });

      // Remove old series AFTER adding new ones to prevent pane collapse
      oldSeries.forEach((s) => s.remove());
    },

    rebuild() {
      generation++;
      initialLoadComplete = false; // Reset to prevent saving stale ranges during load
      panes.initialized = false;
      time.setIndex(index.get());
      time.fetch();
      this.rebuildPane(0);
      this.rebuildPane(1);
      requestAnimationFrame(() => panes.setup());
    },
  };

  // Rebuild when index changes
  index.onChange.add(() => blueprints.rebuild());

  // Index selector + range presets
  let preferredIndex = index.name.value;
  /** @type {HTMLElement | null} */
  let indexField = null;

  /** @param {RangePreset} preset */
  function applyPreset(preset) {
    preferredIndex = preset.index;
    /** @type {HTMLSelectElement} */ (getElementById("index")).value =
      preset.index;
    index.name.set(preset.index);

    const targetGen = generation;
    const waitAndApply = () => {
      if (generation !== targetGen) return;
      if (!initialLoadComplete) {
        requestAnimationFrame(waitAndApply);
        return;
      }
      const data = blueprints.panes[0].series[0]?.getData();
      if (!data?.length) return;
      const fi = data.findIndex(
        (d) => /** @type {number} */ (d.time) >= preset.from,
      );
      const from = fi === -1 ? 0 : fi;
      const padding = Math.round((data.length - from) * 0.025);
      ichart.timeScale().setVisibleLogicalRange({
        from: from - padding,
        to: data.length + padding,
      });
    };
    requestAnimationFrame(waitAndApply);
  }

  const chart = {
    get panes() {
      return blueprints.panes;
    },

    /** @param {{ choices: IndexLabel[], groups: { label: string, items: IndexLabel[] }[] }} arg */
    setIndexChoices({ choices, groups }) {
      if (indexField) indexField.remove();

      let currentValue = choices.includes(preferredIndex)
        ? preferredIndex
        : (choices[0] ?? "1d");

      if (currentValue !== index.name.value) {
        index.name.set(currentValue);
      }

      const legend = createLegend();
      indexField = legend.element;

      legend.setPrefix(
        createSelect({
          initialValue: currentValue,
          onChange: (v) => {
            preferredIndex = v;
            index.name.set(v);
          },
          choices,
          groups,
          id: "index",
        }).element,
      );

      for (const preset of getRangePresets()) {
        const btn = window.document.createElement("button");
        btn.textContent = preset.label;
        btn.title = `${preset.label} at ${preset.index} interval`;
        btn.addEventListener("click", () => applyPreset(preset));
        legend.scroller.append(btn);
      }

      chartEl.append(indexField);
    },

    /**
     * @param {Object} args
     * @param {string} args.name
     * @param {Map<Unit, AnyFetchedSeriesBlueprint[]>} args.top
     * @param {Map<Unit, AnyFetchedSeriesBlueprint[]>} args.bottom
     * @param {VoidFunction} [args.onDataLoaded]
     */
    setBlueprints({ name, top, bottom, onDataLoaded }) {
      storageId = stringToId(name);
      blueprints.panes[0].map = top;
      blueprints.panes[1].map = bottom;
      blueprints.onDataLoaded = onDataLoaded;

      // Set up unit selectors for each pane
      [top, bottom].forEach((map, paneIndex) => {
        const units = Array.from(map.keys());
        if (!units.length) {
          blueprints.panes[paneIndex].unit = null;
          legends[paneIndex].clearPrefix();
          return;
        }

        const defaultUnit = units[0];
        const persistedUnit = createPersistedValue({
          defaultValue: /** @type {string} */ (defaultUnit.id),
          storageKey: `${storageId}-p${paneIndex}-unit`,
          urlKey: paneIndex === 0 ? "u0" : "u1",
          serialize: (v) => v,
          deserialize: (s) => s,
        });

        // Find unit matching persisted value, or use default
        const initialUnit =
          units.find((u) => u.id === persistedUnit.value) ?? defaultUnit;
        blueprints.panes[paneIndex].unit = initialUnit;

        legends[paneIndex].setPrefix(
          createSelect({
            choices: units,
            id: `pane-${paneIndex}-unit`,
            initialValue: blueprints.panes[paneIndex].unit ?? defaultUnit,
            toKey: (u) => u.id,
            toLabel: (u) => u.name,
            sorted: true,
            onChange(unit) {
              generation++;
              persistedUnit.set(unit.id);
              blueprints.panes[paneIndex].unit = unit;
              blueprints.rebuildPane(paneIndex);
            },
          }).element,
        );
      });

      blueprints.rebuild();
    },

    destroy() {
      debouncedSetRange.cancel();
      serieses.all.forEach((s) => s.remove());
      index.onChange.clear();
      onZoomChange.clear();
      removeThemeListener();
      clearInterval(refreshInterval);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      ichart.remove();
    },
  };

  const captureButton = document.createElement("button");
  captureButton.className = "capture";
  captureButton.innerText = "capture";
  captureButton.title = "Capture chart as image";
  captureButton.addEventListener("click", () => {
    capture({
      screenshot: ichart.takeScreenshot(),
      chartWidth: chartEl.clientWidth,
      chartElement: chartEl,
      parent,
      legends,
    });
  });
  chartEl.append(captureButton);

  if (!hintShown) {
    hintShown = true;
    const hint = document.createElement("div");
    hint.className = "chart-hint";
    hint.textContent = matchMedia("(pointer: coarse)").matches
      ? "pinch to zoom · swipe to pan"
      : "scroll to zoom · drag to pan";
    root.append(hint);

    const dismiss = () => hint.classList.add("done");
    chartEl.addEventListener("wheel", dismiss, { once: true, passive: true });
    chartEl.addEventListener("pointerdown", dismiss, { once: true });
  }

  return chart;
}

/**
 * @typedef {typeof createChart} CreateChart
 * @typedef {ReturnType<typeof createChart>} Chart
 */
