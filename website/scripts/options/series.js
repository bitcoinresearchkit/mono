/** Series helpers for creating chart series blueprints */

import { colors } from "../utils/colors.js";
import { Unit } from "../utils/units.js";

// ============================================================================
// Rolling window constants
// ============================================================================

/** @typedef {'_24h' | '_1w' | '_1m' | '_1y'} RollingWindowKey */

/** @type {ReadonlyArray<{key: RollingWindowKey, name: string, title: string, color: Color}>} */
export const ROLLING_WINDOWS = [
  { key: "_24h", name: "24h", title: "Daily", color: colors.time._24h },
  { key: "_1w", name: "1w", title: "Weekly", color: colors.time._1w },
  { key: "_1m", name: "1m", title: "Monthly", color: colors.time._1m },
  { key: "_1y", name: "1y", title: "Yearly", color: colors.time._1y },
];

/** @type {ReadonlyArray<{key: '_24h' | '_1w' | '_1m', name: string, title: string, color: Color}>} */
export const ROLLING_WINDOWS_TO_1M = [
  { key: "_24h", name: "24h", title: "Daily", color: colors.time._24h },
  { key: "_1w", name: "1w", title: "Weekly", color: colors.time._1w },
  { key: "_1m", name: "1m", title: "Monthly", color: colors.time._1m },
];

/**
 * Extract a series from each rolling window via a mapping function
 * @template T
 * @template {AnySeriesPattern} U
 * @param {{ _24h: T, _1w: T, _1m: T, _1y: T }} windows
 * @param {(v: T) => U} extract
 * @returns {{ _24h: U, _1w: U, _1m: U, _1y: U }}
 */
export function mapWindows(windows, extract) {
  return {
    _24h: extract(windows._24h),
    _1w: extract(windows._1w),
    _1m: extract(windows._1m),
    _1y: extract(windows._1y),
  };
}

// ============================================================================
// Price helper for top pane (auto-expands to USD + sats)
// ============================================================================

/**
 * Create a price series for the top pane (auto-expands to USD + sats versions)
 * @param {Object} args
 * @param {AnyPricePattern} args.series - Price pattern with usd and sats
 * @param {string} args.name
 * @param {string} [args.key]
 * @param {LineStyle} [args.style]
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {LineSeriesPartialOptions} [args.options]
 * @returns {FetchedPriceSeriesBlueprint}
 */
export function price({
  series,
  name,
  key,
  style,
  color,
  defaultActive,
  options,
}) {
  return {
    series,
    title: name,
    key,
    color,
    defaultActive,
    options: {
      lineStyle: style,
      ...options,
    },
  };
}

// ============================================================================
// Shared percentile helper
// ============================================================================

/**
 * Create percentile series (max/min/median/pct75/pct25/pct90/pct10) from any stats pattern
 * @param {Object} args
 * @param {DistributionStats} args.pattern
 * @param {Unit} args.unit
 * @param {string} [args.title]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
function percentileSeries({ pattern, unit, title = "" }) {
  const { stat } = colors;
  return [
    line({
      series: pattern.max,
      name: `${title} Max`.trim(),
      color: stat.max,
      unit,
    }),
    line({
      series: pattern.pct90,
      name: `${title} P90`.trim(),
      color: stat.pct90,
      unit,
    }),
    line({
      series: pattern.pct75,
      name: `${title} P75`.trim(),
      color: stat.pct75,
      unit,
    }),
    line({
      series: pattern.median,
      name: `${title} Median`.trim(),
      color: stat.median,
      unit,
    }),
    line({
      series: pattern.pct25,
      name: `${title} P25`.trim(),
      color: stat.pct25,
      unit,
    }),
    line({
      series: pattern.pct10,
      name: `${title} P10`.trim(),
      color: stat.pct10,
      unit,
    }),
    line({
      series: pattern.min,
      name: `${title} Min`.trim(),
      color: stat.min,
      unit,
    }),
  ];
}

/**
 * Create a Line series
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
 * @param {LineStyle} [args.style]
 * @param {Color} [args.color]
 * @param {(value: number) => Color} [args.colorFn]
 * @param {boolean} [args.defaultActive]
 * @param {LineSeriesPartialOptions} [args.options]
 * @returns {FetchedLineSeriesBlueprint}
 */
