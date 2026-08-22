import { brk } from "../../scripts/utils/client.js";
import { numberToShortUSFormat } from "../../scripts/utils/format.js";
import { formatCompact } from "./format.js";
import { createAverageGrid } from "./grid.js";
import {
  DIVERGING_NEGATIVE_LUT,
  DIVERGING_POSITIVE_LUT,
  INFERNO_LUT,
  divergingPowerIntensityColor,
  powerIntensityColor,
} from "./lut.js";
import { defaultTooltip } from "./tooltip/index.js";

/** @typedef {Brk.Cohort} UrpdCohort */
/** @typedef {Brk.UrpdWeight} UrpdWeight */
/**
 * @typedef {Object} UrpdMetric
 * @property {string} name
 * @property {string} title
 * @property {(bucket: Urpd["buckets"][number]) => number} getValue
 * @property {HeatmapColorFn} color
 * @property {{ valueLabel: string, formatValue: (value: number) => string }} tooltip
 */
/** @typedef {{ name: string, cohort: UrpdCohort }} UrpdCohortFolder */

const AGGREGATION = "log2000";
const MIN_LOG = -2;
const MAX_LOG = 6;
const DEFAULT_MIN_LOG = Math.log10(1_000);
const DEFAULT_MAX_LOG = Math.log10(250_000);
const PRICE_CHOICES = [
  { label: "$0.01", key: "0.01", value: Math.log10(0.01) },
  { label: "$0.1", key: "0.1", value: Math.log10(0.1) },
  { label: "$1", key: "1", value: 0 },
  { label: "$10", key: "10", value: 1 },
  { label: "$100", key: "100", value: 2 },
  { label: "$250", key: "250", value: Math.log10(250) },
  { label: "$1k", key: "1k", value: Math.log10(1_000) },
  { label: "$2.5k", key: "2.5k", value: Math.log10(2_500) },
  { label: "$5k", key: "5k", value: Math.log10(5_000) },
  { label: "$10k", key: "10k", value: Math.log10(10_000) },
  { label: "$25k", key: "25k", value: Math.log10(25_000) },
  { label: "$50k", key: "50k", value: Math.log10(50_000) },
  { label: "$100k", key: "100k", value: Math.log10(100_000) },
  { label: "$250k", key: "250k", value: Math.log10(250_000) },
  { label: "$500k", key: "500k", value: Math.log10(500_000) },
  { label: "$1M", key: "1M", value: Math.log10(1_000_000) },
];
const VALUE_COLOR = powerIntensityColor(INFERNO_LUT, 0.4);
const PNL_COLOR = divergingPowerIntensityColor(
  DIVERGING_NEGATIVE_LUT,
  DIVERGING_POSITIVE_LUT,
  0.4,
);

/** @type {UrpdMetric[]} */
const METRICS = [
  {
    name: "supply",
    title: "Supply",
    getValue: (bucket) => bucket.supply,
    color: VALUE_COLOR,
    tooltip: {
      valueLabel: "Supply",
      formatValue: formatBitcoin,
    },
  },
  {
    name: "capital",
    title: "Capital",
    getValue: (bucket) => bucket.realizedCap,
    color: VALUE_COLOR,
    tooltip: {
      valueLabel: "Realized cap",
      formatValue: formatDollar,
    },
  },
  {
    name: "profitability",
    title: "Profitability",
    getValue: (bucket) => bucket.unrealizedPnl,
    color: PNL_COLOR,
    tooltip: {
      valueLabel: "Unrealized PnL",
      formatValue: formatSignedDollar,
    },
  },
];

