import { brk } from "../../scripts/utils/client.js";
import { formatCompact } from "./format.js";
import { createAverageGrid } from "./grid.js";
import { INFERNO_LUT, logIntensityColor } from "./lut.js";
import { defaultTooltip } from "./tooltip/index.js";

const BINS = 2400;
const MIN_LOG = -8;
const BINS_PER_DECADE = 200;
const AMOUNT_CHOICES = [
  { label: "1 sat", key: "1sat", value: -8 },
  { label: "10 sats", key: "10sats", value: -7 },
  { label: "100 sats", key: "100sats", value: -6 },
  { label: "1k sats", key: "1ksats", value: -5 },
  { label: "10k sats", key: "10ksats", value: -4 },
  { label: "100k sats", key: "100ksats", value: -3 },
  { label: "0.01 BTC", key: "0.01btc", value: -2 },
  { label: "0.1 BTC", key: "0.1btc", value: -1 },
  { label: "1 BTC", key: "1btc", value: 0 },
  { label: "10 BTC", key: "10btc", value: 1 },
  { label: "100 BTC", key: "100btc", value: 2 },
  { label: "1k BTC", key: "1kbtc", value: 3 },
  { label: "10k BTC", key: "10kbtc", value: 4 },
];

export const oracleOutputsHeatmapOption = createOracleHeatmapOption(
  "outputs",
  "All",
);
export const oraclePaymentsHeatmapOption = createOracleHeatmapOption(
  "payments",
  "Payments",
);

/**
 * @param {"outputs" | "payments"} mode
 * @param {string} name
 * @returns {PartialHeatmapOption}
 */
function createOracleHeatmapOption(mode, name) {
  return {
    kind: "heatmap",
    name,
    title:
      mode === "outputs" ? "All Output Values" : "Payment Output Values",
    points: {
      fetch: (date, signal, onPoints) =>
        fetchOraclePoints(mode, date, signal, onPoints),
    },
    grid: createAverageGrid({
      yMin: MIN_LOG,
      yMax: MIN_LOG + BINS / BINS_PER_DECADE,
      nativeRows: BINS,
      yOrigin: "top",
    }),
    color: logIntensityColor(INFERNO_LUT),
    axis: {
      y: {
        label: "amount",
        choices: AMOUNT_CHOICES,
        format: formatAmount,
      },
    },
    defaults:
      mode === "payments"
        ? {
            from: "2015",
            to: "today",
            yMin: -5,
            yMax: 2,
          }
        : {
            from: "genesis",
            to: "today",
          },
    tooltip: defaultTooltip(
      mode === "outputs"
        ? { valueLabel: "Outputs" }
        : { valueLabel: "Payment signal" },
    ),
  };
}

/**
 * @param {"outputs" | "payments"} mode
 * @param {string} date
 * @param {AbortSignal} signal
 * @param {(points: HeatmapPoints) => void} [onPoints]
 * @returns {Promise<HeatmapPoints>}
 */
async function fetchOraclePoints(mode, date, signal, onPoints) {
  const values = await fetchOracleValues(
    mode,
    date,
    signal,
    onPoints ? (values) => onPoints(toOraclePoints(values)) : undefined,
  );

  return toOraclePoints(values);
}

/**
 * @param {"outputs" | "payments"} mode
 * @param {string} date
 * @param {AbortSignal} signal
 * @param {(values: number[]) => void} [onValue]
 * @returns {Promise<number[]>}
 */
function fetchOracleValues(mode, date, signal, onValue) {
  return (
    mode === "outputs"
      ? brk.getOracleHistogramOutputs(date, { signal, onValue })
      : brk.getOracleHistogramPayments(date, { signal, onValue })
  );
}

/**
 * @param {number[]} values
 * @returns {HeatmapPoints}
 */
function toOraclePoints(values) {
  return {
    kind: "implicit",
    yStart: MIN_LOG,
    yStep: 1 / BINS_PER_DECADE,
    values,
  };
}

/** @param {number} value */
function formatAmount(value) {
  const rounded = Math.round(value);
  if (Math.abs(value - rounded) < 0.001) {
    const choice = AMOUNT_CHOICES.find((choice) => choice.value === rounded);
    if (choice) return choice.label;
  }
  const btc = 10 ** value;
  if (btc >= 1) return `${formatCompact(btc)} BTC`;
  return `${formatCompact(btc * 100_000_000)} sats`;
}
