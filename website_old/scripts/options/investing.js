/** Investing section - Investment strategy tools and analysis */

import { colors } from "../utils/colors.js";
import { brk } from "../utils/client.js";
import { percentRatioBaseline, price } from "./series.js";
import { satsBtcUsd } from "./shared.js";
import { periodIdToName } from "../utils/time.js";

const SHORT_PERIODS = /** @type {const} */ ([
  "_1w",
  "_1m",
  "_3m",
  "_6m",
  "_1y",
]);
const LONG_PERIODS = /** @type {const} */ ([
  "_2y",
  "_3y",
  "_4y",
  "_5y",
  "_6y",
  "_8y",
  "_10y",
]);

/** @typedef {typeof SHORT_PERIODS[number]} ShortPeriodKey */
/** @typedef {typeof LONG_PERIODS[number]} LongPeriodKey */
/** @typedef {ShortPeriodKey | LongPeriodKey} AllPeriodKey */

/**
 * Add CAGR to a base entry item
 * @param {BaseEntryItem} entry
 * @param {PercentRatioPattern} cagr
 * @returns {LongEntryItem}
 */
const withCagr = (entry, cagr) => ({ ...entry, cagr });

const YEARS_2020S = /** @type {const} */ ([
  2026, 2025, 2024, 2023, 2022, 2021, 2020,
]);
const YEARS_2010S = /** @type {const} */ ([2019, 2018, 2017, 2016, 2015]);

/** @typedef {typeof YEARS_2020S[number] | typeof YEARS_2010S[number]} DcaYear */
/** @typedef {`from${DcaYear}`} DcaYearKey */

/** @param {AllPeriodKey} key */
const periodName = (key) => periodIdToName(key.slice(1), true);

/**
 * @typedef {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} PercentRatioPattern
 */

/**
 * Base entry item for compare and single-entry charts
 * @typedef {Object} BaseEntryItem
 * @property {string} name - Display name
 * @property {Color} color - Item color
 * @property {AnyPricePattern} costBasis - Cost basis series
 * @property {PercentRatioPattern} returns - Returns series
 * @property {AnyValuePattern} stack - Stack pattern
 */

/**
 * Long-term entry item with CAGR
 * @typedef {BaseEntryItem & { cagr: PercentRatioPattern }} LongEntryItem
 */

const ALL_YEARS = /** @type {const} */ ([...YEARS_2020S, ...YEARS_2010S]);

/**
 * Build DCA class entry from year
 * @param {Investing} investing
 * @param {DcaYear} year
 * @param {number} i
 * @returns {BaseEntryItem}
 */
function buildYearEntry(investing, year, i) {
  const key = /** @type {DcaYearKey} */ (`from${year}`);
  return {
    name: `${year}`,
    color: colors.at(i, ALL_YEARS.length),
    costBasis: investing.class.dcaCostBasis[key],
    returns: investing.class.dcaReturn[key],
    stack: investing.class.dcaStack[key],
  };
}

/**
 * Create Investing section
 * @returns {PartialOptionsGroup}
 */
export function createInvestingSection() {
  const { market, investing } = brk.series;
  const { lookback, returns } = market;

  return {
    name: "Investing",
    tree: [
      createDcaVsLumpSumSection({ investing, lookback, returns }),
      createDcaByPeriodSection({ investing, returns }),
      createLumpSumByPeriodSection({ investing, lookback, returns }),
      createDcaByStartYearSection({ investing }),
    ],
  };
}

/**
 * Create compare folder from items
 * @param {string} context
 * @param {Pick<BaseEntryItem, 'name' | 'color' | 'costBasis' | 'returns' | 'stack'>[]} items
 */