/** @type {UrpdCohortFolder[]} */
const AGE_BANDS = [
  { name: "Up to 1h", cohort: "utxos_under_1h_old" },
  { name: "1h to 1d", cohort: "utxos_1h_to_1d_old" },
  { name: "1d to 1w", cohort: "utxos_1d_to_1w_old" },
  { name: "1w to 1m", cohort: "utxos_1w_to_1m_old" },
  { name: "1m to 2m", cohort: "utxos_1m_to_2m_old" },
  { name: "2m to 3m", cohort: "utxos_2m_to_3m_old" },
  { name: "3m to 4m", cohort: "utxos_3m_to_4m_old" },
  { name: "4m to 5m", cohort: "utxos_4m_to_5m_old" },
  { name: "5m to 6m", cohort: "utxos_5m_to_6m_old" },
  { name: "6m to 9m", cohort: "utxos_6m_to_9m_old" },
  { name: "9m to 1y", cohort: "utxos_9m_to_1y_old" },
  { name: "1y to 18m", cohort: "utxos_1y_to_18m_old" },
  { name: "18m to 2y", cohort: "utxos_18m_to_2y_old" },
  { name: "2y to 3y", cohort: "utxos_2y_to_3y_old" },
  { name: "3y to 4y", cohort: "utxos_3y_to_4y_old" },
  { name: "4y to 5y", cohort: "utxos_4y_to_5y_old" },
  { name: "5y to 6y", cohort: "utxos_5y_to_6y_old" },
  { name: "6y to 7y", cohort: "utxos_6y_to_7y_old" },
  { name: "7y to 8y", cohort: "utxos_7y_to_8y_old" },
  { name: "8y to 10y", cohort: "utxos_8y_to_10y_old" },
  { name: "10y to 12y", cohort: "utxos_10y_to_12y_old" },
  { name: "12y to 15y", cohort: "utxos_12y_to_15y_old" },
  { name: "Over 15y", cohort: "utxos_over_15y_old" },
];

export const rawUrpdHeatmapTree = createUrpdHeatmapTree();
export const cointimeWeightedUrpdHeatmapTree = createUrpdHeatmapTree({
  weight: "cointime",
  weightTitle: "Cointime-Weighted",
});
export const coinflowWeightedUrpdHeatmapTree = createUrpdHeatmapTree({
  weight: "coinflow",
  weightTitle: "Coinflow-Weighted",
});

/**
 * @param {Object} args
 * @param {UrpdWeight} [args.weight]
 * @param {string} [args.weightTitle]
 * @returns {PartialOptionsTree}
 */
function createUrpdHeatmapTree({ weight, weightTitle } = {}) {
  return [
    ...createCohortHeatmapOptions({ cohort: "all", weight, weightTitle }),
    {
      name: "STH",
      tree: createCohortHeatmapOptions({
        cohort: "sth",
        titlePrefix: "STH",
        weight,
        weightTitle,
      }),
    },
    {
      name: "LTH",
      tree: createCohortHeatmapOptions({
        cohort: "lth",
        titlePrefix: "LTH",
        weight,
        weightTitle,
      }),
    },
    {
      name: "Age Bands",
      tree: AGE_BANDS.map(({ name, cohort }) => ({
        name,
        tree: createCohortHeatmapOptions({
          cohort,
          titlePrefix: name,
          weight,
          weightTitle,
        }),
      })),
    },
  ];
}

/**
 * @param {Object} args
 * @param {UrpdCohort} args.cohort
 * @param {string} [args.titlePrefix]
 * @param {UrpdWeight} [args.weight]
 * @param {string} [args.weightTitle]
 * @returns {PartialHeatmapOption[]}
 */
function createCohortHeatmapOptions({
  cohort,
  titlePrefix,
  weight,
  weightTitle,
}) {
  return METRICS.map((metric) => {
    const title = [weightTitle, titlePrefix, metric.title, "Distribution"]
      .filter(Boolean)
      .join(" ");
    const tooltip = weightTitle
      ? {
          ...metric.tooltip,
          valueLabel: `${weightTitle} ${metric.tooltip.valueLabel.toLowerCase()}`,
        }
      : metric.tooltip;

    return createUrpdHeatmapOption({
      ...metric,
      cohort,
      title,
      tooltip,
      weight,
    });
  });
}