export function line({
  series,
  name,
  key,
  style,
  color,
  colorFn,
  defaultActive,
  unit,
  options,
}) {
  return {
    series,
    title: name,
    key,
    color,
    colorFn,
    unit,
    defaultActive,
    options: {
      lineStyle: style,
      ...options,
    },
  };
}

/**
 * @param {Omit<Parameters<typeof line>[0], 'style'>} args
 */
export function dotted(args) {
  const _args = /** @type {Parameters<typeof line>[0]} */ (args);
  _args.style = 1;
  return line(_args);
}

/**
 * @param {Omit<Parameters<typeof line>[0], 'style'>} args
 */
export function dashed(args) {
  const _args = /** @type {Parameters<typeof line>[0]} */ (args);
  _args.style = 2;
  return line(_args);
}

/**
 * @param {Omit<Parameters<typeof line>[0], 'style'>} args
 */
export function sparseDotted(args) {
  const _args = /** @type {Parameters<typeof line>[0]} */ (args);
  _args.style = 4;
  return line(_args);
}

/**
 * Create a Dots series (line with only point markers visible)
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {LineSeriesPartialOptions} [args.options]
 * @returns {FetchedDotsSeriesBlueprint}
 */
export function dots({
  series,
  name,
  key,
  color,
  defaultActive,
  unit,
  options,
}) {
  return {
    type: /** @type {const} */ ("Dots"),
    series,
    title: name,
    key,
    color,
    unit,
    defaultActive,
    options,
  };
}

/**
 * Create a Candlestick series
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
 * @param {[Color, Color]} [args.colors] - [upColor, downColor] for legend
 * @param {boolean} [args.defaultActive]
 * @param {CandlestickSeriesPartialOptions} [args.options]
 * @returns {FetchedCandlestickSeriesBlueprint}
 */
export function candlestick({
  series,
  name,
  key,
  defaultActive,
  unit,
  options,
}) {
  return {
    type: /** @type {const} */ ("Candlestick"),
    series,
    title: name,
    key,
    unit,
    defaultActive,
    options,
  };
}

/**
 * Create a Baseline series
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
 * @param {Color | [Color, Color]} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {number | undefined} [args.base]
 * @param {number} [args.style] - Line style (0: Solid, 1: Dotted, 2: Dashed, 3: LargeDashed, 4: SparseDotted)
 * @param {BaselineSeriesPartialOptions} [args.options]
 * @returns {FetchedBaselineSeriesBlueprint}
 */
export function baseline({
  series,
  name,
  key,
  color,
  defaultActive,
  unit,
  base,
  style,
  options,
}) {
  const isTuple = Array.isArray(color);
  return {
    type: /** @type {const} */ ("Baseline"),
    series,
    title: name,
    key,
    color: isTuple ? undefined : color,
    colors: isTuple ? color : undefined,
    unit,
    defaultActive,
    options: {
      baseValue: {
        price: base,
      },
      lineStyle: style,
      ...options,
    },
  };
}

/**
 * @param {Omit<Parameters<typeof baseline>[0], 'style'>} args
 */
export function dottedBaseline(args) {
  const _args = /** @type {Parameters<typeof baseline>[0]} */ (args);
  _args.style = 1;
  return baseline(_args);
}

/**
 * Baseline series rendered as dots (points only, no line)
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key]
 * @param {Color | [Color, Color]} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {number | undefined} [args.base]
 * @param {BaselineSeriesPartialOptions} [args.options]
 * @returns {FetchedDotsBaselineSeriesBlueprint}
 */
export function dotsBaseline({
  series,
  name,
  key,
  color,
  defaultActive,
  unit,
  base,
  options,
}) {
  const isTuple = Array.isArray(color);
  return {
    type: /** @type {const} */ ("DotsBaseline"),
    series,
    title: name,
    key,
    color: isTuple ? undefined : color,
    colors: isTuple ? color : undefined,
    unit,
    defaultActive,
    options: {
      baseValue: {
        price: base,
      },
      ...options,
    },
  };
}

