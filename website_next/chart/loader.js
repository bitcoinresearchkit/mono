import { bitview } from "../utils/client.js";
import { fetchTimeframe } from "./timeframes.js";

/**
 * @param {ChartResult} result
 * @returns {ChartSample[]}
 */
function createSamples(result) {
  /** @type {ChartSample[]} */
  const samples = [];
  /** @type {number | undefined} */
  let lastY;

  for (const [x, y] of result.dateEntries()) {
    if (typeof y === "number" && Number.isFinite(y)) lastY = y;
    if (lastY !== undefined) samples.push({ x, y: lastY });
  }

  return samples;
}

/**
 * @param {Chart} chart
 * @param {TimeframeValue} timeframe
 */
function loadSeries(chart, timeframe) {
  return Promise.all(
    chart.series.map(async (item) => ({
      series: item,
      color: item.color(),
      samples: createSamples(await fetchTimeframe(item.metric(bitview), timeframe)),
    })),
  );
}

/** @param {Chart} chart */
export function createSeriesLoader(chart) {
  /** @type {TimeframeValue | undefined} */
  let cachedTimeframe;
  /** @type {Promise<LoadedSeries[]> | undefined} */
  let cachedPromise;

  /** @param {TimeframeValue} timeframe */
  return function loadTimeframe(timeframe) {
    if (timeframe !== cachedTimeframe || !cachedPromise) {
      cachedTimeframe = timeframe;
      cachedPromise = loadSeries(chart, timeframe).catch((error) => {
        if (timeframe === cachedTimeframe) cachedPromise = undefined;
        throw error;
      });
    }

    return cachedPromise;
  };
}