/**
 * @param {Object} args
 * @param {UrpdCohort} args.cohort
 * @param {string} args.name
 * @param {string} args.title
 * @param {UrpdWeight} [args.weight]
 * @param {(bucket: Urpd["buckets"][number]) => number} args.getValue
 * @param {HeatmapColorFn} args.color
 * @param {{ valueLabel?: string, formatValue?: (value: number) => string }} args.tooltip
 * @returns {PartialHeatmapOption}
 */
function createUrpdHeatmapOption({
  cohort,
  name,
  title,
  weight,
  getValue,
  color,
  tooltip,
}) {
  return {
    kind: "heatmap",
    name,
    title,
    points: {
      fetch: (date, signal, onPoints) =>
        fetchUrpdPoints({
          cohort,
          date,
          weight,
          signal,
          getValue,
          onPoints,
        }),
    },
    grid: createAverageGrid({
      yMin: MIN_LOG,
      yMax: MAX_LOG,
      minCellSize: 2,
    }),
    color,
    axis: {
      y: {
        label: "price",
        choices: PRICE_CHOICES,
        format: formatPrice,
      },
    },
    defaults: {
      from: "2017",
      to: "today",
      yMin: DEFAULT_MIN_LOG,
      yMax: DEFAULT_MAX_LOG,
    },
    tooltip: defaultTooltip(tooltip),
  };
}

/**
 * @param {{ cohort: UrpdCohort, date: string, weight?: UrpdWeight, signal: AbortSignal, getValue: (bucket: Urpd["buckets"][number]) => number, onPoints?: (points: HeatmapPoints) => void }} args
 * @returns {Promise<HeatmapPoints>}
 */
async function fetchUrpdPoints({
  cohort,
  date,
  weight,
  signal,
  getValue,
  onPoints,
}) {
  /** @type {HeatmapPoints | undefined} */
  let points;
  const urpd = await brk.getUrpdAt(
    cohort,
    date,
    AGGREGATION,
    weight,
    {
      signal,
      onValue: onPoints
        ? (urpd) => {
            points = toPoints(urpd, getValue);
            onPoints(points);
          }
        : undefined,
    },
  );

  return points ?? toPoints(urpd, getValue);
}

/**
 * @param {Urpd} urpd
 * @param {(bucket: Urpd["buckets"][number]) => number} getValue
 * @returns {HeatmapPoints}
 */
function toPoints(urpd, getValue) {
  const buckets = urpd.buckets;
  const y = new Float64Array(buckets.length);
  const values = new Float64Array(buckets.length);
  let length = 0;

  for (let i = 0; i < buckets.length; i++) {
    const bucket = buckets[i];
    const pointValue = getValue(bucket);
    if (bucket.priceFloor <= 0 || !Number.isFinite(pointValue)) continue;
    y[length] = Math.log10(bucket.priceFloor);
    values[length] = pointValue;
    length++;
  }

  return {
    kind: "explicit",
    y: y.subarray(0, length),
    values: values.subarray(0, length),
  };
}

/** @param {number} value */
function formatPrice(value) {
  const rounded = Math.round(value);
  if (Math.abs(value - rounded) < 0.001) {
    const choice = PRICE_CHOICES.find((choice) => choice.value === rounded);
    if (choice) return choice.label;
  }

  const price = 10 ** value;
  if (price >= 1_000_000) return `$${formatCompact(price / 1_000_000)}M`;
  if (price >= 1_000) return `$${formatCompact(price / 1_000)}k`;
  return `$${formatCompact(price)}`;
}

/** @param {number} value */
function formatBitcoin(value) {
  return `${numberToShortUSFormat(value)} BTC`;
}

/** @param {number} value */
function formatDollar(value) {
  return `$${numberToShortUSFormat(value)}`;
}

/** @param {number} value */
function formatSignedDollar(value) {
  const formatted = `$${numberToShortUSFormat(Math.abs(value))}`;
  if (value > 0) return `+${formatted}`;
  if (value < 0) return `-${formatted}`;
  return formatted;
}