/**
 * Create a Histogram series
 * @param {Object} args
 * @param {AnySeriesPattern} args.series
 * @param {string} args.name
 * @param {Unit} args.unit
 * @param {string} [args.key] - Optional key for persistence (derived from name if not provided)
 * @param {Color | [Color, Color]} [args.color]
 * @param {(value: number) => Color} [args.colorFn]
 * @param {boolean} [args.defaultActive]
 * @param {HistogramSeriesPartialOptions} [args.options]
 * @returns {FetchedHistogramSeriesBlueprint}
 */
export function histogram({
  series,
  name,
  key,
  color,
  colorFn,
  defaultActive,
  unit,
  options,
}) {
  return {
    type: /** @type {const} */ ("Histogram"),
    series,
    title: name,
    key,
    color,
    colorFn,
    unit,
    defaultActive,
    options,
  };
}

/**
 * Create series from an AverageHeightMaxMedianMinP10P25P75P90Pattern (height + rolling stats)
 * @param {Object} args
 * @param {{ height: AnySeriesPattern } & WindowedStats<AnySeriesPattern>} args.pattern - Pattern with .height and rolling stats
 * @param {string} args.window - Rolling window key (e.g., '_24h', '_1w', '_1m', '_1y')
 * @param {Unit} args.unit
 * @param {string} [args.title]
 * @param {Color} [args.baseColor]
 * @param {boolean} [args.avgActive]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function fromBaseStatsPattern({
  pattern,
  window,
  unit,
  title = "",
  baseColor,
}) {
  const stats = statsAtWindow(pattern, window);
  return [
    dots({
      series: pattern.height,
      name: title || "Base",
      color: baseColor,
      unit,
    }),
    ...percentileSeries({ pattern: stats, unit, title }),
  ];
}

/**
 * Extract stats at a specific rolling window
 * @template T
 * @param {WindowedStats<T>} pattern - Pattern with pct10/pct25/pct75/pct90 and median/max/min at each rolling window
 * @param {string} window
 * @returns {{ median: T, max: T, min: T, pct75: T, pct25: T, pct90: T, pct10: T }}
 */
export function statsAtWindow(pattern, window) {
  return {
    median: pattern.median[window],
    max: pattern.max[window],
    min: pattern.min[window],
    pct75: pattern.pct75[window],
    pct25: pattern.pct25[window],
    pct90: pattern.pct90[window],
    pct10: pattern.pct10[window],
  };
}

/**
 * Rolling folder tree with baseline series
 * @param {Object} args
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.windows
 * @param {string} args.title
 * @param {(w: typeof ROLLING_WINDOWS[number]) => string} args.windowTitle
 * @param {Unit} args.unit
 * @param {string} args.name
 * @param {string} args.legend
 * @returns {PartialOptionsGroup}
 */
function rollingWindowsTreeBaseline({
  windows,
  title,
  windowTitle,
  unit,
  name,
  legend,
}) {
  return {
    name,
    tree: [
      {
        name: "Compare",
        title,
        bottom: ROLLING_WINDOWS.map((w) =>
          baseline({
            series: windows[w.key],
            name: w.name,
            color: w.color,
            unit,
          }),
        ),
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: windowTitle(w),
        bottom: [baseline({ series: windows[w.key], name: legend, unit })],
      })),
    ],
  };
}

/**
 * Generic helper: compare + per-window sum+avg + cumulative.
 * @template P
 * @param {Object} args
 * @param {{ _24h: P, _1w: P, _1m: P, _1y: P }} args.sum
 * @param {{ _24h: P, _1w: P, _1m: P, _1y: P }} args.average
 * @param {P} args.cumulative
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Color} [args.color]
 * @param {(args: { pattern: P, name: string, color?: Color, defaultActive?: boolean }) => AnyFetchedSeriesBlueprint[]} args.series
 * @returns {PartialChartOption[]}
 */