function createCompareFolder(context, items) {
  const topPane = items.map(({ name, color, costBasis }) =>
    price({ series: costBasis, name, color }),
  );
  return {
    name: "Compare",
    tree: [
      {
        name: "Cost Basis",
        title: `Cost Basis: ${context}`,
        top: topPane,
      },
      {
        name: "Returns",
        title: `Returns: ${context}`,
        top: topPane,
        bottom: items.flatMap(({ name, color, returns }) =>
          percentRatioBaseline({
            pattern: returns,
            name,
            color: [color, color],
          }),
        ),
      },
      {
        name: "Accumulated",
        title: `Accumulated Value ($100/day): ${context}`,
        top: topPane,
        bottom: items.flatMap(({ name, color, stack }) =>
          satsBtcUsd({ pattern: stack, name, color }),
        ),
      },
    ],
  };
}

/**
 * Create compare folder from long items (includes CAGR chart)
 * @param {string} context
 * @param {LongEntryItem[]} items
 */
function createLongCompareFolder(context, items) {
  const topPane = items.map(({ name, color, costBasis }) =>
    price({ series: costBasis, name, color }),
  );
  return {
    name: "Compare",
    tree: [
      {
        name: "Cost Basis",
        title: `Cost Basis: ${context}`,
        top: topPane,
      },
      {
        name: "Returns",
        title: `Returns: ${context}`,
        top: topPane,
        bottom: items.flatMap(({ name, color, returns }) =>
          percentRatioBaseline({
            pattern: returns,
            name,
            color: [color, color],
          }),
        ),
      },
      {
        name: "CAGR",
        title: `CAGR: ${context}`,
        top: topPane,
        bottom: items.flatMap(({ name, color, cagr }) =>
          percentRatioBaseline({
            pattern: cagr,
            name,
            color: [color, color],
          }),
        ),
      },
      {
        name: "Accumulated",
        title: `Accumulated Value ($100/day): ${context}`,
        top: topPane,
        bottom: items.flatMap(({ name, color, stack }) =>
          satsBtcUsd({ pattern: stack, name, color }),
        ),
      },
    ],
  };
}

/**
 * Create single entry tree structure
 * @param {BaseEntryItem & { titlePrefix?: string }} item
 * @param {object[]} returnsBottom - Bottom pane items for returns chart
 */
function createSingleEntryTree(item, returnsBottom) {
  const { name, titlePrefix = name, color, costBasis, stack } = item;
  const top = [price({ series: costBasis, name: "Cost Basis", color })];
  return {
    name,
    tree: [
      { name: "Cost Basis", title: `Cost Basis: ${titlePrefix}`, top },
      {
        name: "Returns",
        title: `Returns: ${titlePrefix}`,
        top,
        bottom: returnsBottom,
      },
      {
        name: "Accumulated",
        title: `Accumulated Value ($100/day): ${titlePrefix}`,
        top,
        bottom: satsBtcUsd({ pattern: stack, name: "Value" }),
      },
    ],
  };
}

/**
 * Create a single entry from a base item (no CAGR)
 * @param {BaseEntryItem & { titlePrefix?: string }} item
 */
function createShortSingleEntry(item) {
  return createSingleEntryTree(
    item,
    percentRatioBaseline({ pattern: item.returns, name: "Return" }),
  );
}

/**
 * Create a single entry from a long item (with CAGR as its own chart)
 * @param {LongEntryItem & { titlePrefix?: string }} item
 */
function createLongSingleEntry(item) {
  const {
    name,
    titlePrefix = name,
    color,
    costBasis,
    returns,
    cagr,
    stack,
  } = item;
  const top = [price({ series: costBasis, name: "Cost Basis", color })];
  return {
    name,
    tree: [
      { name: "Cost Basis", title: `Cost Basis: ${titlePrefix}`, top },
      {
        name: "Returns",
        title: `Returns: ${titlePrefix}`,
        top,
        bottom: percentRatioBaseline({ pattern: returns, name: "Return" }),
      },
      {
        name: "CAGR",
        title: `CAGR: ${titlePrefix}`,
        top,
        bottom: percentRatioBaseline({ pattern: cagr, name: "CAGR" }),
      },
      {
        name: "Accumulated",
        title: `Accumulated Value ($100/day): ${titlePrefix}`,
        top,
        bottom: satsBtcUsd({ pattern: stack, name: "Value" }),
      },
    ],
  };
}

