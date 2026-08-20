/** Market section */

import { colors } from "../utils/colors.js";
import { periodIdToName } from "../utils/time.js";
import { brk } from "../utils/client.js";
import { includes } from "../utils/array.js";
import { Unit } from "../utils/units.js";
import { priceLine, priceLines } from "./constants.js";
import {
  baseline,
  deltaTree,
  histogram,
  line,
  price,
  percentRatio,
  percentRatioBaseline,
  ROLLING_WINDOWS,
  ROLLING_WINDOWS_TO_1M,
} from "./series.js";
import { simplePriceRatioTree } from "./shared.js";

/**
 * @typedef {Object} Period
 * @property {string} id
 * @property {Color} color
 * @property {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} returns
 * @property {AnyPricePattern} lookback
 * @property {boolean} [defaultActive]
 */

/**
 * @typedef {Period & { cagr: { percent: AnySeriesPattern, ratio: AnySeriesPattern } }} PeriodWithCagr
 */

/**
 * @typedef {Object} MaPeriod
 * @property {string} id
 * @property {Color} color
 * @property {MaPriceRatioPattern} ratio
 */

/**
 * Create index (percent) + ratio line pair from a BpsPercentRatioPattern
 * @param {{ pattern: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, name: string, color?: Color, defaultActive?: boolean }} args
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
function indexRatio({ pattern, name, color, defaultActive }) {
  return [
    line({
      series: pattern.percent,
      name,
      color,
      defaultActive,
      unit: Unit.index,
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

const commonMaIds = /** @type {const} */ ([
  "1w",
  "1m",
  "200d",
  "1y",
  "200w",
  "4y",
]);

/**
 * @param {string} label
 * @param {MaPeriod[]} averages
 */
function createMaSubSection(label, averages) {
  const common = averages.filter((a) => includes(commonMaIds, a.id));
  const more = averages.filter((a) => !includes(commonMaIds, a.id));

  /** @param {MaPeriod} a */
  const toChart = (a) => {
    const name = periodIdToName(a.id, true);
    const [chart] = simplePriceRatioTree({
      pattern: a.ratio,
      title: `${name} ${label}`,
      legend: "Average",
      color: a.color,
    });
    return { ...chart, name };
  };

  return {
    name: label,
    tree: [
      {
        name: "Compare",
        title: `Price ${label}s`,
        top: averages.map((a) =>
          price({
            series: a.ratio,
            name: a.id,
            color: a.color,
            defaultActive: includes(commonMaIds, a.id),
          }),
        ),
      },
      ...common.map(toChart),
      { name: "More...", tree: more.map(toChart) },
    ],
  };
}

/**
 * @param {string} name
 * @param {Period[]} periods
 */
function returnsSubSection(name, periods) {
  return {
    name,
    tree: [
      {
        name: "Compare",
        title: `${name} Price Returns`,
        bottom: periods.flatMap((p) =>
          percentRatioBaseline({
            pattern: p.returns,
            name: p.id,
            color: p.color,
          }),
        ),
      },
      ...periods.map((p) => ({
        name: periodIdToName(p.id, true),
        title: `${periodIdToName(p.id, true)} Price Returns`,
        bottom: percentRatioBaseline({ pattern: p.returns, name: "Return" }),
      })),
    ],
  };
}

/**
 * @param {string} name
 * @param {PeriodWithCagr[]} periods
 */
function returnsSubSectionWithCagr(name, periods) {
  return {
    name,
    tree: [
      {
        name: "Total",
        tree: [
          {
            name: "Compare",
            title: `${name} Total Price Returns`,
            bottom: periods.flatMap((p) =>
              percentRatioBaseline({
                pattern: p.returns,
                name: p.id,
                color: p.color,
              }),
            ),
          },
          ...periods.map((p) => ({
            name: periodIdToName(p.id, true),
            title: `${periodIdToName(p.id, true)} Total Price Returns`,
            bottom: percentRatioBaseline({
              pattern: p.returns,
              name: "Return",
            }),
          })),
        ],
      },
      {
        name: "CAGR",
        tree: [
          {
            name: "Compare",
            title: `${name} Price CAGR`,
            bottom: periods.flatMap((p) =>
              percentRatioBaseline({
                pattern: p.cagr,
                name: p.id,
                color: p.color,
              }),
            ),
          },
          ...periods.map((p) => ({
            name: periodIdToName(p.id, true),
            title: `${periodIdToName(p.id, true)} Price CAGR`,
            bottom: percentRatioBaseline({ pattern: p.cagr, name: "CAGR" }),
          })),
        ],
      },
    ],
  };
}