export function sumsAndAveragesCumulativeWith({
  sum,
  average,
  cumulative,
  title = (s) => s,
  metric,
  color,
  series,
}) {
  return [
    {
      name: "Compare",
      title: title(`Average ${metric}`),
      bottom: ROLLING_WINDOWS.flatMap((w) =>
        series({
          pattern: average[w.key],
          name: w.name,
          color: w.color,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: [
        ...series({ pattern: sum[w.key], name: "Sum", color: w.color }),
        ...series({
          pattern: average[w.key],
          name: "Avg",
          color: w.color,
          defaultActive: false,
        }),
      ],
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: series({ pattern: cumulative, name: "All Time", color }),
    },
  ];
}

/**
 * Flat array of per-window charts with both sum (active) and average (off by default)
 * @param {Object} args
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.sum
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.average
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialChartOption[]}
 */
export function sumsAndAveragesArray({ sum, average, title = (s) => s, metric, unit }) {
  return ROLLING_WINDOWS.map((w) => ({
    name: w.name,
    title: title(`${w.title} ${metric}`),
    bottom: [
      line({ series: sum[w.key], name: "Sum", color: w.color, unit }),
      line({
        series: average[w.key],
        name: "Avg",
        color: w.color,
        unit,
        defaultActive: false,
      }),
    ],
  }));
}

/**
 * Compare + windowed sum+avg + cumulative (single unit)
 * @param {Object} args
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.sum
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.average
 * @param {AnySeriesPattern} args.cumulative
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {Color} [args.color]
 * @returns {PartialChartOption[]}
 */
export function sumsAndAveragesCumulative({
  sum,
  average,
  cumulative,
  title,
  metric,
  unit,
  color,
}) {
  return sumsAndAveragesCumulativeWith({
    sum,
    average,
    cumulative,
    title,
    metric,
    color,
    series: ({ pattern, name, color, defaultActive }) => [
      line({ series: pattern, name, color, unit, defaultActive }),
    ],
  });
}

/**
 * @param {Object} args
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.windows
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {string} [args.legend]
 * @returns {PartialOptionsGroup}
 */
export function sumsTreeBaseline({ windows, title = (s) => s, metric, unit, legend = "Sum" }) {
  return rollingWindowsTreeBaseline({
    windows,
    title: title(metric),
    windowTitle: (w) => title(`${w.title} ${metric}`),
    unit,
    name: "Sums",
    legend,
  });
}

/**
 * Rolling tree with baseline series for an amount pattern (sats + lazy btc per window).
 * @param {Object} args
 * @param {{ _24h: AmountPattern, _1w: AmountPattern, _1m: AmountPattern, _1y: AmountPattern }} args.windows
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {string} [args.legend]
 * @returns {PartialOptionsGroup}
 */
export function amountSumsTreeBaseline({ windows, title = (s) => s, metric, legend = "Sum" }) {
  return {
    name: "Sums",
    tree: [
      {
        name: "Compare",
        title: title(metric),
        bottom: ROLLING_WINDOWS.flatMap((w) => [
          baseline({ series: windows[w.key].btc, name: w.name, color: w.color, unit: Unit.btc }),
          baseline({ series: windows[w.key].sats, name: w.name, color: w.color, unit: Unit.sats }),
        ]),
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${metric}`),
        bottom: [
          baseline({ series: windows[w.key].btc, name: legend, unit: Unit.btc }),
          baseline({ series: windows[w.key].sats, name: legend, unit: Unit.sats }),
        ],
      })),
    ],
  };
}

/**
 * Flat array of per-window average charts
 * @param {Object} args
 * @param {{ _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} args.windows
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialChartOption[]}
 */
export function averagesArray({ windows, title = (s) => s, metric, unit }) {
  return [
    {
      name: "Compare",
      title: title(metric),
      bottom: ROLLING_WINDOWS.map((w) =>
        line({ series: windows[w.key], name: w.name, color: w.color, unit }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: [
        line({ series: windows[w.key], name: "Average", color: w.color, unit }),
      ],
    })),
  ];
}

/**
 * Create a Distribution folder tree with stats at each rolling window (24h/1w/1m/1y)
 * @param {Object} args
 * @param {WindowedStats<AnySeriesPattern>} args.pattern - Pattern with pct10/pct25/... and average/median/... at each rolling window
 * @param {AnySeriesPattern} [args.base] - Optional base series to show as dots on each chart
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsGroup}
 */
export function distributionWindowsTree({ pattern, base, title = (s) => s, metric, unit }) {
  return {
    name: "Distribution",
    tree: [
      {
        name: "Compare",
        title: title(`${metric} Distribution`),
        bottom: ROLLING_WINDOWS.map((w) =>
          line({
            series: pattern.median[w.key],
            name: w.name,
            color: w.color,
            unit,
          }),
        ),
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${metric} Distribution`),
        bottom: [
          ...(base ? [line({ series: base, name: "Base", unit })] : []),
          ...percentileSeries({ pattern: statsAtWindow(pattern, w.key), unit }),
        ],
      })),
    ],
  };
}