/**
 * Create DCA vs Lump Sum section
 * @param {Object} args
 * @param {Investing} args.investing
 * @param {Market["lookback"]} args.lookback
 * @param {Market["returns"]} args.returns
 */
export function createDcaVsLumpSumSection({ investing, lookback, returns }) {
  /** @param {AllPeriodKey} key */
  const topPane = (key) => [
    price({
      series: investing.period.dcaCostBasis[key],
      name: "DCA",
      color: colors.profit,
    }),
    price({ series: lookback[key], name: "Lump Sum", color: colors.bitcoin }),
  ];

  /** @param {string} name @param {AllPeriodKey} key */
  const costBasisChart = (name, key) => ({
    name: "Cost Basis",
    title: `Cost Basis: ${name} DCA vs Lump Sum`,
    top: topPane(key),
  });

  /** @param {string} name @param {AllPeriodKey} key */
  const returnsChart = (name, key) => ({
    name: "Returns",
    title: `Returns: ${name} DCA vs Lump Sum`,
    top: topPane(key),
    bottom: [
      ...percentRatioBaseline({
        pattern: investing.period.dcaReturn[key],
        name: "DCA",
      }),
      ...percentRatioBaseline({
        pattern: investing.period.lumpSumReturn[key],
        name: "Lump Sum",
        color: colors.bi.p2,
      }),
    ],
  });

  /** @param {string} name @param {LongPeriodKey} key */
  const longCagrChart = (name, key) => ({
    name: "CAGR",
    title: `CAGR: ${name} DCA vs Lump Sum`,
    top: topPane(key),
    bottom: [
      ...percentRatioBaseline({
        pattern: investing.period.dcaCagr[key],
        name: "DCA",
      }),
      ...percentRatioBaseline({
        pattern: returns.cagr[key],
        name: "Lump Sum",
        color: colors.bi.p2,
      }),
    ],
  });

  /** @param {string} name @param {AllPeriodKey} key */
  const stackChart = (name, key) => ({
    name: "Accumulated",
    title: `Accumulated Value ($100/day): ${name} DCA vs Lump Sum`,
    top: topPane(key),
    bottom: [
      ...satsBtcUsd({
        pattern: investing.period.dcaStack[key],
        name: "DCA",
        color: colors.profit,
      }),
      ...satsBtcUsd({
        pattern: investing.period.lumpSumStack[key],
        name: "Lump Sum",
        color: colors.bitcoin,
      }),
    ],
  });

  /** @param {ShortPeriodKey} key */
  const createShortPeriodEntry = (key) => {
    const name = periodName(key);
    return {
      name,
      tree: [
        costBasisChart(name, key),
        returnsChart(name, key),
        stackChart(name, key),
      ],
    };
  };

  /** @param {LongPeriodKey} key */
  const createLongPeriodEntry = (key) => {
    const name = periodName(key);
    return {
      name,
      tree: [
        costBasisChart(name, key),
        returnsChart(name, key),
        longCagrChart(name, key),
        stackChart(name, key),
      ],
    };
  };

  return {
    name: "DCA vs Lump Sum",
    title: "Compare Investment Strategies",
    tree: [
      {
        name: "Short Term",
        title: "Up to 1 Year",
        tree: SHORT_PERIODS.map(createShortPeriodEntry),
      },
      {
        name: "Long Term",
        title: "2+ Years",
        tree: LONG_PERIODS.map(createLongPeriodEntry),
      },
    ],
  };
}

/**
 * Create period-based section (DCA or Lump Sum)
 * @param {Object} args
 * @param {Investing} args.investing
 * @param {Market["lookback"]} [args.lookback]
 * @param {Market["returns"]} args.returns
 */