/**
 * @param {string} name
 * @param {Period[]} periods
 */
function historicalSubSection(name, periods) {
  return {
    name,
    tree: [
      {
        name: "Compare",
        title: `${name} Historical Prices`,
        top: periods.map((p) =>
          price({ series: p.lookback, name: p.id, color: p.color }),
        ),
      },
      ...periods.map((p) => ({
        name: periodIdToName(p.id, true),
        title: `Price ${periodIdToName(p.id)} Ago`,
        top: [price({ series: p.lookback, name: "Price" })],
      })),
    ],
  };
}

/**
 * Create Market section
 * @returns {PartialOptionsGroup}
 */
export function createMarketSection() {
  const {
    market,
    supply,
    cohorts,
    price: prices,
    indicators,
  } = brk.series;
  const { cointime, coinflow } = brk.series.frameworks;
  const { rarityMeter } = brk.series.models;
  const {
    movingAverage: ma,
    ath,
    returns,
    volatility,
    range,
    technical,
    lookback,
  } = market;

  const shortPeriodsBase = [
    { id: "24h", returns: returns.periods._24h, lookback: lookback._24h },
    { id: "1w", returns: returns.periods._1w, lookback: lookback._1w },
    { id: "1m", returns: returns.periods._1m, lookback: lookback._1m },
    {
      id: "3m",
      returns: returns.periods._3m,
      lookback: lookback._3m,
      defaultActive: false,
    },
    {
      id: "6m",
      returns: returns.periods._6m,
      lookback: lookback._6m,
      defaultActive: false,
    },
    { id: "1y", returns: returns.periods._1y, lookback: lookback._1y },
  ];

  const longPeriodsBase = [
    {
      id: "2y",
      returns: returns.periods._2y,
      cagr: returns.cagr._2y,
      lookback: lookback._2y,
      defaultActive: false,
    },
    {
      id: "3y",
      returns: returns.periods._3y,
      cagr: returns.cagr._3y,
      lookback: lookback._3y,
      defaultActive: false,
    },
    {
      id: "4y",
      returns: returns.periods._4y,
      cagr: returns.cagr._4y,
      lookback: lookback._4y,
    },
    {
      id: "5y",
      returns: returns.periods._5y,
      cagr: returns.cagr._5y,
      lookback: lookback._5y,
      defaultActive: false,
    },
    {
      id: "6y",
      returns: returns.periods._6y,
      cagr: returns.cagr._6y,
      lookback: lookback._6y,
      defaultActive: false,
    },
    {
      id: "8y",
      returns: returns.periods._8y,
      cagr: returns.cagr._8y,
      lookback: lookback._8y,
      defaultActive: false,
    },
    {
      id: "10y",
      returns: returns.periods._10y,
      cagr: returns.cagr._10y,
      lookback: lookback._10y,
      defaultActive: false,
    },
  ];

  const totalReturnPeriods = shortPeriodsBase.length + longPeriodsBase.length;

  /** @type {Period[]} */
  const shortPeriods = shortPeriodsBase.map((p, i) => ({
    ...p,
    color: colors.at(i, totalReturnPeriods),
  }));

  /** @type {PeriodWithCagr[]} */
  const longPeriods = longPeriodsBase.map((p, i) => ({
    ...p,
    color: colors.at(shortPeriodsBase.length + i, totalReturnPeriods),
  }));

  /** @type {MaPeriod[]} */
  const sma = [
    { id: "1w", ratio: ma.sma._1w },
    { id: "8d", ratio: ma.sma._8d },
    { id: "13d", ratio: ma.sma._13d },
    { id: "21d", ratio: ma.sma._21d },
    { id: "1m", ratio: ma.sma._1m },
    { id: "34d", ratio: ma.sma._34d },
    { id: "50d", ratio: ma.sma._50d },
    { id: "55d", ratio: ma.sma._55d },
    { id: "89d", ratio: ma.sma._89d },
    { id: "111d", ratio: ma.sma._111d },
    { id: "144d", ratio: ma.sma._144d },
    { id: "200d", ratio: ma.sma._200d },
    { id: "350d", ratio: ma.sma._350d },
    { id: "1y", ratio: ma.sma._1y },
    { id: "2y", ratio: ma.sma._2y },
    { id: "200w", ratio: ma.sma._200w },
    { id: "4y", ratio: ma.sma._4y },
  ].map((p, i, arr) => ({ ...p, color: colors.at(i, arr.length) }));

  /** @type {MaPeriod[]} */
  const ema = [
    { id: "1w", ratio: ma.ema._1w },
    { id: "8d", ratio: ma.ema._8d },
    { id: "12d", ratio: ma.ema._12d },
    { id: "13d", ratio: ma.ema._13d },
    { id: "21d", ratio: ma.ema._21d },
    { id: "26d", ratio: ma.ema._26d },
    { id: "1m", ratio: ma.ema._1m },
    { id: "34d", ratio: ma.ema._34d },
    { id: "55d", ratio: ma.ema._55d },
    { id: "89d", ratio: ma.ema._89d },
    { id: "144d", ratio: ma.ema._144d },
    { id: "200d", ratio: ma.ema._200d },
    { id: "1y", ratio: ma.ema._1y },
    { id: "2y", ratio: ma.ema._2y },
    { id: "200w", ratio: ma.ema._200w },
    { id: "4y", ratio: ma.ema._4y },
  ].map((p, i, arr) => ({ ...p, color: colors.at(i, arr.length) }));

  // SMA vs EMA comparison periods (common periods only)
  const smaVsEma = [
    {
      id: "1w",
      name: "1 Week",
      sma: ma.sma._1w,
      ema: ma.ema._1w,
    },
    {
      id: "1m",
      name: "1 Month",
      sma: ma.sma._1m,
      ema: ma.ema._1m,
    },
    {
      id: "200d",
      name: "200 Day",
      sma: ma.sma._200d,
      ema: ma.ema._200d,
    },
    {
      id: "1y",
      name: "1 Year",
      sma: ma.sma._1y,
      ema: ma.ema._1y,
    },
    {
      id: "200w",
      name: "200 Week",
      sma: ma.sma._200w,
      ema: ma.ema._200w,
    },
    {
      id: "4y",
      name: "4 Year",
      sma: ma.sma._4y,
      ema: ma.ema._4y,
    },
  ].map((p, i, arr) => ({ ...p, color: colors.at(i, arr.length) }));

  return {
    name: "Market",
    tree: [
      // Price
      { name: "Price", title: "Bitcoin Price" },

      // Sats/$
      {
        name: "Sats/$",
        title: "Sats per Dollar",
        bottom: [
          line({
            series: prices.spot.sats,
            name: "Sats/$",
            unit: Unit.sats,
          }),
        ],
      },

      // All Time High
      {
        name: "All Time High",
        tree: [
          {
            name: "Drawdown",
            title: "ATH Drawdown",
            top: [price({ series: ath.high, name: "ATH" })],
            bottom: percentRatio({
              pattern: ath.drawdown,
              name: "Drawdown",
              color: colors.loss,
            }),
          },
          {
            name: "Time Since",
            title: "Time Since ATH",
            top: [price({ series: ath.high, name: "ATH" })],
            bottom: [
              line({
                series: ath.daysSince,
                name: "Days",
                unit: Unit.days,
              }),
              line({
                series: ath.yearsSince,
                name: "Years",
                unit: Unit.years,
              }),
              line({
                series: ath.maxDaysBetween,
                name: "Max Days",
                color: colors.loss,
                unit: Unit.days,
              }),
              line({
                series: ath.maxYearsBetween,
                name: "Max Years",
                color: colors.loss,
                unit: Unit.years,
              }),
            ],
          },
        ],
      },

      // Returns
      {
        name: "Returns",
        tree: [
          {
            name: "Compare",
            title: "Price Returns",
            bottom: [...shortPeriods, ...longPeriods].flatMap((p) =>
              percentRatioBaseline({
                pattern: p.returns,
                name: p.id,
                color: p.color,
                defaultActive: p.defaultActive,
              }),
            ),
          },
          returnsSubSection("Short-term", shortPeriods),
          returnsSubSectionWithCagr("Long-term", longPeriods),
        ],
      },

      // Historical
      {
        name: "Historical",
        tree: [
          {
            name: "Compare",
            title: "Historical Prices",
            top: [...shortPeriods, ...longPeriods].map((p) =>
              price({
                series: p.lookback,
                name: p.id,
                color: p.color,
                defaultActive: p.defaultActive,
              }),
            ),
          },
          historicalSubSection("Short-term", shortPeriods),
          historicalSubSection("Long-term", longPeriods),
        ],
      },

      // Capitalization
      {
        name: "Capitalization",
        tree: [
          {
            name: "Compare",
            title: "Market vs Realized Cap",
            bottom: [
              line({
                series: supply.marketCap.usd,
                name: "Market Cap",
                unit: Unit.usd,
              }),
              line({
                series: cohorts.utxo.all.realized.cap.usd,
                name: "Realized Cap",
                color: colors.realized,
                unit: Unit.usd,
              }),
            ],
          },
          {
            name: "Market Cap",
            tree: [
              {
                name: "Value",
                title: "Market Capitalization",
                bottom: [
                  line({
                    series: supply.marketCap.usd,
                    name: "Market Cap",
                    unit: Unit.usd,
                  }),
                ],
              },
              ...deltaTree({
                delta: supply.marketCap.delta,
                metric: "Market Cap",
                unit: Unit.usd,
                extract: (v) => v.usd,
              }),
            ],
          },
          {
            name: "Realized Cap",
            tree: [
              {
                name: "Value",
                title: "Realized Cap",
                bottom: [
                  line({
                    series: cohorts.utxo.all.realized.cap.usd,
                    name: "Realized Cap",
                    color: colors.realized,
                    unit: Unit.usd,
                  }),
                ],
              },
              ...deltaTree({
                delta: cohorts.utxo.all.realized.cap.delta,
                metric: "Realized Cap",
                unit: Unit.usd,
                extract: (v) => v.usd,
              }),
            ],
          },
          {
            name: "Growth Rate Spread",
            tree: [
              {
                name: "Compare",
                title: "Capitalization Growth Rate Spread",
                bottom: ROLLING_WINDOWS.map((w) =>
                  baseline({
                    series: supply.marketMinusRealizedCapGrowthRate[w.key],
                    name: w.name,
                    color: w.color,
                    unit: Unit.percentage,
                  }),
                ),
              },
              ...ROLLING_WINDOWS.map((w) => ({
                name: w.name,
                title: `${w.title} Capitalization Growth Rate Spread`,
                bottom: [
                  baseline({
                    series: supply.marketMinusRealizedCapGrowthRate[w.key],
                    name: "Spread",
                    unit: Unit.percentage,
                  }),
                ],
              })),
            ],
          },
        ],
      },

      // Technical
      {
        name: "Technical",
        tree: [
          // Moving Averages
          {
            name: "Moving Averages",
            tree: [
              {
                name: "SMA vs EMA",
                tree: [
                  {
                    name: "All Periods",
                    title: "SMA vs EMA",
                    top: smaVsEma.flatMap((p) => [
                      price({
                        series: p.sma,
                        name: `${p.id} SMA`,
                        color: p.color,
                      }),
                      price({
                        series: p.ema,
                        name: `${p.id} EMA`,
                        color: p.color,
                        style: 1,
                      }),
                    ]),
                  },
                  ...smaVsEma.map((p) => ({
                    name: p.name,
                    title: `${p.name} SMA vs EMA`,
                    top: [
                      price({ series: p.sma, name: "SMA", color: p.color }),
                      price({
                        series: p.ema,
                        name: "EMA",
                        color: p.color,
                        style: 1,
                      }),
                    ],
                  })),
                ],
              },
              createMaSubSection("SMA", sma),
              createMaSubSection("EMA", ema),
            ],
          },

          // RSI
          {
            name: "RSI",
            tree: [
              {
                name: "Compare",
                title: "RSI",
                bottom: [
                  ...ROLLING_WINDOWS_TO_1M.flatMap((w) =>
                    indexRatio({
                      pattern: technical.rsi[w.key].rsi,
                      name: w.name,
                      color: w.color,
                    }),
                  ),
                  priceLine({ unit: Unit.index, number: 70 }),
                  priceLine({ unit: Unit.index, number: 30 }),
                ],
              },
              ...ROLLING_WINDOWS_TO_1M.map((w) => {
                const rsi = technical.rsi[w.key];
                return {
                  name: w.name,
                  title: `${w.title} RSI`,
                  bottom: [
                    ...indexRatio({
                      pattern: rsi.rsi,
                      name: "RSI",
                      color: colors.indicator.main,
                    }),
                    priceLine({ unit: Unit.index, number: 70 }),
                    priceLine({
                      unit: Unit.index,
                      number: 50,
                      defaultActive: false,
                    }),
                    priceLine({ unit: Unit.index, number: 30 }),
                  ],
                };
              }),
              {
                name: "Stochastic",
                tree: ROLLING_WINDOWS_TO_1M.map((w) => {
                  const rsi = technical.rsi[w.key];
                  return {
                    name: w.name,
                    title: `${w.title} Stochastic RSI`,
                    bottom: [
                      ...indexRatio({
                        pattern: rsi.stochRsiK,
                        name: "K",
                        color: colors.indicator.fast,
                      }),
                      ...indexRatio({
                        pattern: rsi.stochRsiD,
                        name: "D",
                        color: colors.indicator.slow,
                      }),
                      ...priceLines({
                        unit: Unit.index,
                        numbers: [80, 20],
                      }),
                    ],
                  };
                }),
              },
            ],
          },

          // MACD
          {
            name: "MACD",
            tree: [
              {
                name: "Compare",
                title: "MACD",
                bottom: ROLLING_WINDOWS_TO_1M.map((w) =>
                  line({
                    series: technical.macd[w.key].line,
                    name: w.name,
                    color: w.color,
                    unit: Unit.usd,
                  }),
                ),
              },
              ...ROLLING_WINDOWS_TO_1M.map((w) => ({
                name: w.name,
                title: `${w.title} MACD`,
                bottom: [
                  line({
                    series: technical.macd[w.key].line,
                    name: "MACD",
                    color: colors.indicator.fast,
                    unit: Unit.usd,
                  }),
                  line({
                    series: technical.macd[w.key].signal,
                    name: "Signal",
                    color: colors.indicator.slow,
                    unit: Unit.usd,
                  }),
                  histogram({
                    series: technical.macd[w.key].histogram,
                    name: "Histogram",
                    unit: Unit.usd,
                  }),
                ],
              })),
            ],
          },

          // Volatility
          {
            name: "Volatility",
            tree: [
              {
                name: "Index",
                tree: [
                  {
                    name: "Compare",
                    title: "Volatility Index",
                    bottom: ROLLING_WINDOWS.map((w) =>
                      line({
                        series: volatility[w.key],
                        name: w.name,
                        color: w.color,
                        unit: Unit.percentage,
                      }),
                    ),
                  },
                  ...ROLLING_WINDOWS.map((w) => ({
                    name: w.name,
                    title: `${w.title} Volatility Index`,
                    bottom: [
                      line({
                        series: volatility[w.key],
                        name: "Volatility",
                        color: w.color,
                        unit: Unit.percentage,
                      }),
                    ],
                  })),
                ],
              },
              {
                name: "True Range",
                tree: [
                  {
                    name: "Daily",
                    title: "Daily True Range",
                    bottom: [
                      line({
                        series: range.trueRange,
                        name: "True Range",
                        color: colors.time._24h,
                        unit: Unit.usd,
                      }),
                    ],
                  },
                  {
                    name: "2 Week Sum",
                    title: "2 Week True Range Sum",
                    bottom: [
                      line({
                        series: range.trueRangeSum2w,
                        name: "2w Sum",
                        color: colors.time._1w,
                        unit: Unit.usd,
                      }),
                    ],
                  },
                ],
              },
              {
                name: "Choppiness",
                title: "Choppiness Index",
                bottom: [
                  ...percentRatio({
                    pattern: range.choppinessIndex2w,
                    name: "2w",
                    color: colors.indicator.main,
                  }),
                  ...priceLines({ unit: Unit.index, numbers: [61.8, 38.2] }),
                ],
              },
            ],
          },

          // Price Bands
          {
            name: "Price Bands",
            tree: [
              {
                name: "Mayer Multiple",
                title: "Mayer Multiple",
                top: [
                  price({
                    series: ma.sma._200d,
                    name: "200d SMA",
                    color: colors.indicator.main,
                  }),
                  price({
                    series: ma.sma._200d.x24,
                    name: "200d SMA x2.4",
                    color: colors.indicator.upper,
                  }),
                  price({
                    series: ma.sma._200d.x08,
                    name: "200d SMA x0.8",
                    color: colors.indicator.lower,
                  }),
                ],
              },
              {
                name: "Min/Max",
                tree: [
                  {
                    id: "1w",
                    name: "1 Week",
                    min: range.min._1w,
                    max: range.max._1w,
                  },
                  {
                    id: "2w",
                    name: "2 Week",
                    min: range.min._2w,
                    max: range.max._2w,
                  },
                  {
                    id: "1m",
                    name: "1 Month",
                    min: range.min._1m,
                    max: range.max._1m,
                  },
                  {
                    id: "1y",
                    name: "1 Year",
                    min: range.min._1y,
                    max: range.max._1y,
                  },
                ].map((p) => ({
                  name: p.id,
                  title: `${p.name} Min/Max`,
                  top: [
                    price({
                      series: p.max,
                      name: "Max",
                      key: "price-max",
                      color: colors.stat.max,
                    }),
                    price({
                      series: p.min,
                      name: "Min",
                      key: "price-min",
                      color: colors.stat.min,
                    }),
                  ],
                })),
              },
            ],
          },
        ],
      },

      // Indicators
      {
        name: "Indicators",
        tree: [
          {
            name: "Value Anchors",
            title: "Bitcoin Value Anchors",
            top: [
              price({
                series: cointime.prices.trueMarketMean,
                name: "True Market Mean",
                color: colors.trueMarketMean,
              }),
              price({
                series: cointime.awake.price,
                name: "Awake Price",
                color: colors.awake,
              }),
              price({
                series: coinflow.price,
                name: "Coinflow Price",
                color: colors.coinflow,
              }),
              price({
                series: rarityMeter.cycle.pct50,
                name: "Cycle Rarity Midpoint",
                color: colors.ratioPct._50,
              }),
            ],
          },
          {
            name: "NVT",
            title: "NVT Ratio",
            bottom: [
              line({
                series: indicators.nvt.ratio,
                name: "NVT",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "Thermocap Multiple",
            title: "Thermocap Multiple",
            bottom: [
              line({
                series: indicators.thermoCapMultiple.ratio,
                name: "Thermocap",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "Puell Multiple",
            title: "Puell Multiple",
            bottom: [
              line({
                series: indicators.puellMultiple.ratio,
                name: "Puell",
                color: colors.usd,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "RHODL Ratio",
            title: "RHODL Ratio",
            bottom: [
              line({
                series: indicators.rhodlRatio.ratio,
                name: "RHODL",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "Stock-to-Flow",
            title: "Stock-to-Flow",
            bottom: [
              line({
                series: indicators.stockToFlow,
                name: "S2F",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "Pi Cycle",
            title: "Pi Cycle",
            top: [
              price({
                series: ma.sma._111d,
                name: "111d SMA",
                color: colors.indicator.upper,
              }),
              price({
                series: ma.sma._350d.x2,
                name: "350d SMA x2",
                color: colors.indicator.lower,
              }),
            ],
            bottom: [
              baseline({
                series: technical.piCycle.ratio,
                name: "Pi Cycle",
                unit: Unit.ratio,
                base: 1,
              }),
            ],
          },
          {
            name: "Dormancy",
            title: "Dormancy",
            bottom: [
              line({
                series: indicators.dormancy.supplyAdj,
                name: "Supply Adjusted",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
              line({
                series: indicators.dormancy.flow,
                name: "Flow",
                color: colors.usd,
                unit: Unit.ratio,
                defaultActive: false,
              }),
            ],
          },
          {
            name: "Seller Exhaustion",
            title: "Seller Exhaustion Constant",
            bottom: [
              line({
                series: indicators.sellerExhaustion,
                name: "SEC",
                color: colors.bitcoin,
                unit: Unit.ratio,
              }),
            ],
          },
          {
            name: "Coin Destruction",
            tree: [
              {
                name: "CDD",
                title: "Coindays Destroyed (Supply Adjusted)",
                bottom: [
                  line({
                    series: indicators.coindaysDestroyedSupplyAdj,
                    name: "CDD",
                    color: colors.bitcoin,
                    unit: Unit.ratio,
                  }),
                ],
              },
              {
                name: "CYD",
                title: "Coinyears Destroyed (Supply Adjusted)",
                bottom: [
                  line({
                    series: indicators.coinyearsDestroyedSupplyAdj,
                    name: "CYD",
                    color: colors.usd,
                    unit: Unit.ratio,
                  }),
                ],
              },
            ],
          },
        ],
      },
    ],
  };
}