/**
 * Map a rolling window slot's stats to a specific unit, producing a stats-compatible pattern
 * @param {{ median: Record<string, AnySeriesPattern>, max: Record<string, AnySeriesPattern>, min: Record<string, AnySeriesPattern>, pct75: Record<string, AnySeriesPattern>, pct25: Record<string, AnySeriesPattern>, pct90: Record<string, AnySeriesPattern>, pct10: Record<string, AnySeriesPattern> }} slot - Rolling window slot with multi-currency stats
 * @param {BtcSatsUsdKey} unitKey
 */
function rollingSlotForUnit(slot, unitKey) {
  return {
    median: slot.median[unitKey],
    max: slot.max[unitKey],
    min: slot.min[unitKey],
    pct75: slot.pct75[unitKey],
    pct25: slot.pct25[unitKey],
    pct90: slot.pct90[unitKey],
    pct10: slot.pct10[unitKey],
  };
}

/**
 * Create distribution series for btc/sats/usd from a rolling window slot
 * @param {{ median: Record<string, AnySeriesPattern>, max: Record<string, AnySeriesPattern>, min: Record<string, AnySeriesPattern>, pct75: Record<string, AnySeriesPattern>, pct25: Record<string, AnySeriesPattern>, pct90: Record<string, AnySeriesPattern>, pct10: Record<string, AnySeriesPattern> }} slot - Rolling window slot with multi-currency stats
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export const distributionBtcSatsUsd = (slot) => [
  ...percentileSeries({
    pattern: rollingSlotForUnit(slot, "btc"),
    unit: Unit.btc,
  }),
  ...percentileSeries({
    pattern: rollingSlotForUnit(slot, "sats"),
    unit: Unit.sats,
  }),
  ...percentileSeries({
    pattern: rollingSlotForUnit(slot, "usd"),
    unit: Unit.usd,
  }),
];

/**
 * Create series from a SupplyPattern (sats/btc/usd, no sum/cumulative)
 * @param {Object} args
 * @param {SupplyPattern} args.pattern
 * @param {string} args.title
 * @param {Color} [args.color]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function fromSupplyPattern({ pattern, title, color }) {
  return [
    {
      series: pattern.btc,
      title,
      color,
      unit: Unit.btc,
    },
    {
      series: pattern.sats,
      title,
      color,
      unit: Unit.sats,
    },
    {
      series: pattern.usd,
      title,
      color,
      unit: Unit.usd,
    },
  ];
}

// ============================================================================
// Percent + Ratio helpers
// ============================================================================

/**
 * Create percent + ratio series from a BpsPercentRatioPattern
 * @param {Object} args
 * @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function percentRatio({ pattern, name, color, defaultActive }) {
  return [
    line({
      series: pattern.percent,
      name,
      color,
      defaultActive,
      unit: Unit.percentage,
    }),
    line({
      series: pattern.ratio,
      name,
      color,
      defaultActive,
      unit: Unit.ratio,
    }),
  ];
}

/**
 * Create percent + ratio dots series from a BpsPercentRatioPattern
 * @param {Object} args
 * @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function percentRatioDots({ pattern, name, color, defaultActive }) {
  return [
    dots({
      series: pattern.percent,
      name,
      color,
      defaultActive,
      unit: Unit.percentage,
    }),
    dots({
      series: pattern.ratio,
      name,
      color,
      defaultActive,
      unit: Unit.ratio,
    }),
  ];
}

/**
 * Create percent + ratio baseline series from a BpsPercentRatioPattern
 * @param {Object} args
 * @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} args.pattern
 * @param {string} args.name
 * @param {Color | [Color, Color]} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function percentRatioBaseline({ pattern, name, color, defaultActive }) {
  return [
    baseline({
      series: pattern.percent,
      name,
      color,
      defaultActive,
      unit: Unit.percentage,
    }),
    baseline({
      series: pattern.ratio,
      name,
      color,
      defaultActive,
      unit: Unit.ratio,
    }),
  ];
}

/**
 * Rolling folder tree with percentRatio series (colored in compare, plain in individual)
 * @param {Object} args
 * @param {{ _24h: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1w: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1m: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1y: { percent: AnySeriesPattern, ratio: AnySeriesPattern } }} args.windows
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {string} [args.name]
 * @returns {PartialOptionsGroup}
 */