function createPeriodSection({ investing, lookback, returns }) {
  const isLumpSum = !!lookback;
  const suffix = isLumpSum ? "Lump Sum" : "DCA";

  const allPeriods = /** @type {const} */ ([...SHORT_PERIODS, ...LONG_PERIODS]);

  /** @param {AllPeriodKey} key @param {number} i @returns {BaseEntryItem} */
  const buildBaseEntry = (key, i) => ({
    name: periodName(key),
    color: colors.at(i, allPeriods.length),
    costBasis: isLumpSum ? lookback[key] : investing.period.dcaCostBasis[key],
    returns: isLumpSum
      ? investing.period.lumpSumReturn[key]
      : investing.period.dcaReturn[key],
    stack: isLumpSum
      ? investing.period.lumpSumStack[key]
      : investing.period.dcaStack[key],
  });

  /** @param {LongPeriodKey} key @param {number} i @returns {LongEntryItem} */
  const buildLongEntry = (key, i) =>
    withCagr(
      buildBaseEntry(key, i),
      isLumpSum ? returns.cagr[key] : investing.period.dcaCagr[key],
    );

  /** @param {BaseEntryItem} entry */
  const createShortEntry = (entry) =>
    createShortSingleEntry({
      ...entry,
      titlePrefix: `${entry.name} ${suffix}`,
    });

  /** @param {LongEntryItem} entry */
  const createLongEntry = (entry) =>
    createLongSingleEntry({
      ...entry,
      titlePrefix: `${entry.name} ${suffix}`,
    });

  const shortEntries = SHORT_PERIODS.map((key, i) => buildBaseEntry(key, i));
  const longEntries = LONG_PERIODS.map((key, i) =>
    buildLongEntry(key, SHORT_PERIODS.length + i),
  );

  return {
    name: `${suffix} by Period`,
    title: `${suffix} Performance by Investment Period`,
    tree: [
      {
        name: "Short Term",
        title: "Up to 1 Year",
        tree: [
          createCompareFolder(`Short Term ${suffix}`, shortEntries),
          ...shortEntries.map(createShortEntry),
        ],
      },
      {
        name: "Long Term",
        title: "2+ Years",
        tree: [
          createLongCompareFolder(`Long Term ${suffix}`, longEntries),
          ...longEntries.map(createLongEntry),
        ],
      },
    ],
  };
}

/**
 * Create DCA by Period section
 * @param {Object} args
 * @param {Investing} args.investing
 * @param {Market["returns"]} args.returns
 */
export function createDcaByPeriodSection({ investing, returns }) {
  return createPeriodSection({ investing, returns });
}

/**
 * Create Lump Sum by Period section
 * @param {Object} args
 * @param {Investing} args.investing
 * @param {Market["lookback"]} args.lookback
 * @param {Market["returns"]} args.returns
 */
export function createLumpSumByPeriodSection({ investing, lookback, returns }) {
  return createPeriodSection({ investing, lookback, returns });
}

/**
 * Create DCA by Start Year section
 * @param {Object} args
 * @param {Investing} args.investing
 */
export function createDcaByStartYearSection({ investing }) {
  /** @param {string} name @param {string} title @param {BaseEntryItem[]} entries */
  const createDecadeGroup = (name, title, entries) => ({
    name,
    title,
    tree: [
      createCompareFolder(`${name} DCA`, entries),
      ...entries.map((entry) =>
        createShortSingleEntry({
          ...entry,
          titlePrefix: `${entry.name} DCA`,
        }),
      ),
    ],
  });

  const entries2020s = YEARS_2020S.map((year, i) =>
    buildYearEntry(investing, year, i),
  );
  const entries2010s = YEARS_2010S.map((year, i) =>
    buildYearEntry(investing, year, YEARS_2020S.length + i),
  );

  return {
    name: "DCA by Start Year",
    title: "DCA Performance by When You Started",
    tree: [
      createCompareFolder("All Years DCA", [...entries2020s, ...entries2010s]),
      createDecadeGroup("2020s", "2020-2026", entries2020s),
      createDecadeGroup("2010s", "2015-2019", entries2010s),
    ],
  };
}