export function rollingPercentRatioTree({
  windows,
  title = (s) => s,
  metric,
  name = "Sums",
}) {
  return {
    name,
    tree: [
      {
        name: "Compare",
        title: title(metric),
        bottom: ROLLING_WINDOWS.flatMap((w) =>
          percentRatio({
            pattern: windows[w.key],
            name: w.name,
            color: w.color,
          }),
        ),
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${metric}`),
        bottom: percentRatioBaseline({ pattern: windows[w.key], name: "Rate" }),
      })),
    ],
  };
}

/**
 * Create Change + Growth Rate tree from a delta pattern (absolute + rate)
 * @template T
 * @param {Object} args
 * @param {{ absolute: { _24h: T, _1w: T, _1m: T, _1y: T }, rate: { _24h: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1w: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1m: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1y: { percent: AnySeriesPattern, ratio: AnySeriesPattern } } }} args.delta
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {(v: T) => AnySeriesPattern} args.extract
 * @returns {PartialOptionsTree}
 */
export function deltaTree({ delta, title = (s) => s, metric, unit, extract }) {
  return [
    {
      name: "Change",
      tree: [
        {
          name: "Compare",
          title: title(`${metric} Change`),
          bottom: ROLLING_WINDOWS.map((w) =>
            baseline({
              series: extract(delta.absolute[w.key]),
              name: w.name,
              color: w.color,
              unit,
            }),
          ),
        },
        ...ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: title(`${w.title} ${metric} Change`),
          bottom: [
            baseline({
              series: extract(delta.absolute[w.key]),
              name: "Change",
              unit,
            }),
          ],
        })),
      ],
    },
    rollingPercentRatioTree({
      windows: delta.rate,
      title,
      metric: `${metric} Growth Rate`,
      name: "Growth Rate",
    }),
  ];
}

/**
 * deltaTree where absolute windows are directly AnySeriesPattern (no extract needed)
 * @param {Object} args
 * @param {{ absolute: { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }, rate: { _24h: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1w: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1m: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1y: { percent: AnySeriesPattern, ratio: AnySeriesPattern } } }} args.delta
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function simpleDeltaTree({ delta, title = (s) => s, metric, unit }) {
  return deltaTree({ delta, title, metric, unit, extract: (v) => v });
}

// ============================================================================
// Chart-generating helpers (return PartialOptionsTree for folder structures)
// ============================================================================
// These split patterns into separate Sum/Distribution/Cumulative charts

/**
 * Split flat per-block pattern into charts (Averages/Sums/Distribution/Cumulative)
 * Pattern has: .height, .cumulative, .sum (windowed), .average/.pct10/... (windowed, flat)
 * @param {Object} args
 * @param {FullPerBlockPattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {string} [args.distributionSuffix]
 * @returns {PartialOptionsTree}
 */
export function chartsFromFull({
  pattern,
  title = (s) => s,
  metric,
  unit,
  distributionSuffix = "",
}) {
  const distMetric = distributionSuffix
    ? `${metric} ${distributionSuffix}`
    : metric;
  return [
    ...sumsAndAveragesCumulative({
      sum: pattern.sum,
      average: pattern.average,
      cumulative: pattern.cumulative,
      title,
      metric,
      unit,
    }),
    distributionWindowsTree({ pattern, title, metric: distMetric, unit }),
  ];
}

/**
 * Split pattern into 4 charts with "per Block" in distribution title
 * @param {Object} args
 * @param {FullPerBlockPattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export const chartsFromFullPerBlock = (args) =>
  chartsFromFull({ ...args, distributionSuffix: "per Block" });

/**
 * Split pattern with sum + distribution + cumulative into 3 charts (no base)
 * @param {Object} args
 * @param {AggregatedPattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {string} [args.distributionSuffix]
 * @returns {PartialOptionsTree}
 */
export function chartsFromAggregated({
  pattern,
  title = (s) => s,
  metric,
  unit,
  distributionSuffix = "",
}) {
  const distMetric = distributionSuffix
    ? `${metric} ${distributionSuffix}`
    : metric;
  return [
    ...sumsAndAveragesCumulative({
      sum: pattern.rolling.sum,
      average: pattern.rolling.average,
      cumulative: pattern.cumulative,
      title,
      metric,
      unit,
    }),
    distributionWindowsTree({
      pattern: pattern.rolling,
      title,
      metric: distMetric,
      unit,
    }),
  ];
}

/**
 * Split pattern into 3 charts with "per Block" in distribution title (no base)
 * @param {Object} args
 * @param {AggregatedPattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export const chartsFromAggregatedPerBlock = (args) =>
  chartsFromAggregated({ ...args, distributionSuffix: "per Block" });

/**
 * Create Per Block + Per 6 Blocks stats charts from a _6bBlockTxPattern
 * @param {Object} args
 * @param {{ block: DistributionStats, _6b: DistributionStats }} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function chartsFromBlockAnd6b({ pattern, title = (s) => s, metric, unit }) {
  return [
    {
      name: "Block",
      title: title(`${metric} (Block)`),
      bottom: percentileSeries({ pattern: pattern.block, unit }),
    },
    {
      name: "~Hourly",
      title: title(`${metric} (~Hourly)`),
      bottom: percentileSeries({ pattern: pattern._6b, unit }),
    },
  ];
}

/**
 * Averages + Sums + Cumulative charts
 * @param {Object} args
 * @param {Omit<CountPattern<number>, "block">} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {Color} [args.color]
 * @returns {PartialOptionsTree}
 */
export function chartsFromCount({ pattern, title = (s) => s, metric, unit, color }) {
  return sumsAndAveragesCumulative({
    sum: pattern.sum,
    average: pattern.average,
    cumulative: pattern.cumulative,
    title,
    metric,
    unit,
    color,
  });
}

/**
 * Windowed sums + cumulative chart for count patterns without averages.
 * @param {Object} args
 * @param {{ sum: { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }, cumulative: AnySeriesPattern }} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @param {Color} [args.color]
 * @returns {PartialOptionsTree}
 */
export function chartsFromSumCumulative({
  pattern,
  title = (s) => s,
  metric,
  unit,
  color,
}) {
  return [
    {
      name: "Compare",
      title: title(metric),
      bottom: ROLLING_WINDOWS.map((w) =>
        line({
          series: pattern.sum[w.key],
          name: w.name,
          color: w.color,
          unit,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: [
        line({
          series: pattern.sum[w.key],
          name: "Sum",
          color: w.color,
          unit,
        }),
      ],
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: [
        line({
          series: pattern.cumulative,
          name: "All Time",
          color,
          unit,
        }),
      ],
    },
  ];
}

/**
 * Percent + ratio per rolling window + cumulative, mirroring chartsFromCount for percent data.
 * @param {Object} args
 * @param {PercentRatioCumulativePattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Color} [args.color]
 * @returns {PartialOptionsTree}
 */
export function chartsFromPercentCumulative({
  pattern,
  title = (s) => s,
  metric,
  color,
}) {
  return [
    {
      name: "Compare",
      title: title(metric),
      bottom: ROLLING_WINDOWS.flatMap((w) =>
        percentRatio({
          pattern: pattern[w.key],
          name: w.name,
          color: w.color,
        }),
      ).concat(
        percentRatio({
          pattern,
          name: "All Time",
          color: colors.time.all,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: percentRatio({
        pattern: pattern[w.key],
        name: w.name,
        color: color ?? w.color,
      }),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: percentRatio({
        pattern,
        name: "All Time",
        color: color ?? colors.time.all,
      }),
    },
  ];
}

/**
 * N-pattern variant of {@link chartsFromPercentCumulative}: each chart
 * overlays one `percentRatio` pair per entry (percent + ratio lines).
 * Use when comparing alternate views of the same metric (e.g. a share
 * with two different denominators). Each entry MUST provide a `name`;
 * if `color` is omitted the entry inherits the per-slot default colors
 * (window color in Compare/per-window, all-time color for cumulative).
 * @param {Object} args
 * @param {ReadonlyArray<{
 *   name: string,
 *   pattern: PercentRatioCumulativePattern,
 *   color?: Color,
 * }>} args.entries
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @returns {PartialOptionsTree}
 */
export function chartsFromPercentCumulativeEntries({
  entries,
  title = (s) => s,
  metric,
}) {
  return [
    {
      name: "Compare",
      title: title(metric),
      bottom: ROLLING_WINDOWS.flatMap((w) =>
        entries.flatMap((e) =>
          percentRatio({
            pattern: e.pattern[w.key],
            name: `${w.name} ${e.name}`,
            color: e.color ?? w.color,
          }),
        ),
      ).concat(
        entries.flatMap((e) =>
          percentRatio({
            pattern: e.pattern,
            name: `All Time ${e.name}`,
            color: e.color ?? colors.time.all,
          }),
        ),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: entries.flatMap((e) =>
        percentRatio({
          pattern: e.pattern[w.key],
          name: e.name,
          color: e.color ?? w.color,
        }),
      ),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: entries.flatMap((e) =>
        percentRatio({
          pattern: e.pattern,
          name: e.name,
          color: e.color ?? colors.time.all,
        }),
      ),
    },
  ];
}

/**
 * Windowed sums + optional averages + cumulative for multiple named entries.
 * @param {Object} args
 * @param {Array<[string, CountPattern<number>]>} args.entries
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function chartsFromCountEntries({ entries, title = (s) => s, metric, unit }) {
  const items = entries.map(([name, data], i, arr) => ({
    name,
    color: colors.at(i, arr.length),
    sum: data.sum,
    average: data.average,
    cumulative: data.cumulative,
  }));
  return [
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: items.flatMap((e) => [
        line({ series: e.sum[w.key], name: e.name, color: e.color, unit }),
        line({
          series: e.average[w.key],
          name: `${e.name} Avg`,
          color: e.color,
          unit,
          defaultActive: false,
          style: 1,
        }),
      ]),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: items.map((e) =>
        line({ series: e.cumulative, name: e.name, color: e.color, unit }),
      ),
    },
  ];
}

/**
 * Windowed averages + sums + cumulative for multiple named series (e.g. UTXO flow)
 * @param {Object} args
 * @param {Array<{ name: string, color: Color, average: { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }, sum: { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }, cumulative: AnySeriesPattern }>} args.entries
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function multiSeriesTree({ entries, title = (s) => s, metric, unit }) {
  return [
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metric}`),
      bottom: entries.map((e) =>
        line({ series: e.average[w.key], name: e.name, color: e.color, unit }),
      ),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: entries.map((e) =>
        line({ series: e.cumulative, name: e.name, color: e.color, unit }),
      ),
    },
  ];
}
